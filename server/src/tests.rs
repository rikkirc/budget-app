// These tests run the real routes against a throwaway data folder, never a real vault
use super::*;
use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

const TOKEN: &str = "test-token-0123456789";

fn test_state() -> Shared {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let data_dir = std::env::temp_dir().join(format!("vault_server_test_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    let (events, _) = broadcast::channel(16);
    Arc::new(AppState { data_dir, token: TOKEN.to_string(), db_lock: Mutex::new(()), events })
}

async fn send(state: &Shared, method: &str, uri: &str, token: Option<&str>, body: Body) -> (StatusCode, Bytes) {
    let mut req = Request::builder().method(method).uri(uri).header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = token { req = req.header(header::AUTHORIZATION, format!("Bearer {}", t)); }
    let res = app(state.clone()).oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    (status, res.into_body().collect().await.unwrap().to_bytes())
}

async fn call(state: &Shared, cmd: &str, args: Value, client_id: &str) -> (StatusCode, Value) {
    let body = json!({ "profile": "Juan Dela Cruz", "cmd": cmd, "args": args, "clientId": client_id }).to_string();
    let (status, bytes) = send(state, "POST", "/api/call", Some(TOKEN), Body::from(body)).await;
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn rejects_requests_without_the_right_token() {
    let state = test_state();
    let body = || Body::from(json!({ "profile": "Juan" }).to_string());
    assert_eq!(send(&state, "POST", "/api/login", None, body()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&state, "POST", "/api/login", Some("test-token-0123456780"), body()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&state, "POST", "/api/login", Some(TOKEN), body()).await.0, StatusCode::OK);
    assert_eq!(send(&state, "GET", "/health", None, Body::empty()).await.0, StatusCode::OK);
    assert_eq!(send(&state, "GET", "/api/ping", None, Body::empty()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&state, "GET", "/api/ping", Some(TOKEN), Body::empty()).await.0, StatusCode::OK);
}

#[tokio::test]
async fn rejects_profile_names_that_could_escape_the_data_folder() {
    let state = test_state();
    for bad in ["../../etc/passwd", "a/b", "", "x\\y"] {
        let (status, _) = send(&state, "POST", "/api/login", Some(TOKEN), Body::from(json!({ "profile": bad }).to_string())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{:?} should be rejected", bad);
    }
}

#[tokio::test]
async fn commands_run_on_the_server_vault_and_other_devices_hear_about_changes() {
    let state = test_state();
    let mut listener = state.events.subscribe();

    let (status, added) = call(&state, "add_account", json!({ "name": "Wallet" }), "phone").await;
    assert_eq!(status, StatusCode::OK);
    assert!(added["result"].as_str().unwrap().starts_with("SUCCESS"));

    let event: Value = serde_json::from_str(&listener.try_recv().unwrap()).unwrap();
    assert_eq!(event["profile"], "juan_dela_cruz");
    assert_eq!(event["clientId"], "phone");

    let (_, accounts) = call(&state, "get_accounts", json!({}), "desktop").await;
    assert_eq!(accounts["result"][0]["name"], "Wallet");
    // Reading never sends a change notice
    assert!(listener.try_recv().is_err());
}

#[tokio::test]
async fn bad_commands_are_reported_not_crashed_on() {
    let state = test_state();
    let (status, body) = call(&state, "drop_everything", json!({}), "phone").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("Unknown command"));
}

#[tokio::test]
async fn upload_seeds_an_empty_server_but_never_silently_overwrites_data() {
    let desktop = test_state();
    call(&desktop, "add_account", json!({ "name": "Desktop Savings" }), "desktop").await;
    let (status, vault_file) = send(&desktop, "GET", "/api/vault/Juan%20Dela%20Cruz", Some(TOKEN), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);

    let server = test_state();
    let (status, _) = send(&server, "PUT", "/api/vault/Juan%20Dela%20Cruz", Some(TOKEN), Body::from(vault_file.clone())).await;
    assert_eq!(status, StatusCode::OK);
    let (_, accounts) = call(&server, "get_accounts", json!({}), "phone").await;
    assert_eq!(accounts["result"][0]["name"], "Desktop Savings");

    // The server now has data, so a second upload must ask for replace explicitly
    let (status, _) = send(&server, "PUT", "/api/vault/Juan%20Dela%20Cruz", Some(TOKEN), Body::from(vault_file.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = send(&server, "PUT", "/api/vault/Juan%20Dela%20Cruz?replace=true", Some(TOKEN), Body::from(vault_file)).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn upload_rejects_files_that_are_not_vaults() {
    let state = test_state();
    let (status, _) = send(&state, "PUT", "/api/vault/Juan", Some(TOKEN), Body::from("hello")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
