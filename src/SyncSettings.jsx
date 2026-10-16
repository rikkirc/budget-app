import { useState } from "react";
import { SYNC_ONLY, SYNC_AVAILABLE, signIn, signUp, sendPasswordReset, changePassword, signOut, uploadLocalVault } from "./api";

// Your family account: sign in (or create an account with the family invite code) so this device syncs with your
// other devices. On the desktop you can also sign out and go back to "This computer only".
export default function SyncSettings({ account, profileName, beforeSignOut, onDone, onCancel }) {
  const [mode, setMode] = useState("signin"); // signin | signup | reset | password
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [inviteCode, setInviteCode] = useState("");
  const [busy, setBusy] = useState(false);

  const message = (e) => e?.message ?? String(e);

  // A desktop that was already in use brings its data along, so the phone starts from the same numbers
  const offerUpload = async () => {
    if (SYNC_ONLY || !profileName) return;
    if (!window.confirm(`Upload this computer's "${profileName}" vault to your account so your phone can use it?\n\nPress Cancel if you'd rather use what is already in your account.`)) return;
    try {
      const { upload } = await uploadLocalVault(profileName);
      try {
        await upload(false);
      } catch (e) {
        if (e.status !== 409) throw e;
        if (window.confirm(`Your account already has data for "${profileName}".\n\nPress OK to REPLACE it with this computer's copy (on every device).\nPress Cancel to keep your account's copy and use it here instead.`)) await upload(true);
      }
    } catch (e) {
      alert(`Your vault was not uploaded: ${message(e)}`);
    }
  };

  const run = async (work) => {
    setBusy(true);
    try { await work(); } catch (e) { alert(message(e)); } finally { setBusy(false); }
  };

  const submit = () => run(async () => {
    if (mode === "reset") {
      await sendPasswordReset(email);
      alert(`If ${email.trim()} has an account, a password-reset link is on its way. Check your inbox (and spam folder).`);
      setMode("signin");
      return;
    }
    if (mode === "signup") await signUp(email, password, inviteCode);
    else await signIn(email, password);
    setPassword("");
    await offerUpload();
    onDone();
  });

  const doChangePassword = () => run(async () => {
    await changePassword(password, newPassword);
    setPassword(""); setNewPassword("");
    alert("Password changed. Your other devices will be signed out and need the new password.");
    setMode("signin");
  });

  const doSignOut = () => {
    const warning = SYNC_ONLY
      ? `Sign out of ${account.email} on this device? Your data stays safe in your account.`
      : `Sign out of ${account.email} on this computer?\n\nThe app will go back to this computer's own vault file, which does NOT include changes made on your phone (or here) while you were signed in.`;
    if (!window.confirm(warning)) return;
    run(async () => {
      beforeSignOut?.();
      await signOut();
      onDone({ signedOut: true });
    });
  };

  const inputStyle = { padding: "12px", width: "100%", boxSizing: "border-box", borderRadius: "5px", border: "1px solid #ccc", fontSize: "16px", marginBottom: "15px" };
  const labelStyle = { fontWeight: "bold", fontSize: "14px", color: "#374151" };
  const primary = { padding: "12px 24px", backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", opacity: busy ? 0.6 : 1 };
  const secondary = { padding: "12px 24px", backgroundColor: "#e5e7eb", color: "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" };
  const link = { background: "none", border: "none", color: "#2563eb", cursor: "pointer", fontSize: "14px", padding: 0, textDecoration: "underline" };
  const card = { backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)", maxWidth: "460px", width: "100%", boxSizing: "border-box" };

  if (!SYNC_AVAILABLE) {
    return (
      <div style={card}>
        <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>☁️ Sync isn't set up yet</h2>
        <p style={{ color: "#6b7280", lineHeight: 1.5 }}>This copy of Vault was built without a Firebase project, so it can't sync. Whoever builds the app needs to fill in <code>src/firebaseConfig.js</code> (see docs/SYNC_AND_MOBILE.md).</p>
        {onCancel && <button onClick={onCancel} style={secondary}>Close</button>}
      </div>
    );
  }

  if (account && mode === "password") {
    return (
      <div style={card}>
        <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>🔑 Change password</h2>
        <p style={{ color: "#6b7280", marginTop: 0, lineHeight: 1.5 }}>Your other devices will be signed out and need the new password. Do this if you lose a phone.</p>
        <label style={labelStyle}>Current password</label>
        <input type="password" autoComplete="current-password" value={password} onChange={e => setPassword(e.target.value)} style={inputStyle} />
        <label style={labelStyle}>New password (at least 8 characters)</label>
        <input type="password" autoComplete="new-password" value={newPassword} onChange={e => setNewPassword(e.target.value)} style={inputStyle} />
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={doChangePassword} disabled={busy} style={primary}>{busy ? "Please wait…" : "Change password"}</button>
          <button onClick={() => setMode("signin")} disabled={busy} style={secondary}>Back</button>
        </div>
      </div>
    );
  }

  if (account) {
    return (
      <div style={card}>
        <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>☁️ Your account</h2>
        <p style={{ backgroundColor: "#ecfdf5", border: "1px solid #a7f3d0", padding: "10px", borderRadius: "5px", color: "#047857", fontSize: "14px", overflowWrap: "anywhere" }}>✅ Signed in as <b>{account.email}</b></p>
        <p style={{ color: "#6b7280", lineHeight: 1.5 }}>Sign in with the same email on your other devices, and every change shows up on all of them right away.</p>
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={doSignOut} disabled={busy} style={{ ...primary, backgroundColor: "#dc2626" }}>Sign out</button>
          {onCancel && <button onClick={onCancel} disabled={busy} style={secondary}>Close</button>}
        </div>
        <p style={{ marginBottom: 0 }}><button onClick={() => setMode("password")} style={link}>Change password (lost a phone? this signs it out)</button></p>
      </div>
    );
  }

  const titles = { signin: "Sign in", signup: "Create your account", reset: "Reset your password" };
  return (
    <div style={card}>
      <h2 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>☁️ {titles[mode]}</h2>
      <p style={{ color: "#6b7280", marginTop: 0, lineHeight: 1.5 }}>
        {mode === "signup" ? "Use the family invite code to create your own account. Your vaults stay private to you, and they sync between your phone and computer."
          : mode === "reset" ? "Enter your account's email and we'll send you a link to choose a new password."
          : "Sign in to sync your vaults between your phone and computer."}
      </p>
      {!SYNC_ONLY && !profileName && mode !== "reset" && <p style={{ backgroundColor: "#fffbeb", border: "1px solid #fde68a", padding: "10px", borderRadius: "5px", color: "#92400e", fontSize: "14px", lineHeight: 1.5 }}>💡 Already using Vault on this computer? Unlock your vault first, then sign in from the top bar, so your data is uploaded for your phone.</p>}

      <label style={labelStyle}>Email</label>
      <input type="email" autoCapitalize="off" autoCorrect="off" autoComplete="username" value={email} onChange={e => setEmail(e.target.value)} style={inputStyle} />
      {mode !== "reset" && <>
        <label style={labelStyle}>{mode === "signin" ? "Password" : "Password (at least 8 characters)"}</label>
        <input type="password" autoComplete={mode === "signin" ? "current-password" : "new-password"} value={password} onChange={e => setPassword(e.target.value)} style={inputStyle} />
      </>}
      {mode === "signup" && <>
        <label style={labelStyle}>Family invite code</label>
        <input type="text" autoCapitalize="off" autoCorrect="off" placeholder="Ask whoever set up Vault's sync" value={inviteCode} onChange={e => setInviteCode(e.target.value)} style={inputStyle} />
      </>}

      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
        <button onClick={submit} disabled={busy} style={primary}>{busy ? "Please wait…" : mode === "signin" ? "Sign in" : mode === "signup" ? "Create account" : "Email me a reset link"}</button>
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
