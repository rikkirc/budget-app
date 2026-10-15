// The one door every screen uses to reach the vault.
//   Not signed in ("This computer only"): commands run on the local vault through the Rust backend.
//   Signed in to a family account: commands run on this device's synced copy of the vault; changes go through
//   Firebase first so every phone and computer stays identical (see src/sync/engine.js).
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { initializeApp } from "firebase/app";
import { connectAuthEmulator, getAuth, onAuthStateChanged } from "firebase/auth";
import { connectFirestoreEmulator, getFirestore } from "firebase/firestore";
import { firebaseConfig, isFirebaseConfigured } from "./firebaseConfig";
import * as account from "./sync/account";
import { createSyncEngine, uploadVault } from "./sync/engine";
import { firestoreRemote } from "./sync/firestoreRemote";
import { tauriReplica } from "./sync/tauriReplica";
import { bridgeReplica } from "./sync/bridgeReplica";
import { fromBase64 } from "./sync/base64";

// The phone app has no local vault: it always works through a family account
export const IS_MOBILE = ["android", "ios"].includes(import.meta.env.TAURI_ENV_PLATFORM);

// For automated tests only: the Firebase emulators, and a replica served over HTTP instead of by Tauri
const USE_EMULATOR = import.meta.env.VITE_FIREBASE_EMULATOR === "1";
const REPLICA_BRIDGE = import.meta.env.VITE_REPLICA_BRIDGE;

/** False until firebaseConfig.js has been filled in, in which case the app works exactly as before */
export const SYNC_AVAILABLE = isFirebaseConfigured || USE_EMULATOR;

// Lets each device tell its own changes apart from the ones made elsewhere
const CLIENT_ID = (globalThis.crypto?.randomUUID?.() ?? String(Math.random()).slice(2)).replaceAll("-", "");

let firebase = null;
function services() {
  if (!firebase) {
    const app = initializeApp(USE_EMULATOR ? { projectId: "demo-vault", apiKey: "demo-key", authDomain: "demo-vault.firebaseapp.com" } : firebaseConfig);
    const auth = getAuth(app);
    const db = getFirestore(app);
    if (USE_EMULATOR) {
      connectAuthEmulator(auth, "http://127.0.0.1:9099", { disableWarnings: true });
      connectFirestoreEmulator(db, "127.0.0.1", 8080);
    }
    firebase = { auth, db };
  }
  return firebase;
}

let user = null;     // the signed-in family account, or null
let engine = null;   // the synced vault that is open, if any
const watchers = new Set();

/** Calls back with { email } or null now and whenever the sign-in changes (also when a session ends elsewhere) */
export function onAccountChange(callback) {
  if (!SYNC_AVAILABLE) { callback(null); return () => {}; }
  return onAuthStateChanged(services().auth, (u) => {
    user = u;
    if (!u) { engine?.close(); engine = null; }
    callback(u ? { email: u.email } : null);
  });
}

// Remember the account straight away, so an upload right after signing in never waits on Firebase's sign-in event
export async function signUp(email, password, inviteCode) { user = await account.signUp(services(), email, password, inviteCode); }
export async function signIn(email, password) { user = await account.signIn(services(), email, password); }
/** Whether commands currently go to a synced vault */
export const isSignedIn = () => !!user;
export const sendPasswordReset = (email) => account.sendReset(services(), email);
export const changePassword = (current, next) => account.changePassword(services(), current, next);
export async function signOut() {
  engine?.close();
  engine = null;
  await account.signOut(services());
}

async function openSyncedVault(profile) {
  const key = account.vaultKey(profile);
  engine?.close();
  const replica = REPLICA_BRIDGE ? bridgeReplica(REPLICA_BRIDGE, CLIENT_ID, user.uid, key) : tauriReplica(user.uid, profile);
  engine = createSyncEngine({
    remote: firestoreRemote(services().db, user.uid, key),
    replica,
    clientId: CLIENT_ID,
    onChange: () => watchers.forEach((w) => w.onChange()),
    onStatus: (live) => watchers.forEach((w) => w.onStatus(live)),
  });
  const { offline } = await engine.open();
  return offline
    ? "OFFLINE: Couldn't reach the sync service. You're seeing the last data synced to this device; changes can't be saved until you reconnect."
    : `SUCCESS: Logged in as ${profile}`;
}

/** Drop-in replacement for Tauri's invoke(): same command names, same camelCase arguments, same results */
export async function invoke(cmd, args = {}) {
  if (!user) {
    if (cmd === "login_user") return tauriInvoke("login_user", args);
    return tauriInvoke("call", { cmd, args });
  }
  if (cmd === "login_user") return openSyncedVault(args.profile);
  if (!engine) throw "ERROR: No vault is unlocked.";
  return engine.call(cmd, args);
}

/** Desktop only: copies this computer's unlocked local vault into your account. replace=false never overwrites data. */
export async function uploadLocalVault(profile) {
  const bytes = fromBase64(await tauriInvoke("export_vault"));
  const remote = firestoreRemote(services().db, user.uid, account.vaultKey(profile));
  return {
    upload: (replace) => uploadVault(remote, bytes, replace),
  };
}

/**
 * Listens for changes made on your other devices. onChange() fires when the open vault changed elsewhere;
 * onStatus(true/false) reports whether the live link to Firebase is up. Returns a function that stops listening.
 */
export function subscribeToChanges(_profileName, { onChange, onStatus }) {
  const watcher = { onChange, onStatus };
  watchers.add(watcher);
  return () => watchers.delete(watcher);
}
