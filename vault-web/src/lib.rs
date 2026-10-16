//! Vault in the browser. The web app (PWA) runs the same vault-core as the desktop and Android apps, compiled to
//! WebAssembly, so every device applies the same rules to the same list of changes and ends up with identical balances.
//!
//! A browser has no files, so the vault lives in memory. After every change the web app saves `snapshot()` in the
//! browser's own storage (src/sync/webReplica.js), and opens it again from those bytes next time.
//! Like the phone app, the browser only ever holds a sync replica: its data changes only by applying the shared list.
use serde_json::Value;
use vault_core::rusqlite::Connection;
use wasm_bindgen::prelude::*;

fn fail(e: impl std::fmt::Display) -> JsError { JsError::new(&e.to_string()) }

#[wasm_bindgen]
pub struct WebVault { conn: Connection }

#[wasm_bindgen]
impl WebVault {
    /// The vault this browser saved last time (the bytes from `snapshot()`), or a brand-new empty one
    #[wasm_bindgen(constructor)]
    pub fn new(saved: Option<Vec<u8>>) -> Result<WebVault, JsError> {
        let conn = match saved {
            Some(bytes) => vault_core::open_vault_bytes(&bytes).map_err(fail)?,
            None => {
                let conn = Connection::open_in_memory().map_err(fail)?;
                vault_core::init_schema(&conn);
                conn
            }
        };
        Ok(WebVault { conn })
    }

    /// The number of the last change this vault has applied (0 for a new one)
    #[wasm_bindgen(js_name = lastSeq)]
    pub fn last_seq(&self) -> Result<f64, JsError> { vault_core::last_applied_seq(&self.conn).map(|s| s as f64).map_err(fail) }

    /// Applies the next changes from the shared list, in order. `ops` is a JSON array of {seq, cmd, args, at};
    /// returns a JSON array with each change's outcome ({"result": …} or {"error": …}).
    pub fn apply(&self, ops: &str) -> Result<String, JsError> {
        let ops: Vec<Value> = serde_json::from_str(ops).map_err(fail)?;
        let mut outcomes = Vec::with_capacity(ops.len());
        for op in &ops {
            let (Some(seq), Some(cmd), Some(at)) = (op["seq"].as_i64(), op["cmd"].as_str(), op["at"].as_str()) else { return Err(fail("ERROR: A change is missing its number, command or time.")); };
            outcomes.push(vault_core::apply_op(&self.conn, seq, cmd, &op["args"], at).map_err(fail)?);
        }
        Ok(Value::Array(outcomes).to_string())
    }

    /// Runs a read (any get_… command; `args` is a JSON object). Returns {"result": …} or {"error": …} as JSON.
    /// Changes are refused here: a synced vault changes only through apply(), so every device stays identical.
    pub fn call(&self, cmd: &str, args: &str) -> String {
        if vault_core::is_mutation(cmd) { return serde_json::json!({ "error": "ERROR: Synced vaults change only through sync." }).to_string(); }
        let reply = serde_json::from_str::<Value>(args).map_err(|e| e.to_string()).and_then(|args| vault_core::dispatch(&self.conn, cmd, &args));
        match reply { Ok(v) => serde_json::json!({ "result": v }), Err(e) => serde_json::json!({ "error": e }) }.to_string()
    }

    /// A complete copy of the vault: saved by the browser, and uploaded as a sync checkpoint
    pub fn snapshot(&self) -> Result<Vec<u8>, JsError> { vault_core::vault_bytes(&self.conn).map_err(fail) }

    /// Replaces the vault with a checkpoint (the bytes of a vault file) taken at change `seq`. A damaged checkpoint is
    /// refused and the current vault is kept.
    pub fn restore(&mut self, checkpoint: &[u8], seq: f64) -> Result<(), JsError> {
        self.conn = vault_core::restore_replica_bytes(checkpoint, seq as i64).map_err(fail)?;
        Ok(())
    }
}
