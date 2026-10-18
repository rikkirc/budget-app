// Small things this device remembers between visits (kept in the browser's localStorage, never synced):
// the profile to open automatically, and whether the Terms & Privacy Notice was accepted.
const PROFILE_KEY = "vault.autoOpenProfile";
const TERMS_KEY = "vault.termsAccepted";
export const TERMS_VERSION = "2026-10";

function read(key) {
  try { return JSON.parse(localStorage.getItem(key)); } catch { return null; }
}
function write(key, value) {
  try { value === null ? localStorage.removeItem(key) : localStorage.setItem(key, JSON.stringify(value)); } catch { /* storage unavailable: just ask again next time */ }
}

/** The profile to open automatically for this account (email, or null for "This computer only"), if one was saved */
export function rememberedProfile(email) {
  const saved = read(PROFILE_KEY);
  return saved && saved.email === (email ?? null) ? saved.profile : null;
}
export const rememberProfile = (profile, email) => write(PROFILE_KEY, { profile, email: email ?? null });
export const forgetProfile = () => write(PROFILE_KEY, null);

export const termsAccepted = () => read(TERMS_KEY)?.version === TERMS_VERSION;
export const acceptTerms = () => write(TERMS_KEY, { version: TERMS_VERSION, at: new Date().toISOString() });
