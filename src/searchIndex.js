// Search across bills, loans, and transactions. Kept free of React so it can be tested with `npm test`.

export const CATEGORIES = [
  { id: "all", label: "🔎 Everything", placeholder: "Search names, descriptions, or amounts..." },
  { id: "bills", label: "📅 Bills", placeholder: "Search bill names, e.g. Internet" },
  { id: "loans", label: "📉 Loans", placeholder: "Search loan names, e.g. SLOAN" },
  { id: "amount", label: "💵 Amount", placeholder: "Type an amount, e.g. 1500 or 1,234.56" },
];

export const GROUPS = [
  { kind: "bill", title: "📅 Unpaid Bills", tab: "Budget" },
  { kind: "receipt", title: "🧾 Paid Bills (Receipts)", tab: "Budget" },
  { kind: "loan", title: "📉 Active Loans", tab: "Debt" },
  { kind: "completedLoan", title: "🏁 Completed Loans", tab: "Debt" },
  { kind: "transaction", title: "💰 Transactions", tab: "CashFlow" },
];

const centavos = (value) => Math.round(value * 100);

// "₱1,234.56" -> { value: 1234.56, hasDecimals: true }; anything that isn't an amount -> null
export function parseAmountQuery(query) {
  const cleaned = (query || "").replace(/[₱,\s]/g, "");
  if (!/^\d+(\.\d{1,2})?$/.test(cleaned)) return null;
  return { value: parseFloat(cleaned), hasDecimals: cleaned.includes(".") };
}

// "1500" matches any amount from ₱1,500.00 to ₱1,500.99; "1500.5" matches exactly ₱1,500.50
export function amountMatches(value, parsed) {
  if (!parsed || !value) return false;
  const shown = centavos(value); // compare what the user sees on screen, rounded to the centavo
  return parsed.hasDecimals ? shown === centavos(parsed.value) : Math.floor(shown / 100) === parsed.value;
}

// Every word typed must appear somewhere in the text, in any order, ignoring case
export function textMatches(text, query) {
  const words = (query || "").toLowerCase().split(/\s+/).filter(Boolean);
  const haystack = (text || "").toLowerCase();
  return words.length > 0 && words.every(word => haystack.includes(word));
}

const isHalf = (name) => /\((1st|2nd) Half/.test(name);

// Turn the app's data into one flat list of searchable items
export function buildIndex({ bills = [], receipts = [], loans = [], completedLoans = [], transactions = [] }) {
  return [
    ...bills.map(b => ({
      kind: "bill", id: `bill-${b.id}`, title: b.name, text: b.name, date: b.due_date, payday: b.payday_date,
      amounts: [{ label: "amount", value: b.amount }, ...(isHalf(b.name) ? [{ label: "full installment", value: b.amount * 2 }] : [])],
    })),
    ...receipts.map(r => ({
      kind: "receipt", id: `receipt-${r.id}`, title: r.name, text: r.name, date: r.due_date,
      amounts: [{ label: "amount", value: r.amount }],
    })),
    ...loans.map(l => ({
      kind: "loan", id: `loan-${l.id}`, title: l.name, text: l.name, date: l.next_due_date, detail: `${l.frequency}${l.frequency === "Bi-Monthly" ? ` · ${l.due_structure}` : ""}`,
      amounts: [
        { label: "remaining", value: l.remaining_balance },
        { label: "payment", value: l.payment_amount },
        ...(l.frequency === "Bi-Monthly" ? [{ label: "per half", value: l.payment_amount / 2 }] : []),
        { label: "principal", value: l.principal },
        { label: "total incurred", value: l.total_incurred },
      ],
    })),
    ...completedLoans.map(l => ({
      kind: "completedLoan", id: `completed-${l.id}`, title: l.name, text: l.name, date: l.completed_date,
      amounts: [{ label: "payment", value: l.payment_amount }, { label: "principal", value: l.principal }, { label: "total incurred", value: l.total_incurred }],
    })),
    ...transactions.map(t => ({
      kind: "transaction", id: `tx-${t.id}`, title: t.description, text: `${t.description} ${t.account_name} ${t.linked_account_name}`, date: (t.created_at || "").split(" ")[0],
      detail: t.linked_account_name ? (t.tx_type === "TRANSFER_OUT" ? `${t.account_name} ➡ ${t.linked_account_name}` : `${t.linked_account_name} ➡ ${t.account_name}`) : t.account_name,
      isReversed: t.is_reversed, sign: t.tx_type === "INCOME" || t.tx_type === "TRANSFER_IN" ? "+" : "-",
      amounts: [{ label: "amount", value: t.amount }],
    })),
  ];
}

const KINDS_BY_CATEGORY = {
  all: ["bill", "receipt", "loan", "completedLoan", "transaction"],
  bills: ["bill", "receipt"],
  loans: ["loan", "completedLoan"],
  amount: ["bill", "receipt", "loan", "completedLoan", "transaction"],
};

// Returns matching items, each with `matchedAmount` set when the match came from an amount
export function search(items, query, category = "all") {
  const trimmed = (query || "").trim();
  if (!trimmed) return [];
  const kinds = KINDS_BY_CATEGORY[category] || KINDS_BY_CATEGORY.all;
  const parsed = category === "bills" || category === "loans" ? null : parseAmountQuery(trimmed);

  const results = [];
  for (const item of items) {
    if (!kinds.includes(item.kind)) continue;
    const byText = category !== "amount" && textMatches(item.text, trimmed);
    const matchedAmount = parsed ? item.amounts.find(a => amountMatches(a.value, parsed)) : undefined;
    if (byText || matchedAmount) results.push({ ...item, matchedAmount: byText ? undefined : matchedAmount });
  }
  return results;
}
