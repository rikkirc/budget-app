//! Family accounts: anyone with the family invite code can create their own account (email + password), sign in on
//! any device, and reset a forgotten password with the recovery code they were shown at sign-up. No email service
//! and no administrator are needed.
//!
//! Passwords and recovery codes are stored only as Argon2 hashes; session tokens only as SHA-256 hashes, so a copy
//! of accounts.db alone never lets anyone sign in.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand_core::{OsRng, RngCore};
use argon2::Argon2;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use vault_core::rusqlite::{params, Connection, OptionalExtension};

pub const MIN_PASSWORD_LEN: usize = 8;
// After this many wrong guesses for one email (or wrong invite codes overall), wait LOCKOUT before trying again
const MAX_FAILURES: u32 = 5;
const LOCKOUT: Duration = Duration::from_secs(15 * 60);
// A device that hasn't been used for this long has to sign in again
const SESSION_IDLE_DAYS: i64 = 180;
// Recovery codes avoid look-alike characters (0/O, 1/I/L) so they are easy to copy from paper
const RECOVERY_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

pub struct Accounts {
    conn: Mutex<Connection>,
    invite_code: String,
    failures: Mutex<HashMap<String, (u32, Instant)>>,
}

#[derive(Debug)]
pub struct SignedIn { pub user_id: i64, pub email: String, pub token: String }

fn hash_secret(secret: &str) -> String {
    Argon2::default().hash_password(secret.as_bytes(), &SaltString::generate(&mut OsRng)).expect("hashing never fails with default settings").to_string()
}
fn secret_matches(secret: &str, stored: &str) -> bool {
    PasswordHash::new(stored).map(|h| Argon2::default().verify_password(secret.as_bytes(), &h).is_ok()).unwrap_or(false)
}
fn token_hash(token: &str) -> String { Sha256::digest(token.as_bytes()).iter().map(|b| format!("{:02x}", b)).collect() }

fn new_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
/// e.g. "K7QMX-2HFPA-R9WTE-NB4CZ": 20 random characters, about 99 bits
fn new_recovery_code() -> String {
    let mut bytes = [0u8; 20];
    OsRng.fill_bytes(&mut bytes);
    let chars: String = bytes.iter().map(|b| RECOVERY_ALPHABET[*b as usize % RECOVERY_ALPHABET.len()] as char).collect();
    chars.as_bytes().chunks(5).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join("-")
}
/// People may type a recovery code in lower case, with spaces, or without the dashes
fn normalize_recovery(code: &str) -> String {
    let c: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect();
    c.as_bytes().chunks(5).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join("-")
}
pub fn normalize_email(email: &str) -> String { email.trim().to_lowercase() }

// Compares every byte so the time taken never hints at how much of a guessed code was right
fn same_text(a: &str, b: &str) -> bool { a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0 }

