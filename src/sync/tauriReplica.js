// This device's copy of a synced vault, run by vault-core through the app's Tauri commands (src-tauri/src/lib.rs)
import { invoke } from "@tauri-apps/api/core";
import { fromBase64, toBase64 } from "./base64.js";

export function tauriReplica(uid, profile) {
  return {
    open: () => invoke("open_replica", { uid, profile }),
    apply: (ops) => invoke("apply_ops", { ops }),
    call: (cmd, args) => invoke("call", { cmd, args }).then((result) => ({ result }), (error) => ({ error })),
    snapshot: async () => fromBase64(await invoke("export_vault")),
    restore: (bytes, seq) => invoke("restore_replica", { checkpoint: toBase64(bytes), seq }),
  };
}

// The name src/api.js imports, whichever replica this build uses (see vite.config.js)
export const deviceReplica = tauriReplica;

// "Try the demo" is only in the web app (src/sync/webReplica.js)
export const demoReplica = null;
