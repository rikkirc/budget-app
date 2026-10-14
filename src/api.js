// The one door every screen uses to reach the vault. On "This computer only" it calls the local Rust backend; once
// sync is set up it calls the Vault sync server, so the desktop and the phone always work on the same data.
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

const SYNC_KEY = "vault.sync";
const CACHE_PREFIX = "vault.cache.";

// The phone app has no local vault: it always works through the sync server
export const IS_MOBILE = ["android", "ios"].includes(import.meta.env.TAURI_ENV_PLATFORM);

// Lets the server tell this device apart from the others, so it only refreshes for changes made elsewhere
const CLIENT_ID = (globalThis.crypto?.randomUUID?.() ?? String(Math.random()).slice(2));

let profile = "";

// Browser storage can be unavailable (private modes, cleared data); the app must still work without it
function readStorage(key) { try { return localStorage.getItem(key); } catch { return null; } }
function writeStorage(key, value) { try { value === null ? localStorage.removeItem(key) : localStorage.setItem(key, value); } catch { /* not fatal */ } }

export function getSyncSettings() {
  try { const s = JSON.parse(readStorage(SYNC_KEY)); return s?.url && s?.token ? s : null; } catch { return null; }
}
export function saveSyncSettings(settings) {
  writeStorage(SYNC_KEY, settings ? JSON.stringify({ url: normalizeUrl(settings.url), token: settings.token.trim() }) : null);
}
export function normalizeUrl(url) {
  let u = url.trim().replace(/\/+$/, "");
  if (u && !/^https?:\/\//i.test(u)) u = `https://${u}`;
  return u;
}

// Same rule the server uses to name a vault, so change notices can be matched to the vault that is open
export const profileKey = (name) => name.toLowerCase().replaceAll(" ", "_");

async function request(settings, method, path, body, isJson = true) {
  const res = await fetch(`${settings.url}${path}`, {
    method,
    headers: { Authorization: `Bearer ${settings.token}`, ...(isJson && body !== undefined ? { "Content-Type": "application/json" } : {}) },
    body: body === undefined ? undefined : isJson ? JSON.stringify(body) : body,
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw Object.assign(new Error(data.error || `ERROR: The sync server answered ${res.status}`), { status: res.status });
  return data.result;
}

// A network failure (no signal, server down) rather than the server saying no
const isOffline = (e) => e instanceof TypeError;
// A dropped connection can't tell us whether the server got the change, so never invite a blind retry (it could pay a bill twice)
const OFFLINE_MESSAGE = "ERROR: Couldn't reach the sync server, so this change may not have been saved. Check your connection, then press Refresh to see what was saved before trying again.";

function cacheKey(cmd, args) { return `${CACHE_PREFIX}${profileKey(profile)}.${cmd}.${JSON.stringify(args)}`; }

/** Drop-in replacement for Tauri's invoke(): same command names, same camelCase arguments, same results */
export async function invoke(cmd, args = {}) {
  const sync = getSyncSettings();
  if (cmd === "login_user") profile = args.profile;

  if (!sync) {
    if (cmd === "login_user") return tauriInvoke("login_user", args);
    return tauriInvoke("call", { cmd, args });
  }

  if (cmd === "login_user") {
    try {
      return await request(sync, "POST", "/api/login", { profile: args.profile });
    } catch (e) {
      if (!isOffline(e)) throw e.message;
      return "OFFLINE: Couldn't reach the sync server. You're seeing the last data synced to this device; changes can't be saved until you reconnect.";
    }
  }
  const isRead = cmd.startsWith("get_");
  try {
    const result = await request(sync, "POST", "/api/call", { profile, cmd, args, clientId: CLIENT_ID });
    // Keep the latest copy of every screen's data so the app can still show it without a connection
    if (isRead) writeStorage(cacheKey(cmd, args), JSON.stringify(result));
    return result;
  } catch (e) {
    if (!isOffline(e)) throw e.message;
    if (isRead) {
      const cached = readStorage(cacheKey(cmd, args));
      if (cached !== null) return JSON.parse(cached);
      throw OFFLINE_MESSAGE;
    }
    // Screens already show a command's text result, so the offline warning appears exactly where "SUCCESS" would
    return OFFLINE_MESSAGE;
  }
}

/** Checks the address and token before switching this device over to sync */
export async function testConnection(settings) {
  await request({ url: normalizeUrl(settings.url), token: settings.token.trim() }, "GET", "/api/ping");
}

/** Desktop only: copies this computer's unlocked vault up to the server. replace=false refuses to overwrite server data. */
export async function uploadLocalVault(settings, profileName, replace) {
  const bytes = await tauriInvoke("export_vault");
  const query = `?clientId=${encodeURIComponent(CLIENT_ID)}${replace ? "&replace=true" : ""}`;
  return request({ url: normalizeUrl(settings.url), token: settings.token.trim() }, "PUT", `/api/vault/${encodeURIComponent(profileName)}${query}`, bytes, false);
}

/**
 * Listens for changes made on other devices. onChange() fires when this vault changed elsewhere (or after a
 * reconnect, when changes may have been missed); onStatus(true/false) reports whether the live link is up.
 * Returns a function that stops listening.
 */
export function subscribeToChanges(profileName, { onChange, onStatus }) {
  const sync = getSyncSettings();
  if (!sync) return () => {};
  let socket = null, retryTimer = null, stopped = false, delay = 1000, wasConnected = false;
  const key = profileKey(profileName);

  const connect = () => {
    socket = new WebSocket(`${sync.url.replace(/^http/i, "ws")}/api/events?token=${encodeURIComponent(sync.token)}`);
    socket.onopen = () => {
      onStatus(true);
      delay = 1000;
      if (wasConnected) onChange(); // anything could have changed while the link was down
      wasConnected = true;
    };
    socket.onmessage = (msg) => {
      try {
        const event = JSON.parse(msg.data);
        if (event.type === "changed" && event.clientId !== CLIENT_ID && (event.profile === null || event.profile === key)) onChange();
      } catch { /* ignore anything that isn't a change notice */ }
    };
    socket.onclose = () => {
      onStatus(false);
      if (stopped) return;
      retryTimer = setTimeout(connect, delay);
      delay = Math.min(delay * 2, 30000);
    };
  };
  connect();
  return () => { stopped = true; clearTimeout(retryTimer); socket?.close(); };
}
