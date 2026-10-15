// Test helpers for the sync engine: a real vault-core replica (run through vault-core's replica_cli example) and an
// in-memory stand-in for the shared list of changes. Only used by tests.
import { spawn, execFileSync } from "node:child_process";
import { createInterface } from "node:readline";
import { existsSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { OfflineError } from "./engine.js";

const coreDir = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "vault-core");
const cliPath = join(coreDir, "target", "debug", "examples", process.platform === "win32" ? "replica_cli.exe" : "replica_cli");

export const scratchDir = () => mkdtempSync(join(tmpdir(), "vault-sync-test-"));

/** A replica backed by the real Rust code, exactly as the app's Tauri commands run it */
export function cliReplica(dbPath) {
  if (!existsSync(cliPath)) execFileSync("cargo", ["build", "--quiet", "--example", "replica_cli"], { cwd: coreDir, stdio: "inherit" });
  const child = spawn(cliPath, [], { stdio: ["pipe", "pipe", "inherit"] });
  const lines = createInterface({ input: child.stdout });
  const queue = [];
  lines.on("line", (line) => queue.shift()(JSON.parse(line)));
  const send = (req) => new Promise((resolve, reject) => {
    queue.push((reply) => ("fatal" in reply ? reject(new Error(reply.fatal)) : resolve(reply)));
    child.stdin.write(JSON.stringify(req) + "\n");
  });
  const toHex = (bytes) => Buffer.from(bytes).toString("hex");
  return {
    open: async () => (await send({ op: "open", path: dbPath })).lastSeq,
    apply: async (ops) => (await send({ op: "apply", ops })).results,
    call: (cmd, args) => send({ op: "call", cmd, args }),
    snapshot: async () => new Uint8Array(Buffer.from((await send({ op: "snapshot" })).hex, "hex")),
    restore: async (bytes, seq) => { await send({ op: "restore", hex: toHex(bytes), seq }); },
    close: () => child.stdin.end(),
  };
}

/** One vault's shared list of changes, kept in memory, with the same behaviour as the Firestore version */
export function memoryVault() {
  const state = { vault: null, ops: [], checkpoints: new Map(), watchers: new Set(), reads: 0 };
  // Each device gets its own view, so a test can take one device offline
  const connect = () => {
    let offline = false;
    const guard = () => { if (offline) throw new OfflineError("offline"); };
    const remote = {
      async getVault() { guard(); return state.vault && structuredClone(state.vault); },
      async getOrCreateVault() { guard(); state.vault ??= { nextSeq: 1, checkpoint: null }; return structuredClone(state.vault); },
      async appendOp({ cmd, args, at, clientId }) {
        guard();
        state.vault ??= { nextSeq: 1, checkpoint: null };
        const op = { seq: state.vault.nextSeq++, cmd, args, at, clientId };
        state.ops.push(op);
        queueMicrotask(() => state.watchers.forEach((w) => w([op])));
        return op.seq;
      },
      async appendRestoreOp(checkpointId) {
        const seq = await remote.appendOp({ cmd: "__restore__", args: { checkpoint: checkpointId }, at: "2026-01-01 00:00:00", clientId: "upload" });
        state.vault.checkpoint = { id: checkpointId, seq };
        return seq;
      },
      watchOps(afterSeq, onOps) {
        const deliver = (ops) => { if (!offline) { state.reads += ops.length; onOps(structuredClone(ops), true); } };
        const watcher = (ops) => deliver(ops.filter((o) => o.seq > afterSeq));
        state.watchers.add(watcher);
        queueMicrotask(() => deliver(state.ops.filter((o) => o.seq > afterSeq)));
        return () => state.watchers.delete(watcher);
      },
      async fetchOps(afterSeq) { guard(); const ops = state.ops.filter((o) => o.seq > afterSeq); state.reads += ops.length; return structuredClone(ops); },
      async putCheckpoint(id, bytes) { guard(); state.checkpoints.set(id, new Uint8Array(bytes)); },
      async getCheckpoint(id) { guard(); return state.checkpoints.get(id) ?? null; },
      async publishCheckpoint(cp) {
        guard();
        if ((state.vault.checkpoint?.seq ?? 0) >= cp.seq) return false;
        state.vault.checkpoint = cp;
        return true;
      },
      async deleteCheckpoint(id) { state.checkpoints.delete(id); },
      setOffline(value) { offline = value; },
    };
    return remote;
  };
  return { state, connect };
}
