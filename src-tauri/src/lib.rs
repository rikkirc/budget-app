// The desktop and mobile app. All business logic lives in vault-core; this file only remembers which vault is
// unlocked and hands each command to vault-core.
//
// Two kinds of vault can be unlocked:
//   - a local vault (Documents\FinancialCommandCenter), used when you're not signed in to sync; and
//   - a sync replica (in the app's data folder): this device's copy of a synced vault. Its data only ever changes by
//     applying the shared, numbered list of changes (apply_ops / restore_replica), never by direct commands, so every
//     device stays identical. The frontend's sync engine (src/sync/engine.js) drives it.
use base64::Engine;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};
use vault_core::rusqlite::Connection;

// The Receptionist's memory: the full path of the currently unlocked safe, and whether it is a sync replica
struct AppState {
    db_path: Mutex<Option<PathBuf>>,
    is_replica: Mutex<bool>,
}

// Always save to Documents\FinancialCommandCenter\ — a fixed, permanent home
// that every future version of the app will automatically find, regardless of where the .exe itself sits.
fn vault_folder() -> PathBuf {
    let mut app_folder = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
    app_folder.push("FinancialCommandCenter");
    let _ = std::fs::create_dir_all(&app_folder); // Creates the folder if it doesn't exist yet, safely does nothing if it already does
    app_folder
}

// Profile names become file names; never let one reach outside its folder
fn checked_profile(profile: &str) -> Result<&str, String> {
    if profile.trim().is_empty() || profile.contains(['/', '\\']) || profile.contains("..") { return Err("ERROR: That profile name can't be used.".into()); }
    Ok(profile)
}

fn unlock(state: &State<'_, AppState>, path: PathBuf, is_replica: bool) {
    *state.db_path.lock().unwrap() = Some(path);
    *state.is_replica.lock().unwrap() = is_replica;
}

// The Front Door Login Command. It creates a brand new safe if it doesn't exist!
#[tauri::command]
fn login_user(profile: &str, state: State<'_, AppState>) -> String {
    if let Err(e) = checked_profile(profile) { return e; }
    let folder = vault_folder();
    match vault_core::open_vault(&folder, profile) {
        Ok(_) => {
            unlock(&state, folder.join(vault_core::vault_file_name(profile)), false);
            format!("SUCCESS: Logged in as {}", profile)
        }
        Err(e) => format!("ERROR: Could not open the vault: {}", e),
    }
}

fn open_current(state: &State<'_, AppState>) -> Result<(Connection, PathBuf), String> {
    let path = state.db_path.lock().unwrap().clone().ok_or("ERROR: No vault is unlocked.")?;
    let conn = Connection::open(&path).map_err(|e| format!("ERROR: Could not open the vault: {}", e))?;
    Ok((conn, path))
}

// Every command goes through here. A sync replica only answers reads: its changes arrive through apply_ops.
#[tauri::command]
fn call(cmd: &str, args: serde_json::Value, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    if *state.is_replica.lock().unwrap() && vault_core::is_mutation(cmd) {
        return Err("ERROR: Synced vaults change only through sync.".into());
    }
    let (conn, _) = open_current(&state)?;
    vault_core::run_command(&conn, cmd, &args)
}

// Vault files cross between the app's screens and Rust as base64 text: Android's IPC can't carry raw bytes
// (it turns them into JSON), while plain text arrives intact on every platform
fn to_base64(bytes: &[u8]) -> String { base64::engine::general_purpose::STANDARD.encode(bytes) }
fn from_base64(text: &str) -> Result<Vec<u8>, String> { base64::engine::general_purpose::STANDARD.decode(text).map_err(|_| "ERROR: That vault data is damaged.".to_string()) }

// A complete copy of the unlocked vault (base64): to upload a local vault when you first sign in, or to save a sync checkpoint
#[tauri::command]
fn export_vault(state: State<'_, AppState>) -> Result<String, String> {
    let (conn, path) = open_current(&state)?;
    Ok(to_base64(&vault_core::snapshot(&conn, &path.with_extension("export"))?))
}

// ---- Sync replicas ----

/// Unlocks this device's copy of a synced vault (creating it if needed) and returns the last change it has applied
#[tauri::command]
fn open_replica<R: tauri::Runtime>(app: tauri::AppHandle<R>, uid: &str, profile: &str, state: State<'_, AppState>) -> Result<i64, String> {
    if uid.is_empty() || !uid.chars().all(|c| c.is_ascii_alphanumeric()) { return Err("ERROR: Invalid account.".into()); }
    let profile = checked_profile(profile)?;
    let folder = app.path().app_data_dir().map_err(|e| e.to_string())?.join("synced").join(uid);
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let path = folder.join(vault_core::vault_file_name(profile));
    let conn = vault_core::open_vault_file(&path).map_err(|e| format!("ERROR: Could not open the vault: {}", e))?;
    let seq = vault_core::last_applied_seq(&conn)?;
    unlock(&state, path, true);
    Ok(seq)
}

