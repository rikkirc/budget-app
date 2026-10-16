// The web app's replica (vault-core compiled to WebAssembly) against the desktop's (native vault-core): both must
// turn the same list of changes into exactly the same vault, and their checkpoints must open on each other.
// Needs the WebAssembly build: `npm run build:wasm`. Skipped without it, except in CI (REQUIRE_WASM=1), where
// a missing build fails the run instead of passing silently.
import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { createSyncEngine } from "./engine.js";
import { cliReplica, memoryVault, scratchDir } from "./testSupport.js";

const wasmPath = new URL("./vault-web-pkg/vault_web_bg.wasm", import.meta.url);
const built = existsSync(wasmPath);
if (!built && process.env.REQUIRE_WASM === "1") throw new Error("The WebAssembly build is missing: run `npm run build:wasm` first");
const skip = built ? false : "the WebAssembly build is missing (npm run build:wasm)";
const { webReplica } = built ? await import("./webReplica.js") : {};
const wasm = built ? readFileSync(wasmPath) : null;

/** The browser's storage, kept in a Map */
const memoryStore = () => {
  const saved = new Map();
  return { saved, load: async (key) => saved.get(key), save: async (key, bytes) => { saved.set(key, new Uint8Array(bytes)); } };
};

const settle = () => new Promise((r) => setTimeout(r, 50));
const everything = async (engine) => JSON.stringify(await Promise.all(
  ["get_accounts", "get_all_transactions", "get_smart_bills", "get_paid_bills", "get_loans", "get_archived_bills"].map((cmd) => engine.call(cmd)),
));

function device(shared, name, replica, opts = {}) {
  const engine = createSyncEngine({ remote: shared.connect(), replica, clientId: name, ...opts });
  return { engine, replica, close: () => { engine.close(); replica.close?.(); } };
}

test("a browser and a desktop applying the same changes end up with identical vaults", { skip }, async () => {
  const shared = memoryVault();
  const desktop = device(shared, "desktop", cliReplica(join(scratchDir(), "desktop.db")));
  const browser = device(shared, "browser", webReplica("uid1", "Demo", { store: memoryStore(), wasm }));
  await desktop.engine.open();
  await browser.engine.open();

  // A month of real use, split between the two devices, including the half-centavo Sinking Fund maths
  await desktop.engine.call("add_account", { name: "BPI" });
  await browser.engine.call("add_account", { name: "GCash" });
  await browser.engine.call("add_account", { name: "Car Fund" });
  await desktop.engine.call("log_income", { accountId: 1, amount: 25000.255, description: "Salary" });
  await browser.engine.call("transfer_money", { fromAccountId: 1, toAccountId: 2, amount: 1234.5, description: "Load" });
  await browser.engine.call("add_smart_bill", { name: "Rent", amount: 9000.35, dueDate: "2026-11-30", paydayDate: "2026-11-15", isRecurring: true, split: true });
  await desktop.engine.call("add_smart_bill", { name: "Internet", amount: 1699, dueDate: "2026-11-20", paydayDate: "2026-10-31", isRecurring: true });
  await settle();
  const bills = await browser.engine.call("get_smart_bills");
  const first = bills.find((b) => b.name === "Rent (1st Half)");
  const second = bills.find((b) => b.name === "Rent (2nd Half (Due))");
  const internet = bills.find((b) => b.name === "Internet");
  await browser.engine.call("fund_bill_to_savings", { billId: first.id, fromAccountId: 1, toAccountId: 3, amount: first.amount, description: "Funded" });
  await browser.engine.call("mark_first_half_handled", { billId: first.id, accountName: "Car Fund" });
  await desktop.engine.call("rename_account", { accountId: 3, name: "House Fund" });
  await desktop.engine.call("pay_bill", { billId: second.id, accountId: 1, amount: second.amount, description: "Paid: Rent" });
  await browser.engine.call("edit_bill", { billId: internet.id, name: "PLDT Fibr", amount: 1799, dueDate: "2026-12-05" });
  assert.match(await browser.engine.call("transfer_money", { fromAccountId: 2, toAccountId: 1, amount: 999999, description: "Too much" }), /^ERROR/);
  await settle();

  assert.equal(await everything(browser.engine), await everything(desktop.engine));
  assert.match(JSON.stringify(await browser.engine.call("get_paid_bills")), /Handled in House Fund/);
  desktop.close(); browser.close();
});

test("checkpoints move both ways between the browser and the desktop", { skip }, async () => {
  const shared = memoryVault();
  const dir = scratchDir();
  // A desktop makes changes and checkpoints; a new browser starts from the desktop's checkpoint
  const desktop = device(shared, "desktop", cliReplica(join(dir, "desktop.db")), { checkpointEvery: 3 });
  await desktop.engine.open();
  await desktop.engine.call("add_account", { name: "Wallet" });
  for (let i = 0; i < 4; i++) await desktop.engine.call("log_income", { accountId: 1, amount: 100.1, description: `Pay ${i}` });
  await settle();
  assert.ok(shared.state.vault.checkpoint?.id.startsWith("s"), "the desktop saved a checkpoint");

  const browser = device(shared, "browser", webReplica("uid1", "Demo", { store: memoryStore(), wasm }), { checkpointEvery: 3 });
  await browser.engine.open();
  await settle();
  assert.equal(await everything(browser.engine), await everything(desktop.engine));

  // Now the browser makes the next checkpoint, and a new desktop starts from that
  for (let i = 0; i < 4; i++) await browser.engine.call("log_expense", { accountId: 1, amount: 10.05, description: `Snack ${i}` });
  await settle();
  const before = shared.state.vault.checkpoint.seq;
  await new Promise((r) => setTimeout(r, 300));
  assert.ok(shared.state.vault.checkpoint.seq >= before);
  const laptop = device(shared, "laptop", cliReplica(join(dir, "laptop.db")));
  await laptop.engine.open();
  await settle();
  assert.equal(await everything(laptop.engine), await everything(browser.engine));
  desktop.close(); browser.close(); laptop.close();
});

test("the browser keeps its copy between visits, and downloads again if that copy is damaged", { skip }, async () => {
  const shared = memoryVault();
  const store = memoryStore();
  const first = device(shared, "browser", webReplica("uid1", "Demo", { store, wasm }));
  await first.engine.open();
  await first.engine.call("add_account", { name: "GCash" });
  await first.engine.call("log_income", { accountId: 1, amount: 500, description: "Allowance" });
  first.close();
  assert.ok(store.saved.has("uid1/Demo"), "saved under the account and profile");

  // Next visit, offline: the saved copy opens without the network
  const offlineRemote = shared.connect();
  offlineRemote.setOffline(true);
  const again = webReplica("uid1", "Demo", { store, wasm });
  assert.equal(await again.open(), 2, "it remembers how far it got");
  assert.equal((await again.call("get_accounts")).result[0].balance, 500);
  assert.match((await again.call("add_account", { name: "Sneaky" })).error, /only through sync/, "changes go through sync, never straight in");

  // A damaged copy is replaced by a fresh download
  store.saved.set("uid1/Demo", store.saved.get("uid1/Demo").slice(0, 100));
  const fresh = device(shared, "browser2", webReplica("uid1", "Demo", { store, wasm }));
  await fresh.engine.open();
  await settle();
  assert.equal((await fresh.engine.call("get_accounts"))[0].balance, 500);
  fresh.close();
});