impl Accounts {
    pub fn open(path: &Path, invite_code: String) -> vault_core::rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, email TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL, recovery_hash TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (datetime('now')));
             CREATE TABLE IF NOT EXISTS sessions (token_hash TEXT PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), created_at TEXT NOT NULL DEFAULT (datetime('now')), last_used TEXT NOT NULL DEFAULT (datetime('now')));",
        )?;
        Ok(Accounts { conn: Mutex::new(conn), invite_code, failures: Mutex::new(HashMap::new()) })
    }

    fn db(&self) -> std::sync::MutexGuard<'_, Connection> { self.conn.lock().unwrap_or_else(|p| p.into_inner()) }

    // ---- Guessing protection ----
    fn check_not_locked(&self, key: &str) -> Result<(), String> {
        let mut failures = self.failures.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((count, since)) = failures.get(key).copied() {
            if since.elapsed() >= LOCKOUT { failures.remove(key); }
            else if count >= MAX_FAILURES {
                let minutes = (LOCKOUT - since.elapsed()).as_secs() / 60 + 1;
                return Err(format!("ERROR: Too many wrong attempts. Please wait {} minute(s) and try again.", minutes));
            }
        }
        Ok(())
    }
    fn record_failure(&self, key: &str) {
        let mut failures = self.failures.lock().unwrap_or_else(|p| p.into_inner());
        let entry = failures.entry(key.to_string()).or_insert((0, Instant::now()));
        if entry.1.elapsed() >= LOCKOUT { *entry = (0, Instant::now()); }
        entry.0 += 1;
    }
    fn clear_failures(&self, key: &str) { self.failures.lock().unwrap_or_else(|p| p.into_inner()).remove(key); }

    fn start_session(&self, user_id: i64, email: String) -> SignedIn {
        let token = new_token();
        let _ = self.db().execute("INSERT INTO sessions (token_hash, user_id) VALUES (?1, ?2)", params![token_hash(&token), user_id]);
        SignedIn { user_id, email, token }
    }

    /// Creates an account. Returns the new session and the recovery code (shown to the person exactly once).
    pub fn sign_up(&self, email: &str, password: &str, invite_code: &str) -> Result<(SignedIn, String), String> {
        let email = normalize_email(email);
        if email.len() > 254 || !email.contains('@') || email.starts_with('@') || email.ends_with('@') { return Err("ERROR: Please enter a valid email address.".into()); }
        if password.chars().count() < MIN_PASSWORD_LEN { return Err(format!("ERROR: Your password needs at least {} characters.", MIN_PASSWORD_LEN)); }
        self.check_not_locked("invite")?;
        if !same_text(invite_code.trim(), &self.invite_code) {
            self.record_failure("invite");
            return Err("ERROR: That family invite code isn't right. Ask the person who set up the server.".into());
        }
        let recovery = new_recovery_code();
        let inserted = self.db().execute("INSERT INTO users (email, password_hash, recovery_hash) VALUES (?1, ?2, ?3)", params![email, hash_secret(password), hash_secret(&recovery)]);
        if inserted.is_err() { return Err("ERROR: There is already an account with that email. Sign in instead.".into()); }
        let user_id = self.db().last_insert_rowid();
        Ok((self.start_session(user_id, email), recovery))
    }

    pub fn sign_in(&self, email: &str, password: &str) -> Result<SignedIn, String> {
        let email = normalize_email(email);
        let key = format!("email:{}", email);
        self.check_not_locked(&key)?;
        let user: Option<(i64, String)> = self.db().query_row("SELECT id, password_hash FROM users WHERE email = ?1", params![email], |r| Ok((r.get(0)?, r.get(1)?))).optional().unwrap_or(None);
        match user {
            Some((id, hash)) if secret_matches(password, &hash) => { self.clear_failures(&key); Ok(self.start_session(id, email)) }
            _ => { self.record_failure(&key); Err("ERROR: Wrong email or password.".into()) }
        }
    }

    /// Forgot password: the recovery code proves who you are. Signs out every device and issues a fresh recovery code.
    pub fn reset_password(&self, email: &str, recovery_code: &str, new_password: &str) -> Result<(SignedIn, String), String> {
        let email = normalize_email(email);
        if new_password.chars().count() < MIN_PASSWORD_LEN { return Err(format!("ERROR: Your password needs at least {} characters.", MIN_PASSWORD_LEN)); }
        let key = format!("email:{}", email);
        self.check_not_locked(&key)?;
        let user: Option<(i64, String)> = self.db().query_row("SELECT id, recovery_hash FROM users WHERE email = ?1", params![email], |r| Ok((r.get(0)?, r.get(1)?))).optional().unwrap_or(None);
        let Some((id, _)) = user.filter(|(_, hash)| secret_matches(&normalize_recovery(recovery_code), hash)) else {
            self.record_failure(&key);
            return Err("ERROR: That email and recovery code don't match.".into());
        };
        self.clear_failures(&key);
        let recovery = new_recovery_code();
        {
            let db = self.db();
            let _ = db.execute("UPDATE users SET password_hash = ?1, recovery_hash = ?2 WHERE id = ?3", params![hash_secret(new_password), hash_secret(&recovery), id]);
            let _ = db.execute("DELETE FROM sessions WHERE user_id = ?1", params![id]);
        }
        Ok((self.start_session(id, email), recovery))
    }

    pub fn change_password(&self, user_id: i64, current: &str, new_password: &str) -> Result<(), String> {
        if new_password.chars().count() < MIN_PASSWORD_LEN { return Err(format!("ERROR: Your password needs at least {} characters.", MIN_PASSWORD_LEN)); }
        let key = format!("user:{}", user_id);
        self.check_not_locked(&key)?;
        let hash: String = self.db().query_row("SELECT password_hash FROM users WHERE id = ?1", params![user_id], |r| r.get(0)).map_err(|_| "ERROR: Account not found.".to_string())?;
        if !secret_matches(current, &hash) { self.record_failure(&key); return Err("ERROR: Your current password isn't right.".into()); }
        self.clear_failures(&key);
        let _ = self.db().execute("UPDATE users SET password_hash = ?1 WHERE id = ?2", params![hash_secret(new_password), user_id]);
        Ok(())
    }

    /// Who a session token belongs to, or None if it is unknown or has gone unused too long
    pub fn session_user(&self, token: &str) -> Option<(i64, String)> {
        let db = self.db();
        let hash = token_hash(token);
        let _ = db.execute("DELETE FROM sessions WHERE last_used < datetime('now', ?1)", params![format!("-{} days", SESSION_IDLE_DAYS)]);
        let user = db.query_row("SELECT u.id, u.email FROM sessions s JOIN users u ON u.id = s.user_id WHERE s.token_hash = ?1", params![hash], |r| Ok((r.get(0)?, r.get(1)?))).optional().unwrap_or(None);
        if user.is_some() { let _ = db.execute("UPDATE sessions SET last_used = datetime('now') WHERE token_hash = ?1", params![hash]); }
        user
    }

    pub fn sign_out(&self, token: &str) { let _ = self.db().execute("DELETE FROM sessions WHERE token_hash = ?1", params![token_hash(token)]); }
    pub fn sign_out_everywhere(&self, user_id: i64) { let _ = self.db().execute("DELETE FROM sessions WHERE user_id = ?1", params![user_id]); }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accounts() -> Accounts { Accounts::open(Path::new(":memory:"), "family-code-2026".into()).unwrap() }

    #[test]
    fn sign_up_needs_the_invite_code_and_a_real_password() {
        let a = accounts();
        assert!(a.sign_up("mama@example.com", "long enough pw", "wrong").unwrap_err().contains("invite code"));
        assert!(a.sign_up("mama@example.com", "short", "family-code-2026").unwrap_err().contains("at least"));
        assert!(a.sign_up("not-an-email", "long enough pw", "family-code-2026").unwrap_err().contains("email"));
        let (session, recovery) = a.sign_up("Mama@Example.com ", "long enough pw", "family-code-2026").unwrap();
        assert_eq!(session.email, "mama@example.com");
        assert_eq!(recovery.len(), 23);
        assert!(a.sign_up("mama@example.com", "another password", "family-code-2026").unwrap_err().contains("already"));
    }

    #[test]
    fn sign_in_and_sessions() {
        let a = accounts();
        let (first, _) = a.sign_up("papa@example.com", "correct horse", "family-code-2026").unwrap();
        assert!(a.sign_in("papa@example.com", "wrong horse").is_err());
        let second = a.sign_in("PAPA@example.com", "correct horse").unwrap();
        assert_eq!(a.session_user(&second.token).unwrap().0, first.user_id);
        assert!(a.session_user("not-a-token").is_none());

        a.sign_out(&second.token);
        assert!(a.session_user(&second.token).is_none());
        assert!(a.session_user(&first.token).is_some());
        a.sign_out_everywhere(first.user_id);
        assert!(a.session_user(&first.token).is_none());
    }

    #[test]
    fn repeated_wrong_passwords_lock_the_account_for_a_while() {
        let a = accounts();
        a.sign_up("kuya@example.com", "correct horse", "family-code-2026").unwrap();
        for _ in 0..MAX_FAILURES { assert!(a.sign_in("kuya@example.com", "guess").unwrap_err().contains("Wrong")); }
        // Even the right password is refused during the lockout
        assert!(a.sign_in("kuya@example.com", "correct horse").unwrap_err().contains("Too many"));
    }

    #[test]
    fn recovery_code_resets_the_password_and_signs_out_other_devices() {
        let a = accounts();
        let (old_session, recovery) = a.sign_up("ate@example.com", "forgotten pw", "family-code-2026").unwrap();
        assert!(a.reset_password("ate@example.com", "AAAAA-BBBBB-CCCCC-DDDDD", "brand new pw").is_err());

        // Typed sloppily: lower case, spaces instead of dashes
        let sloppy = recovery.to_lowercase().replace('-', " ");
        let (_, new_recovery) = a.reset_password("ate@example.com", &sloppy, "brand new pw").unwrap();
        assert!(a.session_user(&old_session.token).is_none());
        assert!(a.sign_in("ate@example.com", "forgotten pw").is_err());
        assert!(a.sign_in("ate@example.com", "brand new pw").is_ok());
        // The old recovery code is used up
        assert_ne!(recovery, new_recovery);
        assert!(a.reset_password("ate@example.com", &recovery, "another pw!").is_err());
    }

    #[test]
    fn nothing_secret_is_stored_in_plain_text() {
        let a = accounts();
        let (session, recovery) = a.sign_up("lola@example.com", "plain text pw", "family-code-2026").unwrap();
        let db = a.db();
        let (pw, rc): (String, String) = db.query_row("SELECT password_hash, recovery_hash FROM users", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        let token: String = db.query_row("SELECT token_hash FROM sessions", [], |r| r.get(0)).unwrap();
        assert!(!pw.contains("plain text pw") && !rc.contains(&recovery) && token != session.token);
    }
}
