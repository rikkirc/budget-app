import { openUrl } from "@tauri-apps/plugin-opener";

// The web app serves its own copy of the manual (vite.config.js) and keeps it for offline use; the retired desktop
// app opens the online copy in the browser
export const MANUAL_FILE = "The-Vault-User-Manual.pdf";
const MANUAL_URL = `https://thevault-rc.web.app/${MANUAL_FILE}`;
const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Downloads the User's Manual (PDF). Rendered as a link in the web app, a button in the desktop app. */
export default function ManualLink({ style, children = "📄 Download the User's Manual (PDF)" }) {
  if (inTauri) {
    return <button onClick={() => openUrl(MANUAL_URL).catch(() => alert(`Open this link in your browser:\n${MANUAL_URL}`))} style={style}>{children}</button>;
  }
  return <a href={`/${MANUAL_FILE}`} download={MANUAL_FILE} style={{ display: "inline-block", textDecoration: "none", ...style }}>{children}</a>;
}
