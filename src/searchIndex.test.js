import { test } from "node:test";
import assert from "node:assert/strict";
import { parseAmountQuery, amountMatches, textMatches, buildIndex, search } from "./searchIndex.js";

const data = {
  bills: [
    { id: 1, name: "Internet (1st Half)", amount: 750, due_date: "2026-11-05", payday_date: "2026-10-15" },
    { id: 2, name: "Internet (2nd Half (Due))", amount: 750, due_date: "2026-11-20", payday_date: "2026-10-31" },
    { id: 3, name: "SLOAN3 (2nd Half (Due))", amount: 687.645, due_date: "2026-11-20", payday_date: "2026-10-31" },
    { id: 4, name: "Netflix", amount: 549, due_date: "2026-11-18", payday_date: "2026-10-31" },
  ],
  receipts: [{ id: 9, name: "Netflix", amount: 549, due_date: "2026-10-18" }],
  loans: [{ id: 5, name: "SLOAN3", remaining_balance: 4125.87, payment_amount: 1375.29, principal: 4000, total_incurred: 4125.87, frequency: "Bi-Monthly", due_structure: "Sinking Fund", next_due_date: "2026-11-20" }],
  completedLoans: [{ id: 6, name: "GLoan1 - Tuition", payment_amount: 2000, principal: 5500, total_incurred: 6000, completed_date: "2026-09-20" }],
  transactions: [{ id: 7, account_name: "GCash", linked_account_name: "", tx_type: "EXPENSE", amount: 1500.5, description: "Groceries", is_reversed: false, created_at: "2026-10-14 18:20:00" }],
};
const items = buildIndex(data);
const titles = (results) => results.map(r => `${r.kind}:${r.title}`);

test("parseAmountQuery accepts peso signs, commas, and up to 2 decimals", () => {
  assert.deepEqual(parseAmountQuery("₱1,500"), { value: 1500, hasDecimals: false });
  assert.deepEqual(parseAmountQuery(" 1234.56 "), { value: 1234.56, hasDecimals: true });
  assert.equal(parseAmountQuery("internet"), null);
  assert.equal(parseAmountQuery("12.345"), null);
  assert.equal(parseAmountQuery(""), null);
});

test("a whole-peso amount matches any centavos; a decimal amount matches exactly", () => {
  assert.ok(amountMatches(1500.5, parseAmountQuery("1500")));
  assert.ok(amountMatches(1500.5, parseAmountQuery("1500.50")));
  assert.ok(!amountMatches(1500.5, parseAmountQuery("1500.51")));
  assert.ok(!amountMatches(1501, parseAmountQuery("1500")));
});

test("half-centavo amounts match what the screen shows", () => {
  // 687.645 is displayed as ₱687.64 or ₱687.65 depending on float rounding; search follows the same rounding as the screen
  const shown = Number((687.645).toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2, useGrouping: false }));
  assert.ok(amountMatches(687.645, parseAmountQuery(String(shown))));
});

test("text search ignores case and word order", () => {
  assert.ok(textMatches("GLoan1 - Tuition", "tuition gloan1"));
  assert.ok(!textMatches("GLoan1 - Tuition", "laptop"));
  assert.ok(!textMatches("anything", "   "));
});

test("Bills category searches unpaid bills and receipts by name only", () => {
  assert.deepEqual(titles(search(items, "netflix", "bills")), ["bill:Netflix", "receipt:Netflix"]);
  assert.deepEqual(search(items, "549", "bills"), [], "amounts aren't searched in the Bills category");
  assert.deepEqual(search(items, "sloan3", "bills").map(r => r.kind), ["bill"], "loan records are excluded");
});

test("Loans category searches active and completed loans by name", () => {
  assert.deepEqual(titles(search(items, "sloan3", "loans")), ["loan:SLOAN3"]);
  assert.deepEqual(titles(search(items, "tuition", "loans")), ["completedLoan:GLoan1 - Tuition"]);
});

test("Amount category finds bills, loans, and transactions and says which amount matched", () => {
  const results = search(items, "1,500", "amount");
  assert.deepEqual(titles(results), ["bill:Internet — 1st Half", "bill:Internet — 2nd Half (Due)", "transaction:Groceries"]);
  assert.equal(results[0].matchedAmount.label, "full installment");

  const loanHit = search(items, "4125.87", "amount");
  assert.deepEqual(titles(loanHit), ["loan:SLOAN3"]);
  assert.equal(loanHit[0].matchedAmount.label, "remaining");

  assert.deepEqual(search(items, "internet", "amount"), [], "text isn't searched in the Amount category");
});

test("Everything searches names, transaction descriptions and accounts, and amounts", () => {
  assert.deepEqual(titles(search(items, "groceries")), ["transaction:Groceries"]);
  assert.deepEqual(titles(search(items, "gcash")), ["transaction:Groceries"]);
  assert.deepEqual(titles(search(items, "549")), ["bill:Netflix", "receipt:Netflix"]);
  assert.deepEqual(search(items, ""), []);
});
