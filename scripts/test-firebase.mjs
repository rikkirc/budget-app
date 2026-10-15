// End-to-end tests against the Firebase emulators: the security rules, family sign-up, and the sync engine running
// over real Firestore with real vault-core replicas. Run with:  npm run test:firebase
// (needs Java; the first run downloads the Firestore emulator)
import test from "node:test";
import assert from "node:assert/strict";
import { join } from "node:path";
import { initializeApp, deleteApp } from "firebase/app";
import { getAuth, connectAuthEmulator } from "firebase/auth";
import {
  getFirestore, connectFirestoreEmulator, doc, getDoc, setDoc, updateDoc, deleteDoc, collection, getDocs, terminate,
} from "firebase/firestore";
import { signUp, signIn, vaultKey } from "../src/sync/account.js";
import { firestoreRemote } from "../src/sync/firestoreRemote.js";
import { createSyncEngine, uploadVault } from "../src/sync/engine.js";
import { cliReplica, scratchDir } from "../src/sync/testSupport.js";

const PROJECT = "demo-vault";
const INVITE = "adobo-sinigang-2026";
const settle = (ms = 300) => new Promise((r) => setTimeout(r, ms));
// Waits for something that happens in the background (like a checkpoint upload), instead of guessing how long it takes
async function eventually(check, what, ms = 15000) {
  for (const end = Date.now() + ms; Date.now() < end; await settle(200)) if (await check()) return;
  assert.fail(`timed out waiting for ${what}`);
}

// Admin access to the emulator (bypasses the rules) for seeding and cleanup
const emulator = (path, init) => fetch(`http://127.0.0.1:8080/v1/projects/${PROJECT}/databases/(default)/documents${path}`, { ...init, headers: { Authorization: "Bearer owner", "Content-Type": "application/json" } });
await emulator(`/invites/${INVITE}`, { method: "PATCH", body: JSON.stringify({ fields: {} }) });

let apps = 0;
const opened = [];
/** A separate Firebase "device" (its own sign-in) */
function device() {
  const app = initializeApp({ projectId: PROJECT, apiKey: "demo-key", authDomain: `${PROJECT}.firebaseapp.com` }, `device${apps++}`);
  const auth = getAuth(app);
  connectAuthEmulator(auth, "http://127.0.0.1:9099", { disableWarnings: true });
  const db = getFirestore(app);
  connectFirestoreEmulator(db, "127.0.0.1", 8080);
  opened.push({ app, db });
  return { auth, db };
}
test.after(async () => { for (const { app, db } of opened) { await terminate(db); await deleteApp(app); } });

const email = (name) => `${name}.${Date.now()}${Math.random().toString(36).slice(2, 6)}@example.com`;
const denied = (promise) => assert.rejects(promise, (e) => e.code === "permission-denied");

test("sign-up needs the family invite code", async () => {
  const d = device();
  await assert.rejects(signUp(d, email("stranger"), "a good password", "guessed-code"), /invite code/);
  assert.equal(d.auth.currentUser, null, "the half-made account is removed");

  const user = await signUp(d, email("juan"), "a good password", INVITE);
  assert.ok((await getDoc(doc(d.db, "members", user.uid))).exists());
});

test("accounts without the invite code can't store anything, and nobody can read invite codes", async () => {
  const d = device();
  const { createUserWithEmailAndPassword } = await import("firebase/auth");
  const cred = await createUserWithEmailAndPassword(d.auth, email("sneaky"), "a good password");
  await denied(setDoc(doc(d.db, "users", cred.user.uid, "vaults", "x"), { nextSeq: 1, checkpoint: null }));
  await denied(getDoc(doc(d.db, "invites", INVITE)));
  await denied(getDocs(collection(d.db, "invites")));
});

test("family members never see each other's data", async () => {
  const juan = device();
  const maria = device();
  const juanUser = await signUp(juan, email("juan"), "a good password", INVITE);
  await signUp(maria, email("maria"), "a good password", INVITE);

  const remote = firestoreRemote(juan.db, juanUser.uid, vaultKey("Juan Dela Cruz"));
  await remote.getOrCreateVault();
  await remote.appendOp({ cmd: "add_account", args: { name: "GCash" }, at: "2026-10-15 08:00:00", clientId: "x" });

  await denied(getDoc(doc(maria.db, "users", juanUser.uid, "vaults", "juan_dela_cruz")));
  await denied(getDocs(collection(maria.db, "users", juanUser.uid, "vaults", "juan_dela_cruz", "ops")));
  await denied(setDoc(doc(maria.db, "users", juanUser.uid, "vaults", "other"), { nextSeq: 1, checkpoint: null }));
});

