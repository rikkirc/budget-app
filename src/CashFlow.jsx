import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function CashFlow() {
  const [accounts, setAccounts] = useState([]);
  const [transactions, setTransactions] = useState([]);
  const [newAccountName, setNewAccountName] = useState("");

  const [selectedAccountId, setSelectedAccountId] = useState("");
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [txType, setTxType] = useState("INCOME");
  const [transferToId, setTransferToId] = useState("");
  const [showDevTools, setShowDevTools] = useState(false);
  const [revealedTxIds, setRevealedTxIds] = useState(new Set());
  const [txOffset, setTxOffset] = useState(0);
  const [hasMoreTx, setHasMoreTx] = useState(true);
  const TX_PAGE_SIZE = 50;

  // Which history to show: an account id, or "ALL" for the complete history. null = not picked yet (defaults to the first account)
  const [historyFilter, setHistoryFilter] = useState(null);
  const [accountsLoaded, setAccountsLoaded] = useState(false);
  const latestTxRequest = useRef(0);

  const filterIsValid = historyFilter === "ALL" || accounts.some(a => a.id === historyFilter);
  const activeFilter = filterIsValid ? historyFilter : (accounts.length > 0 ? accounts[0].id : "ALL");
  const historyAccount = accounts.find(a => a.id === activeFilter);

  useEffect(() => {
    fetchAccounts();
  }, []);

  useEffect(() => {
    if (accountsLoaded) fetchTransactions();
  }, [accountsLoaded, activeFilter]);

  const fetchAccounts = async () => {
    const data = await invoke("get_accounts");
    setAccounts(data);
    setAccountsLoaded(true);
  };

  const txQuery = (offset) => ({ offset, limit: TX_PAGE_SIZE, accountId: activeFilter === "ALL" ? null : activeFilter });

  const fetchTransactions = async () => {
    // Ignore replies from older requests if the user switched accounts quickly
    const requestId = ++latestTxRequest.current;
    const data = await invoke("get_transactions_page", txQuery(0));
    if (requestId !== latestTxRequest.current) return;
    setTransactions(data);
    setTxOffset(data.length);
    setHasMoreTx(data.length === TX_PAGE_SIZE);
  };

  const loadMoreTransactions = async () => {
    const requestId = latestTxRequest.current;
    const data = await invoke("get_transactions_page", txQuery(txOffset));
    if (requestId !== latestTxRequest.current) return;
    setTransactions(prev => [...prev, ...data]);
    setTxOffset(prev => prev + data.length);
    setHasMoreTx(data.length === TX_PAGE_SIZE);
  };


  const formatTxDateTime = (stamp) => {
    if (!stamp) return "";
    const [datePart, timePart] = stamp.split(" ");
    if (!datePart || !timePart) return "";
    const [year, month, day] = datePart.split("-").map(Number);
    const [hour, minute] = timePart.split(":").map(Number);
    return new Date(year, month - 1, day, hour, minute).toLocaleString("en-US", { month: "short", day: "2-digit", year: "numeric", hour: "numeric", minute: "2-digit", hour12: true });
  };
  
  const isBillRelatedTx = (t) => {
    // These already have their own dedicated undo in the Receipts section of Bi-Monthly Budget — skip them here to avoid double-undo conflicts
    return t.description.startsWith("Paid:") || t.description.startsWith("Funded to Savings:") || t.description.startsWith("REFUND (Undo):") || t.description.startsWith("Undo Fund Transfer:");
  };

  const handleUndoTransaction = async (t) => {
    if (window.confirm(`Undo this transaction: "${t.description}" (₱${t.amount.toLocaleString("en-US", { minimumFractionDigits: 2 })})? This cannot be re-done automatically.`)) {
        const response = await invoke("undo_transaction", { transactionId: t.id });
        alert(response);
        fetchAccounts();
        fetchTransactions();
    }
  };

  const handleAddAccount = async () => {
    if (!newAccountName) return alert("Please enter an account name!");
    const response = await invoke("add_account", { name: newAccountName });
    alert(response);
    setNewAccountName("");
    fetchAccounts();
  };

  
  const handleRepairRounding = async () => {
    const response = await invoke("repair_balance_rounding");
    alert(response);
    fetchAccounts();
  };

  const handleDeleteAccount = async (id, name) => {
    if (window.confirm(`Are you sure you want to delete ${name}?`)) {
        const response = await invoke("delete_account", { accountId: id });
        alert(response);
        fetchAccounts();
    }
  };

  // 🚨 THE FIX: This forces commas into the box as you type!
  const formatMoneyInput = (value, setter) => {
    let rawText = value.replace(/[^0-9.]/g, "");
    let parts = rawText.split(".");
    if (parts.length > 2) rawText = parts[0] + "." + parts.slice(1).join("");
    if (parts[0]) { 
        parts[0] = parseInt(parts[0], 10).toLocaleString("en-US"); 
        rawText = parts.join("."); 
    }
    setter(rawText);
  };

  const handleTransaction = async () => {
    if (!selectedAccountId || !amount || !description) return alert("Please fill all fields!");
    const cleanAmount = parseFloat(amount.replace(/,/g, ""));

    let response;
    if (txType === "INCOME") {
        response = await invoke("log_income", { accountId: parseInt(selectedAccountId), amount: cleanAmount, description });
    } else if (txType === "EXPENSE") {
        response = await invoke("log_expense", { accountId: parseInt(selectedAccountId), amount: cleanAmount, description });
    } else if (txType === "TRANSFER") {
        if (!transferToId) return alert("Please select an account to transfer to!");
        response = await invoke("transfer_money", { fromAccountId: parseInt(selectedAccountId), toAccountId: parseInt(transferToId), amount: cleanAmount, description });
    }
    
    alert(response);
    setAmount("");
    setDescription("");
    fetchAccounts();
    fetchTransactions();
  };

  return (
    <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)" }}>
      <h2 style={{ color: "#047857", marginTop: "0", fontSize: "28px" }}>Cash Flow Manager</h2>
      
      <div style={{ marginBottom: "20px", padding: "20px", backgroundColor: "#ecfdf5", borderRadius: "8px", border: "1px solid #a7f3d0" }}>
         <h3 style={{ margin: "0 0 10px 0", color: "#065f46" }}>🏦 Add New Account</h3>
         <div style={{ display: "flex", gap: "10px", alignItems: "center", flexWrap: "wrap" }}>
             <input type="text" placeholder="e.g. GCash, BPI..." value={newAccountName} onChange={e => setNewAccountName(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", flex: 1 }} />
             <button onClick={handleAddAccount} style={{ padding: "10px 20px", backgroundColor: "#059669", color: "white", border: "none", borderRadius: "5px", fontWeight: "bold", cursor: "pointer" }}>+ Add</button>
             {showDevTools ? (
               <>
                 <button onClick={handleRepairRounding} style={{ padding: "10px 16px", backgroundColor: "#f59e0b", color: "white", border: "none", borderRadius: "5px", fontWeight: "bold", cursor: "pointer", fontSize: "13px" }}>🧹 Fix Stray Rounding (Run Once)</button>
                 <button onClick={() => setShowDevTools(false)} style={{ padding: "6px 10px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "11px" }}>
                   Hide
                 </button>
               </>
             ) : (
               <button onClick={() => setShowDevTools(true)} style={{ padding: "6px 10px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "11px" }}>
                 Show advanced tools
               </button>
             )}
         </div>
      </div>

      <div style={{ display: "flex", gap: "15px", flexWrap: "wrap", marginBottom: "30px" }}>
          {accounts.map(acc => (
              <div key={acc.id} onClick={() => setHistoryFilter(acc.id)} title={`Show ${acc.name}'s transaction history`} style={{ padding: "20px", backgroundColor: activeFilter === acc.id ? "#ecfdf5" : "#f9fafb", border: activeFilter === acc.id ? "2px solid #059669" : "1px solid #e5e7eb", borderRadius: "8px", minWidth: "200px", position: "relative", cursor: "pointer" }}>
                  <button onClick={(e) => { e.stopPropagation(); handleDeleteAccount(acc.id, acc.name); }} style={{ position: "absolute", top: "10px", right: "10px", background: "none", border: "none", color: "#dc2626", cursor: "pointer", fontSize: "16px" }}>🗑️</button>
                  <div style={{ fontSize: "18px", fontWeight: "bold", color: "#374151" }}>{acc.name}</div>
                  
                  {/* 🚨 THE FIX: Adding toLocaleString() for beautiful commas on your balances! */}
                  <div style={{ fontSize: "24px", fontWeight: "bold", color: "#047857", marginTop: "10px" }}>
                      ₱{acc.balance.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                  </div>
              </div>
          ))}
      </div>

      <div style={{ marginBottom: "30px", padding: "20px", backgroundColor: "#f3f4f6", borderRadius: "8px", border: "1px solid #d1d5db" }}>
         <h3 style={{ margin: "0 0 15px 0", color: "#1f2937" }}>✍️ Log Transaction</h3>
         <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
             <select value={txType} onChange={e => setTxType(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
                 <option value="INCOME">Income (+)</option>
                 <option value="EXPENSE">Expense (-)</option>
                 <option value="TRANSFER">Transfer (↔)</option>
             </select>
             
             <select value={selectedAccountId} onChange={e => setSelectedAccountId(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
                 <option value="">Select Account...</option>
                 {accounts.map(acc => <option key={acc.id} value={acc.id}>{acc.name}</option>)}
             </select>

             {txType === "TRANSFER" && (
                 <>
                 <span style={{ padding: "10px 0", fontWeight: "bold" }}>to</span>
                 <select value={transferToId} onChange={e => setTransferToId(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc" }}>
                     <option value="">Receiving Account...</option>
                     {accounts.filter(a => a.id.toString() !== selectedAccountId).map(acc => <option key={acc.id} value={acc.id}>{acc.name}</option>)}
                 </select>
                 </>
             )}

             <input type="text" placeholder="Amount (₱)" value={amount} onChange={e => formatMoneyInput(e.target.value, setAmount)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", width: "150px" }} />
             <input type="text" placeholder="Description (e.g. Salary, Groceries)" value={description} onChange={e => setDescription(e.target.value)} style={{ padding: "10px", borderRadius: "5px", border: "1px solid #ccc", flex: 1, minWidth: "200px" }} />
             
             <button onClick={handleTransaction} style={{ padding: "10px 20px", backgroundColor: "#1e3a8a", color: "white", border: "none", borderRadius: "5px", fontWeight: "bold", cursor: "pointer" }}>Save</button>
         </div>
      </div>

      <div>
          <h3 style={{ borderBottom: "2px solid #e5e7eb", paddingBottom: "10px", color: "#1f2937", marginBottom: "12px" }}>
              📜 {historyAccount ? `${historyAccount.name} — Transaction History` : "Complete Transaction History"}
          </h3>
          <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", marginBottom: "12px" }}>
              {accounts.map(acc => (
                  <button key={acc.id} onClick={() => setHistoryFilter(acc.id)} style={{ padding: "6px 14px", backgroundColor: activeFilter === acc.id ? "#047857" : "#f3f4f6", color: activeFilter === acc.id ? "white" : "#374151", border: "1px solid #d1d5db", borderRadius: "999px", cursor: "pointer", fontWeight: "bold", fontSize: "13px" }}>
                      {acc.name}
                  </button>
              ))}
              <button onClick={() => setHistoryFilter("ALL")} style={{ padding: "6px 14px", backgroundColor: activeFilter === "ALL" ? "#1e3a8a" : "#f3f4f6", color: activeFilter === "ALL" ? "white" : "#374151", border: "1px solid #d1d5db", borderRadius: "999px", cursor: "pointer", fontWeight: "bold", fontSize: "13px" }}>
                  📚 All Accounts
              </button>
          </div>
          {transactions.length === 0 ? <p style={{ fontStyle: "italic", color: "#6b7280" }}>{historyAccount ? `No transactions for ${historyAccount.name} yet.` : "No transactions yet."}</p> : (
              <div style={{ maxHeight: "300px", overflowY: "auto" }}>
                  {(() => {
                      let undoableCount = 0;
                      return transactions.map(t => {
                          const billRelated = isBillRelatedTx(t);
                          const isUndoable = !billRelated && !t.is_reversed;
                          const isRecent = isUndoable && undoableCount < 3;
                          const isRevealed = revealedTxIds.has(t.id);
                          if (isUndoable) undoableCount++;

                          return (
                              <div key={t.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "10px", borderBottom: "1px solid #e5e7eb", opacity: t.is_reversed ? 0.5 : 1 }}>
                                  <div>
                                      <div>
                                          {(t.tx_type === "TRANSFER_OUT" || t.tx_type === "TRANSFER_IN") && t.linked_account_name ? (
                                              <span style={{ fontWeight: "bold", color: "#374151" }}>
                                                  🔁 {t.tx_type === "TRANSFER_OUT" ? t.account_name : t.linked_account_name} ➡ {t.tx_type === "TRANSFER_OUT" ? t.linked_account_name : t.account_name}
                                              </span>
                                          ) : (
                                              <span style={{ fontWeight: "bold", color: "#374151" }}>{t.account_name}</span>
                                          )}
                                          <span style={{ color: "#6b7280", marginLeft: "10px" }}>{t.description}</span>
                                          {t.is_reversed && <span style={{ color: "#f59e0b", marginLeft: "10px", fontSize: "12px", fontStyle: "italic" }}>(Undone)</span>}
                                      </div>
                                      <div style={{ fontSize: "12px", color: "#9ca3af", marginTop: "3px" }}>
                                          {t.created_at ? `🕒 ${formatTxDateTime(t.created_at)}` : <span style={{ fontStyle: "italic" }}>Date not recorded</span>}
                                      </div>
                                  </div>

                                  <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                                      <div style={{ fontWeight: "bold", color: t.tx_type === "INCOME" || t.tx_type === "TRANSFER_IN" ? "#059669" : "#dc2626" }}>
                                          {t.tx_type === "INCOME" || t.tx_type === "TRANSFER_IN" ? "+" : "-"} ₱{t.amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                      </div>
                                      {isUndoable && (
                                          isRecent ? (
                                              <button onClick={() => handleUndoTransaction(t)} style={{ padding: "5px 10px", backgroundColor: "#f59e0b", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "11px" }}>
                                                  ↩️ Undo
                                              </button>
                                          ) : isRevealed ? (
                                              <>
                                                  <button onClick={() => handleUndoTransaction(t)} style={{ padding: "5px 10px", backgroundColor: "#f59e0b", color: "white", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "11px" }}>
                                                      ↩️ Undo
                                                  </button>
                                                  <button onClick={() => setRevealedTxIds(prev => { const next = new Set(prev); next.delete(t.id); return next; })} style={{ padding: "5px 8px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "10px", fontStyle: "italic" }}>
                                                      Hide undo
                                                  </button>
                                              </>
                                          ) : (
                                              <button onClick={() => setRevealedTxIds(prev => new Set(prev).add(t.id))} style={{ padding: "5px 8px", backgroundColor: "transparent", color: "#9ca3af", border: "1px dashed #d1d5db", borderRadius: "5px", cursor: "pointer", fontSize: "10px", fontStyle: "italic" }}>
                                                  Show undo
                                              </button>
                                          )
                                      )}
                                  </div>
                              </div>
                          );
                      });
                  })()}
                  {hasMoreTx && (
                      <button onClick={loadMoreTransactions} style={{ width: "100%", padding: "10px", marginTop: "10px", backgroundColor: "#f3f4f6", color: "#374151", border: "1px solid #d1d5db", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "13px" }}>
                          Load More Transactions
                      </button>
                  )}
              </div>
          )}
      </div>
    </div>
  );
}