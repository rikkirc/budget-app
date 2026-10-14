//! Vault sync server: holds the always-online copy of every family member's vaults and runs every command through the
//! same vault-core logic the desktop app uses. Whenever a device changes something, that person's other devices are
//! told over a WebSocket so their screens refresh right away.
//!
//! Each person creates their own account with the family invite code; their vaults live in their own folder and no
//! one else's session can reach them.
//!
//! Configuration (environment variables):
//!   VAULT_INVITE_CODE  required: the code family members enter to create an account (at least 8 characters)
//!   VAULT_DATA_DIR     where accounts and vault files live (default: ./data)
//!   PORT               port to listen on (default: 8080)

mod accounts;

use accounts::Accounts;
use axum::{
    body::Bytes,
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, DefaultBodyLimit, Extension, Path, Query, Request, State},
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
    accounts: Accounts,
    // One command at a time: a family's budgets never need more, and it keeps every balance update strictly ordered
    db_lock: Mutex<()>,
    // (account id, change notice): each socket only forwards its own account's notices
    events: broadcast::Sender<(i64, String)>,
}
type Shared = Arc<AppState>;

/// The signed-in person behind a request, put there by require_session
#[derive(Clone)]
struct User { id: i64, email: String, token: String }

fn api_error(status: StatusCode, message: impl Into<String>) -> Response { (status, Json(json!({ "error": message.into() }))).into_response() }
fn ok(result: Value) -> Response { Json(json!({ "result": result })).into_response() }

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

/// Every account's vaults live in their own folder
fn user_dir(state: &AppState, user_id: i64) -> Result<PathBuf, Response> {
    let dir = state.data_dir.join("users").join(user_id.to_string());
    std::fs::create_dir_all(&dir).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("ERROR: Could not create your folder: {}", e)))?;
    Ok(dir)
}

#[derive(Deserialize)] struct TokenQuery { token: Option<String> }
async fn require_session(State(state): State<Shared>, Query(q): Query<TokenQuery>, headers: HeaderMap, mut req: Request, next: Next) -> Response {
    let bearer = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).map(str::to_string);
    // Browsers cannot set headers on a WebSocket, so the live-updates socket passes the token in the URL instead
    let Some(token) = bearer.or(q.token) else { return api_error(StatusCode::UNAUTHORIZED, "ERROR: Please sign in.") };
    let s = state.clone();
    let lookup_token = token.clone();
    let user = tokio::task::spawn_blocking(move || s.accounts.session_user(&lookup_token)).await.ok().flatten();
    let Some((id, email)) = user else { return api_error(StatusCode::UNAUTHORIZED, "ERROR: You've been signed out. Please sign in again.") };
    req.extensions_mut().insert(User { id, email, token });
    next.run(req).await
}

/// Runs password hashing off the async threads (it is deliberately slow)
async fn blocking<T: Send + 'static>(state: &Shared, work: impl FnOnce(&AppState) -> T + Send + 'static) -> Result<T, Response> {
    let state = state.clone();
    tokio::task::spawn_blocking(move || work(&state)).await.map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "ERROR: Something went wrong on the server"))
}

/// Runs blocking SQLite work off the async threads, holding the vault lock (a crashed command never locks it for good)
async fn with_vault<T: Send + 'static>(state: &Shared, work: impl FnOnce(&Shared) -> Result<T, Response> + Send + 'static) -> Result<T, Response> {
    let state = state.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = state.db_lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        work(&state)
    }).await.unwrap_or_else(|_| Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, "ERROR: The command crashed; nothing was saved")))
}

fn open(state: &AppState, user_id: i64, profile: &str) -> Result<Connection, Response> {
    vault_core::open_vault(&user_dir(state, user_id)?, profile).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("ERROR: Could not open the vault: {}", e)))
}

fn notify(state: &AppState, user_id: i64, profile: &str, client_id: Option<&str>, cmd: &str) {
    let _ = state.events.send((user_id, json!({ "type": "changed", "profile": profile_key(profile), "clientId": client_id, "cmd": cmd }).to_string()));
}

// ---- Accounts ----

fn signed_in_json(s: &accounts::SignedIn) -> Value { json!({ "token": s.token, "email": s.email }) }

#[derive(Deserialize)] #[serde(rename_all = "camelCase")]
struct SignUpBody { email: String, password: String, invite_code: String }
async fn sign_up(State(state): State<Shared>, Json(b): Json<SignUpBody>) -> Response {
    match blocking(&state, move |s| s.accounts.sign_up(&b.email, &b.password, &b.invite_code)).await {
        Ok(Ok((session, recovery))) => { let mut v = signed_in_json(&session); v["recoveryCode"] = json!(recovery); ok(v) }
        Ok(Err(e)) => api_error(StatusCode::BAD_REQUEST, e),
        Err(e) => e,
    }
}

