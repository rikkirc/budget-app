// "Try the demo": the made-up vault must load cleanly on any day of the year, hold what the welcome screen promises
// (accounts, bills with one TBD, a split loan, receipts), work like a real vault, and never be saved anywhere.
// Runs on native vault-core (as the desktop does) and, when it's built, on the WebAssembly build the web app uses.
import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { DEMO_ACCOUNTS, DEMO_LOAN, lastPaydayBefore, localEngine, openDemoVault } from "./demo.js";
import { cliReplica, scratchDir } from "./testSupport.js";

const wasmPath = new URL("./vault-web-pkg/vault_web_bg.wasm", import.meta.url);
const built = existsSync(wasmPath);
if (!built && process.env.REQUIRE_WASM === "1") throw new Error("The WebAssembly build is missing: run `npm run build:wasm` first");
const skip = built ? false : "the WebAssembly build is missing (npm run build:wasm)";
const { demoReplica, nothingSaved } = built ? await import("./webReplica.js") : {};
const wasm = built ? readFileSync(wasmPath) : null;

const day = (s) => { const [y, m, d] = s.split("-").map(Number); return new Date(y, m - 1, d); };
const daysBetween = (from, isoTo) => Math.round((day(isoTo) - from) / 86400000);

async function contents(engine) {
  const [accounts, bills, loans, receipts, transactions] = await Promise.all(
    ["get_accounts", "get_smart_bills", "get_loans", "get_paid_bills", "get_all_transactions"].map((cmd) => engine.call(cmd)),
  );
  return { accounts, bills, loans, receipts, transactions };
}

function checkDemo({ accounts, bills, loans, receipts, transactions }, today) {
  assert.deepEqual(accounts.map((a) => a.name).sort(), [...DEMO_ACCOUNTS].sort());
  for (const a of accounts) assert.ok(a.balance >= 0, `${a.name} is overdrawn: ${a.balance}`);
  assert.ok(transactions.length >= 5);

  // One TBD bill due within the next 10 days, so the "enter the amount" reminder shows on opening. (Paying a recurring
  // bill adds next month's copy as TBD too, as in a real vault, but that one is due later.)
  const tbdSoon = bills.filter((b) => b.is_recurring && b.amount === 0 && daysBetween(today, b.due_date) <= 10);
  assert.deepEqual(tbdSoon.map((b) => b.name), ["Electricity"]);
  assert.ok(daysBetween(today, tbdSoon[0].due_date) >= 0);

  // A bi-monthly Sinking Fund loan: a 1st Half and a 2nd Half each month
  assert.equal(loans.length, 1);
  assert.equal(loans[0].name, DEMO_LOAN);
  assert.equal(loans[0].frequency, "Bi-Monthly");
  assert.equal(loans[0].due_structure, "Sinking Fund");
  const halves = bills.filter((b) => b.name.startsWith(DEMO_LOAN));
  // (the first 1st Half is already saved to the fund, so it's a receipt now)
  assert.equal(halves.filter((b) => b.name.endsWith("(1st Half)")).length, 11);
  assert.equal(halves.filter((b) => b.name.endsWith("(2nd Half (Due))")).length, 12);
  // Every bill lands on a payday before it's due
  for (const b of bills) assert.ok(b.payday_date <= b.due_date, `${b.name}: payday ${b.payday_date} after due ${b.due_date}`);

  // Receipts: two bills paid and one 1st Half saved to the fund, each with the time it was settled
  assert.equal(receipts.length, 3);
  assert.deepEqual(receipts.map((r) => r.name).sort(), [`${DEMO_LOAN} (1st Half) (Handled in Sinking Fund Savings)`, "Internet", "Phone Load"].sort());
  for (const r of receipts) assert.ok(r.paid_at, `${r.name} has no settled time`);
  assert.equal(accounts.find((a) => a.name === "Sinking Fund Savings").balance, 600);
}

test("the last payday before a date is the 15th or the end of the previous month", () => {
  const iso = (d) => d.toISOString().slice(0, 10);
  assert.equal(lastPaydayBefore(day("2026-10-18")).getDate(), 15);
  assert.equal(lastPaydayBefore(day("2026-10-16")).getDate(), 15);
  assert.equal(lastPaydayBefore(day("2026-10-15")).getDate(), 30); // Sep 30
  assert.equal(lastPaydayBefore(day("2026-03-01")).getMonth(), 1); // Feb 28
  assert.equal(lastPaydayBefore(day("2027-01-05")).getFullYear(), 2026); // Dec 31
  assert.ok(iso(lastPaydayBefore(day("2026-10-31"))));
});

test("the demo vault loads with realistic data on native vault-core", async () => {
  const today = day("2026-10-18");
  const replica = cliReplica(join(scratchDir(), "demo.db"));
  try {
    const engine = await openDemoVault(replica, today);
    checkDemo(await contents(engine), today);
  } finally {
    replica.close();
  }
});

test("the demo vault loads on any day of the year, including month and year ends", async () => {
  for (const date of ["2026-01-01", "2026-02-28", "2028-02-29", "2026-03-31", "2026-06-15", "2026-06-16", "2026-12-31"]) {
    const replica = cliReplica(join(scratchDir(), "demo.db"));
    try {
      const engine = await openDemoVault(replica, day(date));
      checkDemo(await contents(engine), day(date));
    } catch (e) {
      throw new Error(`On ${date}: ${e.message ?? e}`);
    } finally {
      replica.close();
    }
  }
});

test("the demo works like a real vault: changes apply, and mistakes are refused as usual", async () => {
  const replica = cliReplica(join(scratchDir(), "demo.db"));
  try {
    const engine = await openDemoVault(replica, day("2026-10-18"));
    const wallet = (await engine.call("get_accounts")).find((a) => a.name === "Cash Wallet");
    assert.match(await engine.call("log_expense", { accountId: wallet.id, amount: 100, description: "Snacks" }), /^SUCCESS/);
    const after = (await engine.call("get_accounts")).find((a) => a.name === "Cash Wallet");
    assert.equal(after.balance, wallet.balance - 100);
    // Paying more than the account holds is refused, exactly as in a real vault
    const rent = (await engine.call("get_smart_bills")).find((b) => b.name === "Rent");
    assert.match(await engine.call("pay_bill", { billId: rent.id, accountId: wallet.id, amount: rent.amount, description: "Paid: Rent" }), /Insufficient/i);
  } finally {
    replica.close();
  }
});

test("the web app's demo vault matches the native one and is never saved", { skip }, async () => {
  const today = day("2026-10-18");
  const native = cliReplica(join(scratchDir(), "demo.db"));
  try {
    const fromNative = await contents(await openDemoVault(native, today));
    const web = await openDemoVault(demoReplica({ wasm }), today);
    const fromWeb = await contents(web);
    checkDemo(fromWeb, today);
    assert.deepEqual(fromWeb, fromNative);

    // Its store keeps nothing, so a change in the demo is gone next time: a new demo starts empty
    await web.call("add_account", { name: "Extra" });
    assert.equal(await nothingSaved.load("demo/demo"), undefined);
    const again = localEngine(demoReplica({ wasm }));
    await again.open();
    assert.deepEqual(await again.call("get_accounts"), []);
  } finally {
    native.close();
  }
});
