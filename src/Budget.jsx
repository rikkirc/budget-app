import { useState, useEffect } from "react";
import { invoke } from "./api";

export default function Budget() {
  const [bills, setBills] = useState([]);
  const [accounts, setAccounts] = useState([]);
  const [paidBills, setPaidBills] = useState([]);
  const [archivedBills, setArchivedBills] = useState([]);
  const [loans, setLoans] = useState([]);
  
  const [activeBillId, setActiveBillId] = useState(null);
  const [actionType, setActionType] = useState("PAY"); 
  const [fromAccountId, setFromAccountId] = useState("");
  const [toAccountId, setToAccountId] = useState("");
  const [activeTab, setActiveTab] = useState("");

  // 🚨 NEW: States for the "Log a New Bill" form
  const [newBillName, setNewBillName] = useState("");
  const [newBillAmount, setNewBillAmount] = useState("");
  const [newBillDue, setNewBillDue] = useState("");
  const [newBillPayday, setNewBillPayday] = useState("");
  const [newBillRecurring, setNewBillRecurring] = useState(false);
  const [newBillSplit, setNewBillSplit] = useState(false);
  const [editingAmountId, setEditingAmountId] = useState(null);
  const [editAmountValue, setEditAmountValue] = useState("");
  const [movingBillId, setMovingBillId] = useState(null);
  const [moveTargetPayday, setMoveTargetPayday] = useState("");
  const [revealedReceiptIds, setRevealedReceiptIds] = useState(new Set());
  const [receiptOffset, setReceiptOffset] = useState(0);
  const [hasMoreReceipts, setHasMoreReceipts] = useState(true);
  const RECEIPT_PAGE_SIZE = 50;
  

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setBills(await invoke("get_smart_bills"));
    setAccounts(await invoke("get_accounts"));
    try {
      const receipts = await invoke("get_paid_bills_page", { offset: 0, limit: RECEIPT_PAGE_SIZE });
      setPaidBills(receipts);
      setReceiptOffset(receipts.length);
      setHasMoreReceipts(receipts.length === RECEIPT_PAGE_SIZE);
    } catch (e) { console.log("No paid bills found."); }
    try { setArchivedBills(await invoke("get_archived_bills")); } catch (e) { console.log("No archived bills found."); }
    try { setLoans(await invoke("get_loans")); } catch (e) { console.log("No loans found."); }
  };

  const loadMoreReceipts = async () => {
    const receipts = await invoke("get_paid_bills_page", { offset: receiptOffset, limit: RECEIPT_PAGE_SIZE });
    setPaidBills(prev => [...prev, ...receipts]);
    setReceiptOffset(prev => prev + receipts.length);
    setHasMoreReceipts(receipts.length === RECEIPT_PAGE_SIZE);
  };

  
  // A split recurring bill (series_id set) is one bill paid in two halves, like a loan's Sinking Fund
  const isSplitBill = (bill) => bill.series_id > 0;

  const isSinkingFundFirstHalf = (bill) => {
    if (!bill.name.includes("(1st Half)")) return false;
    if (isSplitBill(bill)) return true;
    const loanName = bill.name.split(" (")[0];
    const matchedLoan = loans.find(l => l.name === loanName);
    return matchedLoan ? matchedLoan.due_structure === "Sinking Fund" : true; // default to safe/old behavior if loan not found
  };

  // 🚨 NEW: Function to handle adding a brand new bill
  const handleAddBill = async (e) => {
    e.preventDefault();
    // Amount is now optional ONLY if this is a recurring bill (TBD placeholder). Otherwise it's still required.
    if (!newBillName || !newBillDue || !newBillPayday || (!newBillRecurring && !newBillAmount)) {
        return alert("Please fill out all fields!");
    }
    await invoke("add_smart_bill", {
        name: newBillName,
        amount: newBillAmount ? parseFloat(newBillAmount) : 0,
        dueDate: newBillDue,
        paydayDate: newBillPayday,
        isRecurring: newBillRecurring,
        split: newBillRecurring && newBillSplit
    });
    alert(newBillRecurring && newBillSplit ? "Success: Bill Added as a 1st Half and 2nd Half (Due)!" : "Success: Bill Added!");
    setNewBillName("");
    setNewBillAmount("");
    setNewBillDue("");
    setNewBillPayday("");
    setNewBillRecurring(false);
    setNewBillSplit(false);
    loadData();
  };

  const handleSplitBill = async (bill) => {
    if (!window.confirm(`Split "${bill.name}" into two halves?\n\n• 1st Half: due 15 days earlier, saved to a fund\n• 2nd Half (Due): paid on the real due date, taking the full amount\n\nEvery future month will be split too.`)) return;
    const response = await invoke("split_recurring_bill", { billId: bill.id });
    alert(response);
    loadData();
  };

  
  const handleSaveAmount = async (billId) => {
    if (!editAmountValue || parseFloat(editAmountValue) <= 0) {
        return alert("Please enter a valid amount greater than 0!");
    }
    await invoke("update_bill_amount", { billId: billId, newAmount: parseFloat(editAmountValue) });
    setEditingAmountId(null);
    setEditAmountValue("");
    loadData();
  };

  const handleProcessBill = async (bill) => {
    if (!fromAccountId) return alert("Please select an account to take the money from!");

    if (actionType === "PAY") {
        // Check if this is a Sinking Fund 2nd Half with a matching saved fund somewhere
        if (bill.name.includes("(2nd Half (Due))") || bill.name.includes("(2nd Half)")) {
            const fundInfo = await invoke("get_sinking_fund_info", { billId: bill.id });

            if (fundInfo.found) {
                const choice = window.confirm(
                    `💡 This is a Sinking Fund bill. Your 1st Half (₱${fundInfo.amount.toLocaleString("en-US", { minimumFractionDigits: 2 })}) is saved in "${fundInfo.account_name}".\n\n` +
                    `Click OK to AUTO-TRANSFER that amount into your selected account first, then complete the payment.\n` +
                    `Click Cancel to just pay from your selected account as-is, leaving the sinking fund untouched.`
                );

                if (choice) {
                    const transferResponse = await invoke("transfer_money", {
                        fromAccountId: fundInfo.account_id,
                        toAccountId: parseInt(fromAccountId),
                        amount: fundInfo.amount,
                        description: `Auto-Transfer: Sinking Fund for ${bill.name}`
                    });

                    if (transferResponse.startsWith("ERROR")) {
                        alert(`Transfer failed: ${transferResponse}`);
                        return; // Stop here, don't attempt the payment
                    }
                }
            }
        }

        const response = await invoke("pay_bill", {
            billId: bill.id,
            accountId: parseInt(fromAccountId),
            amount: bill.amount,
            description: `Paid: ${bill.name}`
        });
        alert(response);
    } else if (actionType === "FUND") {
        if (!toAccountId) return alert("Please select a Savings account to fund!");
        if (fromAccountId === toAccountId) return alert("Source and Destination accounts must be different!");

        const selectedAccount = accounts.find(acc => acc.id === parseInt(toAccountId));

        // 1. Move the money to the sinking fund
        await invoke("fund_bill_to_savings", {
            billId: bill.id,
            fromAccountId: parseInt(fromAccountId),
            toAccountId: parseInt(toAccountId),
            amount: bill.amount,
            description: `Funded to Savings: ${bill.name}`
        });

        // 2. Tag the 1st half as "Handled" visually
        await invoke("mark_first_half_handled", {
            billId: bill.id,
            accountName: selectedAccount ? selectedAccount.name : "Savings"
        });
        
        alert(`Success! 1st Half held safely in ${selectedAccount ? selectedAccount.name : "Savings"}.`);
    }

    setActiveBillId(null);
    setFromAccountId("");
    setToAccountId("");
    loadData();
  };

  const handleMoveBill = async (bill) => {
    if (!moveTargetPayday) return alert("Please pick a paycheck to move this bill to!");
    if (moveTargetPayday === bill.payday_date) return alert("This bill is already allocated to that paycheck.");
    if (moveTargetPayday > bill.due_date && !window.confirm(`That paycheck lands after this bill's due date (${formatDate(bill.due_date).replace(" BILLS", "")}). Move it anyway?`)) return;

    const response = await invoke("move_bill_allocation", { billId: bill.id, newPayday: moveTargetPayday });
    alert(response);
    if (response.startsWith("SUCCESS")) {
        setMovingBillId(null);
        setMoveTargetPayday("");
        loadData();
    }
  };

  const handleDeleteBill = async (id) => {
    const response = await invoke("delete_bill", { billId: id });
    alert(response);
    loadData();
  };

  const handleRestoreBill = async (id) => {
    const response = await invoke("undo_delete_bill", { archiveId: id });
    alert(response);
    loadData();
  };

  
  const handleUndoReceipt = async (receipt) => {
    const isFundTransfer = receipt.from_account_id && receipt.from_account_id !== 0;
    const confirmMsg = isFundTransfer
        ? `Undo this Sinking Fund transfer for "${receipt.name}"? The money will move back to its original account.`
        : `Undo this payment for "${receipt.name}"? The full amount will be refunded to the account it was paid from.`;

    if (window.confirm(confirmMsg)) {
        const response = isFundTransfer
            ? await invoke("undo_fund_to_savings", { paidBillId: receipt.id })
            : await invoke("undo_bill_payment", { paidBillId: receipt.id });
        alert(response);
        loadData();
    }
  };

  const isRealDate = (dateStr) => {
    if (!dateStr || dateStr.length < 10) return true; // empty or still-typing (incomplete) is fine, don't interrupt the user
    const [year, month, day] = dateStr.split('-').map(Number);
    const test = new Date(year, month - 1, day);
    return test.getFullYear() === year && test.getMonth() === month - 1 && test.getDate() === day;
  };

  const formatDate = (dateStr) => {
    if (!dateStr) return "";
    const [year, month, day] = dateStr.split('-');
    const formattedDate = new Date(year, month - 1, day).toLocaleDateString("en-US", { month: "long", day: "2-digit", year: "numeric" });
    return `${formattedDate} BILLS`;
  };
  
  const formatReceiptDate = (dateStr) => {
    if (!dateStr) return "";
    const [year, month, day] = dateStr.split('-');
    return new Date(year, month - 1, day).toLocaleDateString("en-US", { month: "short", day: "2-digit", year: "numeric" });
  };

  const groupedBills = bills.reduce((groups, bill) => {
    const date = bill.payday_date;
    if (!groups[date]) groups[date] = [];
    groups[date].push(bill);
    return groups;
  }, {});

  // Sort each paycheck's bill list so the nearest due_date shows up on top
  Object.keys(groupedBills).forEach(date => {
    groupedBills[date].sort((a, b) => new Date(a.due_date) - new Date(b.due_date));
  });

  const sortedPaydays = Object.keys(groupedBills).sort((a, b) => new Date(a) - new Date(b));
  // Fall back to the first tab if the selected paycheck no longer has bills (e.g. its last bill was paid or moved away)
  const currentTab = activeTab && groupedBills[activeTab] ? activeTab : (sortedPaydays.length > 0 ? sortedPaydays[0] : null);

  return (
    <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)" }}>
      {/* 1. THE NEW TITLE */}
      <h2 style={{ color: "#2563eb", marginTop: "0", fontSize: "28px" }}>📅 Smart Bill Tracker</h2>

      {/* 2. LOG A NEW BILL FORM */}
      <div style={{ marginBottom: "30px", padding: "20px", backgroundColor: "#f3f4f6", borderRadius: "8px", border: "1px solid #d1d5db" }}>
        <h3 style={{ marginTop: 0, color: "#374151" }}>➕ Log a New Bill</h3>
        <form onSubmit={handleAddBill} style={{ display: "flex", gap: "10px", flexWrap: "wrap", alignItems: "flex-end" }}>
            <div style={{ flex: 1, minWidth: "150px" }}>
                <label style={{ display: "block", fontSize: "14px", fontWeight: "bold", color: "#4b5563", marginBottom: "5px" }}>Bill Name</label>
                <input type="text" value={newBillName} onChange={e => setNewBillName(e.target.value)} required placeholder="e.g. Internet" style={{ width: "100%", padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }} />
            </div>
            <div style={{ flex: 1, minWidth: "100px" }}>
                <label style={{ display: "block", fontSize: "14px", fontWeight: "bold", color: "#4b5563", marginBottom: "5px" }}>{newBillRecurring && newBillSplit ? "Full Amount (₱)" : "Amount (₱)"} {newBillRecurring && <span style={{ fontWeight: "normal", color: "#9ca3af" }}>(optional)</span>}</label>
                <input type="number" step="0.01" value={newBillAmount} onChange={e => setNewBillAmount(e.target.value)} required={!newBillRecurring} placeholder={newBillRecurring ? "Leave blank for TBD" : "0.00"} style={{ width: "100%", padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }} />
            </div>
            <div style={{ flex: 1, minWidth: "130px" }}>
                <label style={{ display: "block", fontSize: "14px", fontWeight: "bold", color: "#4b5563", marginBottom: "5px" }}>Actual Due Date</label>
                <input type="date" value={newBillDue} onChange={e => setNewBillDue(e.target.value)} onBlur={e => { if (!isRealDate(e.target.value)) { alert("That date doesn't exist on the calendar! Please pick a valid date."); setNewBillDue(""); } }} required style={{ width: "100%", padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }} />
            </div>
            <div style={{ flex: 1, minWidth: "130px" }}>
                <label style={{ display: "block", fontSize: "14px", fontWeight: "bold", color: "#4b5563", marginBottom: "5px" }}>{newBillRecurring && newBillSplit ? "2nd Half Paycheck" : "Paycheck Allocation"}</label>
                <input type="date" value={newBillPayday} onChange={e => setNewBillPayday(e.target.value)} onBlur={e => { if (!isRealDate(e.target.value)) { alert("That date doesn't exist on the calendar! Please pick a valid date."); setNewBillPayday(""); } }} required style={{ width: "100%", padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }} />
            </div>
            <div style={{ width: "100%", display: "flex", alignItems: "center", gap: "8px", marginTop: "5px" }}>
                <input type="checkbox" id="recurringCheck" checked={newBillRecurring} onChange={e => { setNewBillRecurring(e.target.checked); if (!e.target.checked) setNewBillSplit(false); }} style={{ width: "18px", height: "18px", cursor: "pointer" }} />
                <label htmlFor="recurringCheck" style={{ fontSize: "14px", fontWeight: "bold", color: "#4b5563", cursor: "pointer" }}>🔁 Recurring Bill (same day every month)</label>
            </div>
            {newBillRecurring && (
                <div style={{ width: "100%", marginLeft: "26px" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                        <input type="checkbox" id="splitCheck" checked={newBillSplit} onChange={e => setNewBillSplit(e.target.checked)} style={{ width: "18px", height: "18px", cursor: "pointer" }} />
                        <label htmlFor="splitCheck" style={{ fontSize: "14px", fontWeight: "bold", color: "#4b5563", cursor: "pointer" }}>✂️ Split into Bi-Monthly Halves (Sinking Fund)</label>
                    </div>
                    {newBillSplit && (
                        <p style={{ margin: "6px 0 0 26px", fontSize: "12px", color: "#2563eb", fontStyle: "italic" }}>
                            ℹ️ Creates a 1st Half (due 15 days earlier, saved to a fund from its own paycheck) and a 2nd Half (Due) that pays the full bill on the real due date, just like a Sinking Fund loan.
                        </p>
                    )}
                </div>
            )}
            <button type="submit" style={{ padding: "10px 20px", backgroundColor: "#10b981", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", height: "40px" }}>
                Save Bill
            </button>
        </form>
      </div>

      {/* 3. PAYCHECK ALLOCATIONS TABS */}
      <div style={{ marginBottom: "40px" }}>
        {sortedPaydays.length === 0 ? <p style={{ fontStyle: "italic", color: "#6b7280" }}>No upcoming bills. You are clear!</p> : (
          <div>
            <div style={{ display: "flex", gap: "10px", borderBottom: "2px solid #e5e7eb", paddingBottom: "15px", marginBottom: "20px", overflowX: "auto" }}>
                {sortedPaydays.map(payday => (
                    <button 
                        key={payday}
                        onClick={() => { setActiveTab(payday); setActiveBillId(null); setMovingBillId(null); }}
                        style={{ padding: "10px 20px", backgroundColor: currentTab === payday ? "#2563eb" : "#f3f4f6", color: currentTab === payday ? "white" : "#4b5563", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", whiteSpace: "nowrap" }}
                    >
                        💼 {formatDate(payday)}
                    </button>
                ))}
            </div>

            {currentTab && (
                <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                    <div style={{ paddingBottom: "10px", marginBottom: "10px", borderBottom: "1px solid #e5e7eb", color: "#4b5563", fontWeight: "bold", textAlign: "right" }}>
                        Total Tab Due: ₱{groupedBills[currentTab].reduce((sum, b) => sum + b.amount, 0).toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                    </div>

                    {groupedBills[currentTab].map(bill => (
                        <div key={bill.id} style={{ padding: "15px", backgroundColor: "#f9fafb", borderRadius: "8px", border: "1px solid #e5e7eb" }}>
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: activeBillId === bill.id || movingBillId === bill.id ? "15px" : "0" }}>
                                <div>
                                    <div style={{ fontWeight: "bold", color: "#374151", fontSize: "16px" }}>{bill.name}</div>
                                    <div style={{ color: "#dc2626", fontSize: "14px", marginTop: "5px" }}>Target Due Date: {formatDate(bill.due_date).replace(" BILLS", "")}</div>
                                </div>
                                <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                    {editingAmountId === bill.id ? (
                                        <div style={{ display: "flex", alignItems: "center", gap: "5px" }}>
                                            <input type="number" step="0.01" value={editAmountValue} onChange={e => setEditAmountValue(e.target.value)} placeholder={isSplitBill(bill) ? "Full bill" : "0.00"} title={isSplitBill(bill) ? "Enter the FULL bill amount. Each half becomes half of it." : undefined} autoFocus style={{ width: "110px", padding: "6px", borderRadius: "5px", border: "1px solid #9333ea" }} />
                                            <button onClick={() => handleSaveAmount(bill.id)} style={{ padding: "6px 10px", backgroundColor: "#059669", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>✓</button>
                                            <button onClick={() => { setEditingAmountId(null); setEditAmountValue(""); }} style={{ padding: "6px 10px", backgroundColor: "#e5e7eb", color: "#4b5563", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>✕</button>
                                        </div>
                                    ) : bill.amount === 0 ? (
                                        <button onClick={() => { setEditingAmountId(bill.id); setEditAmountValue(""); }} style={{ padding: "8px 12px", backgroundColor: "#fef3c7", color: "#92400e", border: "1px solid #fde68a", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "13px" }}>
                                            {isSplitBill(bill) ? "⚠️ TBD — Enter Full Bill Amount" : "⚠️ TBD — Click to Enter Amount"}
                                        </button>
                                    ) : (
                                        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                                            <div style={{ textAlign: "right" }}>
                                                <div style={{ fontWeight: "bold", color: "#dc2626", fontSize: "18px" }}>
                                                    ₱{bill.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                                </div>
                                                {isSplitBill(bill) && (
                                                    <div style={{ fontSize: "11px", color: "#6b7280" }}>
                                                        half of ₱{(bill.amount * 2).toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                                    </div>
                                                )}
                                            </div>
                                            {bill.is_recurring && (
                                                <button onClick={() => { setEditingAmountId(bill.id); setEditAmountValue((isSplitBill(bill) ? bill.amount * 2 : bill.amount).toString()); }} style={{ padding: "4px 8px", backgroundColor: "#f3f4f6", color: "#6b7280", border: "1px solid #d1d5db", borderRadius: "4px", cursor: "pointer", fontSize: "11px" }}>
                                                    ✏️ Edit
                                                </button>
                                            )}
                                        </div>
                                    )}
                                    {bill.is_recurring && <span style={{ fontSize: "16px" }} title={isSplitBill(bill) ? "Recurring bill, split into halves" : "Recurring bill"}>{isSplitBill(bill) ? "🔁✂️" : "🔁"}</span>}
                                    {bill.is_recurring && bill.loan_id === 0 && !isSplitBill(bill) && (
                                        <button onClick={() => handleSplitBill(bill)} title="Split into a 1st Half and 2nd Half (Due)" style={{ padding: "8px 12px", backgroundColor: "#eff6ff", color: "#1d4ed8", border: "1px solid #bfdbfe", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                            ✂️ Split
                                        </button>
                                    )}
                                    <button onClick={() => handleDeleteBill(bill.id)} style={{ padding: "8px 12px", backgroundColor: "#ef4444", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                        🗑️ Delete
                                    </button>
                                    <button onClick={() => {
                                        const opening = movingBillId !== bill.id;
                                        setMovingBillId(opening ? bill.id : null);
                                        setMoveTargetPayday("");
                                        if (opening) setActiveBillId(null);
                                    }} style={{ padding: "8px 12px", backgroundColor: movingBillId === bill.id ? "#4b5563" : "#f3f4f6", color: movingBillId === bill.id ? "white" : "#374151", border: "1px solid #d1d5db", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                        {movingBillId === bill.id ? "Cancel" : "📅 Move"}
                                    </button>
                                    <button onClick={() => {
                                        const opening = activeBillId !== bill.id;
                                        setActiveBillId(opening ? bill.id : null);
                                        if (opening) setMovingBillId(null);
                                        if (opening) setActionType(isSinkingFundFirstHalf(bill) ? "FUND" : "PAY");
                                    }} disabled={bill.amount === 0} style={{ padding: "8px 16px", backgroundColor: bill.amount === 0 ? "#d1d5db" : "#2563eb", color: "white", border: "none", borderRadius: "5px", cursor: bill.amount === 0 ? "not-allowed" : "pointer", fontWeight: "bold" }}>
                                        {activeBillId === bill.id ? "Cancel" : "Handle Bill"}
                                    </button>
                                </div>
                            </div>

                            {movingBillId === bill.id && (
                                <div style={{ padding: "15px", backgroundColor: "#f9fafb", borderRadius: "8px", border: "1px solid #d1d5db" }}>
                                    <p style={{ margin: "0 0 10px 0", fontSize: "13px", color: "#4b5563" }}>
                                        Move this bill from the <strong>{formatDate(bill.payday_date).replace(" BILLS", "")}</strong> paycheck to another one. Only this occurrence moves.
                                    </p>
                                    <div style={{ display: "flex", gap: "10px", alignItems: "center", flexWrap: "wrap" }}>
                                        <select value={sortedPaydays.includes(moveTargetPayday) ? moveTargetPayday : ""} onChange={e => setMoveTargetPayday(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", flex: 1, minWidth: "200px" }}>
                                            <option value="">Pick an existing paycheck...</option>
                                            {sortedPaydays.filter(p => p !== bill.payday_date).map(p => <option key={p} value={p}>💼 {formatDate(p)}</option>)}
                                        </select>
                                        <span style={{ color: "#6b7280", fontSize: "13px" }}>or</span>
                                        <input type="date" value={moveTargetPayday} onChange={e => setMoveTargetPayday(e.target.value)} onBlur={e => { if (!isRealDate(e.target.value)) { alert("That date doesn't exist on the calendar! Please pick a valid date."); setMoveTargetPayday(""); } }} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }} />
                                        <button onClick={() => handleMoveBill(bill)} style={{ padding: "10px 20px", backgroundColor: "#4b5563", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                            Confirm Move
                                        </button>
                                    </div>
                                </div>
                            )}

                            {activeBillId === bill.id && (
                                <div style={{ padding: "15px", backgroundColor: "#eff6ff", borderRadius: "8px", border: "1px solid #bfdbfe" }}>
                                    <div style={{ display: "flex", gap: "10px", marginBottom: "15px" }}>
                                        {!isSinkingFundFirstHalf(bill) && (
                                            <button onClick={() => setActionType("PAY")} style={{ padding: "8px 16px", flex: 1, backgroundColor: actionType === "PAY" ? "#1d4ed8" : "#e5e7eb", color: actionType === "PAY" ? "white" : "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                                🏦 Pay Bank (Lowers Debt)
                                            </button>
                                        )}
                                        <button onClick={() => setActionType("FUND")} style={{ padding: "8px 16px", flex: 1, backgroundColor: actionType === "FUND" ? "#047857" : "#e5e7eb", color: actionType === "FUND" ? "white" : "#374151", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                            📥 Save to Fund (Moves Cash)
                                        </button>
                                    </div>
                                    {isSinkingFundFirstHalf(bill) && (
                                        <p style={{ margin: "0 0 10px 0", fontSize: "12px", color: "#2563eb", fontStyle: "italic" }}>
                                            ℹ️ This is a Sinking Fund 1st Half — it can only be saved to a fund, since the real due date is the 2nd Half.
                                        </p>
                                    )}

                                    <div style={{ display: "flex", gap: "10px", alignItems: "center", flexWrap: "wrap" }}>
                                        <select value={fromAccountId} onChange={e => setFromAccountId(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", flex: 1, minWidth: "200px" }}>
                                            <option value="">Take money from...</option>
                                            {accounts.map(acc => <option key={acc.id} value={acc.id}>{acc.name} (₱{acc.balance.toLocaleString()})</option>)}
                                        </select>

                                        {actionType === "FUND" && (
                                            <>
                                                <span style={{ fontWeight: "bold", color: "#374151" }}>➡</span>
                                                <select value={toAccountId} onChange={e => setToAccountId(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", flex: 1, minWidth: "200px" }}>
                                                    <option value="">Move to Savings...</option>
                                                    {accounts.map(acc => <option key={acc.id} value={acc.id}>{acc.name} (₱{acc.balance.toLocaleString()})</option>)}
                                                </select>
                                            </>
                                        )}

                                        <button onClick={() => handleProcessBill(bill)} style={{ padding: "10px 20px", backgroundColor: actionType === "PAY" ? "#1d4ed8" : "#047857", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>
                                            Confirm {actionType === "PAY" ? "Payment" : "Transfer"}
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            )}
          </div>
        )}
      </div>

      {/* 4. PAID BILLS (RECEIPTS) */}
      <div style={{ marginTop: "30px", paddingTop: "20px", borderTop: "2px dashed #d1d5db" }}>
        <h3 style={{ color: "#374151", marginBottom: "15px" }}>🧾 Receipts (Paid & Settled Bills)</h3>
        {paidBills.length === 0 ? (
            <p style={{ fontStyle: "italic", color: "#9ca3af" }}>No receipts to display yet.</p>
        ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "10px", maxHeight: "280px", overflowY: "auto", paddingRight: "8px" }}>
                {paidBills.map((receipt, index) => {
                    const isRecent = index < 3;
                    const isRevealed = revealedReceiptIds.has(receipt.id);
                    return (
                        <div key={index} style={{ padding: "12px 15px", backgroundColor: "#f3f4f6", borderRadius: "6px", borderLeft: "4px solid #10b981", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                            <div>
                                <div style={{ fontWeight: "bold", color: "#1f2937" }}>{receipt.name}</div>
                                <div style={{ color: "#6b7280", fontSize: "13px" }}>Original Due: {formatReceiptDate(receipt.due_date)}</div>
                            </div>
                            <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                                <div style={{ fontWeight: "bold", color: "#10b981" }}>
                                    ₱{receipt.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                </div>
                                {isRecent ? (
                                    <button onClick={() => handleUndoReceipt(receipt)} style={{ padding: "6px 10px", backgroundColor: "#f59e0b", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>
                                        ↩️ Undo
                                    </button>
                                ) : isRevealed ? (
                                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                                        <button onClick={() => handleUndoReceipt(receipt)} style={{ padding: "6px 10px", backgroundColor: "#f59e0b", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>
                                            ↩️ Undo
                                        </button>
                                        <button onClick={() => setRevealedReceiptIds(prev => { const next = new Set(prev); next.delete(receipt.id); return next; })} style={{ padding: "6px 8px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "11px", fontStyle: "italic" }}>
                                            Hide undo
                                        </button>
                                    </div>
                                ) : (
                                    <button onClick={() => setRevealedReceiptIds(prev => new Set(prev).add(receipt.id))} style={{ padding: "6px 8px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "11px", fontStyle: "italic" }}>
                                        Show undo
                                    </button>
                                )}
                            </div>
                        </div>
                    );
                })}
                {hasMoreReceipts && (
                    <button onClick={loadMoreReceipts} style={{ padding: "10px", backgroundColor: "#f3f4f6", color: "#374151", border: "1px solid #d1d5db", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "13px" }}>
                        Load More Receipts
                    </button>
                )}
            </div>
        )}
      </div>

      {/* 5. DELETE BILLS RECYCLE BIN */}
      <div style={{ marginTop: "30px", paddingTop: "20px", borderTop: "2px dashed #d1d5db" }}>
        <h3 style={{ color: "#374151", marginBottom: "15px" }}>🗑️ Recycle Bin (Deleted Bills)</h3>
        {archivedBills.length === 0 ? (
            <p style={{ fontStyle: "italic", color: "#9ca3af" }}>No deleted bills.</p>
        ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "10px", maxHeight: "280px", overflowY: "auto", paddingRight: "8px" }}>
                {archivedBills.map((archived) => (
                    <div key={archived.id} style={{ padding: "12px 15px", backgroundColor: "#fee2e2", borderRadius: "6px", borderLeft: "4px solid #ef4444", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <div>
                            <div style={{ fontWeight: "bold", color: "#1f2937", textDecoration: "line-through" }}>{archived.name}</div>
                            <div style={{ color: "#6b7280", fontSize: "13px" }}>Original Due: {formatReceiptDate(archived.due_date)}</div>
                        </div>
                        <div style={{ display: "flex", alignItems: "center", gap: "15px" }}>
                            <div style={{ fontWeight: "bold", color: "#ef4444" }}>
                                ₱{archived.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                            </div>
                            <button onClick={() => handleRestoreBill(archived.id)} style={{ padding: "6px 12px", backgroundColor: "#10b981", color: "white", border: "none", borderRadius: "4px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>
                                ↩️ Restore
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