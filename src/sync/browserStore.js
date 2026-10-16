// Where the web app keeps this device's copy of each synced vault: the browser's IndexedDB, one entry per vault.
// Everything here is also in Firebase, so if the browser ever clears it, signing in brings the vault back.
const DB_NAME = "vault";
const STORE = "replicas";

let opening = null;
function database() {
  opening ??= new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1);
    request.onupgradeneeded = () => request.result.createObjectStore(STORE);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => { opening = null; reject(request.error); };
  });
  return opening;
}

async function run(mode, work) {
  const db = await database();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, mode);
    const request = work(tx.objectStore(STORE));
    tx.oncomplete = () => resolve(request.result);
    tx.onerror = tx.onabort = () => reject(tx.error);
  });
}

export const indexedDbStore = {
  /** The saved bytes of a vault, or undefined */
  load: (key) => run("readonly", (store) => store.get(key)),
  save: (key, bytes) => run("readwrite", (store) => store.put(bytes, key)),
};

/** Asks the browser not to clear Vault's storage when space runs low (installed home-screen apps usually get this) */
export function keepStorage() {
  navigator.storage?.persist?.().catch(() => {});
}
