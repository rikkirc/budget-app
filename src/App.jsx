import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Dashboard from "./Dashboard";
import Budget from "./Budget";
import Debt from "./Debt";
import CashFlow from "./CashFlow"; 

export default function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [profileName, setProfileName] = useState("");
  const [activeTab, setActiveTab] = useState("Dashboard");
  const [refreshKey, setRefreshKey] = useState(0);

  const handleLogin = async () => {
    if (!profileName) {
      alert("Please enter a profile name!");
      return;
    }
    const response = await invoke("login_user", { profile: profileName });
    alert(response);
    setIsLoggedIn(true);
    await checkUpcomingRecurringBills();
  };

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
          return `• ${b.name} — due ${dueDate.toLocaleDateString("en-US", { month: "long", day: "2-digit", year: "numeric" })}`;
        }).join("\n");

        alert(`💡 You have upcoming utility bill(s) due soon!\n\n${billList}\n\nPlease input the exact amount now if you have the billing statement, so it's ready to pay on time.`);
      }
    } catch (e) {
      console.log("Could not check upcoming recurring bills.");
    }
  };

  const handleLogout = () => {
    if (window.confirm("Are you sure you want to lock the vault and log out?")) {
      setIsLoggedIn(false);
      setProfileName(""); 
      setActiveTab("Dashboard"); 
    }
  };

  if (!isLoggedIn) {
    return (
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100vh", backgroundColor: "#f3f4f6" }}>
        <div style={{ backgroundColor: "white", padding: "40px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)", textAlign: "center" }}>
          <h1 style={{ color: "#1e3a8a", margin: "0 0 10px 0" }}>Financial Command Center</h1>
          <p style={{ color: "#6b7280", marginBottom: "30px" }}>Please enter your secure profile name to unlock your vault.</p>
          
          <input 
            type="text" 
            placeholder="Enter your desired profile name" 
            value={profileName} 
            onChange={(e) => setProfileName(e.target.value)}
            style={{ padding: "12px", width: "250px", borderRadius: "5px", border: "1px solid #ccc", fontSize: "16px", marginBottom: "20px" }}
          />
          <br />
          <button 
            onClick={handleLogin}
            style={{ padding: "12px 24px", backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" }}
          >
            🔐 Unlock Vault
          </button>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: "20px", fontFamily: "Arial, sans-serif", backgroundColor: "#f3f4f6", minHeight: "100vh" }}>
      
      <div style={{ display: "flex", gap: "10px", marginBottom: "20px", backgroundColor: "white", padding: "15px", borderRadius: "8px", boxShadow: "0 2px 4px rgba(0,0,0,0.1)", flexWrap: "wrap", justifyContent: "space-between", alignItems: "center" }}>
        
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={() => setActiveTab("Dashboard")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Dashboard" ? "#1e3a8a" : "#e5e7eb", color: activeTab === "Dashboard" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>🏠 Dashboard</button>
          <button onClick={() => setActiveTab("CashFlow")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "CashFlow" ? "#047857" : "#e5e7eb", color: activeTab === "CashFlow" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>💰 Cash Flow</button>
          <button onClick={() => setActiveTab("Budget")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Budget" ? "#2563eb" : "#e5e7eb", color: activeTab === "Budget" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>📅 Bi-Monthly Budget</button>
          <button onClick={() => setActiveTab("Debt")} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: activeTab === "Debt" ? "#9333ea" : "#e5e7eb", color: activeTab === "Debt" ? "white" : "#374151", border: "none", borderRadius: "5px", transition: "0.2s" }}>📉 Loan Generator</button>
        </div>

        {/* 🚨 NEW: The Profile Name Tag sits right next to the Log Out button! */}
        <div style={{ display: "flex", alignItems: "center", gap: "15px" }}>
          <span style={{ fontSize: "16px", fontWeight: "bold", color: "#1f2937", backgroundColor: "#f3f4f6", padding: "10px 15px", borderRadius: "5px", border: "1px solid #d1d5db" }}>
            👤 Vault: {profileName}
          </span>
          <button onClick={() => setRefreshKey(prev => prev + 1)} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: "#0891b2", color: "white", border: "none", borderRadius: "5px", transition: "0.2s" }}>
            🔄 Refresh
          </button>
          <button onClick={handleLogout} style={{ padding: "12px 24px", cursor: "pointer", fontWeight: "bold", fontSize: "16px", backgroundColor: "#dc2626", color: "white", border: "none", borderRadius: "5px", transition: "0.2s" }}>
            🔒 Log Out
          </button>
        </div>

      </div>

      <div>
        {activeTab === "Dashboard" && <Dashboard key={refreshKey} />}
        {activeTab === "CashFlow" && <CashFlow key={refreshKey} />}
        {activeTab === "Budget" && <Budget key={refreshKey} />}
        {activeTab === "Debt" && <Debt key={refreshKey} />}
      </div>
      
    </div>
  );
}