import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function Debt() {
  const [loans, setLoans] = useState([]);
  const [archivedLoans, setArchivedLoans] = useState([]);
  const [completedLoans, setCompletedLoans] = useState([]);
  const [bills, setBills] = useState([]); 
  
  const [name, setName] = useState("");
  const [principal, setPrincipal] = useState("");
  const [remainingBalance, setRemainingBalance] = useState("");
  const [term, setTerm] = useState("");
  const [termType, setTermType] = useState("Months"); 
  const [frequency, setFrequency] = useState("Monthly"); 
  const [paymentAmount, setPaymentAmount] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [interestRate, setInterestRate] = useState("");
  const [interestType, setInterestType] = useState("Flat");
  const [dueStructure, setDueStructure] = useState("Sinking Fund");
  const [showDevTools, setShowDevTools] = useState(false);
  const [dateAcquired, setDateAcquired] = useState("");
  const [isManualEntry, setIsManualEntry] = useState(false);
  
  useEffect(() => {
    if (!isManualEntry && principal && interestRate && term) {
        const totals = calculateLoanTotals();
        setRemainingBalance(totals.totalIncurred.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
        setPaymentAmount(totals.monthlyPayment.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
    }
}, [principal, interestRate, interestType, term, termType, isManualEntry]);

  const [paydayOne, setPaydayOne] = useState("15");
  const [paydayTwo, setPaydayTwo] = useState("30");

  const [expandedLoanId, setExpandedLoanId] = useState(null);
  const [draftSchedule, setDraftSchedule] = useState([]);

  useEffect(() => { 
    fetchLoans(); 
    fetchArchivedLoans(); 
    fetchBills();
    fetchCompletedLoans();
  }, []);

  const fetchLoans = async () => {
    const data = await invoke("get_loans");
    setLoans(data);
  };

  const fetchArchivedLoans = async () => {
    const data = await invoke("get_archived_loans");
    setArchivedLoans(data);
  };

  const fetchCompletedLoans = async () => {
    const data = await invoke("get_completed_loans");
    setCompletedLoans(data);
  };

  const fetchBills = async () => {
    try {
      const activeBills = await invoke("get_smart_bills");
      const paidBills = await invoke("get_paid_bills");
      setBills([...activeBills, ...paidBills]); // 👈 Now checks both active bills and paid sinking funds!
      await fetchLoans();
      await fetchCompletedLoans();
    } catch (e) {
      console.log("No bills found.");
    }
  };

  const calculateLoanTotals = () => {
    const cleanPrincipal = parseFloat((principal || "0").replace(/,/g, ""));
    const cleanRate = parseFloat(interestRate || "0");
    const cleanTerm = parseInt(term || "0", 10);

    if (!cleanPrincipal || !cleanRate || !cleanTerm) {
        return { totalInterest: 0, totalIncurred: cleanPrincipal, monthlyPayment: 0, biMonthlyPayment: 0 };
    }

    const totalMonths = termType === "Years" ? cleanTerm * 12 : cleanTerm;
    let totalInterest = 0;

    if (interestType === "Flat") {
        // One-time percentage on top of the principal, regardless of term length
        totalInterest = cleanPrincipal * (cleanRate / 100);
    } else if (interestType === "Monthly") {
        // The rate repeats every single month of the term
        totalInterest = cleanPrincipal * (cleanRate / 100) * totalMonths;
    } else if (interestType === "Annual") {
        // The rate is yearly, so we prorate it across however many months the term covers
        totalInterest = cleanPrincipal * (cleanRate / 100) * (totalMonths / 12);
    }

    const totalIncurred = cleanPrincipal + totalInterest;
    const monthlyPayment = totalMonths > 0 ? totalIncurred / totalMonths : 0;
    const biMonthlyPayment = monthlyPayment / 2;

    return { totalInterest, totalIncurred, monthlyPayment, biMonthlyPayment };
};

  const formatMoney = (value, setter) => {
    let rawText = value.replace(/[^0-9.]/g, "");
    let parts = rawText.split(".");
    if (parts.length > 2) rawText = parts[0] + "." + parts.slice(1).join("");
    if (parts[0]) { parts[0] = parseInt(parts[0], 10).toLocaleString("en-US"); rawText = parts.join("."); }
    setter(rawText);
  };

  const isRealDate = (dateStr) => {
    if (!dateStr || dateStr.length < 10) return true;
    const [year, month, day] = dateStr.split('-').map(Number);
    const test = new Date(year, month - 1, day);
    return test.getFullYear() === year && test.getMonth() === month - 1 && test.getDate() === day;
  };

  const formatDate = (dateStr) => {
    if (!dateStr) return "";
    const [year, month, day] = dateStr.split('-');
    const date = new Date(year, month - 1, day);
    return date.toLocaleDateString("en-US", { month: "long", day: "2-digit", year: "numeric" });
  };

  const getPrecedingPayday = (targetDate, p1, p2) => {
    let gapDate = new Date(targetDate);
    gapDate.setUTCDate(gapDate.getUTCDate() - 11);
    let year = gapDate.getUTCFullYear();
    let month = gapDate.getUTCMonth();
    const getPaydate = (y, m, d) => (d >= 30) ? new Date(Date.UTC(y, m + 1, 0)) : new Date(Date.UTC(y, m, d));
    let candidates = [
        getPaydate(year, month - 1, p1),
        getPaydate(year, month - 1, p2),
        getPaydate(year, month, p1),
        getPaydate(year, month, p2),
        getPaydate(year, month + 1, p1)
    ];
    let valid = candidates.filter(c => c <= gapDate);
    valid.sort((a, b) => b - a);
    return valid[0]; 
  };

  const generateScheduleList = (baseDateStr, termVal, type, freq, amount, p1, p2) => {
    let schedule = [];
    let totalMonths = type === "Years" ? termVal * 12 : termVal;
    let isBiMonthly = freq === "Bi-Monthly";
    
    for (let i = 0; i < totalMonths; i++) {
        let currentDue = new Date(baseDateStr);
        currentDue.setUTCMonth(currentDue.getUTCMonth() + i);
        let dueStr = currentDue.toISOString().split('T')[0];
        
        if (isBiMonthly) {
            let secondHalfPayday = getPrecedingPayday(currentDue, p1, p2);
            let firstHalfDue = new Date(currentDue);
            firstHalfDue.setUTCDate(firstHalfDue.getUTCDate() - 15);
            let firstHalfDueStr = firstHalfDue.toISOString().split('T')[0];
            let firstHalfPayday = getPrecedingPayday(firstHalfDue, p1, p2);
            
            schedule.push({ label: "1st Half", dueDate: firstHalfDueStr, paydayDate: firstHalfPayday.toISOString().split('T')[0], amount: amount / 2 });
            schedule.push({ label: "2nd Half (Due)", dueDate: dueStr, paydayDate: secondHalfPayday.toISOString().split('T')[0], amount: amount / 2 });
        } else {
            let payday = getPrecedingPayday(currentDue, p1, p2);
            schedule.push({ label: "Due Date", dueDate: dueStr, paydayDate: payday.toISOString().split('T')[0], amount: amount });
        }
    }
    return schedule;
  };

  const handlePreviewSchedule = () => {
    const principalRequired = isManualEntry ? true : !!principal; // Still require SOME principal value, even in Manual mode, but skip interest-specific checks

    if (!dueDate) {
      alert("⚠️ Invalid or missing Next Due Date! Please pick a valid date using the calendar icon.");
      return;
    }

    if (!name || !principalRequired || !remainingBalance || !term || !paymentAmount || !paydayOne || !paydayTwo) {
      alert("Please fill out all the boxes, including your custom Paycheck Schedule!");
      return;
    }
    const cleanPayment = parseFloat(paymentAmount.replace(/,/g, ""));
    const cleanTerm = parseInt(term, 10);
    const p1 = parseInt(paydayOne, 10);
    const p2 = parseInt(paydayTwo, 10);
    
    const list = generateScheduleList(dueDate, cleanTerm, termType, frequency, cleanPayment, p1, p2);
    setDraftSchedule(list);
  };

  const handleUpdateDraftDate = (index, newDate) => {
    const updatedDraft = [...draftSchedule];
    updatedDraft[index].paydayDate = newDate;
    setDraftSchedule(updatedDraft);
  };

  const handleConfirmAndSaveLoan = async () => {
    const cleanPrincipal = parseFloat(principal.replace(/,/g, ""));
    const cleanRemaining = parseFloat(remainingBalance.replace(/,/g, ""));
    const cleanPayment = parseFloat(paymentAmount.replace(/,/g, ""));
    const cleanTerm = parseInt(term, 10);
    const cleanTotalIncurred = calculateLoanTotals().totalIncurred;
    const finalDueStructure = frequency === "Bi-Monthly" ? dueStructure : "N/A";

    const namesList = draftSchedule.map(item => `${name} (${item.label})`);
    const amountsList = draftSchedule.map(item => item.amount);
    const dueDatesList = draftSchedule.map(item => item.dueDate);
    const paydayDatesList = draftSchedule.map(item => item.paydayDate);

    const response = await invoke("add_loan", {
      name, principal: cleanPrincipal, remainingBalance: cleanRemaining, term: cleanTerm,
      termType, frequency, paymentAmount: cleanPayment, dueDate, paydayDate: "", totalIncurred: cleanTotalIncurred, dueStructure: finalDueStructure, dateAcquired: dateAcquired,
      billNames: namesList, billAmounts: amountsList, billDues: dueDatesList, billPaydays: paydayDatesList
    });
    
    alert(response);
    await fetchLoans(); 
    await fetchBills();
    setName(""); setPrincipal(""); setRemainingBalance(""); setTerm(""); setPaymentAmount(""); setDueDate(""); 
    setInterestRate(""); setInterestType("Flat"); setDueStructure("Sinking Fund"); setIsManualEntry(false); setDateAcquired("");
    setDraftSchedule([]); 
  };

  const handleDeleteLoan = async (loanId, loanName) => {
    const isConfirmed = window.confirm(`Are you sure you want to delete the loan "${loanName}"?\n\nIt will be moved to the Recycle Bin, and all future unpaid budget schedules for this loan will be instantly wiped to keep your Budget clean.`);
    if (isConfirmed) {
      const response = await invoke("delete_loan", { loanId: loanId });
      alert(response);
      await fetchLoans(); 
      await fetchArchivedLoans(); 
      await fetchBills();
    }
  };

  const handleUndoDeleteLoan = async (archivedLoan) => {
    if (window.confirm(`Restore "${archivedLoan.name}"? This will put the loan back on your dashboard and completely rebuild its future payment schedule in your Budget.`)) {
      const scheduleList = generateScheduleList(archivedLoan.next_due_date, archivedLoan.term, archivedLoan.term_type, archivedLoan.frequency, archivedLoan.payment_amount, parseInt(paydayOne, 10), parseInt(paydayTwo, 10));
      const namesList = scheduleList.map(item => `${archivedLoan.name} (${item.label})`);
      const amountsList = scheduleList.map(item => item.amount);
      const dueDatesList = scheduleList.map(item => item.dueDate);
      const paydayDatesList = scheduleList.map(item => item.paydayDate);

      const response = await invoke("undo_delete_loan", {
        archiveId: archivedLoan.id, billNames: namesList, billAmounts: amountsList, billDues: dueDatesList, billPaydays: paydayDatesList
      });

      alert(response);
      await fetchLoans(); 
      await fetchArchivedLoans(); 
      await fetchBills();
    }
  };

  const renderFullSchedule = (loan) => {
    const p1 = paydayOne ? parseInt(paydayOne, 10) : 15;
    const p2 = paydayTwo ? parseInt(paydayTwo, 10) : 30;
    const scheduleList = generateScheduleList(loan.next_due_date, loan.term, loan.term_type, loan.frequency, loan.payment_amount, p1, p2);
    
    const totalScheduled = scheduleList.reduce((sum, item) => sum + item.amount, 0);
    const amountPaid = totalScheduled - loan.remaining_balance;
    let accumulated = 0;

    return (
        <div style={{ marginTop: "15px", padding: "15px", backgroundColor: "#f3f4f6", borderRadius: "8px", border: "1px solid #d1d5db", maxHeight: "350px", overflowY: "auto" }}>
            <div style={{ marginBottom: "12px", padding: "10px 15px", backgroundColor: "#faf5ff", border: "1px solid #e9d5ff", borderRadius: "6px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span style={{ fontWeight: "bold", color: "#6b21a8", fontSize: "14px" }}>Total Loan Incurred (Principal + Interest):</span>
                <span style={{ fontWeight: "bold", color: "#6b21a8", fontSize: "16px" }}>₱{loan.total_incurred.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
            </div>
            <h4 style={{ margin: "0 0 12px 0", color: "#374151", fontSize: "16px" }}>Full {scheduleList.length}-Payment Schedule:</h4>

            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
                <thead>
                    <tr style={{ borderBottom: "2px solid #d1d5db" }}>
                        <th style={{ textAlign: "left", padding: "6px 8px", color: "#6b7280" }}>Installment</th>
                        <th style={{ textAlign: "left", padding: "6px 8px", color: "#6b7280" }}>Due Date</th>
                        <th style={{ textAlign: "right", padding: "6px 8px", color: "#6b7280" }}>Amount</th>
                        <th style={{ textAlign: "left", padding: "6px 8px", color: "#6b7280" }}>Paycheck</th>
                        <th style={{ textAlign: "center", padding: "6px 8px", color: "#6b7280" }}>Status</th>
                    </tr>
                </thead>
                <tbody>
                    {scheduleList.map((item, index) => {
                        accumulated += item.amount;
                        const isPaid = accumulated <= (amountPaid + 0.05);

                        let displayLabel = item.label;
                        let isHandledInFund = false;

                        if (item.label === "1st Half" && !isPaid) {
                            const handledBill = bills.find(b =>
                                b.loan_id === loan.id &&
                                b.due_date === item.dueDate &&
                                b.name.includes("(1st Half)") &&
                                b.name.includes("(Handled in")
                            );

                            if (handledBill) {
                                const match = handledBill.name.match(/\(Handled in (.*?)\)/);
                                if (match) {
                                    displayLabel = `1st Half (Handled - ${match[1]})`;
                                    isHandledInFund = true;
                                }
                            }
                        }

                        // Shade every "pair" (1st Half + 2nd Half) the same soft color so months are easy to tell apart
                        const pairIndex = Math.floor(index / 2);
                        const rowBg = pairIndex % 2 === 0 ? "#ffffff" : "#eef0f3";

                        return (
                            <tr key={index} style={{ backgroundColor: rowBg, borderBottom: "1px solid #e5e7eb" }}>
                                <td style={{ padding: "8px", fontWeight: "bold", color: "#6b21a8" }}>{displayLabel}</td>
                                <td style={{ padding: "8px", color: "#374151" }}>📅 {formatDate(item.dueDate)}</td>
                                <td style={{ padding: "8px", textAlign: "right", color: "#374151" }}>
                                    ₱{item.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                </td>
                                <td style={{ padding: "8px", color: "#6b7280" }}>{formatDate(item.paydayDate)}</td>
                                <td style={{ padding: "8px", textAlign: "center" }}>
                                    {isPaid ? (
                                        <span style={{ color: "#059669", fontWeight: "bold" }}>✅ Paid</span>
                                    ) : isHandledInFund ? (
                                        <span style={{ color: "#2563eb", fontWeight: "bold", fontStyle: "italic" }}>📥 Saved</span>
                                    ) : (
                                        <span style={{ color: "#9ca3af", fontStyle: "italic" }}>unpaid</span>
                                    )}
                                </td>
                            </tr>
                        );
                    })}
                </tbody>
            </table>
        </div>
    );
};

  const totalDebt = loans.reduce((sum, loan) => sum + loan.remaining_balance, 0);

  const handleRepairLoanIds = async () => {
    if (window.confirm("This will scan all your bills and link them to their correct loans by ID. Run this once after migrating data from an older app version. Continue?")) {
      const response = await invoke("repair_loan_ids");
      alert(response);
      await fetchBills();
    }
  };

  return (
    <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)" }}>
      <h2 style={{ color: "#9333ea", marginTop: "0", fontSize: "28px" }}>Loan Generator</h2>
      
      <div style={{ marginTop: "20px", padding: "20px", backgroundColor: "#fdf4ff", border: "1px solid #fbcfe8", borderRadius: "8px", display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "10px" }}>
        <h3 style={{ margin: "0", color: "#86198f" }}>Total Remaining Debt: ₱{totalDebt.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</h3>
        {showDevTools ? (
          <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
            <button onClick={handleRepairLoanIds} style={{ padding: "10px 16px", backgroundColor: "#6b21a8", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "13px" }}>
              🔧 Repair Loan Links (Run Once After Migration)
            </button>
            <button onClick={() => setShowDevTools(false)} style={{ padding: "6px 10px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "11px" }}>
              Hide
            </button>
          </div>
        ) : (
          <button onClick={() => setShowDevTools(true)} style={{ padding: "6px 10px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "11px" }}>
            Show advanced tools
          </button>
        )}
      </div>

      <div style={{ marginTop: "30px", padding: "20px", backgroundColor: "#f9fafb", borderRadius: "8px", border: "1px solid #e5e7eb" }}>
        <h3 style={{ marginTop: "0", color: "#374151" }}>+ Log a New Loan & Automate Payments</h3>
        
         <div style={{ display: "flex", gap: "10px", marginBottom: "15px" }}>
          <button onClick={() => setIsManualEntry(false)} style={{ padding: "10px 16px", flex: 1, backgroundColor: !isManualEntry ? "#9333ea" : "#e5e7eb", color: !isManualEntry ? "white" : "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
            🆕 New Loan (Auto-Calculate)
          </button>
          <button onClick={() => setIsManualEntry(true)} style={{ padding: "10px 16px", flex: 1, backgroundColor: isManualEntry ? "#9333ea" : "#e5e7eb", color: isManualEntry ? "white" : "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
            📋 Existing Loan (Manual Entry)
          </button>
        </div>

       <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", marginBottom: "10px" }}>
          <input type="text" placeholder="Loan Name (e.g. Maya Personal)" value={name} onChange={(e) => setName(e.target.value)} style={{ padding: "10px", flex: 1, minWidth: "150px", borderRadius: "5px", border: "1px solid #ccc" }} />
          <input type="text" placeholder="Principal (₱)" value={principal} onChange={(e) => formatMoney(e.target.value, setPrincipal)} style={{ padding: "10px", width: "150px", borderRadius: "5px", border: "1px solid #ccc" }} />
          {!isManualEntry && (
            <>
              <input type="text" placeholder="Interest Rate (%)" value={interestRate} onChange={(e) => setInterestRate(e.target.value.replace(/[^0-9.]/g, ""))} style={{ padding: "10px", width: "130px", borderRadius: "5px", border: "1px solid #ccc" }} />
              <select value={interestType} onChange={(e) => setInterestType(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
                <option value="Flat">Flat (one-time)</option>
                <option value="Monthly">Per Month</option>
                <option value="Annual">Per Year</option>
              </select>
            </>
          )}
        </div>

        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", marginBottom: "15px", alignItems: "center" }}>
          {isManualEntry ? (
            <>
              <input type="text" placeholder="Remaining Balance (₱)" value={remainingBalance} onChange={(e) => formatMoney(e.target.value, setRemainingBalance)} style={{ padding: "10px", width: "180px", borderRadius: "5px", border: "1px solid #ccc" }} />
              <span style={{ fontSize: "12px", color: "#9ca3af", fontStyle: "italic" }}>👆 Enter exactly what's left to pay on your existing loan</span>
            </>
          ) : (
            <>
              <input type="text" placeholder="Remaining (₱)" value={remainingBalance} readOnly style={{ padding: "10px", width: "180px", borderRadius: "5px", border: "1px solid #ccc", backgroundColor: "#f3f4f6", color: "#6b7280", cursor: "not-allowed" }} />
              <span style={{ fontSize: "12px", color: "#9ca3af", fontStyle: "italic" }}>👆 Auto-calculated (Principal + Interest)</span>
            </>
          )}
        </div>

        {!isManualEntry && principal && interestRate && term && (
          <div style={{ display: "flex", gap: "25px", flexWrap: "wrap", marginBottom: "15px", padding: "15px", backgroundColor: "#ecfdf5", border: "1px solid #a7f3d0", borderRadius: "8px" }}>
            <div>
              <div style={{ fontSize: "12px", color: "#047857", fontWeight: "bold" }}>Total Interest</div>
              <div style={{ fontSize: "16px", fontWeight: "bold", color: "#065f46" }}>₱{calculateLoanTotals().totalInterest.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
            </div>
            <div>
              <div style={{ fontSize: "12px", color: "#047857", fontWeight: "bold" }}>Total Loan Incurred</div>
              <div style={{ fontSize: "16px", fontWeight: "bold", color: "#065f46" }}>₱{calculateLoanTotals().totalIncurred.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
            </div>
            <div>
              <div style={{ fontSize: "12px", color: "#047857", fontWeight: "bold" }}>Per Month</div>
              <div style={{ fontSize: "16px", fontWeight: "bold", color: "#065f46" }}>₱{calculateLoanTotals().monthlyPayment.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
            </div>
            <div>
              <div style={{ fontSize: "12px", color: "#047857", fontWeight: "bold" }}>Per Bi-Monthly Half</div>
              <div style={{ fontSize: "16px", fontWeight: "bold", color: "#065f46" }}>₱{calculateLoanTotals().biMonthlyPayment.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
            </div>
          </div>
        )}

        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", marginBottom: "15px", alignItems: "center" }}>
          <label style={{ fontWeight: "bold", color: "#374151" }}>Duration:</label>
          <input type="number" placeholder="e.g. 1, 12" value={term} onChange={(e) => setTerm(e.target.value)} style={{ padding: "10px", width: "90px", borderRadius: "5px", border: "1px solid #ccc" }} />
          <select value={termType} onChange={(e) => setTermType(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
            <option value="Months">Months</option>
            <option value="Years">Years</option>
          </select>
          
          <label style={{ fontWeight: "bold", color: "#374151", marginLeft: "10px" }}>Preferred Schedule:</label>
          <select value={frequency} onChange={(e) => setFrequency(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
            <option value="Monthly">Monthly</option>
            <option value="Bi-Monthly">Bi-Monthly</option>
          </select>

          {frequency === "Bi-Monthly" && (
            <>
              <label style={{ fontWeight: "bold", color: "#374151", marginLeft: "10px" }}>Bi-Monthly Type:</label>
              <select value={dueStructure} onChange={(e) => setDueStructure(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
                <option value="Sinking Fund">Sinking Fund (only 2nd half is real debt)</option>
                <option value="True Bi-Monthly">True Bi-Monthly (both halves are real debt)</option>
              </select>
            </>
          )}
        </div>

        <div style={{ display: "flex", gap: "15px", flexWrap: "wrap", marginBottom: "15px", padding: "15px", backgroundColor: "#fff7ed", border: "1px solid #fed7aa", borderRadius: "8px", alignItems: "center" }}>
          <label style={{ fontWeight: "bold", color: "#9a3412" }}>⚙️ Set Your Paycheck Routine:</label>
          <div style={{ display: "flex", alignItems: "center", gap: "5px" }}>
            <label style={{ fontSize: "14px", color: "#c2410c" }}>1st Payday (Day):</label>
            <input type="number" min="1" max="28" value={paydayOne} onChange={(e) => setPaydayOne(e.target.value)} style={{ padding: "8px", width: "60px", borderRadius: "5px", border: "1px solid #fdba74" }} />
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "5px" }}>
            <label style={{ fontSize: "14px", color: "#c2410c" }}>2nd Payday (Day):</label>
            <input type="number" min="2" max="31" value={paydayTwo} onChange={(e) => setPaydayTwo(e.target.value)} placeholder="30" style={{ padding: "8px", width: "60px", borderRadius: "5px", border: "1px solid #fdba74" }} />
            <span style={{ fontSize: "12px", color: "#fb923c", fontStyle: "italic" }}>(Use 30 or 31 for End of Month)</span>
          </div>
        </div>

        <div style={{ display: "flex", gap: "15px", flexWrap: "wrap", alignItems: "center", backgroundColor: "#f3e8ff", padding: "15px", borderRadius: "8px" }}>
          <div style={{ display: "flex", flexDirection: "column" }}>
             <label style={{ fontWeight: "bold", color: "#6b21a8", marginBottom: "5px", fontSize: "14px" }}>💸 Total Monthly Payment (₱)</label>
             {isManualEntry ? (
                <input type="text" value={paymentAmount} onChange={(e) => formatMoney(e.target.value, setPaymentAmount)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", width: "180px" }} />
             ) : (
                <input type="text" value={paymentAmount} readOnly style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", width: "180px", backgroundColor: "#f3f4f6", color: "#6b7280", cursor: "not-allowed" }} />
             )}
          </div>
          <div style={{ display: "flex", flexDirection: "column" }}>
            <label style={{ fontWeight: "bold", color: "#6b21a8", marginBottom: "5px", fontSize: "14px" }}>📅 Next Due Date</label>
            <input type="date" value={dueDate} onChange={(e) => setDueDate(e.target.value)} onBlur={(e) => { if (!isRealDate(e.target.value)) { alert("That date doesn't exist on the calendar! Please pick a valid date."); setDueDate(""); } }} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", cursor: "pointer" }} />  
          </div>
                    <div style={{ display: "flex", flexDirection: "column" }}>
            <label style={{ fontWeight: "bold", color: "#6b21a8", marginBottom: "5px", fontSize: "14px" }}>📆 Date Acquired <span style={{ fontWeight: "normal", color: "#9ca3af" }}>(optional)</span></label>
            <input type="date" value={dateAcquired} onChange={(e) => setDateAcquired(e.target.value)} onBlur={(e) => { if (!isRealDate(e.target.value)) { alert("That date doesn't exist on the calendar! Please pick a valid date."); setDateAcquired(""); } }} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", cursor: "pointer" }} />
          </div>
          <button onClick={handlePreviewSchedule} style={{ padding: "12px 20px", marginLeft: "auto", marginTop: "15px", backgroundColor: "#9333ea", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" }}>
            + Preview Draft Schedule
          </button>
        </div>

        {draftSchedule.length > 0 && (
          <div style={{ marginTop: "20px", padding: "20px", backgroundColor: "white", border: "2px dashed #9333ea", borderRadius: "8px" }}>
            <h4 style={{ margin: "0 0 15px 0", color: "#6b21a8", fontSize: "18px" }}>📝 Review & Edit Paycheck Allocations</h4>
            <p style={{ margin: "0 0 15px 0", color: "#4b5563", fontSize: "14px", fontStyle: "italic" }}>
              The Smart Calendar mapped these to guarantee a 10-day safety gap! You can review or manually tweak any payday below before saving.
            </p>
            
            <div style={{ maxHeight: "300px", overflowY: "auto", marginBottom: "20px", paddingRight: "10px" }}>
              {draftSchedule.map((item, index) => (
                <div key={index} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", borderBottom: "1px solid #e5e7eb", paddingBottom: "10px", marginBottom: "10px" }}>
                  <div>
                    <div style={{ fontWeight: "bold", color: "#374151" }}>Due: {formatDate(item.dueDate)} ({item.label})</div>
                    <div style={{ color: "#dc2626", fontWeight: "bold" }}>Amount: ₱{item.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
                  </div>
                  
                  <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end" }}>
                    <label style={{ fontSize: "12px", color: "#6b7280", marginBottom: "2px", fontWeight: "bold" }}>Allocate to Paycheck:</label>
                    <input 
                      type="date" 
                      value={item.paydayDate}
                      onChange={(e) => handleUpdateDraftDate(index, e.target.value)}
                      onBlur={(e) => { if (!isRealDate(e.target.value)) { alert("That date doesn't exist on the calendar! Please pick a valid date."); } }}
                      style={{ padding: "8px", borderRadius: "5px", border: "1px solid #9333ea", cursor: "pointer", backgroundColor: "#faf5ff" }}
                    />
                    <div style={{ fontSize: "12px", color: "#9333ea", marginTop: "3px" }}>{formatDate(item.paydayDate)}</div>
                  </div>
                </div>
              ))}
            </div>

            <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
              <button onClick={() => setDraftSchedule([])} style={{ padding: "10px 20px", backgroundColor: "#f3f4f6", color: "#4b5563", border: "1px solid #ccc", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                Cancel
              </button>
              <button onClick={handleConfirmAndSaveLoan} style={{ padding: "10px 20px", backgroundColor: "#059669", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "16px" }}>
                ✅ Confirm & Save Loan
              </button>
            </div>
          </div>
        )}
      </div>

      <div style={{ marginTop: "40px" }}>
        <h3 style={{ borderBottom: "2px solid #e5e7eb", paddingBottom: "10px", color: "#1f2937" }}>Active Loans</h3>
        
        {loans.length === 0 ? <p style={{ fontStyle: "italic", color: "#6b7280" }}>No loans logged yet.</p> : null}
        
        <div style={{ display: "grid", gap: "15px", gridTemplateColumns: "repeat(auto-fill, minmax(350px, 1fr))" }}>
          {loans.map(loan => (
            <div key={loan.id} style={{ padding: "20px", backgroundColor: "white", border: "1px solid #e5e7eb", borderRadius: "8px", boxShadow: "0 2px 4px rgba(0,0,0,0.05)" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <div>
                  <div style={{ fontSize: "20px", fontWeight: "bold", color: "#4c1d95", marginBottom: "5px" }}>{loan.name}</div>
                  <div style={{ fontSize: "14px", color: "#6b7280", fontStyle: "italic", marginBottom: "5px" }}>
                    {loan.term} {loan.term_type} • Paid {loan.frequency}
                  </div>
                  <div style={{ fontSize: "13px", color: "#7c3aed", marginBottom: "15px" }}>
                    📆 Acquired: {loan.date_acquired ? formatDate(loan.date_acquired) : <span style={{ color: "#9ca3af", fontStyle: "italic" }}>not set</span>}
                  </div>
                </div>
                
                <div style={{ display: "flex", gap: "10px" }}>
                  <button onClick={() => setExpandedLoanId(expandedLoanId === loan.id ? null : loan.id)} style={{ padding: "6px 12px", backgroundColor: "#e9d5ff", color: "#6b21a8", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>
                    {expandedLoanId === loan.id ? "Hide" : "View"}
                  </button>
                  <button onClick={() => handleDeleteLoan(loan.id, loan.name)} style={{ padding: "6px 12px", backgroundColor: "#fee2e2", color: "#dc2626", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>
                    🗑️ Delete
                  </button>
                </div>
              </div>
              
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "5px", fontSize: "14px", color: "#4b5563" }}>
                <span>Principal:</span>
                <span>₱{loan.principal.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
              </div>
              
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "16px", fontWeight: "bold", color: "#dc2626", marginTop: "10px", paddingTop: "10px", borderTop: "1px dashed #ccc" }}>
                <span>Remaining:</span>
                <span>₱{loan.remaining_balance.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
              </div>

              {expandedLoanId === loan.id && renderFullSchedule(loan)}
            </div>
          ))}
        </div>
      </div>

      <div style={{ marginTop: "50px", borderTop: "3px solid #e5e7eb", paddingTop: "20px" }}>
        <h3 style={{ color: "#047857", marginBottom: "15px" }}>✅ Completed Loans</h3>

        {completedLoans.length === 0 ? (
           <p style={{ fontStyle: "italic", color: "#6b7280" }}>No loans fully paid off yet.</p>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "10px", maxHeight: "300px", overflowY: "auto", paddingRight: "8px" }}>
            {completedLoans.map(cl => (
              <div key={cl.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "15px", backgroundColor: "#ecfdf5", border: "1px solid #a7f3d0", borderRadius: "8px" }}>
                <div>
                  <div style={{ fontSize: "16px", fontWeight: "bold", color: "#065f46" }}>🎉 {cl.name}</div>
                  <div style={{ fontSize: "14px", color: "#047857", marginTop: "5px" }}>
                    Principal: ₱{cl.principal.toLocaleString("en-US")} • {cl.term} {cl.term_type} • Paid {cl.frequency}
                  </div>
                  <div style={{ fontSize: "13px", color: "#059669", marginTop: "3px", fontStyle: "italic" }}>
                    Completed on: {formatDate(cl.completed_date)}
                  </div>
                </div>
                <div style={{ fontSize: "18px", fontWeight: "bold", color: "#065f46" }}>
                  Total Paid: ₱{cl.total_incurred.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>


      <div style={{ marginTop: "50px", borderTop: "3px solid #e5e7eb", paddingTop: "20px" }}>
        <h3 style={{ color: "#b91c1c", marginBottom: "15px" }}>🗑️ Loan Recycle Bin</h3>
        
        {archivedLoans.length === 0 ? (
           <p style={{ fontStyle: "italic", color: "#6b7280" }}>No loans have been deleted.</p>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            {archivedLoans.map(al => (
              <div key={al.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "15px", backgroundColor: "#fef2f2", border: "1px solid #fecaca", borderRadius: "8px" }}>
                <div>
                  <div style={{ fontSize: "16px", fontWeight: "bold", color: "#991b1b", textDecoration: "line-through" }}>{al.name}</div>
                  <div style={{ fontSize: "14px", color: "#b91c1c", marginTop: "5px" }}>
                    Principal: ₱{al.principal.toLocaleString("en-US")}
                  </div>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "20px" }}>
                  <div style={{ fontSize: "18px", fontWeight: "bold", color: "#991b1b" }}>
                    Remaining: ₱{al.remaining_balance.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                  </div>
                  
                  <button onClick={() => handleUndoDeleteLoan(al)} style={{ padding: "6px 12px", backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "14px" }}>
                    ↩️ Undo & Restore
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}