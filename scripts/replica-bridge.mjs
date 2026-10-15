// Test-only: serves vault-core replicas over HTTP, so the app can be tested end to end in a plain browser, where
// there is no Tauri. Build the app with VITE_REPLICA_BRIDGE=http://127.0.0.1:9300 to use it (see src/sync/bridgeReplica.js).
import { createServer } from "node:http";
import { join } from "node:path";
import { cliReplica, scratchDir } from "../src/sync/testSupport.js";

const dir = scratchDir();
const devices = new Map(); // device id -> replica process

createServer(async (req, res) => {
  res.setHeader("Access-Control-Allow-Origin", "*");
  if (req.method !== "POST") { res.end(); return; }
  let body = "";
  for await (const chunk of req) body += chunk;
  const { device, op, ...rest } = JSON.parse(body);
  try {
    if (op === "open") {
      devices.get(device)?.close();
      const replica = cliReplica(join(dir, `${device}_${rest.name}.db`));
      devices.set(device, replica);
      res.end(JSON.stringify({ lastSeq: await replica.open() }));
      return;
    }
    const replica = devices.get(device);
    const reply = op === "apply" ? { results: await replica.apply(rest.ops) }
      : op === "call" ? await replica.call(rest.cmd, rest.args)
      : op === "snapshot" ? { hex: Buffer.from(await replica.snapshot()).toString("hex") }
      : (await replica.restore(new Uint8Array(Buffer.from(rest.hex, "hex")), rest.seq), {});
    res.end(JSON.stringify(reply));
  } catch (e) {
    res.end(JSON.stringify({ fatal: String(e.message ?? e) }));
  }
}).listen(9300, "127.0.0.1", () => console.log(`Replica bridge on http://127.0.0.1:9300 (replicas in ${dir})`));
