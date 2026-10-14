import { useState } from "react";
import { IS_MOBILE, getSyncSettings, saveSyncSettings, testConnection, uploadLocalVault } from "./api";

// Connect this device to the Vault sync server (or, on the desktop, go back to "This computer only")
export default function SyncSettings({ profileName, onDone, onCancel }) {
  const current = getSyncSettings();
  const [url, setUrl] = useState(current?.url ?? "");
  const [token, setToken] = useState(current?.token ?? "");
  const [busy, setBusy] = useState(false);

  const connect = async () => {
    if (!url.trim() || !token.trim()) { alert("Please enter both the server address and the sync token."); return; }
    const settings = { url, token };
    setBusy(true);
    try {
      await testConnection(settings);
    } catch (e) {
      setBusy(false);
      alert(e instanceof TypeError ? "Couldn't reach the sync server. Check the address and your internet connection." : e.message);
      return;
    }

    // A desktop that was already in use brings its data along, so the phone starts from the same numbers
    if (!IS_MOBILE && profileName && !current) {
      if (window.confirm(`Upload this computer's "${profileName}" vault to the sync server so your phone can use it?\n\nPress Cancel if you'd rather use what is already on the server.`)) {
        try {
          await uploadLocalVault(settings, profileName, false);
        } catch (e) {
          if (e.status !== 409) { setBusy(false); alert(e.message); return; }
          const replace = window.confirm(`The sync server already has data for "${profileName}".\n\nPress OK to REPLACE the server's copy with this computer's copy.\nPress Cancel to keep the server's copy and use it here instead.`);
          if (replace) {
            try { await uploadLocalVault(settings, profileName, true); } catch (e2) { setBusy(false); alert(e2.message); return; }
          }
        }
      }
    }

    saveSyncSettings(settings);
    setBusy(false);
    onDone();
  };

  const disconnect = () => {
    const warning = IS_MOBILE
      ? "Disconnect this phone from the sync server? Your data stays safe on the server."
      : "Turn off sync on this computer?\n\nThe app will go back to this computer's own vault file, which does NOT include changes made on your phone (or here) since sync was turned on.";
    if (!window.confirm(warning)) return;
    saveSyncSettings(null);
    onDone();
  };

  const inputStyle = { padding: "12px", width: "100%", boxSizing: "border-box", borderRadius: "5px", border: "1px solid #ccc", fontSize: "16px", marginBottom: "15px" };

  return (
    <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)", maxWidth: "460px", width: "100%", boxSizing: "border-box" }}>
      <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>☁️ Sync</h2>
      <p style={{ color: "#6b7280", marginTop: 0, lineHeight: 1.5 }}>
        {IS_MOBILE
          ? "Connect this phone to your Vault sync server. Use the same address and token as your desktop app."
          : "Connect to your Vault sync server to keep this computer and your phone in sync. Every change shows up on the other device right away."}
      </p>
      {!IS_MOBILE && !profileName && !current && <p style={{ backgroundColor: "#fffbeb", border: "1px solid #fde68a", padding: "10px", borderRadius: "5px", color: "#92400e", fontSize: "14px", lineHeight: 1.5 }}>💡 Already using Vault on this computer? Unlock your vault first, then connect from the top bar, so your data is uploaded for your phone.</p>}
      {current && <p style={{ backgroundColor: "#ecfdf5", border: "1px solid #a7f3d0", padding: "10px", borderRadius: "5px", color: "#047857", fontSize: "14px", overflowWrap: "anywhere" }}>✅ Syncing with {current.url}</p>}

      <label style={{ fontWeight: "bold", fontSize: "14px", color: "#374151" }}>Server address</label>
      <input type="url" inputMode="url" autoCapitalize="off" autoCorrect="off" placeholder="https://my-vault.fly.dev" value={url} onChange={e => setUrl(e.target.value)} style={inputStyle} />
      <label style={{ fontWeight: "bold", fontSize: "14px", color: "#374151" }}>Sync token</label>
      <input type="password" autoCapitalize="off" autoCorrect="off" placeholder="The VAULT_TOKEN you set on the server" value={token} onChange={e => setToken(e.target.value)} style={inputStyle} />

      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
        <button onClick={connect} disabled={busy} style={{ padding: "12px 24px", backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", opacity: busy ? 0.6 : 1 }}>
          {busy ? "Connecting…" : current ? "Save" : "🔗 Connect"}
        </button>
        {current && <button onClick={disconnect} disabled={busy} style={{ padding: "12px 24px", backgroundColor: "#dc2626", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" }}>{IS_MOBILE ? "Disconnect" : "Turn off sync"}</button>}
        {onCancel && <button onClick={onCancel} disabled={busy} style={{ padding: "12px 24px", backgroundColor: "#e5e7eb", color: "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" }}>Cancel</button>}
      </div>
    </div>
  );
}