#[derive(Deserialize)] struct SignInBody { email: String, password: String }
async fn sign_in(State(state): State<Shared>, Json(b): Json<SignInBody>) -> Response {
    match blocking(&state, move |s| s.accounts.sign_in(&b.email, &b.password)).await {
        Ok(Ok(session)) => ok(signed_in_json(&session)),
        Ok(Err(e)) => api_error(StatusCode::UNAUTHORIZED, e),
        Err(e) => e,
    }
}

#[derive(Deserialize)] #[serde(rename_all = "camelCase")]
struct ResetBody { email: String, recovery_code: String, new_password: String }
async fn reset_password(State(state): State<Shared>, Json(b): Json<ResetBody>) -> Response {
    match blocking(&state, move |s| s.accounts.reset_password(&b.email, &b.recovery_code, &b.new_password)).await {
        Ok(Ok((session, recovery))) => { let mut v = signed_in_json(&session); v["recoveryCode"] = json!(recovery); ok(v) }
        Ok(Err(e)) => api_error(StatusCode::BAD_REQUEST, e),
        Err(e) => e,
    }
}

#[derive(Deserialize)] #[serde(rename_all = "camelCase")]
struct ChangePasswordBody { current_password: String, new_password: String }
async fn change_password(State(state): State<Shared>, Extension(user): Extension<User>, Json(b): Json<ChangePasswordBody>) -> Response {
    match blocking(&state, move |s| s.accounts.change_password(user.id, &b.current_password, &b.new_password)).await {
        Ok(Ok(())) => ok(json!("SUCCESS: Password changed!")),
        Ok(Err(e)) => api_error(StatusCode::BAD_REQUEST, e),
        Err(e) => e,
    }
}

async fn me(Extension(user): Extension<User>) -> Response { ok(json!({ "email": user.email })) }

async fn sign_out(State(state): State<Shared>, Extension(user): Extension<User>) -> Response {
    match blocking(&state, move |s| s.accounts.sign_out(&user.token)).await { Ok(()) => ok(json!("SUCCESS: Signed out.")), Err(e) => e }
}
async fn sign_out_everywhere(State(state): State<Shared>, Extension(user): Extension<User>) -> Response {
    match blocking(&state, move |s| s.accounts.sign_out_everywhere(user.id)).await { Ok(()) => ok(json!("SUCCESS: Signed out on every device.")), Err(e) => e }
}

// ---- Vaults ----

#[derive(Deserialize)] struct LoginBody { profile: String }
async fn login(State(state): State<Shared>, Extension(user): Extension<User>, Json(body): Json<LoginBody>) -> Response {
    let profile = match checked_profile(&body.profile) { Ok(p) => p, Err(e) => return e };
    match with_vault(&state, move |s| open(s, user.id, &profile).map(|_| profile)).await {
        Ok(profile) => ok(json!(format!("SUCCESS: Logged in as {}", profile))),
        Err(e) => e,
    }
}

#[derive(Deserialize)] #[serde(rename_all = "camelCase")]
struct CallBody { profile: String, cmd: String, #[serde(default)] args: Value, client_id: Option<String> }
async fn call(State(state): State<Shared>, Extension(user): Extension<User>, Json(body): Json<CallBody>) -> Response {
    let profile = match checked_profile(&body.profile) { Ok(p) => p, Err(e) => return e };
    let args = if body.args.is_null() { json!({}) } else { body.args };
    let result = with_vault(&state, move |s| {
        let conn = open(s, user.id, &profile)?;
        let result = vault_core::run_command(&conn, &body.cmd, &args).map_err(|e| api_error(StatusCode::BAD_REQUEST, e))?;
        if vault_core::is_mutation(&body.cmd) { notify(s, user.id, &profile, body.client_id.as_deref(), &body.cmd); }
        Ok(result)
    }).await;
    match result { Ok(value) => ok(value), Err(e) => e }
}

// Download a complete copy of one of your vaults, for backups or to move it back onto a desktop
async fn download_vault(State(state): State<Shared>, Extension(user): Extension<User>, Path(profile): Path<String>) -> Response {
    let profile = match checked_profile(&profile) { Ok(p) => p, Err(e) => return e };
    let result = with_vault(&state, move |s| {
        let dir = user_dir(s, user.id)?;
        if !dir.join(vault_core::vault_file_name(&profile)).exists() { return Err(api_error(StatusCode::NOT_FOUND, "ERROR: You have no vault with that profile name on this server")); }
        let conn = open(s, user.id, &profile)?;
        vault_core::snapshot(&conn, &dir.join(format!(".download_{}", vault_core::vault_file_name(&profile)))).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, e))
    }).await;
    match result { Ok(bytes) => ([(header::CONTENT_TYPE, "application/vnd.sqlite3")], bytes).into_response(), Err(e) => e }
}

