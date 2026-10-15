import { useState, useEffect } from "react";
import { displayBillName } from "./billNames";
import { invoke } from "./api";

export default function Dashboard() {
  const [totalCash, setTotalCash] = useState(0);
  const [totalDebt, setTotalDebt] = useState(0);
  const [upcomingBills, setUpcomingBills] = useState([]);
  const [allBills, setAllBills] = useState([]);
  const [activeLoans, setActiveLoans] = useState([]);

  // 🚨 Wake up and ask the Engine for the grand totals!
  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    // 1. Calculate Total Liquid Cash
    const accounts = await invoke("get_accounts");
    const cash = accounts.reduce((sum, acc) => sum + acc.balance, 0);
    setTotalCash(cash);

    // 2. Calculate Total Remaining Debt
    const loans = await invoke("get_loans");
    const debt = loans.reduce((sum, loan) => sum + loan.remaining_balance, 0);
    setTotalDebt(debt);
    setActiveLoans(loans);

    // 3. Find the Next 5 Upcoming Bills (and keep the full sorted list for the summary table)
    const bills = await invoke("get_smart_bills");
    
    // Sort them by date so the most urgent ones are at the top
    const sortedBills = [...bills].sort((a, b) => new Date(a.due_date) - new Date(b.due_date));
        
    setUpcomingBills(sortedBills.slice(0, 5));
    setAllBills(sortedBills);
  };

  const upcomingTotal = upcomingBills.reduce((sum, bill) => sum + bill.amount, 0);
  const upcomingTbdCount = upcomingBills.filter(bill => bill.amount === 0).length;

  const formatDate = (dateStr) => {
    if (!dateStr) return "";
    const [year, month, day] = dateStr.split('-');
    const date = new Date(year, month - 1, day);
    return date.toLocaleDateString("en-US", { month: "long", day: "2-digit", year: "numeric" });
  };

  return (
    <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)" }}>
      <h2 style={{ color: "#1e3a8a", marginTop: "0", fontSize: "32px" }}>Financial Command Center</h2>
      <p style={{ color: "#6b7280", fontSize: "16px", marginBottom: "30px" }}>Welcome to your personal wealth dashboard.</p>

      {/* 📊 The Big Scoreboards */}
      <div style={{ display: "flex", gap: "20px", flexWrap: "wrap", marginBottom: "40px" }}>
        
        <div style={{ flex: 1, minWidth: "250px", backgroundColor: "#ecfdf5", padding: "25px", borderRadius: "10px", border: "1px solid #a7f3d0" }}>
          <h3 style={{ margin: "0 0 10px 0", color: "#065f46", fontSize: "18px" }}>💰 Total Liquid Cash</h3>
          <div style={{ fontSize: "36px", fontWeight: "bold", color: "#047857" }}>
            ₱{totalCash.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </div>
        </div>

        <div style={{ flex: 1, minWidth: "250px", backgroundColor: "#fef2f2", padding: "25px", borderRadius: "10px", border: "1px solid #fecaca" }}>
          <h3 style={{ margin: "0 0 10px 0", color: "#991b1b", fontSize: "18px" }}>📉 Total Remaining Debt</h3>
          <div style={{ fontSize: "36px", fontWeight: "bold", color: "#b91c1c" }}>
            ₱{totalDebt.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </div>
        </div>

      </div>

      {/* 📅 The Urgent Bills Alert */}
      <div>
        <h3 style={{ color: "#1f2937", borderBottom: "2px solid #e5e7eb", paddingBottom: "10px", marginBottom: "20px", display: "flex", justifyContent: "space-between", alignItems: "baseline", flexWrap: "wrap", gap: "10px" }}>
          <span>📅 Next {upcomingBills.length || 5} Upcoming Bills</span>
          {upcomingBills.length > 0 && (
            <span style={{ fontSize: "16px", color: "#4b5563" }}>
              Total: <span style={{ color: "#dc2626", fontSize: "20px" }}>₱{upcomingTotal.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
              {upcomingTbdCount > 0 && (
                <span style={{ fontSize: "12px", fontWeight: "normal", color: "#92400e", marginLeft: "8px" }}>
                  + {upcomingTbdCount} TBD {upcomingTbdCount === 1 ? "bill" : "bills"} not yet included
                </span>
              )}
            </span>
          )}
        </h3>
        
        {upcomingBills.length === 0 ? (
          <p style={{ fontStyle: "italic", color: "#6b7280" }}>No upcoming bills! You are all caught up.</p>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            {upcomingBills.map((bill, index) => (
              <div key={index} style={{ display: "flex", justifyContent: "space-between", padding: "15px", backgroundColor: "#f9fafb", borderRadius: "8px", border: "1px solid #e5e7eb" }}>
                <div>
                  <div style={{ fontWeight: "bold", color: "#374151", fontSize: "16px" }}>{displayBillName(bill.name)}</div>
                  <div style={{ color: "#dc2626", fontSize: "14px", marginTop: "5px" }}>Due: {formatDate(bill.due_date)}</div>
                </div>
                <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end" }}>
                    <div style={{ fontWeight: "bold", color: bill.amount === 0 ? "#f59e0b" : "#dc2626", fontSize: "18px" }}>
                    {bill.amount === 0 ? "TBD" : `₱${bill.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
                    </div>
                    <div style={{ color: "#6b7280", fontSize: "12px", marginTop: "3px" }}>Allocated to: {formatDate(bill.payday_date)} Paycheck</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 📅 BI-MONTHLY BUDGET QUICK SUMMARY (read-only table) */}
      <div style={{ marginTop: "40px" }}>
        <h3 style={{ color: "#1f2937", borderBottom: "2px solid #e5e7eb", paddingBottom: "10px", marginBottom: "20px" }}>
          📅 Bi-Monthly Budget Summary
        </h3>

        {allBills.length === 0 ? (
          <p style={{ fontStyle: "italic", color: "#6b7280" }}>No bills logged yet.</p>
        ) : (
          <div style={{ maxHeight: "320px", overflowY: "auto", border: "1px solid #e5e7eb", borderRadius: "8px" }}>
            <table className="summary-table" style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
              <thead style={{ position: "sticky", top: 0, backgroundColor: "#f3f4f6" }}>
                <tr style={{ borderBottom: "2px solid #d1d5db" }}>
                  <th style={{ textAlign: "left", padding: "10px 12px", color: "#6b7280" }}>Bill Name</th>
                  <th style={{ textAlign: "left", padding: "10px 12px", color: "#6b7280" }}>Due Date</th>
                  <th style={{ textAlign: "right", padding: "10px 12px", color: "#6b7280" }}>Amount</th>
                  <th style={{ textAlign: "left", padding: "10px 12px", color: "#6b7280" }}>Paycheck</th>
                </tr>
              </thead>
              <tbody>
                {allBills.map((bill, index) => (
                  <tr key={bill.id} style={{ backgroundColor: index % 2 === 0 ? "#ffffff" : "#f9fafb", borderBottom: "1px solid #e5e7eb" }}>
                    <td style={{ padding: "10px 12px", fontWeight: "bold", color: "#374151" }}>{displayBillName(bill.name)}</td>
                    <td style={{ padding: "10px 12px", color: "#dc2626" }}>{formatDate(bill.due_date)}</td>
                    <td style={{ padding: "10px 12px", textAlign: "right", color: "#374151" }}>
                      {bill.amount === 0 ? <span style={{ color: "#f59e0b", fontStyle: "italic" }}>TBD</span> : `₱${bill.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
                    </td>
                    <td style={{ padding: "10px 12px", color: "#6b7280" }}>{formatDate(bill.payday_date)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 📉 ACTIVE LOANS QUICK SUMMARY (read-only table) */}
      <div style={{ marginTop: "40px" }}>
        <h3 style={{ color: "#1f2937", borderBottom: "2px solid #e5e7eb", paddingBottom: "10px", marginBottom: "20px" }}>
          📉 Active Loans Summary
        </h3>

        {activeLoans.length === 0 ? (
          <p style={{ fontStyle: "italic", color: "#6b7280" }}>No active loans.</p>
        ) : (
          <div style={{ maxHeight: "320px", overflowY: "auto", border: "1px solid #e5e7eb", borderRadius: "8px" }}>
            <table className="summary-table" style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
              <thead style={{ position: "sticky", top: 0, backgroundColor: "#f3f4f6" }}>
                <tr style={{ borderBottom: "2px solid #d1d5db" }}>
                  <th style={{ textAlign: "left", padding: "10px 12px", color: "#6b7280" }}>Loan Name</th>
                  <th style={{ textAlign: "left", padding: "10px 12px", color: "#6b7280" }}>Term</th>
                  <th style={{ textAlign: "left", padding: "10px 12px", color: "#6b7280" }}>Frequency</th>
                  <th style={{ textAlign: "right", padding: "10px 12px", color: "#6b7280" }}>Payment</th>
                  <th style={{ textAlign: "right", padding: "10px 12px", color: "#6b7280" }}>Remaining</th>
                </tr>
              </thead>
              <tbody>
                {activeLoans.map((loan, index) => (
                  <tr key={loan.id} style={{ backgroundColor: index % 2 === 0 ? "#ffffff" : "#f9fafb", borderBottom: "1px solid #e5e7eb" }}>
                    <td style={{ padding: "10px 12px", fontWeight: "bold", color: "#374151" }}>{loan.name}</td>
                    <td style={{ padding: "10px 12px", color: "#6b7280" }}>{loan.term} {loan.term_type}</td>
                    <td style={{ padding: "10px 12px", color: "#6b7280" }}>{loan.frequency}</td>
                    <td style={{ padding: "10px 12px", textAlign: "right", color: "#374151" }}>₱{loan.payment_amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</td>
                    <td style={{ padding: "10px 12px", textAlign: "right", fontWeight: "bold", color: "#dc2626" }}>₱{loan.remaining_balance.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

    </div>
  );
}