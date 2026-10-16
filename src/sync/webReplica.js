// This browser's copy of a synced vault, for the web app (PWA). It runs vault-core compiled to WebAssembly
// (vault-web/), so the browser applies exactly the same rules as the desktop and Android apps. The vault lives in
// memory and is saved to the browser's storage after every change, so it opens straight away and works offline.
import initWasm, { WebVault } from "./vault-web-pkg/vault_web.js";
import { indexedDbStore, keepStorage } from "./browserStore.js";

let loading = null;

/**
 * The same replica interface as tauriReplica. `store` (load/save by key) and `wasm` (the compiled module's bytes)
 * can be passed in for tests running outside a browser.
 */
export function webReplica(uid, profile, { store = indexedDbStore, wasm } = {}) {
  const key = `${uid}/${profile}`;
  let vault = null;
  const save = () => store.save(key, vault.snapshot());

  return {
    async open() {
      loading ??= initWasm(wasm ? { module_or_path: wasm } : undefined);
      await loading;
      const saved = await store.load(key);
      try {
        vault = new WebVault(saved ?? undefined);
      } catch (e) {
        // A damaged copy is replaced by an empty one, which then downloads everything again from the family account
        console.warn("Vault: the saved copy couldn't be opened, so it will be downloaded again", e);
        vault = new WebVault(undefined);
      }
      if (!wasm) keepStorage();
      return vault.lastSeq();
    },
    async apply(ops) {
      const outcomes = JSON.parse(vault.apply(JSON.stringify(ops)));
      await save();
      return outcomes;
    },
    call: async (cmd, args) => JSON.parse(vault.call(cmd, JSON.stringify(args ?? {}))),
    snapshot: async () => vault.snapshot(),
    async restore(bytes, seq) {
      vault.restore(bytes, seq);
      await save();
    },
  };
}

// The name src/api.js imports, whichever replica this build uses (see vite.config.js)
export const deviceReplica = webReplica;
