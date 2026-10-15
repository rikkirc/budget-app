// Loan installments and split bills are stored with the part in brackets, e.g. "Car Loan (2nd Half (Due))".
// The payment rules recognise each half by those exact names, so they stay as stored; only what people read changes:
// "Car Loan (2nd Half (Due))" is shown as "Car Loan — 2nd Half (Due)", without the nested brackets.
export function displayBillName(name) {
  return String(name ?? "").replace(/ \((1st Half|2nd Half \(Due\)|Due Date)\)/, " — $1");
}
