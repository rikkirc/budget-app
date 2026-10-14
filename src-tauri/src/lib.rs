// The desktop and mobile app. All business logic lives in vault-core; this file only remembers which vault is
// unlocked and hands each command to vault-core. When sync is on, the frontend talks to the sync server instead.
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use vault_core::rusqlite::Connection;

// The Receptionist's memory: the full path of the currently logged-in user's safe
struct AppState {
    db_path: Mutex<Option<PathBuf>>
}

// Always save to Documents\FinancialCommandCenter\ — a fixed, permanent home
// that every future version of the app will automatically find, regardless of where the .exe itself sits.
fn vault_folder() -> PathBuf {
    let mut app_folder = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
    app_folder.push("FinancialCommandCenter");
    let _ = std::fs::create_dir_all(&app_folder); // Creates the folder if it doesn't exist yet, safely does nothing if it already does
    app_folder
}

// The Front Door Login Command. It creates a brand new safe if it doesn't exist!
#[tauri::command]
fn login_user(profile: &str, state: State<'_, AppState>) -> String {
    let folder = vault_folder();
    match vault_core::open_vault(&folder, profile) {
        Ok(_) => {
            *state.db_path.lock().unwrap() = Some(folder.join(vault_core::vault_file_name(profile)));
            format!("SUCCESS: Logged in as {}", profile)
        }
        Err(e) => format!("ERROR: Could not open the vault: {}", e),
    }
}

fn open_current(state: &State<'_, AppState>) -> Result<Connection, String> {
    let path = state.db_path.lock().unwrap().clone().ok_or("ERROR: No vault is unlocked.")?;
    Connection::open(&path).map_err(|e| format!("ERROR: Could not open the vault: {}", e))
}

// Every other command goes through here: the same vault-core logic the sync server runs
#[tauri::command]
fn call(cmd: &str, args: serde_json::Value, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let conn = open_current(&state)?;
    vault_core::run_command(&conn, cmd, &args)
}

// A complete copy of the unlocked vault, so it can be uploaded to the sync server
#[tauri::command]
fn export_vault(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let conn = open_current(&state)?;
    let bytes = vault_core::snapshot(&conn, &std::env::temp_dir().join(format!("vault_export_{}.db", std::process::id())))?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize the Receptionist memory with a blank slate!
        .manage(AppState { db_path: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![login_user, call, export_vault])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
