//! Vault sync server: holds the one always-online copy of each vault and runs every command through the same
//! vault-core logic the desktop app uses. Whenever a device changes something, every other connected device is told
//! over a WebSocket so its screens refresh right away.
//!
//! Configuration (environment variables):
//!   VAULT_TOKEN     required: the secret every device must present (at least 16 characters)
//!   VAULT_DATA_DIR  where the vault files live (default: ./data)
//!   PORT            port to listen on (default: 8080)

use axum::{
    body::Bytes,
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, DefaultBodyLimit, Path, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::{Arc, Mutex}, time::Duration};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use vault_core::rusqlite::Connection;

const MAX_VAULT_UPLOAD: usize = 64 * 1024 * 1024;

struct AppState {
    data_dir: PathBuf,
    token: String,
    // One command at a time: a personal budget never needs more, and it keeps every balance update strictly ordered
    db_lock: Mutex<()>,
    events: broadcast::Sender<String>,
}
type Shared = Arc<AppState>;

fn api_error(status: StatusCode, message: impl Into<String>) -> Response { (status, Json(json!({ "error": message.into() }))).into_response() }

// Profile names become file names, so only allow the characters a person would type for a name
fn checked_profile(profile: &str) -> Result<String, Response> {
    let p = profile.trim();
    if p.is_empty() || p.len() > 64 || !p.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
        return Err(api_error(StatusCode::BAD_REQUEST, "ERROR: Profile names may only use letters, numbers, spaces, - and _"));
    }
    Ok(p.to_string())
}

/// The key devices use to tell whether a change notice is about the vault they have open
fn profile_key(profile: &str) -> String { profile.to_lowercase().replace(' ', "_") }

// Compares every byte so the time taken never hints at how much of a guessed token was right
fn tokens_match(given: &str, expected: &str) -> bool {
    given.len() == expected.len() && given.bytes().zip(expected.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

async fn require_token(State(state): State<Shared>, Query(q): Query<TokenQuery>, headers: HeaderMap, req: Request, next: Next) -> Response {
    let bearer = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer "));
    // Browsers cannot set headers on a WebSocket, so the live-updates socket passes the token in the URL instead
    let given = bearer.or(q.token.as_deref()).unwrap_or("");
    if !tokens_match(given, &state.token) { return api_error(StatusCode::UNAUTHORIZED, "ERROR: Wrong or missing sync token"); }
    next.run(req).await
}
#[derive(Deserialize)] struct TokenQuery { token: Option<String> }

/// Runs blocking SQLite work off the async threads, holding the vault lock (a crashed command never locks it for good)
async fn with_vault<T: Send + 'static>(state: &Shared, work: impl FnOnce(&Shared) -> Result<T, Response> + Send + 'static) -> Result<T, Response> {
    let state = state.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = state.db_lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        work(&state)
    }).await.unwrap_or_else(|_| Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, "ERROR: The command crashed; nothing was saved")))
}

fn open(state: &AppState, profile: &str) -> Result<Connection, Response> {
    vault_core::open_vault(&state.data_dir, profile).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("ERROR: Could not open the vault: {}", e)))
}

fn notify(state: &AppState, profile: &str, client_id: Option<&str>, cmd: &str) {
    let _ = state.events.send(json!({ "type": "changed", "profile": profile_key(profile), "clientId": client_id, "cmd": cmd }).to_string());
}

#[derive(Deserialize)] struct LoginBody { profile: String }
async fn login(State(state): State<Shared>, Json(body): Json<LoginBody>) -> Response {
    let profile = match checked_profile(&body.profile) { Ok(p) => p, Err(e) => return e };
    match with_vault(&state, move |s| open(s, &profile).map(|_| profile)).await {
        Ok(profile) => Json(json!({ "result": format!("SUCCESS: Logged in as {}", profile) })).into_response(),
        Err(e) => e,
    }
}

#[derive(Deserialize)] #[serde(rename_all = "camelCase")]
struct CallBody { profile: String, cmd: String, #[serde(default)] args: Value, client_id: Option<String> }
async fn call(State(state): State<Shared>, Json(body): Json<CallBody>) -> Response {
    let profile = match checked_profile(&body.profile) { Ok(p) => p, Err(e) => return e };
    let args = if body.args.is_null() { json!({}) } else { body.args };
    let result = with_vault(&state, move |s| {
        let conn = open(s, &profile)?;
        let result = vault_core::run_command(&conn, &body.cmd, &args).map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
        if vault_core::is_mutation(&body.cmd) { notify(s, &profile, body.client_id.as_deref(), &body.cmd); }
        Ok(result)
    }).await;
    match result { Ok(value) => Json(json!({ "result": value })).into_response(), Err(e) => e }
}

// Download a complete copy of a vault, for backups or to move it back onto a desktop
async fn download_vault(State(state): State<Shared>, Path(profile): Path<String>) -> Response {
    let profile = match checked_profile(&profile) { Ok(p) => p, Err(e) => return e };
    let result = with_vault(&state, move |s| {
        if !s.data_dir.join(vault_core::vault_file_name(&profile)).exists() { return Err(api_error(StatusCode::NOT_FOUND, "ERROR: No vault with that profile name on this server")); }
        let conn = open(s, &profile)?;
        vault_core::snapshot(&conn, &s.data_dir.join(format!(".download_{}", vault_core::vault_file_name(&profile)))).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, e))
    }).await;
    match result { Ok(bytes) => ([(header::CONTENT_TYPE, "application/vnd.sqlite3")], bytes).into_response(), Err(e) => e }
}