test("the list of changes is append-only and strictly numbered", async () => {
  const d = device();
  const user = await signUp(d, email("kuya"), "a good password", INVITE);
  const remote = firestoreRemote(d.db, user.uid, "kuya");
  await remote.getOrCreateVault();
  const seq = await remote.appendOp({ cmd: "add_account", args: { name: "Wallet" }, at: "2026-10-15 08:00:00", clientId: "x" });
  assert.equal(seq, 1);

  const vaultPath = ["users", user.uid, "vaults", "kuya"];
  const op1 = doc(d.db, ...vaultPath, "ops", "0000000001");
  await denied(updateDoc(op1, { cmd: "delete_account" }));
  await denied(deleteDoc(op1));
  // Writing a change without taking its number (or skipping ahead) is refused
  await denied(setDoc(doc(d.db, ...vaultPath, "ops", "0000000002"), { seq: 2, cmd: "add_account", args: {}, at: "2026-10-15 08:00:00", clientId: "x" }));
  await denied(updateDoc(doc(d.db, ...vaultPath), { nextSeq: 5 }));
  assert.equal(await remote.appendOp({ cmd: "add_account", args: { name: "Savings" }, at: "2026-10-15 08:00:00", clientId: "x" }), 2);
});

test("two devices signed in to one account stay identical through Firestore", async (t) => {
  const dir = scratchDir();
  const phoneApp = device();
  const desktopApp = device();
  const address = email("ate");
  const user = await signUp(phoneApp, address, "a good password", INVITE);
  await signIn(desktopApp, address, "a good password");

  // The desktop brings its existing data along
  const local = cliReplica(join(dir, "local.db"));
  t.after(() => local.close());
  await local.open();
  await local.apply([
    { seq: 1, cmd: "add_account", args: { name: "BPI" }, at: "2025-05-01 09:00:00" },
    { seq: 2, cmd: "log_income", args: { accountId: 1, amount: 20000, description: "Salary" }, at: "2025-05-15 09:00:00" },
  ]);
  const key = vaultKey("Ate Dela Cruz");
  await uploadVault(firestoreRemote(desktopApp.db, user.uid, key), await local.snapshot(), false);

  let phoneHeard = 0;
  const make = (fb, name, onChange = () => {}) => {
    const replica = cliReplica(join(dir, `${name}.db`));
    const engine = createSyncEngine({ remote: firestoreRemote(fb.db, user.uid, key), replica, clientId: name, onChange, checkpointEvery: 5 });
    // Always stop the replica processes, even if an assertion fails, or the test run never exits
    t.after(() => { engine.close(); replica.close(); });
    return { engine, close: () => {} };
  };
  const desktop = make(desktopApp, "desktop");
  const phone = make(phoneApp, "phone", () => phoneHeard++);
  await desktop.engine.open();
  await phone.engine.open();
  await settle();

  await phone.engine.call("add_account", { name: "GCash" });
  for (let i = 0; i < 6; i++) await desktop.engine.call("transfer_money", { fromAccountId: 1, toAccountId: 2, amount: 100.25, description: `Load ${i}` });
  assert.match(await phone.engine.call("transfer_money", { fromAccountId: 2, toAccountId: 1, amount: 99999, description: "Too much" }), /^ERROR/);
  const balances = async (e) => JSON.stringify(await e.call("get_accounts"));
  await eventually(async () => (await balances(phone.engine)) === (await balances(desktop.engine)), "both devices to agree");
  assert.match(await balances(phone.engine), /"balance":19398.5.*"balance":601.5/);
  assert.ok(phoneHeard > 0, "the phone was told about the desktop's changes");

  // A checkpoint was saved, and a brand-new device starts from it
  const vaultRef = doc(phoneApp.db, "users", user.uid, "vaults", key);
  await eventually(async () => (await getDoc(vaultRef)).data().checkpoint?.id.startsWith("s"), "a regular checkpoint");
  const tabletApp = device();
  await signIn(tabletApp, address, "a good password");
  const tablet = make(tabletApp, "tablet");
  await tablet.engine.open();
  await eventually(async () => (await balances(tablet.engine)) === (await balances(phone.engine)), "the tablet to catch up");
});

test("large checkpoints are split across documents and come back intact", async () => {
  const d = device();
  const user = await signUp(d, email("lola"), "a good password", INVITE);
  const remote = firestoreRemote(d.db, user.uid, "lola");
  await remote.getOrCreateVault();
  const bytes = new Uint8Array(2_500_000).map((_, i) => (i * 7919) % 251);
  await remote.putCheckpoint("big", bytes);
  assert.deepEqual(await remote.getCheckpoint("big"), bytes);
  await remote.deleteCheckpoint("big");
  assert.equal(await remote.getCheckpoint("big"), null);
});
