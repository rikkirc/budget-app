import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SYNC_ONLY } from "./api";
import { termsAccepted, acceptTerms } from "./preferences";

// The online copy of the manual, for the desktop app (the web app serves its own copy, see vite.config.js)
const MANUAL_URL = "https://thevault-rc.web.app/The-Vault-User-Manual.pdf";
const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// Shown before signing in or unlocking: what The Vault is, how to start, the manual, and the Terms & Privacy Notice
export default function Welcome({ onProceed }) {
  const alreadyAccepted = termsAccepted();
  const [agreed, setAgreed] = useState(alreadyAccepted);
  const [showTerms, setShowTerms] = useState(!alreadyAccepted);

  const proceed = () => {
    if (!agreed) return;
    acceptTerms();
    onProceed();
  };

  const h3 = { margin: "22px 0 8px 0", color: "#1f2937", fontSize: "17px" };
  return (
    <div style={{ display: "flex", justifyContent: "center", minHeight: "100vh", backgroundColor: "#f3f4f6", padding: "20px", boxSizing: "border-box", fontFamily: "Arial, sans-serif" }}>
      <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)", maxWidth: "620px", width: "100%", boxSizing: "border-box", alignSelf: "flex-start" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "15px" }}>
          <img src="/icons/icon-192.png" alt="" width="64" height="64" style={{ borderRadius: "14px", flexShrink: 0 }} />
          <div>
            <h1 style={{ margin: 0, color: "#1e3a8a", fontSize: "30px" }}>The Vault</h1>
            <div style={{ color: "#6b7280", fontSize: "14px" }}>by RCbreezy ©</div>
          </div>
        </div>

        <p style={{ color: "#374151", lineHeight: 1.6, marginTop: "20px" }}>
          A budgeting app for ordinary paycheck to paycheck people. The Vault plans every bill and loan payment against the paycheck
          that will cover it, tracks the money in each of your accounts, and keeps your phone and computer in step.
        </p>

        <h3 style={h3}>How to get started</h3>
        <ol style={{ color: "#374151", lineHeight: 1.6, paddingLeft: "22px", margin: 0 }}>
          <li>{SYNC_ONLY ? "Sign in with your email, or create an account with the invite code." : "Unlock a profile on this computer, or sign in to your family account to sync with your phone."}</li>
          <li>Open a <b>profile</b>. One account can hold several profiles, each a separate budget (for example "Personal" and "Household").</li>
          <li>Add your money accounts (GCash, BPI, cash…) in <b>💰 Cash Flow</b>.</li>
          <li>Log your bills in <b>📅 Bi-Monthly Budget</b> and your loans in <b>📉 Loan Generator</b>. Each lands on the paycheck that pays it.</li>
          <li>On payday, open that paycheck's tab and tap <b>Handle Bill</b> for each bill.</li>
        </ol>

        <div style={{ marginTop: "20px" }}>
          {inTauri ? (
            <button onClick={() => openUrl(MANUAL_URL).catch(() => alert(`Open this link in your browser:\n${MANUAL_URL}`))} style={manualButton}>📄 Download the User's Manual (PDF)</button>
          ) : (
            <a href="/The-Vault-User-Manual.pdf" download="The-Vault-User-Manual.pdf" style={{ ...manualButton, display: "inline-block", textDecoration: "none" }}>📄 Download the User's Manual (PDF)</a>
          )}
        </div>

        <h3 style={h3}>Terms & Privacy Notice</h3>
        {showTerms ? (
          <div style={{ maxHeight: "220px", overflowY: "auto", padding: "12px 15px", backgroundColor: "#f3f4f6", border: "1px solid #d1d5db", borderRadius: "8px", fontSize: "14px", color: "#374151", lineHeight: 1.55 }}>
            <p style={{ marginTop: 0 }}><b>About the app.</b> The Vault is a free personal budgeting app made by RCbreezy for family and friends. It is provided as is, without any warranty. It is a planning tool: always check amounts against your bank, e-wallet and billers.</p>
            <p><b>Your data.</b> What you enter is kept on your device and, when you sign in, in your family account on Google Firebase (Singapore data centre), so your devices stay in step.</p>
            <p><b>Privacy.</b> Every account is private: other members can never see your vaults. Your information is confidential. It is never shared, sold, or used for anything other than running the app for you.</p>
            <p><b>Administrator access.</b> Your data is protected by your password and Firebase's security rules, but it is not end-to-end encrypted. The administrator of the family's Firebase project (RCbreezy) therefore has technical access to stored data, like the owner of any app's database, but does not look at it.</p>
            <p><b>Please don't store</b> passwords, PINs, card numbers or full account numbers in The Vault.</p>
            <p style={{ marginBottom: 0 }}><b>Leaving.</b> You can stop using The Vault at any time and ask RCbreezy to delete your account and data.</p>
          </div>
        ) : (
          <p style={{ fontSize: "14px", color: "#047857", margin: 0 }}>
            ✅ You accepted the Terms & Privacy Notice.{" "}
            <button onClick={() => setShowTerms(true)} style={{ background: "none", border: "none", color: "#2563eb", cursor: "pointer", fontSize: "14px", padding: 0, textDecoration: "underline" }}>Read it again</button>
          </p>
        )}
        {!alreadyAccepted && (
          <label style={{ display: "flex", alignItems: "flex-start", gap: "10px", marginTop: "12px", fontSize: "14px", color: "#1f2937", cursor: "pointer", lineHeight: 1.4 }}>
            <input type="checkbox" checked={agreed} onChange={e => setAgreed(e.target.checked)} style={{ width: "18px", height: "18px", marginTop: "1px", flexShrink: 0 }} />
            I have read and agree to the Terms & Privacy Notice.
          </label>
        )}

        <button onClick={proceed} disabled={!agreed} style={{ marginTop: "22px", width: "100%", padding: "14px", backgroundColor: agreed ? "#1e3a8a" : "#9ca3af", color: "white", border: "none", borderRadius: "6px", cursor: agreed ? "pointer" : "not-allowed", fontWeight: "bold", fontSize: "17px" }}>
          Proceed →
        </button>
      </div>
    </div>
  );
}

const manualButton = { padding: "10px 16px", backgroundColor: "#eff6ff", color: "#1d4ed8", border: "1px solid #bfdbfe", borderRadius: "6px", cursor: "pointer", fontWeight: "bold", fontSize: "14px" };
