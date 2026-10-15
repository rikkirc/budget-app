// The sync engine against real vault-core replicas and an in-memory shared list of changes.
// (The same engine runs against the Firestore emulator in scripts/test-firebase.mjs.)
import test from "node:test";
import assert from "node:assert/strict";
import { join } from "node:path";
import { createSyncEngine, uploadVault, OFFLINE_MESSAGE } from "./engine.js";
import { cliReplica, memoryVault, scratchDir } from "./testSupport.js";

const settle = () => new Promise((r) => setTimeout(r, 50));

function device(shared, name, dir, opts = {}) {
  const replica = cliReplica(join(dir, `${name}.db`));
  const remote = shared.connect();
  let changes = 0;
  const engine = createSyncEngine({ remote, replica, clientId: name, onChange: () => changes++, ...opts });
  return { engine, replica, remote, changes: () => changes, close: () => { engine.close(); replica.close(); } };
}

const balances = async (d) => (await d.engine.call("get_accounts")).map((a) => `${a.name}=${a.balance}`).join(", ");

test("two devices stay identical, and each hears only about the other's changes", async () => {
  const dir = scratchDir();
  const shared = memoryVault();
  const phone = device(shared, "phone", dir);
  const desktop = device(shared, "desktop", dir);
  await phone.engine.open();
  await desktop.engine.open();

  assert.match(await phone.engine.call("add_account", { name: "GCash" }), /^SUCCESS/);
  assert.match(await desktop.engine.call("add_account", { name: "BPI" }), /^SUCCESS/);
  await desktop.engine.call("log_income", { accountId: 1, amount: 1000, description: "Salary" });
  await phone.engine.call("transfer_money", { fromAccountId: 1, toAccountId: 2, amount: 250.5, description: "Savings" });
  await settle();

  assert.equal(await balances(phone), "GCash=749.5, BPI=250.5");
  assert.equal(await balances(desktop), await balances(phone));
  assert.equal(JSON.stringify(await phone.engine.call("get_all_transactions")), JSON.stringify(await desktop.engine.call("get_all_transactions")));
  assert.ok(phone.changes() >= 1 && desktop.changes() >= 1);
  phone.close(); desktop.close();
});

test("a change the rules reject gives the same answer and changes nothing on every device", async () => {
  const dir = scratchDir();
  const shared = memoryVault();
  const phone = device(shared, "phone", dir);
  const desktop = device(shared, "desktop", dir);
  await phone.engine.open(); await desktop.engine.open();
  await phone.engine.call("add_account", { name: "GCash" });
  await phone.engine.call("add_account", { name: "BPI" });

  const reply = await phone.engine.call("transfer_money", { fromAccountId: 1, toAccountId: 2, amount: 500, description: "Too much" });
  assert.match(reply, /^ERROR/);
  await settle();
  assert.equal(await balances(desktop), "GCash=0, BPI=0");
  phone.close(); desktop.close();
});

test("the desktop's existing vault is uploaded once and never silently replaced", async () => {
  const dir = scratchDir();
  // An existing desktop vault with real data
  const local = cliReplica(join(dir, "local.db"));
  await local.open();
  await local.apply([
    { seq: 1, cmd: "add_account", args: { name: "Old Savings" }, at: "2025-01-01 09:00:00" },
    { seq: 2, cmd: "log_income", args: { accountId: 1, amount: 5000, description: "Bonus" }, at: "2025-01-02 09:00:00" },
  ]);
  const bytes = await local.snapshot();
  local.close();

  const shared = memoryVault();
  await uploadVault(shared.connect(), bytes, false);
  const phone = device(shared, "phone", dir);
  await phone.engine.open();
  await settle();
  assert.equal(await balances(phone), "Old Savings=5000");

  await phone.engine.call("log_expense", { accountId: 1, amount: 100, description: "Lunch" });
  await assert.rejects(uploadVault(shared.connect(), bytes, false), (e) => e.status === 409);

  // Replacing on purpose switches every device back to the uploaded copy
  await uploadVault(shared.connect(), bytes, true);
  await settle();
  assert.equal(await balances(phone), "Old Savings=5000");
  phone.close();
});

test("a new device starts from the latest checkpoint instead of replaying everything", async () => {
  const dir = scratchDir();
  const shared = memoryVault();
  const busy = device(shared, "busy", dir, { checkpointEvery: 20 });
  await busy.engine.open();
  await busy.engine.call("add_account", { name: "Wallet" });
  for (let i = 0; i < 99; i++) await busy.engine.call("log_income", { accountId: 1, amount: 1, description: `Tip ${i}` });
  await settle();
  assert.ok(shared.state.vault.checkpoint.seq >= 80, `checkpoint at ${shared.state.vault.checkpoint?.seq}`);
  // Older regular checkpoints are tidied away
  assert.equal([...shared.state.checkpoints.keys()].length, 1);

  shared.state.reads = 0;
  const fresh = device(shared, "fresh", dir);
  await fresh.engine.open();
  await settle();
  assert.equal(await balances(fresh), "Wallet=99");
  assert.ok(shared.state.reads < 25, `read ${shared.state.reads} changes`);
  busy.close(); fresh.close();
});

test("offline: reads still work, and changes are refused without pretending to save", async () => {
  const dir = scratchDir();
  const shared = memoryVault();
  const phone = device(shared, "phone", dir);
  await phone.engine.open();
  await phone.engine.call("add_account", { name: "GCash" });

  phone.remote.setOffline(true);
  assert.equal(await balances(phone), "GCash=0");
  assert.equal(await phone.engine.call("log_income", { accountId: 1, amount: 5, description: "x" }), OFFLINE_MESSAGE);
  assert.equal(shared.state.ops.length, 1);

  // Reopening with no connection still shows the last data this device had
  phone.close();
  const again = device(shared, "phone", dir);
  again.remote.setOffline(true);
  assert.deepEqual(await again.engine.open(), { offline: true });
  assert.equal(await balances(again), "GCash=0");
  again.close();
});
