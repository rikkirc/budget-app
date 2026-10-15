// Test-only: a replica run by scripts/replica-bridge.mjs, so the app can be tested end to end in a plain browser
// (where there is no Tauri). Only used when the app is built with VITE_REPLICA_BRIDGE set.
export function bridgeReplica(bridgeUrl, deviceId, uid, profile) {
  const send = async (req) => {
    const res = await fetch(bridgeUrl, { method: "POST", body: JSON.stringify({ device: deviceId, ...req }) });
    const reply = await res.json();
    if ("fatal" in reply) throw reply.fatal;
    return reply;
  };
  const toHex = (bytes) => Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  const fromHex = (hex) => new Uint8Array(hex.match(/../g).map((h) => parseInt(h, 16)));
  return {
    open: async () => (await send({ op: "open", name: `${uid}_${profile}` })).lastSeq,
    apply: async (ops) => (await send({ op: "apply", ops })).results,
    call: (cmd, args) => send({ op: "call", cmd, args }),
    snapshot: async () => fromHex((await send({ op: "snapshot" })).hex),
    restore: (bytes, seq) => send({ op: "restore", hex: toHex(bytes), seq }),
  };
}
