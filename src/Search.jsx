import { useState, useEffect, useMemo } from "react";
import { invoke } from "./api";
import { CATEGORIES, GROUPS, buildIndex, search, parseAmountQuery } from "./searchIndex";

const RESULTS_PER_GROUP = 50;

export default function Search({ onNavigate }) {
  const [category, setCategory] = useState("all");
  const [query, setQuery] = useState("");
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    const safe = async (command) => { try { return await invoke(command); } catch (e) { return []; } };
    const [bills, receipts, loans, completedLoans, transactions] = await Promise.all([
      safe("get_smart_bills"), safe("get_paid_bills"), safe("get_loans"), safe("get_completed_loans"), safe("get_all_transactions"),
    ]);
    setItems(buildIndex({ bills, receipts, loans, completedLoans, transactions }));
    setLoading(false);
  };

  const results = useMemo(() => search(items, query, category), [items, query, category]);
  const activeCategory = CATEGORIES.find(c => c.id === category);
  const needsAmount = category === "amount" && query.trim() && !parseAmountQuery(query);

  const formatMoney = (value) => `₱${value.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  const formatDate = (dateStr) => {
    if (!dateStr) return "";
    const [year, month, day] = dateStr.split("-");
    return new Date(year, month - 1, day).toLocaleDateString("en-US", { month: "short", day: "2-digit", year: "numeric" });
  };

  const describe = (item) => {
    switch (item.kind) {
      case "bill": return `Unpaid · Due ${formatDate(item.date)} · ${formatDate(item.payday)} paycheck`;
      case "receipt": return `Paid · Originally due ${formatDate(item.date)}`;
      case "loan": return `${item.detail} · Next due ${formatDate(item.date)}`;
      case "completedLoan": return `Completed ${formatDate(item.date)}`;
      case "transaction": return `${item.detail} · ${formatDate(item.date)}${item.isReversed ? " · Undone" : ""}`;
      default: return "";
    }
  };

  // The headline amount for each result: what matched, or the most useful amount for that kind
  const headline = (item) => item.matchedAmount || item.amounts[0];

  return (
    <div style={{ backgroundColor: "white", padding: "30px", borderRadius: "10px", boxShadow: "0 4px 6px rgba(0,0,0,0.1)" }}>
      <h2 style={{ color: "#4f46e5", marginTop: "0", fontSize: "28px" }}>🔍 Search</h2>

      <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", marginBottom: "12px" }}>
        {CATEGORIES.map(c => (
          <button key={c.id} onClick={() => setCategory(c.id)} style={{ padding: "8px 16px", backgroundColor: category === c.id ? "#4f46e5" : "#f3f4f6", color: category === c.id ? "white" : "#374151", border: "1px solid #d1d5db", borderRadius: "999px", cursor: "pointer", fontWeight: "bold", fontSize: "14px" }}>
            {c.label}
          </button>
        ))}
      </div>

      <input
        type="text"
        value={query}
        onChange={e => setQuery(e.target.value)}
        placeholder={activeCategory.placeholder}
        inputMode={category === "amount" ? "decimal" : "text"}
        autoFocus
        style={{ width: "100%", boxSizing: "border-box", padding: "14px", fontSize: "16px", borderRadius: "8px", border: "2px solid #c7d2fe", marginBottom: "20px" }}
      />

      {loading ? (
        <p style={{ fontStyle: "italic", color: "#6b7280" }}>Loading your vault...</p>
      ) : !query.trim() ? (
        <p style={{ fontStyle: "italic", color: "#6b7280" }}>Pick what you're looking for above, then start typing.</p>
      ) : needsAmount ? (
        <p style={{ color: "#92400e" }}>💵 Type an amount, like <strong>1500</strong> or <strong>1,234.56</strong>. A whole number matches any centavos.</p>
      ) : results.length === 0 ? (
        <p style={{ fontStyle: "italic", color: "#6b7280" }}>No matches for "{query.trim()}" in {activeCategory.label.replace(/^\S+\s/, "")}.</p>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
          <div style={{ color: "#6b7280", fontSize: "14px" }}>{results.length} {results.length === 1 ? "match" : "matches"}</div>
          {GROUPS.map(group => {
            const groupResults = results.filter(r => r.kind === group.kind);
            if (groupResults.length === 0) return null;
            return (
              <div key={group.kind}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", borderBottom: "2px solid #e5e7eb", paddingBottom: "8px", marginBottom: "10px" }}>
                  <h3 style={{ margin: 0, color: "#1f2937", fontSize: "17px" }}>{group.title} <span style={{ color: "#9ca3af", fontWeight: "normal" }}>({groupResults.length})</span></h3>
                  <button onClick={() => onNavigate(group.tab)} style={{ padding: "5px 12px", backgroundColor: "#eef2ff", color: "#4338ca", border: "1px solid #c7d2fe", borderRadius: "5px", cursor: "pointer", fontWeight: "bold", fontSize: "12px" }}>
                    Open {group.tab === "Budget" ? "Bi-Monthly Budget" : group.tab === "Debt" ? "Loan Generator" : "Cash Flow"} ›
                  </button>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                  {groupResults.slice(0, RESULTS_PER_GROUP).map(item => {
                    const amount = headline(item);
                    return (
                      <div key={item.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: "12px", padding: "12px 15px", backgroundColor: "#f9fafb", borderRadius: "8px", border: "1px solid #e5e7eb", opacity: item.isReversed ? 0.6 : 1 }}>
                        <div style={{ minWidth: 0 }}>
                          <div style={{ fontWeight: "bold", color: "#374151" }}>{item.title}</div>
                          <div style={{ color: "#6b7280", fontSize: "13px", marginTop: "3px" }}>{describe(item)}</div>
                        </div>
                        <div style={{ textAlign: "right", whiteSpace: "nowrap" }}>
                          <div style={{ fontWeight: "bold", color: item.sign === "+" ? "#059669" : "#dc2626" }}>
                            {item.sign ? `${item.sign} ` : ""}{amount.value === 0 ? "TBD" : formatMoney(amount.value)}
                          </div>
                          {(item.matchedAmount || item.amounts.length > 1) && amount.label !== "amount" && (
                            <div style={{ fontSize: "11px", color: item.matchedAmount ? "#4338ca" : "#9ca3af", fontWeight: item.matchedAmount ? "bold" : "normal" }}>
                              {item.matchedAmount ? `matched ${amount.label}` : amount.label}
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })}
                  {groupResults.length > RESULTS_PER_GROUP && (
                    <div style={{ fontSize: "13px", color: "#6b7280", fontStyle: "italic" }}>
                      Showing the first {RESULTS_PER_GROUP} of {groupResults.length}. Type more to narrow it down.
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
