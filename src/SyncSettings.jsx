import { useState } from "react";
import { IS_MOBILE, getSyncSettings, getLastServer, signIn, signUp, resetPassword, signOut, uploadLocalVault } from "./api";

// Your family account: sign in (or create an account with the family invite code) so this device syncs with your
// other devices. On the desktop you can also sign out and go back to "This computer only".
export default function SyncSettings({ profileName, onDone, onCancel }) {
  const current = getSyncSettings();
  const last = getLastServer();
  const [mode, setMode] = useState(last.email ? "signin" : "signup"); // signin | signup | reset | recovery
  const [url, setUrl] = useState(last.url);
  const [email, setEmail] = useState(last.email);
  const [password, setPassword] = useState("");
  const [inviteCode, setInviteCode] = useState("");
  const [recoveryInput, setRecoveryInput] = useState("");
  const [recoveryCode, setRecoveryCode] = useState("");
  const [savedCode, setSavedCode] = useState(false);
  const [busy, setBusy] = useState(false);

  const explain = (e) => e instanceof TypeError ? "Couldn't reach the server. Check the address and your internet connection." : e.message;

  // A desktop that was already in use brings its data along, so the phone starts from the same numbers
  const offerUpload = async () => {
    if (IS_MOBILE || !profileName) return;
    if (!window.confirm(`Upload this computer's "${profileName}" vault to your account so your phone can use it?\n\nPress Cancel if you'd rather use what is already in your account.`)) return;
    try {
      await uploadLocalVault(profileName, false);
    } catch (e) {
      if (e.status !== 409) { alert(explain(e)); return; }
      if (window.confirm(`Your account already has data for "${profileName}".\n\nPress OK to REPLACE it with this computer's copy.\nPress Cancel to keep your account's copy and use it here instead.`)) {
        try { await uploadLocalVault(profileName, true); } catch (e2) { alert(explain(e2)); }
      }
    }
  };

  const submit = async () => {
    if (!url.trim() || !email.trim()) { alert("Please enter the server address and your email."); return; }
    setBusy(true);
    try {
      if (mode === "signin") {
        await signIn(url, email, password);
      } else if (mode === "signup") {
        const result = await signUp(url, email, password, inviteCode);
        setRecoveryCode(result.recoveryCode);
      } else {
        const result = await resetPassword(url, email, recoveryInput, password);
        setRecoveryCode(result.recoveryCode);
      }
    } catch (e) {
      setBusy(false);
      alert(explain(e));
      return;
    }
    setPassword("");
    // After sign-up or a reset, show the new recovery code first; the upload offer follows on Continue
    if (mode === "signin") { await offerUpload(); setBusy(false); onDone(); }
    else { setBusy(false); setMode("recovery"); }
  };

  const doSignOut = async (everywhere) => {
    const warning = IS_MOBILE
      ? `Sign out of ${current.email} on this phone? Your data stays safe in your account.`
      : `Sign out of ${current.email} on this computer?\n\nThe app will go back to this computer's own vault file, which does NOT include changes made on your phone (or here) while you were signed in.`;
    if (!window.confirm(everywhere ? `Sign out of ${current.email} on EVERY device, including this one?` : warning)) return;
    setBusy(true);
    await signOut(everywhere);
    setBusy(false);
    onDone();
  };

  const inputStyle = { padding: "12px", width: "100%", boxSizing: "border-box", borderRadius: "5px", border: "1px solid #ccc", fontSize: "16px", marginBottom: "15px" };
  const labelStyle = { fontWeight: "bold", fontSize: "14px", color: "#374151" };
  const primary = { padding: "12px 24px", backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", opacity: busy ? 0.6 : 1 };
  const secondary = { padding: "12px 24px", backgroundColor: "#e5e7eb", color: "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" };
  const link = { background: "none", border: "none", color: "#2563eb", cursor: "pointer", fontSize: "14px", padding: 0, textDecoration: "underline" };
  const card = { backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)", maxWidth: "460px", width: "100%", boxSizing: "border-box" };

  if (mode === "recovery") {
    return (
      <div style={card}>
        <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>🔑 Save your recovery code</h2>
        <p style={{ color: "#6b7280", marginTop: 0, lineHeight: 1.5 }}>If you ever forget your password, this code is the <b>only</b> way back into your account. Write it down or save it in a safe place. It won't be shown again.</p>
        <div style={{ fontFamily: "monospace", fontSize: "22px", fontWeight: "bold", letterSpacing: "1px", textAlign: "center", padding: "15px", backgroundColor: "#fffbeb", border: "2px dashed #f59e0b", borderRadius: "8px", marginBottom: "15px", userSelect: "all", overflowWrap: "anywhere" }}>{recoveryCode}</div>
        <label style={{ display: "flex", gap: "8px", alignItems: "center", marginBottom: "15px", color: "#374151" }}>
          <input type="checkbox" checked={savedCode} onChange={e => setSavedCode(e.target.checked)} /> I've saved my recovery code
        </label>
        <button onClick={async () => { setBusy(true); await offerUpload(); setBusy(false); onDone(); }} disabled={!savedCode || busy} style={{ ...primary, opacity: savedCode && !busy ? 1 : 0.5 }}>Continue</button>
      </div>
    );
  }

  if (current) {
    return (
      <div style={card}>
        <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>☁️ Your account</h2>
        <p style={{ backgroundColor: "#ecfdf5", border: "1px solid #a7f3d0", padding: "10px", borderRadius: "5px", color: "#047857", fontSize: "14px", overflowWrap: "anywhere" }}>✅ Signed in as <b>{current.email}</b><br />on {current.url}</p>
        <p style={{ color: "#6b7280", lineHeight: 1.5 }}>Sign in with the same email on your other devices, and every change shows up on all of them right away.</p>
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={() => doSignOut(false)} disabled={busy} style={{ ...primary, backgroundColor: "#dc2626" }}>Sign out</button>
          {onCancel && <button onClick={onCancel} disabled={busy} style={secondary}>Close</button>}
        </div>
        <p style={{ marginBottom: 0 }}><button onClick={() => doSignOut(true)} disabled={busy} style={link}>Lost a phone? Sign out on every device</button></p>
      </div>
    );
  }

  const titles = { signin: "Sign in", signup: "Create your account", reset: "Reset your password" };
  return (
    <div style={card}>
      <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>☁️ {titles[mode]}</h2>
      <p style={{ color: "#6b7280", marginTop: 0, lineHeight: 1.5 }}>
        {mode === "signup" ? "Use the family invite code to create your own account. Your vaults stay private to you, and they sync between your phone and computer."
          : mode === "reset" ? "Enter the recovery code you saved when you created your account, and choose a new password."
          : "Sign in to sync your vaults between your phone and computer."}
      </p>
      {!IS_MOBILE && !profileName && mode !== "reset" && <p style={{ backgroundColor: "#fffbeb", border: "1px solid #fde68a", padding: "10px", borderRadius: "5px", color: "#92400e", fontSize: "14px", lineHeight: 1.5 }}>💡 Already using Vault on this computer? Unlock your vault first, then sign in from the top bar, so your data is uploaded for your phone.</p>}

      <label style={labelStyle}>Server address</label>
      <input type="url" inputMode="url" autoCapitalize="off" autoCorrect="off" placeholder="https://our-vault.duckdns.org" value={url} onChange={e => setUrl(e.target.value)} style={inputStyle} />
      <label style={labelStyle}>Email</label>
      <input type="email" autoCapitalize="off" autoCorrect="off" autoComplete="username" value={email} onChange={e => setEmail(e.target.value)} style={inputStyle} />
      {mode === "reset" && <>
        <label style={labelStyle}>Recovery code</label>
        <input type="text" autoCapitalize="characters" autoCorrect="off" placeholder="XXXXX-XXXXX-XXXXX-XXXXX" value={recoveryInput} onChange={e => setRecoveryInput(e.target.value)} style={inputStyle} />
      </>}
      <label style={labelStyle}>{mode === "signin" ? "Password" : "New password (at least 8 characters)"}</label>
      <input type="password" autoComplete={mode === "signin" ? "current-password" : "new-password"} value={password} onChange={e => setPassword(e.target.value)} style={inputStyle} />
      {mode === "signup" && <>
        <label style={labelStyle}>Family invite code</label>
        <input type="text" autoCapitalize="off" autoCorrect="off" placeholder="Ask whoever set up the server" value={inviteCode} onChange={e => setInviteCode(e.target.value)} style={inputStyle} />
      </>}

      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
        <button onClick={submit} disabled={busy} style={primary}>{busy ? "Please wait…" : mode === "signin" ? "Sign in" : mode === "signup" ? "Create account" : "Reset password"}</button>
        {onCancel && <button onClick={onCancel} disabled={busy} style={secondary}>Cancel</button>}
      </div>
      <p style={{ marginBottom: 0, fontSize: "14px", color: "#6b7280", display: "flex", gap: "15px", flexWrap: "wrap" }}>
        {mode !== "signin" && <button onClick={() => setMode("signin")} style={link}>I already have an account</button>}
        {mode !== "signup" && <button onClick={() => setMode("signup")} style={link}>Create an account</button>}
        {mode === "signin" && <button onClick={() => setMode("reset")} style={link}>Forgot password?</button>}
      </p>
    </div>
  );
}