#[derive(Deserialize)]
struct Op { seq: i64, cmd: String, args: serde_json::Value, at: String }

/// Applies the next changes from the shared list, in order; returns each one's outcome
#[tauri::command]
fn apply_ops(ops: Vec<Op>, state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    if !*state.is_replica.lock().unwrap() { return Err("ERROR: No synced vault is unlocked.".into()); }
    let (conn, _) = open_current(&state)?;
    ops.iter().map(|op| vault_core::apply_op(&conn, op.seq, &op.cmd, &op.args, &op.at)).collect()
}

/// Replaces the replica with a checkpoint (a complete vault file, base64) taken at change number `seq`
#[tauri::command]
fn restore_replica(checkpoint: String, seq: i64, state: State<'_, AppState>) -> Result<(), String> {
    if !*state.is_replica.lock().unwrap() { return Err("ERROR: No synced vault is unlocked.".into()); }
    let path = state.db_path.lock().unwrap().clone().ok_or("ERROR: No vault is unlocked.")?;
    vault_core::restore_replica(&path, &from_base64(&checkpoint)?, seq).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
    use tauri::webview::InvokeRequest;

    fn ipc(webview: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str, body: serde_json::Value) -> Result<serde_json::Value, serde_json::Value> {
        get_ipc_response(webview, InvokeRequest {
            cmd: cmd.into(), callback: tauri::ipc::CallbackFn(0), error: tauri::ipc::CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(), body: tauri::ipc::InvokeBody::Json(body),
            headers: Default::default(), invoke_key: INVOKE_KEY.to_string(),
        }).map(|r| r.deserialize::<serde_json::Value>().unwrap())
    }

    // Calls the commands exactly as the screens do: JSON arguments only, which is all Android's IPC carries
    #[test]
    fn replica_commands_work_with_plain_json_arguments() {
        let app = builder(mock_builder()).build(mock_context(noop_assets())).unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        let uid = format!("test{}", std::process::id());

        assert_eq!(ipc(&webview, "open_replica", serde_json::json!({ "uid": uid, "profile": "Juan" })).unwrap(), 0);
        let ops = serde_json::json!({ "ops": [
            { "seq": 1, "cmd": "add_account", "args": { "name": "GCash" }, "at": "2026-10-15 08:00:00" },
            { "seq": 2, "cmd": "log_income", "args": { "accountId": 1, "amount": 500.0, "description": "Allowance" }, "at": "2026-10-15 08:01:00" },
        ] });
        ipc(&webview, "apply_ops", ops).unwrap();
        // Synced vaults refuse direct changes; reads work
        assert!(ipc(&webview, "call", serde_json::json!({ "cmd": "add_account", "args": { "name": "Sneaky" } })).is_err());
        let snapshot = ipc(&webview, "export_vault", serde_json::json!({})).unwrap();
        assert!(snapshot.as_str().unwrap().starts_with("U1FMaXRl"), "base64 of 'SQLite format 3'");

        // Restore the checkpoint into a second device's replica
        assert_eq!(ipc(&webview, "open_replica", serde_json::json!({ "uid": uid, "profile": "Phone" })).unwrap(), 0);
        ipc(&webview, "restore_replica", serde_json::json!({ "checkpoint": snapshot, "seq": 2 })).unwrap();
        let accounts = ipc(&webview, "call", serde_json::json!({ "cmd": "get_accounts", "args": {} })).unwrap();
        assert_eq!(accounts[0]["name"], "GCash");
        assert_eq!(accounts[0]["balance"], 500.0);
        assert_eq!(ipc(&webview, "open_replica", serde_json::json!({ "uid": uid, "profile": "Phone" })).unwrap(), 2);
        assert!(ipc(&webview, "restore_replica", serde_json::json!({ "checkpoint": "garbage", "seq": 3 })).is_err());

        let _ = std::fs::remove_dir_all(app.path().app_data_dir().unwrap().join("synced").join(&uid));
    }

    #[test]
    fn vault_bytes_survive_the_trip_through_base64() {
        let bytes: Vec<u8> = (0..=255u8).cycle().take(70_000).collect();
        assert_eq!(from_base64(&to_base64(&bytes)).unwrap(), bytes);
        assert!(from_base64("not base64!").is_err());
    }
}

fn builder<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        // Initialize the Receptionist memory with a blank slate!
        .manage(AppState { db_path: Mutex::new(None), is_replica: Mutex::new(false) })
        .invoke_handler(tauri::generate_handler![login_user, call, export_vault, open_replica, apply_ops, restore_replica])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    builder(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
