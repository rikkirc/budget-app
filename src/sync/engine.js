// The sync engine keeps this device's copy ("replica") of one vault in step with the vault's shared, numbered list
// of changes in Firebase. Every device applies the same changes in the same order with the same Rust rules, so
// every phone and computer ends up with identical balances.
//
// It talks to two things, passed in so they can be swapped for test doubles:
//   remote  — the shared list of changes for one vault (src/sync/firestoreRemote.js)
//   replica — this device's copy of the vault, run by vault-core (src/sync/tauriReplica.js)

// After this many changes, a device uploads a checkpoint (a complete copy of the vault), so a new phone downloads
// one file instead of replaying the whole history
export const CHECKPOINT_EVERY = 200;
// A replica this far behind the latest checkpoint jumps straight to it instead of replaying every change
const JUMP_TO_CHECKPOINT_AFTER = 50;

// A dropped connection can't tell us whether the change was recorded, so never invite a blind retry (it could pay a bill twice)
export const OFFLINE_MESSAGE = "ERROR: Couldn't reach the sync service, so this change may not have been saved. Check your connection, then press Refresh to see what was saved before trying again.";

export class OfflineError extends Error {}

/** "YYYY-MM-DD HH:MM:SS" in this device's local time: the time stamped on transactions, the same on every device */
export function localTimestamp(date = new Date()) {
  const p = (n) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${p(date.getMonth() + 1)}-${p(date.getDate())} ${p(date.getHours())}:${p(date.getMinutes())}:${p(date.getSeconds())}`;
}

/**
 * Puts a complete vault (e.g. the desktop's existing data) into the shared vault as a "restore" change, so every
 * device switches to exactly that data. Refuses to replace a vault that already has changes unless replace=true.
 */
export async function uploadVault(remote, bytes, replace) {
  const vault = await remote.getOrCreateVault();
  if (vault.nextSeq > 1 && !replace) {
    throw Object.assign(new Error("ERROR: Your account already has data for this vault."), { status: 409 });
  }
  const id = `u${Date.now()}${Math.random().toString(36).slice(2, 8)}`;
  await remote.putCheckpoint(id, bytes);
  return remote.appendRestoreOp(id);
}

export function createSyncEngine({ remote, replica, clientId, onChange = () => {}, onStatus = () => {}, checkpointEvery = CHECKPOINT_EVERY }) {
  let lastSeq = 0;
  let latestCheckpoint = null; // { id, seq } as last seen in the vault document
  const pending = new Map();   // seq -> change received but not yet applied
  const results = new Map();   // seq -> outcome of a change this device made, until its caller collects it
  const waiters = new Map();   // seq -> resolve() for callers waiting on their change
  let draining = Promise.resolve();
  let stopWatching = () => {};
  let checkpointing = false;
  let closed = false;

  const restoreFrom = async (checkpoint) => {
    const bytes = await remote.getCheckpoint(checkpoint.id);
    if (!bytes) return false;
    await replica.restore(bytes, checkpoint.seq);
    lastSeq = checkpoint.seq;
    for (const seq of [...pending.keys()]) if (seq <= lastSeq) pending.delete(seq);
    return true;
  };

  const settleWaiters = () => {
    for (const [seq, resolve] of waiters) {
      if (seq <= lastSeq) { waiters.delete(seq); resolve(); }
    }
  };

  // Applies every change that is next in line. Runs one at a time, however many arrive at once.
  const drain = () => {
    draining = draining.then(async () => {
      let changedElsewhere = false;
      while (!closed && pending.has(lastSeq + 1)) {
        const op = pending.get(lastSeq + 1);
        if (op.cmd === "__restore__") {
          let restored = await restoreFrom({ id: op.args.checkpoint, seq: op.seq });
          // That upload may have been superseded and tidied away; a newer checkpoint covers it just as well
          if (!restored) {
            const vault = await remote.getVault();
            if (vault?.checkpoint && vault.checkpoint.seq >= op.seq) restored = await restoreFrom(vault.checkpoint);
          }
          if (!restored) break; // try again when the next change arrives
          if (op.clientId !== clientId) changedElsewhere = true;
          if (op.clientId === clientId) results.set(op.seq, { result: "SUCCESS: Vault uploaded!" });
          continue;
        }
        // Apply the whole run of ordinary changes up to the next restore in one go
        const batch = [];
        for (let seq = lastSeq + 1; pending.has(seq) && pending.get(seq).cmd !== "__restore__"; seq++) batch.push(pending.get(seq));
        const outcomes = await replica.apply(batch.map(({ seq, cmd, args, at }) => ({ seq, cmd, args, at })));
        batch.forEach((op, i) => {
          pending.delete(op.seq);
          if (op.clientId === clientId) results.set(op.seq, outcomes[i]);
          else changedElsewhere = true;
        });
        lastSeq = batch[batch.length - 1].seq;
      }
      settleWaiters();
      if (changedElsewhere) onChange();
      maybeCheckpoint();
    }).catch((e) => console.error("Vault sync: could not apply changes", e));
    return draining;
  };

  const receive = (ops) => {
    for (const op of ops) if (op.seq > lastSeq) pending.set(op.seq, op);
    return drain();
  };

  // Every so often, save a checkpoint so new devices start from it instead of from the very first change
  const maybeCheckpoint = async () => {
    if (checkpointing || closed || lastSeq - (latestCheckpoint?.seq ?? 0) < checkpointEvery) return;
    checkpointing = true;
    try {
      const seq = lastSeq;
      const bytes = await replica.snapshot();
      if (lastSeq !== seq) return; // more changes arrived meanwhile; the next round will catch up
      const checkpoint = { id: `s${seq}`, seq };
      await remote.putCheckpoint(checkpoint.id, bytes);
      const previous = latestCheckpoint;
      if (await remote.publishCheckpoint(checkpoint)) {
        latestCheckpoint = checkpoint;
        // Uploads are kept (restore changes point at them); regular checkpoints are replaced by newer ones
        if (previous?.id.startsWith("s")) remote.deleteCheckpoint(previous.id).catch(() => {});
      }
    } catch (e) {
      console.warn("Vault sync: checkpoint skipped", e);
    } finally {
      checkpointing = false;
    }
  };

  const waitFor = (seq) => {
    if (seq <= lastSeq) return Promise.resolve(true);
    return new Promise((resolve) => {
      // The live feed normally delivers our own change within a second; if it's slow, fetch it directly once
      const timer = setTimeout(async () => {
        if (!waiters.has(seq)) return;
        try { await receive(await remote.fetchOps(lastSeq)); } catch { /* still offline */ }
        if (waiters.has(seq)) { waiters.delete(seq); resolve(false); }
      }, 15000);
      waiters.set(seq, () => { clearTimeout(timer); resolve(true); });
    });
  };

  return {
    /** Opens the replica and starts following the shared list. Works offline with whatever this device last saw. */
    async open() {
      lastSeq = await replica.open();
      const watch = () => { stopWatching = remote.watchOps(lastSeq, (ops, live) => { onStatus(live); receive(ops); }, () => onStatus(false)); };
      let vault;
      try {
        vault = await remote.getOrCreateVault();
      } catch (e) {
        if (!(e instanceof OfflineError)) throw e;
        onStatus(false);
        // The live feed reconnects by itself, so changes start flowing again as soon as the connection is back
        watch();
        return { offline: true };
      }
      latestCheckpoint = vault.checkpoint;
      if (vault.checkpoint && (lastSeq === 0 || vault.checkpoint.seq > lastSeq + JUMP_TO_CHECKPOINT_AFTER)) {
        await restoreFrom(vault.checkpoint);
      }
      watch();
      return { offline: false };
    },

    /** Runs a command: reads come straight from the replica; changes go through the shared list first. */
    async call(cmd, args = {}) {
      if (cmd.startsWith("get_")) {
        const reply = await replica.call(cmd, args);
        if ("error" in reply) throw reply.error;
        return reply.result;
      }
      let seq;
      try {
        seq = await remote.appendOp({ cmd, args, at: localTimestamp(), clientId });
      } catch (e) {
        if (e instanceof OfflineError) return OFFLINE_MESSAGE;
        throw e.message ?? String(e);
      }
      const applied = await waitFor(seq);
      if (!applied) return "Saved. It's still syncing to this device, so press Refresh in a moment to see it.";
      const outcome = results.get(seq);
      results.delete(seq);
      if (outcome && "error" in outcome) throw outcome.error;
      return outcome?.result;
    },

    get lastSeq() { return lastSeq; },

    close() { closed = true; stopWatching(); },
  };
}
