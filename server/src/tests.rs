// These tests run the real routes against a throwaway data folder, never a real vault
use super::*;
use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

const INVITE: &str = "family-code-2026";

fn test_state() -> Shared {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let data_dir = std::env::temp_dir().join(format!("vault_server_test_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    let accounts = Accounts::open(&data_dir.join("accounts.db"), INVITE.to_string()).unwrap();
    let (events, _) = broadcast::channel(16);
    Arc::new(AppState { data_dir, accounts, db_lock: Mutex::new(()), events })
}

async fn send(state: &Shared, method: &str, uri: &str, token: Option<&str>, body: Body) -> (StatusCode, Bytes) {
    let mut req = Request::builder().method(method).uri(uri).header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = token { req = req.header(header::AUTHORIZATION, format!("Bearer {}", t)); }
    let res = app(state.clone()).oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    (status, res.into_body().collect().await.unwrap().to_bytes())
}

async fn post_json(state: &Shared, uri: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
    let (status, bytes) = send(state, "POST", uri, token, Body::from(body.to_string())).await;
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// Signs up a family member and returns their session token
async fn sign_up(state: &Shared, email: &str) -> String {
    let (status, body) = post_json(state, "/api/sign-up", None, json!({ "email": email, "password": "a good password", "inviteCode": INVITE })).await;
    assert_eq!(status, StatusCode::OK, "{}", body);
    body["result"]["token"].as_str().unwrap().to_string()
}

async fn call(state: &Shared, token: &str, cmd: &str, args: Value, client_id: &str) -> (StatusCode, Value) {
    post_json(state, "/api/call", Some(token), json!({ "profile": "Juan Dela Cruz", "cmd": cmd, "args": args, "clientId": client_id })).await
}

#[tokio::test]
async fn sign_up_needs_the_family_invite_code() {
    let state = test_state();
    let (status, body) = post_json(&state, "/api/sign-up", None, json!({ "email": "x@example.com", "password": "a good password", "inviteCode": "guess" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("invite code"));

    let (status, body) = post_json(&state, "/api/sign-up", None, json!({ "email": "x@example.com", "password": "a good password", "inviteCode": INVITE })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["email"], "x@example.com");
    assert_eq!(body["result"]["recoveryCode"].as_str().unwrap().len(), 23);
}

#[tokio::test]
async fn vault_routes_need_a_valid_session() {
    let state = test_state();
    assert_eq!(send(&state, "GET", "/api/me", None, Body::empty()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&state, "GET", "/api/me", Some("made-up"), Body::empty()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(post_json(&state, "/api/login", None, json!({ "profile": "Juan" })).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&state, "GET", "/health", None, Body::empty()).await.0, StatusCode::OK);

    let token = sign_up(&state, "juan@example.com").await;
    let (status, body) = send(&state, "GET", "/api/me", Some(&token), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8_lossy(&body).contains("juan@example.com"));
}

#[tokio::test]
async fn sign_in_on_a_second_device_reaches_the_same_vault() {
    let state = test_state();
    let desktop = sign_up(&state, "juan@example.com").await;
    call(&state, &desktop, "add_account", json!({ "name": "Wallet" }), "desktop").await;

    let (status, body) = post_json(&state, "/api/sign-in", None, json!({ "email": "JUAN@example.com", "password": "a good password" })).await;
    assert_eq!(status, StatusCode::OK);
    let phone = body["result"]["token"].as_str().unwrap().to_string();
    let (_, accounts) = call(&state, &phone, "get_accounts", json!({}), "phone").await;
    assert_eq!(accounts["result"][0]["name"], "Wallet");

    let (status, _) = post_json(&state, "/api/sign-in", None, json!({ "email": "juan@example.com", "password": "wrong" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn family_members_never_see_each_others_vaults_or_changes() {
    let state = test_state();
    let juan = sign_up(&state, "juan@example.com").await;
    let maria = sign_up(&state, "maria@example.com").await;
    let mut listener = state.events.subscribe();

    // Same profile name, different people: two separate vaults
    call(&state, &juan, "add_account", json!({ "name": "Juan's Wallet" }), "juan-phone").await;
    let (_, marias) = call(&state, &maria, "get_accounts", json!({}), "maria-phone").await;
    assert_eq!(marias["result"], json!([]));

    let (owner, event) = listener.try_recv().unwrap();
    let juan_id = state.accounts.session_user(&juan).unwrap().0;
    assert_eq!(owner, juan_id);
    let event: Value = serde_json::from_str(&event).unwrap();
    assert_eq!(event["profile"], "juan_dela_cruz");
    assert_eq!(event["clientId"], "juan-phone");

    // A profile only Juan has can't be downloaded by Maria: her account has no vault by that name
    assert_eq!(post_json(&state, "/api/login", Some(&juan), json!({ "profile": "Juan Private" })).await.0, StatusCode::OK);
    assert_eq!(send(&state, "GET", "/api/vault/Juan%20Private", Some(&juan), Body::empty()).await.0, StatusCode::OK);
    assert_eq!(send(&state, "GET", "/api/vault/Juan%20Private", Some(&maria), Body::empty()).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reads_send_no_change_notices() {
    let state = test_state();
    let token = sign_up(&state, "juan@example.com").await;
    let mut listener = state.events.subscribe();
    call(&state, &token, "get_accounts", json!({}), "desktop").await;
    assert!(listener.try_recv().is_err());
}

#[tokio::test]
async fn signing_out_ends_the_session_and_reset_signs_out_everywhere() {
    let state = test_state();
    let (_, body) = post_json(&state, "/api/sign-up", None, json!({ "email": "ate@example.com", "password": "a good password", "inviteCode": INVITE })).await;
    let first = body["result"]["token"].as_str().unwrap().to_string();
    let recovery = body["result"]["recoveryCode"].as_str().unwrap().to_string();
    let (_, body) = post_json(&state, "/api/sign-in", None, json!({ "email": "ate@example.com", "password": "a good password" })).await;
    let second = body["result"]["token"].as_str().unwrap().to_string();

    assert_eq!(post_json(&state, "/api/sign-out", Some(&second), json!({})).await.0, StatusCode::OK);
    assert_eq!(send(&state, "GET", "/api/me", Some(&second), Body::empty()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&state, "GET", "/api/me", Some(&first), Body::empty()).await.0, StatusCode::OK);

    let (status, body) = post_json(&state, "/api/reset-password", None, json!({ "email": "ate@example.com", "recoveryCode": recovery, "newPassword": "brand new password" })).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["result"]["token"].is_string() && body["result"]["recoveryCode"].is_string());
    assert_eq!(send(&state, "GET", "/api/me", Some(&first), Body::empty()).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_profile_names_that_could_escape_the_data_folder() {
    let state = test_state();
    let token = sign_up(&state, "juan@example.com").await;
    for bad in ["../../etc/passwd", "a/b", "", "x\\y"] {
        let (status, _) = post_json(&state, "/api/login", Some(&token), json!({ "profile": bad })).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{:?} should be rejected", bad);
    }
}

#[tokio::test]
async fn bad_commands_are_reported_not_crashed_on() {
    let state = test_state();
    let token = sign_up(&state, "juan@example.com").await;
    let (status, body) = call(&state, &token, "drop_everything", json!({}), "phone").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("Unknown command"));
}

#[tokio::test]
async fn upload_seeds_an_empty_server_but_never_silently_overwrites_data() {
    let desktop_server = test_state();
    let desktop = sign_up(&desktop_server, "juan@example.com").await;
    call(&desktop_server, &desktop, "add_account", json!({ "name": "Desktop Savings" }), "desktop").await;
    let (status, vault_file) = send(&desktop_server, "GET", "/api/vault/Juan%20Dela%20Cruz", Some(&desktop), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);

    let server = test_state();
    let token = sign_up(&server, "juan@example.com").await;
    let (status, _) = send(&server, "PUT", "/api/vault/Juan%20Dela%20Cruz", Some(&token), Body::from(vault_file.clone())).await;
    assert_eq!(status, StatusCode::OK);
    let (_, accounts) = call(&server, &token, "get_accounts", json!({}), "phone").await;
    assert_eq!(accounts["result"][0]["name"], "Desktop Savings");

    // The server now has data, so a second upload must ask for replace explicitly
    let (status, _) = send(&server, "PUT", "/api/vault/Juan%20Dela%20Cruz", Some(&token), Body::from(vault_file.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = send(&server, "PUT", "/api/vault/Juan%20Dela%20Cruz?replace=true", Some(&token), Body::from(vault_file)).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn upload_rejects_files_that_are_not_vaults() {
    let state = test_state();
    let token = sign_up(&state, "juan@example.com").await;
    let (status, _) = send(&state, "PUT", "/api/vault/Juan", Some(&token), Body::from("hello")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