#[derive(Deserialize)] struct UploadQuery { replace: Option<bool>, #[serde(rename = "clientId")] client_id: Option<String> }
// Upload a desktop vault so the server starts from the same data. Refuses to overwrite a server vault that already
// holds data unless replace=true, so a second device can never wipe out the first one's records by accident.
async fn upload_vault(State(state): State<Shared>, Path(profile): Path<String>, Query(q): Query<UploadQuery>, body: Bytes) -> Response {
    let profile = match checked_profile(&profile) { Ok(p) => p, Err(e) => return e };
    if !body.starts_with(b"SQLite format 3\0") { return api_error(StatusCode::BAD_REQUEST, "ERROR: That file is not a Vault database"); }
    let result = with_vault(&state, move |s| {
        let target = s.data_dir.join(vault_core::vault_file_name(&profile));
        if target.exists() && !q.replace.unwrap_or(false) {
            let conn = open(s, &profile)?;
            let has_data: i64 = conn.query_row("SELECT (SELECT COUNT(*) FROM accounts) + (SELECT COUNT(*) FROM smart_bills) + (SELECT COUNT(*) FROM loans) + (SELECT COUNT(*) FROM transactions)", [], |r| r.get(0)).unwrap_or(1);
            if has_data > 0 { return Err(api_error(StatusCode::CONFLICT, "ERROR: The server already has data for this vault. Upload again with replace to overwrite it.")); }
        }
        // Write beside the real file, check it opens, then swap it in so a bad upload can never corrupt the live vault
        let incoming = s.data_dir.join(format!(".upload_{}", vault_core::vault_file_name(&profile)));
        std::fs::write(&incoming, &body).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let check = vault_core::open_vault_file(&incoming).and_then(|c| c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0)));
        if !matches!(check.as_deref(), Ok("ok")) {
            let _ = std::fs::remove_file(&incoming);
            return Err(api_error(StatusCode::BAD_REQUEST, "ERROR: That vault file is damaged and was not uploaded"));
        }
        std::fs::rename(&incoming, &target).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        notify(s, &profile, q.client_id.as_deref(), "upload_vault");
        Ok(())
    }).await;
    match result { Ok(()) => Json(json!({ "result": "SUCCESS: Vault uploaded to the sync server!" })).into_response(), Err(e) => e }
}

async fn events(State(state): State<Shared>, ws: WebSocketUpgrade) -> Response {
    let rx = state.events.subscribe();
    ws.on_upgrade(move |socket| stream_events(socket, rx))
}
async fn stream_events(mut socket: WebSocket, mut rx: broadcast::Receiver<String>) {
    // Hosting proxies drop silent connections, so ping regularly to keep the live channel open
    let mut ping = tokio::time::interval(Duration::from_secs(25));
    loop {
        tokio::select! {
            event = rx.recv() => match event {
                Ok(text) => if socket.send(Message::Text(text.into())).await.is_err() { break },
                // This device fell too far behind to replay each change; one "changed" makes it reload everything
                Err(broadcast::error::RecvError::Lagged(_)) => if socket.send(Message::Text(json!({ "type": "changed", "profile": null }).to_string().into())).await.is_err() { break },
                Err(broadcast::error::RecvError::Closed) => break,
            },
            _ = ping.tick() => if socket.send(Message::Ping(Bytes::new())).await.is_err() { break },
            incoming = socket.recv() => match incoming { Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break, _ => {} },
        }
    }
}

fn app(state: Shared) -> Router {
    let api = Router::new()
        // Lets an app check the address and token are right before switching to sync
        .route("/api/ping", get(|| async { Json(json!({ "result": "ok" })) }))
        .route("/api/login", post(login))
        .route("/api/call", post(call))
        .route("/api/vault/{profile}", get(download_vault).put(upload_vault))
        .route("/api/events", get(events))
        .layer(middleware::from_fn_with_state(state.clone(), require_token));
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(api)
        .layer(DefaultBodyLimit::max(MAX_VAULT_UPLOAD))
        // The apps run in a webview with their own origin; every request carries the token, not cookies, so any origin may call
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let token = std::env::var("VAULT_TOKEN").unwrap_or_default();
    if token.len() < 16 {
        eprintln!("Set VAULT_TOKEN to a long random secret (at least 16 characters) before starting the sync server.");
        std::process::exit(1);
    }
    let data_dir = PathBuf::from(std::env::var("VAULT_DATA_DIR").unwrap_or_else(|_| "data".to_string()));
    std::fs::create_dir_all(&data_dir).expect("could not create the data folder");
    let port = std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8080);

    let (events, _) = broadcast::channel(256);
    let state = Arc::new(AppState { data_dir: data_dir.clone(), token, db_lock: Mutex::new(()), events });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("could not open the port");
    println!("Vault sync server listening on port {} (vaults in {})", port, data_dir.display());
    axum::serve(listener, app(state)).with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; }).await.unwrap();
}

#[cfg(test)]
mod tests;
