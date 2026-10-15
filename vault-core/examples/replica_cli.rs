//! A sync replica driven over stdin/stdout, one JSON request per line. Used by the JavaScript sync-engine tests
//! (src/sync/engine.test.js) so they exercise the real vault-core logic rather than a stand-in.
//!   {"op":"open","path":"…"}                         -> {"lastSeq":0}
//!   {"op":"apply","ops":[{"seq":1,"cmd":…,"args":…,"at":…}]} -> {"results":[…]}
//!   {"op":"call","cmd":"get_accounts","args":{}}     -> {"result":…} | {"error":…}
//!   {"op":"snapshot"}                                -> {"hex":"…"}
//!   {"op":"restore","hex":"…","seq":5}               -> {"lastSeq":5}
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use vault_core::rusqlite::Connection;

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{:02x}", b)).collect() }
fn unhex(s: &str) -> Vec<u8> { (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect() }

fn main() {
    let mut conn: Option<Connection> = None;
    let mut path = PathBuf::new();
    let stdout = std::io::stdout();
    for line in std::io::stdin().lock().lines() {
        let req: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let reply = (|| -> Result<Value, String> {
            match req["op"].as_str().unwrap_or("") {
                "open" => {
                    path = PathBuf::from(req["path"].as_str().unwrap());
                    let c = vault_core::open_vault_file(&path).map_err(|e| e.to_string())?;
                    let seq = vault_core::last_applied_seq(&c)?;
                    conn = Some(c);
                    Ok(json!({ "lastSeq": seq }))
                }
                "apply" => {
                    let c = conn.as_ref().ok_or("not open")?;
                    let mut results = vec![];
                    for op in req["ops"].as_array().unwrap() {
                        results.push(vault_core::apply_op(c, op["seq"].as_i64().unwrap(), op["cmd"].as_str().unwrap(), &op["args"], op["at"].as_str().unwrap())?);
                    }
                    Ok(json!({ "results": results }))
                }
                "call" => {
                    let c = conn.as_ref().ok_or("not open")?;
                    Ok(match vault_core::dispatch(c, req["cmd"].as_str().unwrap(), &req["args"]) { Ok(v) => json!({ "result": v }), Err(e) => json!({ "error": e }) })
                }
                "snapshot" => {
                    let c = conn.as_ref().ok_or("not open")?;
                    Ok(json!({ "hex": hex(&vault_core::snapshot(c, &path.with_extension("snapshot"))?) }))
                }
                "restore" => {
                    conn = None;
                    let seq = req["seq"].as_i64().unwrap();
                    conn = Some(vault_core::restore_replica(&path, &unhex(req["hex"].as_str().unwrap()), seq)?);
                    Ok(json!({ "lastSeq": seq }))
                }
                other => Err(format!("unknown op {}", other)),
            }
        })();
        let out = match reply { Ok(v) => v, Err(e) => json!({ "fatal": e }) };
        let mut lock = stdout.lock();
        writeln!(lock, "{}", out).unwrap();
        lock.flush().unwrap();
    }
}
