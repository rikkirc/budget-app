// A loan's payment schedule, as the Loan Generator previews it and saves it: each installment with its due date and
// the paycheck that pays it. Also used to rebuild a schedule (restoring a deleted loan) and to fill the demo vault.

/** The latest payday at least 11 days before targetDate (the 10-day safety gap). p1, p2: days of the month; 30+ means month end. */
export function getPrecedingPayday(targetDate, p1, p2) {
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
}

/** Every installment: { label, dueDate, paydayDate, amount }. Bi-Monthly loans get a 1st Half and a 2nd Half (Due) each month. */
export function generateScheduleList(baseDateStr, termVal, type, freq, amount, p1, p2) {
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
}
