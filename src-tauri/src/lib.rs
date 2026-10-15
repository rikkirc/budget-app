// The desktop and mobile app. All business logic lives in vault-core; this file only remembers which vault is
// unlocked and hands each command to vault-core.
//
// Two kinds of vault can be unlocked:
//   - a local vault (Documents\FinancialCommandCenter), used when you're not signed in to sync; and
//   - a sync replica (in the app's data folder): this device's copy of a synced vault. Its data only ever changes by
//     applying the shared, numbered list of changes (apply_ops / restore_replica), never by direct commands, so every
//     device stays identical. The frontend's sync engine (src/sync/engine.js) drives it.
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

// A complete copy of the unlocked vault: to upload a local vault when you first sign in, or to save a sync checkpoint
#[tauri::command]
fn export_vault(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let (conn, path) = open_current(&state)?;
    let bytes = vault_core::snapshot(&conn, &path.with_extension("export"))?;
    Ok(tauri::ipc::Response::new(bytes))
}

// ---- Sync replicas ----

/// Unlocks this device's copy of a synced vault (creating it if needed) and returns the last change it has applied
#[tauri::command]
fn open_replica(app: tauri::AppHandle, uid: &str, profile: &str, state: State<'_, AppState>) -> Result<i64, String> {
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

/// Replaces the replica with a checkpoint (raw vault file in the request body) taken at change number `x-seq`
#[tauri::command]
fn restore_replica(request: tauri::ipc::Request<'_>, state: State<'_, AppState>) -> Result<(), String> {
    if !*state.is_replica.lock().unwrap() { return Err("ERROR: No synced vault is unlocked.".into()); }
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else { return Err("ERROR: Expected the checkpoint's bytes.".into()) };
    let seq: i64 = request.headers().get("x-seq").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok()).ok_or("ERROR: Missing checkpoint number.")?;
    let path = state.db_path.lock().unwrap().clone().ok_or("ERROR: No vault is unlocked.")?;
    vault_core::restore_replica(&path, bytes, seq).map(|_| ())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize the Receptionist memory with a blank slate!
        .manage(AppState { db_path: Mutex::new(None), is_replica: Mutex::new(false) })
        .invoke_handler(tauri::generate_handler![login_user, call, export_vault, open_replica, apply_ops, restore_replica])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