#[derive(Deserialize)] struct UploadQuery { replace: Option<bool>, #[serde(rename = "clientId")] client_id: Option<String> }
// Upload a desktop vault so the server starts from the same data. Refuses to overwrite a server vault that already
// holds data unless replace=true, so a second device can never wipe out the first one's records by accident.
async fn upload_vault(State(state): State<Shared>, Extension(user): Extension<User>, Path(profile): Path<String>, Query(q): Query<UploadQuery>, body: Bytes) -> Response {
    let profile = match checked_profile(&profile) { Ok(p) => p, Err(e) => return e };
    if !body.starts_with(b"SQLite format 3\0") { return api_error(StatusCode::BAD_REQUEST, "ERROR: That file is not a Vault database"); }
    let result = with_vault(&state, move |s| {
        let dir = user_dir(s, user.id)?;
        let target = dir.join(vault_core::vault_file_name(&profile));
        if target.exists() && !q.replace.unwrap_or(false) {
            let conn = open(s, user.id, &profile)?;
            let has_data: i64 = conn.query_row("SELECT (SELECT COUNT(*) FROM accounts) + (SELECT COUNT(*) FROM smart_bills) + (SELECT COUNT(*) FROM loans) + (SELECT COUNT(*) FROM transactions)", [], |r| r.get(0)).unwrap_or(1);
            if has_data > 0 { return Err(api_error(StatusCode::CONFLICT, "ERROR: The server already has data for this vault. Upload again with replace to overwrite it.")); }
        }
        // Write beside the real file, check it opens, then swap it in so a bad upload can never corrupt the live vault
        let incoming = dir.join(format!(".upload_{}", vault_core::vault_file_name(&profile)));
        std::fs::write(&incoming, &body).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let check = vault_core::open_vault_file(&incoming).and_then(|c| c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0)));
        if !matches!(check.as_deref(), Ok("ok")) {
            let _ = std::fs::remove_file(&incoming);
            return Err(api_error(StatusCode::BAD_REQUEST, "ERROR: That vault file is damaged and was not uploaded"));
        }
        std::fs::rename(&incoming, &target).map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        notify(s, user.id, &profile, q.client_id.as_deref(), "upload_vault");
        Ok(())
    }).await;
    match result { Ok(()) => ok(json!("SUCCESS: Vault uploaded to the sync server!")), Err(e) => e }
}

async fn events(State(state): State<Shared>, Extension(user): Extension<User>, ws: WebSocketUpgrade) -> Response {
    let rx = state.events.subscribe();
    ws.on_upgrade(move |socket| stream_events(socket, rx, user.id))
}
async fn stream_events(mut socket: WebSocket, mut rx: broadcast::Receiver<(i64, String)>, user_id: i64) {
    // Hosting proxies drop silent connections, so ping regularly to keep the live channel open
    let mut ping = tokio::time::interval(Duration::from_secs(25));
    loop {
        tokio::select! {
            event = rx.recv() => match event {
                Ok((owner, text)) => if owner == user_id && socket.send(Message::Text(text.into())).await.is_err() { break },
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
    let signed_in = Router::new()
        .route("/api/me", get(me))
        .route("/api/sign-out", post(sign_out))
        .route("/api/sign-out-everywhere", post(sign_out_everywhere))
        .route("/api/change-password", post(change_password))
        .route("/api/login", post(login))
        .route("/api/call", post(call))
        .route("/api/vault/{profile}", get(download_vault).put(upload_vault))
        .route("/api/events", get(events))
        .layer(middleware::from_fn_with_state(state.clone(), require_session));
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/sign-up", post(sign_up))
        .route("/api/sign-in", post(sign_in))
        .route("/api/reset-password", post(reset_password))
        .merge(signed_in)
        .layer(DefaultBodyLimit::max(MAX_VAULT_UPLOAD))
        // The apps run in a webview with their own origin; every request carries a session token, not cookies, so any origin may call
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let invite_code = std::env::var("VAULT_INVITE_CODE").unwrap_or_default().trim().to_string();
    if invite_code.len() < 8 {
        eprintln!("Set VAULT_INVITE_CODE to the code your family will use to create accounts (at least 8 characters).");
        std::process::exit(1);
    }
    let data_dir = PathBuf::from(std::env::var("VAULT_DATA_DIR").unwrap_or_else(|_| "data".to_string()));
    std::fs::create_dir_all(&data_dir).expect("could not create the data folder");
    let port = std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8080);
    let accounts = Accounts::open(&data_dir.join("accounts.db"), invite_code).expect("could not open the accounts database");

    let (events, _) = broadcast::channel(256);
    let state = Arc::new(AppState { data_dir: data_dir.clone(), accounts, db_lock: Mutex::new(()), events });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("could not open the port");
    println!("Vault sync server listening on port {} (data in {})", port, data_dir.display());
    axum::serve(listener, app(state)).with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; }).await.unwrap();
}

#[cfg(test)]
mod tests;
