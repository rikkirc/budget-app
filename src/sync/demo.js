// "Try the demo": a made-up vault that lives only in this browser tab. It runs the same vault-core as a real vault
// (through the same replica as the web app), but its changes are numbered here instead of in Firebase, and nothing is
// ever saved: closing the demo, or the tab, throws it away. No account, no invite code, no network.
import { localTimestamp } from "./engine.js";
import { generateScheduleList } from "../loanSchedule.js";

/** The demo's profile name, shown in the top bar */
export const DEMO_PROFILE = "Demo";

/**
 * Runs commands on a replica the way the sync engine does (reads straight from it, changes applied in order with a
 * time stamp), but on this device only. `replica` is a replica whose store saves nothing (see demoReplica).
 */
export function localEngine(replica) {
  let lastSeq = 0;
  return {
    async open() { lastSeq = await replica.open(); },
    async call(cmd, args = {}, at = localTimestamp()) {
      if (cmd.startsWith("get_")) {
        const reply = await replica.call(cmd, args);
        if ("error" in reply) throw reply.error;
        return reply.result;
      }
      const [outcome] = await replica.apply([{ seq: lastSeq + 1, cmd, args, at }]);
      lastSeq += 1;
      if ("error" in outcome) throw outcome.error;
      return outcome.result;
    },
    close() {},
  };
}

const pad = (n) => String(n).padStart(2, "0");
const isoDate = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const addDays = (d, days) => new Date(d.getFullYear(), d.getMonth(), d.getDate() + days);
const at = (d, hours, minutes = 0) => localTimestamp(new Date(d.getFullYear(), d.getMonth(), d.getDate(), hours, minutes));

/** The last payday (15th or month end) strictly before `today` */
export function lastPaydayBefore(today) {
  return today.getDate() > 15 ? new Date(today.getFullYear(), today.getMonth(), 15) : new Date(today.getFullYear(), today.getMonth(), 0);
}

export const DEMO_ACCOUNTS = ["Payroll Account", "GCash", "Cash Wallet", "Sinking Fund Savings"];
export const DEMO_LOAN = "Appliance Loan";

/**
 * Fills an empty demo vault with a realistic month, dated around `today`: four accounts, a salary and some spending,
 * five bills (Electricity still TBD and due within 10 days, so the reminder shows), a bi-monthly Sinking Fund loan,
 * and three receipts (two bills paid, one 1st Half saved to the fund). Only made-up names.
 */
export async function fillDemoVault(engine, today = new Date()) {
  const run = async (cmd, args, when) => {
    const result = await engine.call(cmd, args, when);
    if (typeof result === "string" && result.startsWith("ERROR")) throw new Error(`Demo vault: ${cmd} failed: ${result}`);
    return result;
  };
  const payday = lastPaydayBefore(today);
  const yesterday = addDays(today, -1);

  for (const name of DEMO_ACCOUNTS) await run("add_account", { name }, at(payday, 8));
  const ids = Object.fromEntries((await engine.call("get_accounts")).map((a) => [a.name, a.id]));
  const [payroll, gcash, cash, savings] = DEMO_ACCOUNTS.map((name) => ids[name]);

  await run("log_income", { accountId: payroll, amount: 18500, description: "Salary" }, at(payday, 8, 30));
  await run("transfer_money", { fromAccountId: payroll, toAccountId: gcash, amount: 4000, description: "For bills and groceries" }, at(payday, 9));
  await run("transfer_money", { fromAccountId: payroll, toAccountId: cash, amount: 2500, description: "Pocket money" }, at(payday, 9, 5));
  await run("log_expense", { accountId: gcash, amount: 1850.75, description: "Groceries" }, at(payday, 12));
  await run("log_expense", { accountId: cash, amount: 420, description: "Jeepney and tricycle fare" }, at(payday, 18));

  // A blank payday lets the tracker pick the paycheck, as when you add a bill yourself
  const bills = [
    { name: "Internet", amount: 1299, due: 4, isRecurring: true },
    { name: "Phone Load", amount: 299, due: 3, isRecurring: false },
    { name: "Electricity", amount: 0, due: 9, isRecurring: true }, // TBD until the statement arrives
    { name: "Water", amount: 385.5, due: 12, isRecurring: true },
    { name: "Rent", amount: 6500, due: 18, isRecurring: true },
  ];
  for (const b of bills) {
    await run("add_smart_bill", { name: b.name, amount: b.amount, dueDate: isoDate(addDays(today, b.due)), paydayDate: "", isRecurring: b.isRecurring, split: false }, at(payday, 10));
  }

  // A 12-month appliance loan paid twice a month, saving the 1st Half until the 2nd Half pays the full installment
  const loanDue = isoDate(addDays(today, 25));
  const schedule = generateScheduleList(loanDue, 12, "Months", "Bi-Monthly", 1200, 15, 30);
  await run("add_loan", {
    name: DEMO_LOAN, principal: 12000, remainingBalance: 14400, term: 12, termType: "Months", frequency: "Bi-Monthly",
    paymentAmount: 1200, dueDate: loanDue, paydayDate: "", totalIncurred: 14400, dueStructure: "Sinking Fund",
    dateAcquired: isoDate(addDays(today, -40)),
    billNames: schedule.map((s) => `${DEMO_LOAN} (${s.label})`), billAmounts: schedule.map((s) => s.amount),
    billDues: schedule.map((s) => s.dueDate), billPaydays: schedule.map((s) => s.paydayDate),
  }, at(payday, 10, 30));

  // Receipts, settled last night
  const open = await engine.call("get_smart_bills");
  const bill = (name) => open.find((b) => b.name === name);
  const firstHalf = open.filter((b) => b.name === `${DEMO_LOAN} (1st Half)`).sort((a, b) => a.due_date.localeCompare(b.due_date))[0];
  await run("pay_bill", { billId: bill("Internet").id, accountId: gcash, amount: 1299, description: "Paid: Internet" }, at(yesterday, 20));
  await run("pay_bill", { billId: bill("Phone Load").id, accountId: cash, amount: 299, description: "Paid: Phone Load" }, at(yesterday, 20, 10));
  await run("fund_bill_to_savings", { billId: firstHalf.id, fromAccountId: payroll, toAccountId: savings, amount: firstHalf.amount, description: `Funded to Savings: ${firstHalf.name}` }, at(yesterday, 20, 20));
  await run("mark_first_half_handled", { billId: firstHalf.id, accountName: "Sinking Fund Savings" }, at(yesterday, 20, 20));
}

/** Opens a fresh demo vault on `replica` and fills it. Returns the engine that runs its commands. */
export async function openDemoVault(replica, today = new Date()) {
  const engine = localEngine(replica);
  await engine.open();
  await fillDemoVault(engine, today);
  return engine;
}
