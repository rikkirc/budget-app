// One vault's shared, numbered list of changes, stored in Cloud Firestore:
//
//   users/{uid}/vaults/{vaultKey}                      { nextSeq, checkpoint: { id, seq } | null }
//   users/{uid}/vaults/{vaultKey}/ops/{0000000042}     { seq, cmd, args, at, clientId }   (never changed or deleted)
//   users/{uid}/vaults/{vaultKey}/checkpoints/{id}     { chunks, size }  + chunks/{000} { data: Bytes }
//
// Each change gets its number inside a transaction, so two devices can never take the same number; firestore.rules
// enforces the same thing on the server. Everything here fits Firebase's free Spark plan (no Cloud Storage needed).
import {
  Bytes, collection, deleteDoc, doc, getDoc, getDocs, onSnapshot, orderBy, query, runTransaction, serverTimestamp, where, writeBatch,
} from "firebase/firestore";
import { OfflineError } from "./engine.js";

// Firestore documents hold up to 1 MiB, so a checkpoint is stored in slices just under that
const CHUNK_BYTES = 900 * 1024;
const CHUNKS_PER_BATCH = 8;
const TIMEOUT_MS = 15000;

const seqId = (seq) => String(seq).padStart(10, "0");

const isOffline = (e) => e?.code === "unavailable" || e?.code === "deadline-exceeded" || /offline/i.test(e?.message ?? "");

// Firestore keeps retrying while offline; give up after a while and say so, rather than leave the screen waiting
async function online(work) {
  let timer;
  try {
    return await Promise.race([
      work(),
      new Promise((_, reject) => { timer = setTimeout(() => reject(new OfflineError("timed out")), TIMEOUT_MS); }),
    ]);
  } catch (e) {
    if (e instanceof OfflineError || isOffline(e)) throw new OfflineError(e.message);
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

// Firestore can't store `undefined`; commands only ever take plain JSON values
const plain = (value) => JSON.parse(JSON.stringify(value ?? {}));

export function firestoreRemote(db, uid, vaultKey) {
  const vaultRef = doc(db, "users", uid, "vaults", vaultKey);
  const opsRef = collection(vaultRef, "ops");
  const checkpointRef = (id) => doc(vaultRef, "checkpoints", id);
  const opsAfter = (seq) => query(opsRef, where("seq", ">", seq), orderBy("seq"));
  const vaultData = (snap) => ({ nextSeq: snap.data().nextSeq, checkpoint: snap.data().checkpoint ?? null });

  const append = (op, extra = () => ({})) => online(() => runTransaction(db, async (tx) => {
    const snap = await tx.get(vaultRef);
    if (!snap.exists()) throw new Error("ERROR: This vault hasn't been set up yet. Please log in again.");
    const seq = snap.data().nextSeq;
    tx.set(doc(opsRef, seqId(seq)), { seq, ...op });
    tx.update(vaultRef, { nextSeq: seq + 1, ...extra(seq) });
    return seq;
  }));

  return {
    async getVault() {
      const snap = await online(() => getDoc(vaultRef));
      return snap.exists() ? vaultData(snap) : null;
    },

    getOrCreateVault: () => online(() => runTransaction(db, async (tx) => {
      const snap = await tx.get(vaultRef);
      if (snap.exists()) return vaultData(snap);
      tx.set(vaultRef, { nextSeq: 1, checkpoint: null, createdAt: serverTimestamp() });
      return { nextSeq: 1, checkpoint: null };
    })),

    appendOp: ({ cmd, args, at, clientId }) => append({ cmd, args: plain(args), at, clientId }),

    appendRestoreOp: (checkpointId) => append(
      { cmd: "__restore__", args: { checkpoint: checkpointId }, at: "2000-01-01 00:00:00", clientId: "upload" },
      (seq) => ({ checkpoint: { id: checkpointId, seq } }),
    ),

    // Calls onOps(newChanges, live) for everything after afterSeq, then again for every new change.
    // `live` is false while Firestore is answering from its offline cache.
    watchOps(afterSeq, onOps, onError) {
      return onSnapshot(opsAfter(afterSeq), { includeMetadataChanges: true }, (snap) => {
        const ops = snap.docChanges().filter((c) => c.type === "added").map((c) => c.doc.data());
        onOps(ops, !snap.metadata.fromCache);
      }, onError);
    },

    async fetchOps(afterSeq) {
      const snap = await online(() => getDocs(opsAfter(afterSeq)));
      return snap.docs.map((d) => d.data());
    },

    async putCheckpoint(id, bytes) {
      const chunks = [];
      for (let i = 0; i < bytes.length; i += CHUNK_BYTES) chunks.push(bytes.slice(i, i + CHUNK_BYTES));
      for (let i = 0; i < chunks.length; i += CHUNKS_PER_BATCH) {
        const batch = writeBatch(db);
        chunks.slice(i, i + CHUNKS_PER_BATCH).forEach((chunk, j) => {
          batch.set(doc(checkpointRef(id), "chunks", String(i + j).padStart(3, "0")), { data: Bytes.fromUint8Array(chunk) });
        });
        await online(() => batch.commit());
      }
      // The header goes last, so nobody ever sees a half-uploaded checkpoint
      const header = writeBatch(db);
      header.set(checkpointRef(id), { chunks: chunks.length, size: bytes.length, createdAt: serverTimestamp() });
      await online(() => header.commit());
    },

    async getCheckpoint(id) {
      const header = await online(() => getDoc(checkpointRef(id)));
      if (!header.exists()) return null;
      const snap = await online(() => getDocs(collection(checkpointRef(id), "chunks")));
      if (snap.size !== header.data().chunks) return null;
      const out = new Uint8Array(header.data().size);
      let offset = 0;
      for (const chunk of snap.docs) { // document IDs 000, 001, … come back in order
        const part = chunk.data().data.toUint8Array();
        out.set(part, offset);
        offset += part.length;
      }
      return out;
    },

    publishCheckpoint: (checkpoint) => online(() => runTransaction(db, async (tx) => {
      const snap = await tx.get(vaultRef);
      if ((snap.data()?.checkpoint?.seq ?? 0) >= checkpoint.seq) return false;
      tx.update(vaultRef, { checkpoint });
      return true;
    })),

    async deleteCheckpoint(id) {
      const chunks = await getDocs(collection(checkpointRef(id), "chunks"));
      await deleteDoc(checkpointRef(id));
      await Promise.all(chunks.docs.map((c) => deleteDoc(c.ref)));
    },
  };
}
