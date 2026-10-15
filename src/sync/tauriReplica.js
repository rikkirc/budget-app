// This device's copy of a synced vault, run by vault-core through the app's Tauri commands (src-tauri/src/lib.rs)
import { invoke } from "@tauri-apps/api/core";

export function tauriReplica(uid, profile) {
  return {
    open: () => invoke("open_replica", { uid, profile }),
    apply: (ops) => invoke("apply_ops", { ops }),
    call: (cmd, args) => invoke("call", { cmd, args }).then((result) => ({ result }), (error) => ({ error })),
    snapshot: async () => new Uint8Array(await invoke("export_vault")),
    restore: (bytes, seq) => invoke("restore_replica", bytes, { headers: { "x-seq": String(seq) } }),
  };
}
