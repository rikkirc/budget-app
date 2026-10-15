import test from "node:test";
import assert from "node:assert/strict";
import { displayBillName } from "./billNames.js";

test("loan and split-bill parts are shown without nested brackets", () => {
  assert.equal(displayBillName("Demo Loan 1 (2nd Half (Due))"), "Demo Loan 1 — 2nd Half (Due)");
  assert.equal(displayBillName("Rent (1st Half)"), "Rent — 1st Half");
  assert.equal(displayBillName("Car Loan (Due Date)"), "Car Loan — Due Date");
  // A handled 1st Half keeps its note
  assert.equal(displayBillName("Rent (1st Half) (Handled in Emergency Fund)"), "Rent — 1st Half (Handled in Emergency Fund)");
  // Transaction descriptions that mention a bill read the same way
  assert.equal(displayBillName("Paid: Demo Loan 1 (2nd Half (Due))"), "Paid: Demo Loan 1 — 2nd Half (Due)");
});

test("ordinary names are left alone", () => {
  assert.equal(displayBillName("Internet (PLDT Fibr Plan 1699)"), "Internet (PLDT Fibr Plan 1699)");
  assert.equal(displayBillName("Electricity"), "Electricity");
  assert.equal(displayBillName(undefined), "");
});
