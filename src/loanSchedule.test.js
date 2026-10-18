import test from "node:test";
import assert from "node:assert/strict";
import { generateScheduleList, getPrecedingPayday } from "./loanSchedule.js";

const iso = (d) => d.toISOString().slice(0, 10);

test("a bill's payday is the latest one at least 11 days before it's due", () => {
  assert.equal(iso(getPrecedingPayday(new Date("2026-11-20"), 15, 30)), "2026-10-31");
  assert.equal(iso(getPrecedingPayday(new Date("2026-11-26"), 15, 30)), "2026-11-15");
  assert.equal(iso(getPrecedingPayday(new Date("2026-03-10"), 15, 30)), "2026-02-15");
  assert.equal(iso(getPrecedingPayday(new Date("2026-03-12"), 15, 30)), "2026-02-28");
});

test("a bi-monthly loan gets a 1st Half and a 2nd Half (Due) each month, each half the payment", () => {
  const schedule = generateScheduleList("2026-11-20", 2, "Months", "Bi-Monthly", 1200, 15, 30);
  assert.deepEqual(schedule.map((s) => s.label), ["1st Half", "2nd Half (Due)", "1st Half", "2nd Half (Due)"]);
  assert.deepEqual(schedule.map((s) => s.amount), [600, 600, 600, 600]);
  assert.deepEqual(schedule.map((s) => s.dueDate), ["2026-11-05", "2026-11-20", "2026-12-05", "2026-12-20"]);
  for (const s of schedule) assert.ok(s.paydayDate < s.dueDate);
});

test("a monthly loan gets one installment a month, and years count as 12 months", () => {
  const schedule = generateScheduleList("2026-11-20", 1, "Years", "Monthly", 2000, 15, 30);
  assert.equal(schedule.length, 12);
  assert.ok(schedule.every((s) => s.label === "Due Date" && s.amount === 2000));
});
