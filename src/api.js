// The one door every screen uses to reach the vault.
//   Trying the demo (web app): commands run on a made-up vault in this browser tab only (src/sync/demo.js).
//   Not signed in ("This computer only"): commands run on the local vault through the Rust backend.
//   Signed in to a family account: commands run on this device's synced copy of the vault; changes go through
//   Firebase first so every phone and computer stays identical (see src/sync/engine.js).
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { initializeApp } from "firebase/app";
import { browserLocalPersistence, browserSessionPersistence, connectAuthEmulator, getAuth, onAuthStateChanged, setPersistence } from "firebase/auth";
import { connectFirestoreEmulator, getFirestore } from "firebase/firestore";
import { firebaseConfig, isFirebaseConfigured } from "./firebaseConfig";
import * as account from "./sync/account";
import { createSyncEngine, uploadVault } from "./sync/engine";
import { firestoreRemote } from "./sync/firestoreRemote";
// This device's copy of a synced vault: run by the Rust backend in the desktop and Android apps, or by vault-core
// compiled to WebAssembly in the web app. vite.config.js picks the right one for each build.
import { deviceReplica, demoReplica } from "@device-replica";
import { DEMO_PROFILE, openDemoVault } from "./sync/demo";
import { bridgeReplica } from "./sync/bridgeReplica";
import { fromBase64 } from "./sync/base64";

/** The web app (PWA), built with `npm run build:web` */
export const IS_WEB = import.meta.env.MODE === "web";
// The phone and web apps have no local vault: they always work through a family account
export const SYNC_ONLY = IS_WEB || ["android", "ios"].includes(import.meta.env.TAURI_ENV_PLATFORM);

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
let joining = null;  // a sign-up still saving its family membership
let engine = null;   // the synced vault that is open, if any
let demo = null;     // the demo vault, while trying the demo
const watchers = new Set();

/** Calls back with { email } or null now and whenever the sign-in changes (also when a session ends elsewhere) */
export function onAccountChange(callback) {
  if (!SYNC_AVAILABLE) { callback(null); return () => {}; }
  return onAuthStateChanged(services().auth, async (u) => {
    // A new account exists a moment before its membership is saved. Until then it can't open a vault, so don't
    // report it yet; if the invite code was wrong, the account is removed and a signed-out change follows instead.
    if (u && joining) {
      try { await joining; } catch { return; }
    }
    user = u;
    if (!u) { engine?.close(); engine = null; }
    callback(u ? { email: u.email } : null);
  });
}

// Remember the account straight away, so an upload right after signing in never waits on Firebase's sign-in event
// remember = false: signed in only until the app (or browser) is closed. Otherwise the sign-in lasts until you sign out.
const keepSignedIn = (remember) => setPersistence(services().auth, remember ? browserLocalPersistence : browserSessionPersistence);

export async function signUp(email, password, inviteCode, remember = true) {
  await keepSignedIn(remember);
  joining = account.signUp(services(), email, password, inviteCode);
  try { user = await joining; } finally { joining = null; }
}
export async function signIn(email, password, remember = true) {
  await keepSignedIn(remember);
  user = await account.signIn(services(), email, password);
}
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
  const replica = REPLICA_BRIDGE ? bridgeReplica(REPLICA_BRIDGE, CLIENT_ID, user.uid, key) : deviceReplica(user.uid, profile);
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

/** Whether this app offers "Try the demo" (the web app does) */
export const DEMO_AVAILABLE = !!demoReplica;

/** Opens a fresh demo vault: no account, nothing sent to Firebase, nothing saved. Returns its profile name. */
export async function startDemo() {
  demo = null;
  demo = await openDemoVault(demoReplica());
  return DEMO_PROFILE;
}
/** Throws the demo vault away */
export function endDemo() {
  demo?.close();
  demo = null;
}
/** Whether commands currently go to the demo vault */
export const isDemo = () => !!demo;

/** Drop-in replacement for Tauri's invoke(): same command names, same camelCase arguments, same results */
export async function invoke(cmd, args = {}) {
  if (demo) {
    if (cmd === "login_user") return `SUCCESS: Logged in as ${DEMO_PROFILE}`;
    return demo.call(cmd, args);
  }
  if (!user) {
    if (SYNC_ONLY) throw "ERROR: Please sign in to your family account first.";
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
