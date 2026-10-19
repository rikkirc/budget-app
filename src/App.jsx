import { useState, useEffect, useRef } from "react";
import { displayBillName } from "./billNames";
import { invoke, SYNC_ONLY, SYNC_AVAILABLE, DEMO_AVAILABLE, isSignedIn, onAccountChange, subscribeToChanges, startDemo, endDemo } from "./api";
import Dashboard from "./Dashboard";
import Budget from "./Budget";
import Debt from "./Debt";
import CashFlow from "./CashFlow";
import Search from "./Search";
import SyncSettings from "./SyncSettings";
import Welcome from "./Welcome";
import ManualLink from "./ManualLink";
import { rememberedProfile, rememberProfile, forgetProfile } from "./preferences";

// "Sat, Oct 18, 2026 · 9:41 PM", kept current while the app is open
function Clock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 15000);
    return () => clearInterval(timer);
  }, []);
  const date = now.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric", year: "numeric" });
  const time = now.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
  return <span>{date} · {time}</span>;
}

export default function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [profileName, setProfileName] = useState("");
  const [activeTab, setActiveTab] = useState("Dashboard");
  const [refreshKey, setRefreshKey] = useState(0);
  // The signed-in family account: undefined while Firebase is still checking, null when not signed in
  const [account, setAccount] = useState(undefined);
  const sync = account ?? null;
  const [showSync, setShowSync] = useState(false);
  const [liveLink, setLiveLink] = useState(false);
  const [changedElsewhere, setChangedElsewhere] = useState(false);
  // The welcome screen comes first whenever the app opens to the sign-in or unlock screen
  const [welcomeDone, setWelcomeDone] = useState(false);
  // "Open this profile automatically next time" on the unlock screen
  const [rememberMe, setRememberMe] = useState(true);
  const [autoOpening, setAutoOpening] = useState(null); // the profile being opened automatically
  const autoTried = useRef(new Set());
  // "Try the demo": a made-up vault in this browser only. A link ending in ?demo opens it straight away.
  const [demo, setDemo] = useState(false);
  const [startingDemo, setStartingDemo] = useState(false);
  const demoLink = useRef(DEMO_AVAILABLE && new URLSearchParams(window.location.search).has("demo"));

  // Real-time sync: when the phone (or desktop) changes something, every open screen here reloads its data.
  // If you're in the middle of typing, it waits and offers a Refresh instead, so your half-filled form isn't wiped.
  useEffect(() => {
    if (!isLoggedIn || !sync || demo) return;
    return subscribeToChanges(profileName, {
      onStatus: setLiveLink,
      onChange: () => {
        const tag = document.activeElement?.tagName;
        if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") setChangedElsewhere(true);
        else setRefreshKey(prev => prev + 1);
      },
    });
  }, [isLoggedIn, sync, profileName, demo]);

  // Follow the sign-in. If it ends without you signing out here (the password was changed on another device),
  // lock the vault and ask to sign in again rather than quietly switching to other data.
  const signingOut = useRef(false);
  const unlocked = useRef(false);
  const unlockedSynced = useRef(false); // the open vault is a synced one (not this computer's local vault)
  unlocked.current = isLoggedIn;
  useEffect(() => onAccountChange((next) => {
    setAccount((previous) => {
      if (previous && !next && !signingOut.current && unlocked.current && unlockedSynced.current) {
        setIsLoggedIn(false);
        setShowSync(true);
        alert("You've been signed out of your account (was the password changed on another device?). Please sign in again.");
      }
      signingOut.current = false;
      return next;
    });
  }), []);

  const refresh = () => { setChangedElsewhere(false); setRefreshKey(prev => prev + 1); };

  const handleSyncChanged = async ({ signedOut = false } = {}) => {
    setShowSync(false);
    if (!isLoggedIn) return;
    if (signedOut) forgetProfile();
    // The phone and web apps have no vault of their own, so signing out locks the vault
    if (signedOut && SYNC_ONLY) { setIsLoggedIn(false); setProfileName(""); return; }
    // Reopen the same profile on whichever side now holds the data
    try { await invoke("login_user", { profile: profileName }); } catch (e) { alert(e); }
    unlockedSynced.current = isSignedIn();
    refresh();
  };

  // auto: opening the remembered profile when the app starts (no "Logged in" pop-up, and the unlock screen if it fails)
  const handleLogin = async (name = profileName, { auto = false } = {}) => {
    if (!name) {
      alert("Please enter a profile name!");
      return;
    }
    let response;
    try {
      response = await invoke("login_user", { profile: name });
    } catch (e) {
      alert(e);
      return;
    }
    if (!auto || !String(response).startsWith("SUCCESS")) alert(response);
    if (auto || rememberMe) rememberProfile(name, sync?.email); else forgetProfile();
    unlockedSynced.current = isSignedIn();
    setProfileName(name);
    setIsLoggedIn(true);
    await checkUpcomingRecurringBills();
  };

  // Open the remembered profile as soon as we know who is signed in (on the phone and web: once signed in)
  useEffect(() => {
    if (account === undefined || isLoggedIn || (SYNC_ONLY && !account)) return;
    const key = account?.email ?? "(this computer)";
    if (autoTried.current.has(key)) return;
    autoTried.current.add(key);
    if (demoLink.current) return; // a demo link opens the demo instead
    const saved = rememberedProfile(account?.email);
    if (!saved) return;
    setAutoOpening(saved);
    setProfileName(saved);
    handleLogin(saved, { auto: true }).finally(() => setAutoOpening(null));
  }, [account, isLoggedIn]);

  const tryDemo = async () => {
    setStartingDemo(true);
    try {
      const name = await startDemo();
      unlockedSynced.current = false;
      setDemo(true);
      setProfileName(name);
      setActiveTab("Dashboard");
      setIsLoggedIn(true);
      await checkUpcomingRecurringBills();
    } catch (e) {
      alert(`The demo couldn't open: ${e?.message ?? e}`);
    } finally {
      setStartingDemo(false);
    }
  };

  // Leaving the demo throws its vault away and goes back to the welcome screen
  const exitDemo = () => {
    endDemo();
    demoLink.current = false;
    if (window.location.search) window.history.replaceState(null, "", window.location.pathname);
    setDemo(false);
    setIsLoggedIn(false);
    setProfileName("");
    setActiveTab("Dashboard");
    setWelcomeDone(false);
  };

  useEffect(() => { if (demoLink.current) tryDemo(); }, []);

  const checkUpcomingRecurringBills = async () => {
    try {
      const bills = await invoke("get_smart_bills");
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const tenDaysFromNow = new Date(today);
      tenDaysFromNow.setDate(tenDaysFromNow.getDate() + 10);

      const upcomingTBD = bills.filter(bill => {
        if (!bill.is_recurring || bill.amount !== 0) return false;
        const [year, month, day] = bill.due_date.split('-');
        const dueDate = new Date(year, month - 1, day);
        return dueDate >= today && dueDate <= tenDaysFromNow;
      });

      if (upcomingTBD.length > 0) {
        const billList = upcomingTBD.map(b => {
          const [year, month, day] = b.due_date.split('-');
          const dueDate = new Date(year, month - 1, day);
          return `• ${displayBillName(b.name)} — due ${dueDate.toLocaleDateString("en-US", { month: "long", day: "2-digit", year: "numeric" })}`;
        }).join("\n");

        alert(`💡 You have upcoming utility bill(s) due soon!\n\n${billList}\n\nPlease input the exact amount now if you have the billing statement, so it's ready to pay on time.`);
      }
    } catch (e) {
      console.log("Could not check upcoming recurring bills.");
    }
  };

  const handleLogout = () => {
    if (demo) { exitDemo(); return; }
    if (window.confirm("Are you sure you want to lock the vault and log out?")) {
      // Logging out on purpose means: don't open this profile by itself next time
      forgetProfile();
      setIsLoggedIn(false);
      setProfileName(""); 
      setActiveTab("Dashboard"); 
    }
  };

  if (account === undefined) {
    return <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: "100vh", backgroundColor: "#f3f4f6", color: "#6b7280" }}>Opening The Vault…</div>;
  }

  if (startingDemo && !isLoggedIn) {
    return <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: "100vh", backgroundColor: "#f3f4f6", color: "#6b7280" }}>Opening the demo…</div>;
  }

  if (autoOpening && !isLoggedIn) {
    return <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: "100vh", backgroundColor: "#f3f4f6", color: "#6b7280" }}>Opening {autoOpening}…</div>;
  }

  if (!isLoggedIn && !welcomeDone) return <Welcome onProceed={() => setWelcomeDone(true)} onTryDemo={DEMO_AVAILABLE ? tryDemo : null} />;

  // The phone and web apps always work through a family account, so they must be signed in before the first unlock
  if (!isLoggedIn && (showSync || (SYNC_ONLY && !sync))) {
    return (
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: "16px", minHeight: "100vh", backgroundColor: "#f3f4f6", padding: "20px", boxSizing: "border-box" }}>
        <SyncSettings account={account} profileName="" beforeSignOut={() => { signingOut.current = true; }} onDone={handleSyncChanged} onCancel={sync || !SYNC_ONLY ? () => setShowSync(false) : null} />
        <ManualLink style={screenManualLink} />
      </div>
    );
  }

  if (!isLoggedIn) {
    return (
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100vh", backgroundColor: "#f3f4f6" }}>
        <div style={{ backgroundColor: "white", padding: "40px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)", textAlign: "center" }}>
          <h1 style={{ color: "#1e3a8a", margin: "0 0 4px 0" }}>🔐 The Vault</h1>
          <div style={{ color: "#6b7280", fontSize: "13px", marginBottom: "15px" }}>by RCbreezy ©</div>
          <p style={{ color: "#6b7280", marginBottom: "30px" }}>Please enter your secure profile name to unlock your vault.</p>
          {/* A form, so pressing Enter unlocks */}
          <form onSubmit={e => { e.preventDefault(); handleLogin(); }}>
          <input 
            type="text" 
            placeholder="Enter your desired profile name" 
            value={profileName} 
            onChange={(e) => setProfileName(e.target.value)}
            style={{ padding: "12px", width: "250px", borderRadius: "5px", border: "1px solid #ccc", fontSize: "16px", marginBottom: "20px" }}
          />
          <br />
          <label style={{ display: "inline-flex", alignItems: "center", gap: "8px", fontSize: "14px", color: "#374151", marginBottom: "20px", cursor: "pointer" }}>
            <input type="checkbox" checked={rememberMe} onChange={e => setRememberMe(e.target.checked)} style={{ width: "18px", height: "18px" }} />
            Open this profile automatically next time
          </label>
          <br />
          <button 
            type="submit"
            style={{ padding: "12px 24px", backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" }}
          >
            🔐 Unlock Vault
          </button>
          </form>
          <p style={{ marginTop: "20px", marginBottom: 0 }}><ManualLink style={{ color: "#2563eb", fontSize: "14px", textDecoration: "underline", background: "none", border: "none", padding: 0, cursor: "pointer" }} /></p>
          {SYNC_AVAILABLE && (
            <p style={{ color: "#6b7280", fontSize: "14px", marginTop: "12px", marginBottom: 0 }}>
              {sync ? `☁️ Signed in as ${sync.email}` : SYNC_ONLY ? "" : "💻 This computer only"}
              {" · "}
              <button onClick={() => setShowSync(true)} style={{ background: "none", border: "none", color: "#2563eb", cursor: "pointer", fontSize: "14px", padding: 0, textDecoration: "underline" }}>{sync ? "Account" : "Sign in to sync"}</button>
            </p>
          )}
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: "20px", fontFamily: "Arial, sans-serif", backgroundColor: "#f3f4f6", minHeight: "100vh" }}>
      <div className="app-brand" style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", flexWrap: "wrap", gap: "4px 15px", marginBottom: "12px", padding: "0 5px" }}>
        <div>
          <span style={{ fontSize: "22px", fontWeight: "bold", color: "#1e3a8a" }}>The Vault</span>
          <span style={{ fontSize: "13px", color: "#6b7280", marginLeft: "8px" }}>by RCbreezy ©</span>
        </div>
        <div style={{ fontSize: "14px", color: "#4b5563" }}><Clock /></div>
      </div>

      <div className="app-header" style={{ display: "flex", gap: "10px", marginBottom: "20px", backgroundColor: "white", padding: "15px", borderRadius: "8px", boxShadow: "0 2px 4px rgba(0,0,0,0.1)", flexWrap: "wrap", justifyContent: "space-between", alignItems: "center" }}>
        
        <div className="app-tabs" style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={() => setActiveTab("Dashboard")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Dashboard" ? "#1e3a8a" : "#e5e7eb", color: activeTab === "Dashboard" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>🏠 Dashboard</button>
          <button onClick={() => setActiveTab("CashFlow")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "CashFlow" ? "#047857" : "#e5e7eb", color: activeTab === "CashFlow" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>💰 Cash Flow</button>
          <button onClick={() => setActiveTab("Budget")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Budget" ? "#2563eb" : "#e5e7eb", color: activeTab === "Budget" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>📅 Bi-Monthly Budget</button>
          <button onClick={() => setActiveTab("Debt")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Debt" ? "#9333ea" : "#e5e7eb", color: activeTab === "Debt" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>📉 Loan Generator</button>
          <button onClick={() => setActiveTab("Search")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Search" ? "#4f46e5" : "#e5e7eb", color: activeTab === "Search" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>🔍 Search</button>
        </div>

        {/* 🚨 NEW: The Profile Name Tag sits right next to the Log Out button! */}
        <div className="app-actions" style={{ display: "flex", alignItems: "center", gap: "15px", flexWrap: "wrap" }}>
          <span style={{ fontSize: "16px", fontWeight: "bold", color: "#1f2937", backgroundColor: "#f3f4f6", padding: "10px 15px", borderRadius: "5px", border: "1px solid #d1d5db" }}>
            👤 Vault: {profileName}
          </span>
          {SYNC_AVAILABLE && !demo && (
            <button onClick={() => setShowSync(true)} title={sync ? `Signed in as ${sync.email}` : "Sign in to sync with your phone"} style={{ padding: "10px 15px", cursor: "pointer", fontWeight: "bold", fontSize: "14px", backgroundColor: !sync ? "#f3f4f6" : liveLink ? "#ecfdf5" : "#fff7ed", color: !sync ? "#374151" : liveLink ? "#047857" : "#c2410c", border: "1px solid #d1d5db", borderRadius: "5px" }}>
              {!sync ? "💻 Local only" : liveLink ? "🟢 Live sync" : "🟠 Reconnecting…"}
            </button>
          )}
          <ManualLink style={{ padding: "10px 15px", fontWeight: "bold", fontSize: "14px", backgroundColor: "#eff6ff", color: "#1d4ed8", border: "1px solid #bfdbfe", borderRadius: "5px", cursor: "pointer" }}>📄 Manual</ManualLink>
          <button onClick={refresh} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: "#0891b2", color: "white", border: "none", borderRadius: "5px", transition: "0.2s" }}>
            🔄 Refresh
          </button>
          <button onClick={handleLogout} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: "#dc2626", color: "white", border: "none", borderRadius: "5px", transition: "0.2s" }}>
            {demo ? "🚪 Exit Demo" : "🔒 Log Out"}
          </button>
        </div>

      </div>

      {showSync && (
        <div onClick={() => setShowSync(false)} style={{ position: "fixed", inset: 0, backgroundColor: "rgba(0,0,0,0.4)", display: "flex", alignItems: "center", justifyContent: "center", padding: "20px", zIndex: 1000 }}>
          <div onClick={e => e.stopPropagation()} style={{ width: "100%", maxWidth: "460px" }}>
            <SyncSettings account={account} profileName={profileName} beforeSignOut={() => { signingOut.current = true; }} onDone={handleSyncChanged} onCancel={() => setShowSync(false)} />
          </div>
        </div>
      )}

      {demo && (
        <div className="demo-banner" role="status" style={{ display: "flex", flexWrap: "wrap", alignItems: "center", justifyContent: "space-between", gap: "10px", marginBottom: "20px", padding: "12px 15px", backgroundColor: "#fffbeb", border: "1px solid #fcd34d", borderRadius: "8px", color: "#92400e" }}>
          <span><b>🧪 Demo:</b> nothing here is saved. Create an account to use your own vault.</span>
          <button onClick={exitDemo} style={{ padding: "8px 14px", backgroundColor: "#1e3a8a", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "14px" }}>
            ← Back to the welcome screen
          </button>
        </div>
      )}

      {changedElsewhere && (
        <div onClick={refresh} style={{ marginBottom: "20px", padding: "12px 15px", backgroundColor: "#eff6ff", border: "1px solid #bfdbfe", borderRadius: "8px", color: "#1e40af", fontWeight: "bold", cursor: "pointer" }}>
          🔄 Your vault was updated on another device. Tap here to refresh.
        </div>
      )}

      <div>
        {activeTab === "Dashboard" && <Dashboard key={refreshKey} />}
        {activeTab === "CashFlow" && <CashFlow key={refreshKey} />}
        {activeTab === "Budget" && <Budget key={refreshKey} />}
        {activeTab === "Debt" && <Debt key={refreshKey} />}
        {activeTab === "Search" && <Search key={refreshKey} onNavigate={setActiveTab} />}
      </div>
      
    </div>
  );
}

// The manual link under the sign-in card
const screenManualLink = { color: "#2563eb", fontSize: "14px", textDecoration: "underline" };
