//! Vault's business logic: every command, working on an open SQLite connection.
//! Shared by the desktop/mobile app (src-tauri) and the sync server (server/).

pub use rusqlite;
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Serialize)] pub struct Account { pub id: i32, pub name: String, pub balance: f64 }
#[derive(Serialize)] pub struct TransactionLog { pub id: i32, pub account_name: String, pub tx_type: String, pub amount: f64, pub description: String, pub is_reversed: bool, pub created_at: String, pub linked_account_name: String }
#[derive(Serialize)] pub struct SmartBill { pub id: i32, pub name: String, pub amount: f64, pub due_date: String, pub payday_date: String, pub is_recurring: bool, pub loan_id: i32, pub series_id: i32 }
#[derive(Serialize)] pub struct Loan { pub id: i32, pub name: String, pub principal: f64, pub remaining_balance: f64, pub term: i32, pub term_type: String, pub frequency: String, pub payment_amount: f64, pub next_due_date: String, pub payday_date: String, pub total_incurred: f64, pub due_structure: String, pub date_acquired: String }
#[derive(Serialize)] pub struct PaidBill { pub id: i32, pub original_bill_id: i32, pub name: String, pub amount: f64, pub due_date: String, pub payday_date: String, pub account_id: i32, pub from_account_id: i32, pub loan_id: i32, pub paid_at: String }
#[derive(Serialize)] pub struct CompletedLoan { pub id: i32, pub original_id: i32, pub name: String, pub principal: f64, pub term: i32, pub term_type: String, pub frequency: String, pub payment_amount: f64, pub total_incurred: f64, pub completed_date: String }
#[derive(Serialize)] pub struct SinkingFundInfo { pub found: bool, pub account_id: i32, pub account_name: String, pub amount: f64 }
#[derive(Serialize)] pub struct ArchivedBill { pub id: i32, pub original_id: i32, pub name: String, pub amount: f64, pub due_date: String, pub payday_date: String }

/// Turns "John Doe" into "john_doe_finance_data.db", the same file name every version of the app has used
pub fn vault_file_name(profile: &str) -> String { format!("{}_finance_data.db", profile.to_lowercase().replace(" ", "_")) }

/// Opens (creating if needed) the vault for a profile inside `folder`, with every table and migration in place
pub fn open_vault(folder: &Path, profile: &str) -> rusqlite::Result<Connection> { open_vault_file(&folder.join(vault_file_name(profile))) }

/// Opens (creating if needed) a vault file at an exact path, bringing its tables up to date
pub fn open_vault_file(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    init_schema(&conn);
    Ok(conn)
}

// Open the safe and build the empty drawers if this is a brand-new user!
pub fn init_schema(conn: &Connection) {
    conn.execute("CREATE TABLE IF NOT EXISTS accounts (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, balance REAL NOT NULL DEFAULT 0.0)", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS transactions (id INTEGER PRIMARY KEY, account_id INTEGER NOT NULL, tx_type TEXT NOT NULL, amount REAL NOT NULL, description TEXT NOT NULL, is_reversed INTEGER NOT NULL DEFAULT 0, linked_id INTEGER, FOREIGN KEY(account_id) REFERENCES accounts(id))", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS smart_bills (id INTEGER PRIMARY KEY, name TEXT NOT NULL, amount REAL NOT NULL, due_date TEXT NOT NULL, payday_date TEXT NOT NULL, loan_id INTEGER NOT NULL DEFAULT 0, is_recurring INTEGER NOT NULL DEFAULT 0)", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS loans (id INTEGER PRIMARY KEY, name TEXT NOT NULL, principal REAL NOT NULL, remaining_balance REAL NOT NULL, term INTEGER NOT NULL, term_type TEXT NOT NULL, frequency TEXT NOT NULL, payment_amount REAL NOT NULL, next_due_date TEXT NOT NULL, payday_date TEXT NOT NULL, total_incurred REAL NOT NULL DEFAULT 0, due_structure TEXT NOT NULL DEFAULT 'Sinking Fund')", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS paid_bills (id INTEGER PRIMARY KEY, original_bill_id INTEGER, name TEXT NOT NULL, amount REAL NOT NULL, due_date TEXT NOT NULL, payday_date TEXT NOT NULL, account_id INTEGER NOT NULL, loan_id INTEGER NOT NULL DEFAULT 0, from_account_id INTEGER NOT NULL DEFAULT 0)", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS archived_accounts (id INTEGER PRIMARY KEY, original_id INTEGER, name TEXT NOT NULL)", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS archived_loans (id INTEGER PRIMARY KEY, original_id INTEGER, name TEXT NOT NULL, principal REAL NOT NULL, remaining_balance REAL NOT NULL, term INTEGER NOT NULL, term_type TEXT NOT NULL, frequency TEXT NOT NULL, payment_amount REAL NOT NULL, next_due_date TEXT NOT NULL, payday_date TEXT NOT NULL, total_incurred REAL NOT NULL DEFAULT 0, due_structure TEXT NOT NULL DEFAULT 'Sinking Fund')", []).unwrap();

    // 🚨 Safety net: adds the new column to your EXISTING database file too (ignores error if it's already there)
    let _ = conn.execute("ALTER TABLE loans ADD COLUMN total_incurred REAL NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE archived_loans ADD COLUMN total_incurred REAL NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE loans ADD COLUMN due_structure TEXT NOT NULL DEFAULT 'Sinking Fund'", []);
    let _ = conn.execute("ALTER TABLE archived_loans ADD COLUMN due_structure TEXT NOT NULL DEFAULT 'Sinking Fund'", []);
    let _ = conn.execute("ALTER TABLE smart_bills ADD COLUMN loan_id INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE paid_bills ADD COLUMN loan_id INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE smart_bills ADD COLUMN is_recurring INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN is_reversed INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN linked_id INTEGER", []);
    let _ = conn.execute("ALTER TABLE paid_bills ADD COLUMN from_account_id INTEGER NOT NULL DEFAULT 0", []);
    // Split recurring bills: both halves of a pair (and every future month's pair) share one series_id. 0 = not a split bill.
    let _ = conn.execute("ALTER TABLE smart_bills ADD COLUMN series_id INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE paid_bills ADD COLUMN series_id INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE paid_bills ADD COLUMN is_recurring INTEGER NOT NULL DEFAULT 0", []);
    conn.execute("CREATE TABLE IF NOT EXISTS archived_smart_bills (id INTEGER PRIMARY KEY, original_id INTEGER, name TEXT NOT NULL, amount REAL NOT NULL, due_date TEXT NOT NULL, payday_date TEXT NOT NULL)", []).unwrap();
    let _ = conn.execute("ALTER TABLE archived_smart_bills ADD COLUMN series_id INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE archived_smart_bills ADD COLUMN is_recurring INTEGER NOT NULL DEFAULT 0", []);
        conn.execute("CREATE TABLE IF NOT EXISTS completed_loans (id INTEGER PRIMARY KEY, original_id INTEGER, name TEXT NOT NULL, principal REAL NOT NULL, term INTEGER NOT NULL, term_type TEXT NOT NULL, frequency TEXT NOT NULL, payment_amount REAL NOT NULL, total_incurred REAL NOT NULL, completed_date TEXT NOT NULL)", []).unwrap();
    // Date a loan was acquired/taken out (optional; blank for older loans)
    let _ = conn.execute("ALTER TABLE loans ADD COLUMN date_acquired TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE archived_loans ADD COLUMN date_acquired TEXT NOT NULL DEFAULT ''", []);
    // Date & time each transaction was recorded (stamped automatically by the database, in local time)
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN created_at TEXT NOT NULL DEFAULT ''", []);
    // When each receipt was settled (paid or saved to a fund), stamped the same way; blank for receipts from before this was recorded
    let _ = conn.execute("ALTER TABLE paid_bills ADD COLUMN paid_at TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute_batch("CREATE TRIGGER IF NOT EXISTS set_paid_timestamp AFTER INSERT ON paid_bills WHEN NEW.paid_at = '' BEGIN UPDATE paid_bills SET paid_at = datetime('now','localtime') WHERE id = NEW.id; END;");
    let _ = conn.execute_batch("CREATE TRIGGER IF NOT EXISTS set_tx_timestamp AFTER INSERT ON transactions WHEN NEW.created_at = '' BEGIN UPDATE transactions SET created_at = datetime('now','localtime') WHERE id = NEW.id; END;");
}

// Money moves in whole centavos. Sinking Fund halves can be half-centavos (412.35 / 2 = 206.175); rounding each account's
// balance separately could then round one side up and the other down, creating or losing ₱0.01. So every amount is rounded
// ONCE here, before it touches any account, and that same rounded amount is applied to both sides and logged.
pub fn round_to_centavo(amount: f64) -> f64 { (amount * 100.0).round() / 100.0 }

// 🚨 UPDATED: Every single command now asks the Receptionist which safe to open!
pub fn add_account(conn: &Connection, name: &str) -> String { match conn.execute("INSERT INTO accounts (name, balance) VALUES (?1, 0.0)", params![name]) { Ok(_) => format!("SUCCESS: Created new account '{}'!", name), Err(_) => "ERROR: Account might already exist.".to_string() } }
pub fn get_accounts(conn: &Connection) -> Vec<Account> { let mut stmt = conn.prepare("SELECT id, name, balance FROM accounts").unwrap(); let iter = stmt.query_map([], |row| { Ok(Account { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), balance: row.get(2).unwrap_or(0.0) }) }).unwrap(); let mut accounts = Vec::new(); for a in iter { if let Ok(acc) = a { accounts.push(acc); } } accounts }
pub fn delete_account(conn: &Connection, account_id: i32) -> String { let balance: f64 = conn.query_row("SELECT balance FROM accounts WHERE id = ?1", params![account_id], |row| row.get(0)).unwrap_or(0.0); if balance > 0.0 { return format!("ERROR: You cannot delete an account that still holds money. Please transfer the remaining ₱{:.2} first!", balance); } let name: String = conn.query_row("SELECT name FROM accounts WHERE id = ?1", params![account_id], |row| row.get(0)).unwrap_or_else(|_| "".to_string()); let _ = conn.execute("INSERT INTO archived_accounts (original_id, name) VALUES (?1, ?2)", params![account_id, name]); let _ = conn.execute("DELETE FROM accounts WHERE id = ?1", params![account_id]); "SUCCESS: Empty account safely moved to the Recycle Bin!".to_string() }
pub fn log_income(conn: &Connection, account_id: i32, amount: f64, description: &str) -> String { let amount = round_to_centavo(amount); let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, account_id]); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'INCOME', ?2, ?3)", params![account_id, amount, description]); "SUCCESS!".to_string() }
pub fn log_expense(conn: &Connection, account_id: i32, amount: f64, description: &str) -> String { let amount = round_to_centavo(amount); let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, account_id]); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'EXPENSE', ?2, ?3)", params![account_id, amount, description]); "SUCCESS!".to_string() }
pub fn transfer_money(conn: &Connection, from_account_id: i32, to_account_id: i32, amount: f64, description: &str) -> String { transfer_money_impl(&conn, from_account_id, to_account_id, amount, description) }
fn transfer_money_impl(conn: &Connection, from_account_id: i32, to_account_id: i32, amount: f64, description: &str) -> String { let amount = round_to_centavo(amount); let current_balance: f64 = conn.query_row("SELECT balance FROM accounts WHERE id = ?1", params![from_account_id], |r| r.get(0)).unwrap_or(0.0); if current_balance < (amount - 0.01) { return format!("ERROR: Insufficient funds! Source account only has ₱{:.2}, but the transfer needs ₱{:.2}.", current_balance, amount); } let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, from_account_id]); let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, to_account_id]); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'TRANSFER_OUT', ?2, ?3)", params![from_account_id, amount, description]); let out_id = conn.last_insert_rowid(); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description, linked_id) VALUES (?1, 'TRANSFER_IN', ?2, ?3, ?4)", params![to_account_id, amount, description, out_id]); let in_id = conn.last_insert_rowid(); let _ = conn.execute("UPDATE transactions SET linked_id = ?1 WHERE id = ?2", params![in_id, out_id]); "SUCCESS!".to_string() }
pub fn undo_transaction(conn: &Connection, transaction_id: i32) -> String {
    
    let mut stmt = conn.prepare("SELECT account_id, tx_type, amount, is_reversed, linked_id FROM transactions WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![transaction_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let account_id: i32 = row.get(0).unwrap();
        let tx_type: String = row.get(1).unwrap();
        let amount: f64 = round_to_centavo(row.get(2).unwrap());
        let already_reversed: i32 = row.get(3).unwrap_or(0);
        let linked_id: Option<i32> = row.get(4).unwrap_or(None);

        if already_reversed != 0 {
            return "ERROR: This transaction has already been undone.".to_string();
        }

        // Reverse the balance based on what kind of entry this was
        match tx_type.as_str() {
            "INCOME" | "TRANSFER_IN" => { let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, account_id]); },
            "EXPENSE" | "TRANSFER_OUT" => { let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, account_id]); },
            _ => {}
        }
        let _ = conn.execute("UPDATE transactions SET is_reversed = 1 WHERE id = ?1", params![transaction_id]);

        // If this was one half of a transfer, reverse its partner too
        if let Some(partner_id) = linked_id {
            let mut p_stmt = conn.prepare("SELECT account_id, tx_type, amount, is_reversed FROM transactions WHERE id = ?1").unwrap();
            let mut p_rows = p_stmt.query(params![partner_id]).unwrap();
            if let Some(p_row) = p_rows.next().unwrap() {
                let p_account_id: i32 = p_row.get(0).unwrap();
                let p_tx_type: String = p_row.get(1).unwrap();
                let p_amount: f64 = round_to_centavo(p_row.get(2).unwrap());
                let p_already_reversed: i32 = p_row.get(3).unwrap_or(0);

                if p_already_reversed == 0 {
                    match p_tx_type.as_str() {
                        "INCOME" | "TRANSFER_IN" => { let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![p_amount, p_account_id]); },
                        "EXPENSE" | "TRANSFER_OUT" => { let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![p_amount, p_account_id]); },
                        _ => {}
                    }
                    let _ = conn.execute("UPDATE transactions SET is_reversed = 1 WHERE id = ?1", params![partner_id]);
                }
            }
        }
    }
    "SUCCESS: Transaction undone!".to_string()
}
pub fn get_all_transactions(conn: &Connection) -> Vec<TransactionLog> { let mut stmt = conn.prepare("SELECT t.id, a.name, t.tx_type, t.amount, t.description, t.is_reversed, t.created_at, COALESCE(pa.name, '') FROM transactions t JOIN accounts a ON t.account_id = a.id LEFT JOIN transactions p ON t.linked_id = p.id LEFT JOIN accounts pa ON p.account_id = pa.id ORDER BY t.id DESC").unwrap(); let iter = stmt.query_map([], |row| { let reversed_int: i32 = row.get(5).unwrap_or(0); Ok(TransactionLog { id: row.get(0).unwrap_or(0), account_name: row.get(1).unwrap_or_else(|_| "".to_string()), tx_type: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), description: row.get(4).unwrap_or_else(|_| "".to_string()), is_reversed: reversed_int != 0, created_at: row.get(6).unwrap_or_else(|_| "".to_string()), linked_account_name: row.get(7).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut logs = Vec::new(); for t in iter { if let Ok(l) = t { logs.push(l); } } logs }
// account_id = None returns the complete history across all accounts; Some(id) returns only that account's entries
pub fn get_transactions_page(conn: &Connection, offset: i32, limit: i32, account_id: Option<i32>) -> Vec<TransactionLog> { get_transactions_page_impl(&conn, offset, limit, account_id) }
fn get_transactions_page_impl(conn: &Connection, offset: i32, limit: i32, account_id: Option<i32>) -> Vec<TransactionLog> { let mut stmt = conn.prepare("SELECT t.id, a.name, t.tx_type, t.amount, t.description, t.is_reversed, t.created_at, COALESCE(pa.name, '') FROM transactions t JOIN accounts a ON t.account_id = a.id LEFT JOIN transactions p ON t.linked_id = p.id LEFT JOIN accounts pa ON p.account_id = pa.id WHERE (?3 IS NULL OR t.account_id = ?3) ORDER BY t.id DESC LIMIT ?1 OFFSET ?2").unwrap(); let iter = stmt.query_map(params![limit, offset, account_id], |row| { let reversed_int: i32 = row.get(5).unwrap_or(0); Ok(TransactionLog { id: row.get(0).unwrap_or(0), account_name: row.get(1).unwrap_or_else(|_| "".to_string()), tx_type: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), description: row.get(4).unwrap_or_else(|_| "".to_string()), is_reversed: reversed_int != 0, created_at: row.get(6).unwrap_or_else(|_| "".to_string()), linked_account_name: row.get(7).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut logs = Vec::new(); for t in iter { if let Ok(l) = t { logs.push(l); } } logs }
// split = true turns a recurring bill into a Sinking Fund pair: "(1st Half)" saved to a fund, "(2nd Half (Due))" paid on the real due date
pub fn add_smart_bill(conn: &Connection, name: &str, amount: f64, due_date: &str, payday_date: &str, is_recurring: bool, split: Option<bool>) -> String { add_smart_bill_impl(&conn, name, amount, due_date, payday_date, is_recurring, split.unwrap_or(false)) }
// A blank payday_date lets the tracker pick the paycheck: the latest payday at least 11 days before the due date,
// the same rule it uses for loans and for next month's recurring bills
fn add_smart_bill_impl(conn: &Connection, name: &str, amount: f64, due_date: &str, payday_date: &str, is_recurring: bool, split: bool) -> String {
    let picked;
    let payday_date = if payday_date.trim().is_empty() {
        let normalized: Option<String> = conn.query_row("SELECT date(?1)", params![due_date], |r| r.get(0)).unwrap_or(None);
        if normalized.as_deref() != Some(due_date) { return "ERROR: That due date isn't a real calendar date.".to_string(); }
        picked = preceding_payday(conn, due_date);
        picked.as_str()
    } else { payday_date };
    if split {
        if !is_recurring { return "ERROR: Only recurring bills can be split into halves.".to_string(); }
        let series_id = next_series_id(conn);
        create_split_pair(conn, name, amount, due_date, payday_date, series_id);
        return "SUCCESS!".to_string();
    }
    match conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5)", params![name, amount, due_date, payday_date, is_recurring as i32]) { Ok(_) => "SUCCESS!".to_string(), Err(_) => "ERROR".to_string() }
}
// For a split bill, new_amount is the FULL bill amount: both unpaid halves of that month get half each
pub fn update_bill_amount(conn: &Connection, bill_id: i32, new_amount: f64) -> String { update_bill_amount_impl(&conn, bill_id, new_amount) }
fn update_bill_amount_impl(conn: &Connection, bill_id: i32, new_amount: f64) -> String {
    let bill: Option<(String, String, i32)> = conn.query_row("SELECT name, due_date, series_id FROM smart_bills WHERE id = ?1", params![bill_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).ok();
    match bill {
        Some((name, due, series_id)) if series_id != 0 => {
            // Work out this month's real due date (the 2nd Half's), then update whichever halves are still unpaid
            let second_half_due: String = if name.contains("(1st Half)") { conn.query_row("SELECT date(?1, '+15 days')", params![due], |r| r.get(0)).unwrap_or(due) } else { due };
            match conn.execute("UPDATE smart_bills SET amount = ?1 / 2.0 WHERE series_id = ?2 AND (due_date = ?3 OR due_date = date(?3, '-15 days'))", params![new_amount, series_id, second_half_due]) { Ok(_) => "SUCCESS: Amount updated! Each half is now half of the full bill.".to_string(), Err(_) => "ERROR: Could not update amount".to_string() }
        }
        _ => match conn.execute("UPDATE smart_bills SET amount = ?1 WHERE id = ?2", params![new_amount, bill_id]) { Ok(_) => "SUCCESS: Amount updated!".to_string(), Err(_) => "ERROR: Could not update amount".to_string() }
    }
}
// Turn an existing recurring bill into a split pair. Its current paycheck stays with the 2nd Half; the 1st Half gets the usual payday rule.
pub fn split_recurring_bill(conn: &Connection, bill_id: i32) -> String { split_recurring_bill_impl(&conn, bill_id) }
fn split_recurring_bill_impl(conn: &Connection, bill_id: i32) -> String {
    let bill: Option<(String, f64, String, String, i32, i32, i32)> = conn.query_row("SELECT name, amount, due_date, payday_date, is_recurring, loan_id, series_id FROM smart_bills WHERE id = ?1", params![bill_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))).ok();
    let Some((name, amount, due, payday, is_recurring, loan_id, series_id)) = bill else { return "ERROR: Could not find that bill.".to_string(); };
    if is_recurring == 0 || loan_id != 0 { return "ERROR: Only recurring bills (not loan payments) can be split.".to_string(); }
    if series_id != 0 { return "ERROR: This bill is already split into halves.".to_string(); }
    let new_series = next_series_id(conn);
    create_split_pair(conn, &name, amount, &due, &payday, new_series);
    let _ = conn.execute("DELETE FROM smart_bills WHERE id = ?1", params![bill_id]);
    format!("SUCCESS: {} is now split into a 1st Half and 2nd Half (Due)!", name)
}
// Same rule the app uses everywhere: the latest payday (15th or end of month) at least 11 days before the due date
fn preceding_payday(conn: &Connection, due: &str) -> String {
    let gap_date: String = conn.query_row("SELECT date(?1, '-11 days')", params![due], |r| r.get(0)).unwrap_or_else(|_| due.to_string());
    conn.query_row(
        "SELECT MAX(c) FROM (
            SELECT date(?1, 'start of month', '-1 month', '+14 days') AS c
            UNION ALL SELECT date(?1, 'start of month', '-1 day')
            UNION ALL SELECT date(?1, 'start of month', '+14 days')
            UNION ALL SELECT date(?1, 'start of month', '+1 month', '-1 day')
         ) WHERE c <= ?1",
        params![gap_date],
        |r| r.get(0)
    ).unwrap_or_else(|_| due.to_string())
}
fn next_series_id(conn: &Connection) -> i32 {
    conn.query_row("SELECT COALESCE(MAX(s), 0) + 1 FROM (SELECT MAX(series_id) AS s FROM smart_bills UNION ALL SELECT MAX(series_id) FROM paid_bills UNION ALL SELECT MAX(series_id) FROM archived_smart_bills)", [], |r| r.get(0)).unwrap_or(1)
}
// Creates one month's pair. second_half_due is the real due date; the 1st Half is due 15 days earlier, like loan halves.
fn create_split_pair(conn: &Connection, base_name: &str, full_amount: f64, second_half_due: &str, second_half_payday: &str, series_id: i32) {
    let first_half_due: String = conn.query_row("SELECT date(?1, '-15 days')", params![second_half_due], |r| r.get(0)).unwrap();
    let first_half_payday = preceding_payday(conn, &first_half_due);
    // Unrounded on purpose (like loan halves): the 2nd Half charges double its amount, so rounding here could make the full payment a cent off
    let half = full_amount / 2.0;
    let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, is_recurring, series_id) VALUES (?1, ?2, ?3, ?4, 0, 1, ?5)", params![format!("{} (1st Half)", base_name), half, first_half_due, first_half_payday, series_id]);
    let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, is_recurring, series_id) VALUES (?1, ?2, ?3, ?4, 0, 1, ?5)", params![format!("{} (2nd Half (Due))", base_name), half, second_half_due, second_half_payday, series_id]);
}
// Reassign one unpaid bill to a different paycheck allocation (only this occurrence; next month's recurring copy still uses the normal payday rule)
pub fn move_bill_allocation(conn: &Connection, bill_id: i32, new_payday: &str) -> String { move_bill_allocation_impl(&conn, bill_id, new_payday) }
fn move_bill_allocation_impl(conn: &Connection, bill_id: i32, new_payday: &str) -> String {
    // SQLite's date() returns NULL for impossible dates like 2026-02-30, so this doubles as validation
    let normalized: Option<String> = conn.query_row("SELECT date(?1)", params![new_payday], |r| r.get(0)).unwrap_or(None);
    if normalized.as_deref() != Some(new_payday) {
        return "ERROR: That paycheck date isn't a real calendar date.".to_string();
    }
    match conn.execute("UPDATE smart_bills SET payday_date = ?1 WHERE id = ?2", params![new_payday, bill_id]) {
        Ok(1) => "SUCCESS: Bill moved to the new paycheck!".to_string(),
        _ => "ERROR: Could not find that bill. It may have already been paid.".to_string(),
    }
}
pub fn get_smart_bills(conn: &Connection) -> Vec<SmartBill> { let mut stmt = conn.prepare("SELECT id, name, amount, due_date, payday_date, is_recurring, loan_id, series_id FROM smart_bills").unwrap(); let iter = stmt.query_map([], |row| { let recurring_int: i32 = row.get(5).unwrap_or(0); Ok(SmartBill { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), amount: row.get(2).unwrap_or(0.0), due_date: row.get(3).unwrap_or_else(|_| "".to_string()), payday_date: row.get(4).unwrap_or_else(|_| "".to_string()), is_recurring: recurring_int != 0, loan_id: row.get(6).unwrap_or(0), series_id: row.get(7).unwrap_or(0) }) }).unwrap(); let mut bills = Vec::new(); for b in iter { if let Ok(item) = b { bills.push(item); } } bills }
pub fn add_loan(conn: &Connection, name: &str, principal: f64, remaining_balance: f64, term: i32, term_type: &str, frequency: &str, payment_amount: f64, due_date: &str, payday_date: &str, total_incurred: f64, due_structure: &str, date_acquired: &str, bill_names: Vec<String>, bill_amounts: Vec<f64>, bill_dues: Vec<String>, bill_paydays: Vec<String>) -> String { let _ = conn.execute("INSERT INTO loans (name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![name, principal, remaining_balance, term, term_type, frequency, payment_amount, due_date, payday_date, total_incurred, due_structure, date_acquired]); let new_loan_id = conn.last_insert_rowid(); for i in 0..bill_names.len() { let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![bill_names[i], bill_amounts[i], bill_dues[i], bill_paydays[i], new_loan_id]); } "SUCCESS: Loan generated!".to_string() }
pub fn get_loans(conn: &Connection) -> Vec<Loan> { let mut stmt = conn.prepare("SELECT id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM loans").unwrap(); let iter = stmt.query_map([], |row| { Ok(Loan { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), principal: row.get(2).unwrap_or(0.0), remaining_balance: row.get(3).unwrap_or(0.0), term: row.get(4).unwrap_or(0), term_type: row.get(5).unwrap_or_else(|_| "".to_string()), frequency: row.get(6).unwrap_or_else(|_| "".to_string()), payment_amount: row.get(7).unwrap_or(0.0), next_due_date: row.get(8).unwrap_or_else(|_| "".to_string()), payday_date: row.get(9).unwrap_or_else(|_| "".to_string()), total_incurred: row.get(10).unwrap_or(0.0), due_structure: row.get(11).unwrap_or_else(|_| "Sinking Fund".to_string()), date_acquired: row.get(12).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut loans = Vec::new(); for l in iter { if let Ok(item) = l { loans.push(item); } } loans }
pub fn delete_loan(conn: &Connection, loan_id: i32) -> String { let mut stmt = conn.prepare("SELECT name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM loans WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![loan_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let principal: f64 = row.get(1).unwrap(); let remaining_balance: f64 = row.get(2).unwrap(); let term: i32 = row.get(3).unwrap(); let term_type: String = row.get(4).unwrap(); let frequency: String = row.get(5).unwrap(); let payment_amount: f64 = row.get(6).unwrap(); let next_due_date: String = row.get(7).unwrap(); let payday_date: String = row.get(8).unwrap(); let total_incurred: f64 = row.get(9).unwrap_or(0.0); let due_structure: String = row.get(10).unwrap_or_else(|_| "Sinking Fund".to_string()); let date_acquired: String = row.get(11).unwrap_or_else(|_| "".to_string()); let _ = conn.execute("INSERT INTO archived_loans (original_id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", params![loan_id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired]); // Only this loan's installments: matched by loan_id (or, for bills from before loan_id existed, by its exact name), never by a prefix that "Car Loan 2" would also match
 let _ = conn.execute("DELETE FROM smart_bills WHERE loan_id = ?1 OR (loan_id = 0 AND substr(name, 1, length(?2) + 2) = ?2 || ' (')", params![loan_id, name]); } let _ = conn.execute("DELETE FROM loans WHERE id = ?1", params![loan_id]); "SUCCESS: Loan moved to Recycle Bin!".to_string() }

pub fn fund_bill_to_savings(conn: &Connection, bill_id: i32, from_account_id: i32, to_account_id: i32, amount: f64, description: &str) -> String {
        fund_bill_to_savings_impl(&conn, bill_id, from_account_id, to_account_id, amount, description)
}
fn fund_bill_to_savings_impl(conn: &Connection, bill_id: i32, from_account_id: i32, to_account_id: i32, amount: f64, description: &str) -> String {
    // Also stored on the receipt, so the later auto-transfer and Undo move exactly what was funded
    let amount = round_to_centavo(amount);
    // 1. Get bill details (now also grabbing loan_id!)
    let mut stmt = conn.prepare("SELECT name, due_date, payday_date, loan_id, series_id, is_recurring FROM smart_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let bill_name: String = row.get(0).unwrap();
        let due: String = row.get(1).unwrap();
        let payday: String = row.get(2).unwrap();
        let bill_loan_id: i32 = row.get(3).unwrap_or(0);
        let series_id: i32 = row.get(4).unwrap_or(0);
        let is_recurring: i32 = row.get(5).unwrap_or(0);

        // 🛡️ SAFETY CHECK: Does the source account actually have enough money?
        let current_balance: f64 = conn.query_row(
            "SELECT balance FROM accounts WHERE id = ?1",
            params![from_account_id],
            |r| r.get(0)
        ).unwrap_or(0.0);

        if current_balance < (amount - 0.01) {
            return format!("ERROR: Insufficient funds! This account only has ₱{:.2}, but the bill needs ₱{:.2}.", current_balance, amount);
        }
        
        // 2. Move to paid_bills (this acts as your receipt/sinking fund archive) — now correctly tagged with loan_id AND from_account_id
        let _ = conn.execute("INSERT INTO paid_bills (original_bill_id, name, amount, due_date, payday_date, account_id, loan_id, from_account_id, series_id, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)", params![bill_id, bill_name, amount, due, payday, to_account_id, bill_loan_id, from_account_id, series_id, is_recurring]);
        let _ = conn.execute("DELETE FROM smart_bills WHERE id = ?1", params![bill_id]);
        
        // 3. Move money from source account to destination savings account
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, from_account_id]);
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, to_account_id]);
        
        // 4. Log transactions for both accounts, linked together via description matching (used for undo later)
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'EXPENSE', ?2, ?3)", params![from_account_id, amount, description]);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'INCOME', ?2, ?3)", params![to_account_id, amount, description]);
    }
    
    "SUCCESS: Funded to Savings!".to_string()
}
pub fn pay_bill(conn: &Connection, bill_id: i32, account_id: i32, amount: f64, description: &str) -> String {
        pay_bill_impl(&conn, bill_id, account_id, amount, description)
}
fn pay_bill_impl(conn: &Connection, bill_id: i32, account_id: i32, amount: f64, description: &str) -> String {
    // 1. Get details of the bill being paid
    let mut stmt = conn.prepare("SELECT name, due_date, payday_date, loan_id, is_recurring, series_id FROM smart_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let bill_name: String = row.get(0).unwrap();
        let due: String = row.get(1).unwrap();
        let payday: String = row.get(2).unwrap();
        let bill_loan_id: i32 = row.get(3).unwrap_or(0);
        let is_recurring_int: i32 = row.get(4).unwrap_or(0);
        let is_recurring = is_recurring_int != 0;
        let series_id: i32 = row.get(5).unwrap_or(0);

        // Look up this loan's due_structure and name by its actual ID — never guess from text again
        let (loan_name, due_structure): (String, String) = conn.query_row(
            "SELECT name, due_structure FROM loans WHERE id = ?1",
            params![bill_loan_id],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or_else(|_| {
            let fallback_name = if let Some(idx) = bill_name.find(" (") { bill_name[..idx].to_string() } else { bill_name.clone() };
            (fallback_name, "Sinking Fund".to_string())
        });

        let is_sinking_fund_second_half = due_structure == "Sinking Fund" && (bill_name.contains("(2nd Half (Due))") || bill_name.contains("(2nd Half)"));

        // This is the REAL amount that should leave the account AND come off the loan balance
        let real_amount: f64 = round_to_centavo(if is_sinking_fund_second_half { amount * 2.0 } else { amount });

        // 🛡️ SAFETY CHECK: Does this account actually have enough for the REAL amount owed?
        let current_balance: f64 = conn.query_row(
            "SELECT balance FROM accounts WHERE id = ?1",
            params![account_id],
            |r| r.get(0)
        ).unwrap_or(0.0);

        if current_balance < (real_amount - 0.01) {
            return format!("ERROR: Insufficient funds! This account only has ₱{:.2}, but the full payment needs ₱{:.2}.", current_balance, real_amount);
        }
        
        // Move to paid bills (still logs the bill's own listed amount, for accurate receipts) — now also tagged with loan_id
        let _ = conn.execute("INSERT INTO paid_bills (original_bill_id, name, amount, due_date, payday_date, account_id, loan_id, series_id, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![bill_id, bill_name, amount, due, payday, account_id, bill_loan_id, series_id, is_recurring_int]);
        let _ = conn.execute("DELETE FROM smart_bills WHERE id = ?1", params![bill_id]);

        // 🔁 RECURRING BILL: automatically create next month's occurrence, amount reset to TBD (0)
        if is_recurring {
            // Next month's due date, same day (e.g. Sep 20 -> Oct 20)
            let next_due: String = conn.query_row("SELECT date(?1, '+1 month')", params![due], |r| r.get(0)).unwrap_or_else(|_| due.clone());
            // Same 10-day rule as loans: the paycheck must land at least 11 days before the due date
            let next_payday = preceding_payday(conn, &next_due);
            if series_id != 0 {
                // Split bill: next month's pair is created once, when this month's 2nd Half (the real due date) is paid
                if let Some(idx) = bill_name.find(" (2nd Half") {
                    create_split_pair(conn, &bill_name[..idx], 0.0, &next_due, &next_payday, series_id);
                }
            } else {
                let _ = conn.execute(
                    "INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, is_recurring) VALUES (?1, 0, ?2, ?3, 0, 1)",
                    params![bill_name, next_due, next_payday]
                );
            }
        }
        
        // Deduct the REAL full amount from the account and log it
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![real_amount, account_id]);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'EXPENSE', ?2, ?3)", params![account_id, real_amount, description]);

        if is_sinking_fund_second_half {
            // Match the 1st Half by loan_id + series_id + exact 15-day relationship — no more name/date guesswork
            let _ = conn.execute(
                "INSERT INTO paid_bills (original_bill_id, name, amount, due_date, payday_date, account_id, loan_id, series_id, is_recurring)
                 SELECT id, name, amount, due_date, payday_date, ?1, loan_id, series_id, is_recurring FROM smart_bills
                 WHERE loan_id = ?2 AND series_id = ?4 AND name LIKE '%(1st Half)%' AND due_date = date(?3, '-15 days')",
                params![account_id, bill_loan_id, due, series_id]
            );
            let _ = conn.execute(
                "DELETE FROM smart_bills WHERE loan_id = ?1 AND series_id = ?3 AND name LIKE '%(1st Half)%' AND due_date = date(?2, '-15 days')",
                params![bill_loan_id, due, series_id]
            );
        }

        // Loan's remaining balance drops by the same REAL amount that left the account — matched by ID, not name
        let _ = conn.execute("UPDATE loans SET remaining_balance = ROUND(remaining_balance - ?1, 2) WHERE id = ?2", params![real_amount, bill_loan_id]);

        // Check if this payment fully paid off the loan. If so, graduate it to Completed Loans!
        let mut check_stmt = conn.prepare("SELECT id, principal, remaining_balance, term, term_type, frequency, payment_amount, total_incurred FROM loans WHERE id = ?1").unwrap();
        let mut check_rows = check_stmt.query(params![bill_loan_id]).unwrap();
        if let Some(loan_row) = check_rows.next().unwrap() {
            let loan_id: i32 = loan_row.get(0).unwrap();
            let l_principal: f64 = loan_row.get(1).unwrap();
            let l_remaining: f64 = loan_row.get(2).unwrap();
            let l_term: i32 = loan_row.get(3).unwrap();
            let l_term_type: String = loan_row.get(4).unwrap();
            let l_frequency: String = loan_row.get(5).unwrap();
            let l_payment_amount: f64 = loan_row.get(6).unwrap();
            let l_total_incurred: f64 = loan_row.get(7).unwrap();

            if l_remaining <= 0.05 {
                let _ = conn.execute(
                    "INSERT INTO completed_loans (original_id, name, principal, term, term_type, frequency, payment_amount, total_incurred, completed_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![loan_id, loan_name, l_principal, l_term, l_term_type, l_frequency, l_payment_amount, l_total_incurred, due]
                );
                let _ = conn.execute("DELETE FROM smart_bills WHERE loan_id = ?1", params![loan_id]);
                let _ = conn.execute("DELETE FROM loans WHERE id = ?1", params![loan_id]);
            }
        }
    }
    "SUCCESS: Paid!".to_string()
}
pub fn undo_bill_payment(conn: &Connection, paid_bill_id: i32) -> String {
        undo_bill_payment_impl(&conn, paid_bill_id)
}
fn undo_bill_payment_impl(conn: &Connection, paid_bill_id: i32) -> String {
    let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date, account_id, loan_id, series_id, is_recurring FROM paid_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![paid_bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let name: String = row.get(0).unwrap();
        let amount: f64 = row.get(1).unwrap();
        let due: String = row.get(2).unwrap();
        let payday: String = row.get(3).unwrap();
        let account_id: i32 = row.get(4).unwrap();
        let loan_id: i32 = row.get(5).unwrap_or(0);
        let series_id: i32 = row.get(6).unwrap_or(0);
        let is_recurring: i32 = row.get(7).unwrap_or(0);

        // Figure out if this was a Sinking Fund 2nd Half (meaning the REAL amount taken was double the listed amount)
        let due_structure: String = conn.query_row(
            "SELECT due_structure FROM loans WHERE id = ?1",
            params![loan_id],
            |r| r.get(0)
        ).unwrap_or_else(|_| "Sinking Fund".to_string());

        let is_sinking_fund_second_half = due_structure == "Sinking Fund" && (name.contains("(2nd Half (Due))") || name.contains("(2nd Half)"));
        let real_amount: f64 = round_to_centavo(if is_sinking_fund_second_half { amount * 2.0 } else { amount });

        // 1. Refund the REAL amount back to the account it was taken from
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![real_amount, account_id]);
        let refund_desc = format!("REFUND (Undo): {}", name);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'INCOME', ?2, ?3)", params![account_id, real_amount, refund_desc]);

        // 2. If this loan still exists (wasn't graduated to Completed), restore its remaining_balance
        let loan_exists: bool = conn.query_row("SELECT 1 FROM loans WHERE id = ?1", params![loan_id], |_| Ok(true)).unwrap_or(false);
        if loan_exists {
            let _ = conn.execute("UPDATE loans SET remaining_balance = ROUND(remaining_balance + ?1, 2) WHERE id = ?2", params![real_amount, loan_id]);
        } else {
            // The loan was graduated to Completed Loans by this exact payment — bring it back to Active Loans
            let mut cl_stmt = conn.prepare("SELECT original_id, name, principal, term, term_type, frequency, payment_amount, total_incurred FROM completed_loans WHERE original_id = ?1 ORDER BY id DESC LIMIT 1").unwrap();
            let mut cl_rows = cl_stmt.query(params![loan_id]).unwrap();
            if let Some(cl_row) = cl_rows.next().unwrap() {
                let original_id: i32 = cl_row.get(0).unwrap();
                let cl_name: String = cl_row.get(1).unwrap();
                let cl_principal: f64 = cl_row.get(2).unwrap();
                let cl_term: i32 = cl_row.get(3).unwrap();
                let cl_term_type: String = cl_row.get(4).unwrap();
                let cl_frequency: String = cl_row.get(5).unwrap();
                let cl_payment_amount: f64 = cl_row.get(6).unwrap();
                let cl_total_incurred: f64 = cl_row.get(7).unwrap();

                let _ = conn.execute(
                    "INSERT INTO loans (id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![original_id, cl_name, cl_principal, real_amount, cl_term, cl_term_type, cl_frequency, cl_payment_amount, due, payday, cl_total_incurred, due_structure]
                );
                let _ = conn.execute("DELETE FROM completed_loans WHERE original_id = ?1", params![loan_id]);
            }
        }

        // 3. Restore this bill to smart_bills (with its correct loan_id, recurring flag, and split series)
        let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, series_id, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![name, amount, due, payday, loan_id, series_id, is_recurring]);

        // 4. If it was a Sinking Fund 2nd Half, also restore the matching 1st Half that got auto-cleared
        if is_sinking_fund_second_half {
            let _ = conn.execute(
                "INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, series_id, is_recurring)
                 SELECT name, amount, due_date, payday_date, loan_id, series_id, is_recurring FROM paid_bills
                 WHERE loan_id = ?1 AND series_id = ?3 AND name LIKE '%(1st Half)%' AND due_date = date(?2, '-15 days')
                 ORDER BY id DESC LIMIT 1",
                params![loan_id, due, series_id]
            );
            let _ = conn.execute(
                "DELETE FROM paid_bills WHERE id = (SELECT id FROM paid_bills WHERE loan_id = ?1 AND series_id = ?3 AND name LIKE '%(1st Half)%' AND due_date = date(?2, '-15 days') ORDER BY id DESC LIMIT 1)",
                params![loan_id, due, series_id]
            );
        }

        // 5. Remove the auto-generated NEXT recurring bill(s) this payment may have created (only if still untouched/TBD)
        if series_id != 0 {
            let _ = conn.execute(
                "DELETE FROM smart_bills WHERE series_id = ?1 AND loan_id = 0 AND is_recurring = 1 AND amount = 0 AND (due_date = date(?2, '+1 month') OR due_date = date(date(?2, '+1 month'), '-15 days'))",
                params![series_id, due]
            );
        } else {
            let _ = conn.execute(
                "DELETE FROM smart_bills WHERE name = ?1 AND loan_id = 0 AND series_id = 0 AND is_recurring = 1 AND amount = 0 AND due_date = date(?2, '+1 month')",
                params![name, due]
            );
        }

        // 6. Remove this paid receipt
        let _ = conn.execute("DELETE FROM paid_bills WHERE id = ?1", params![paid_bill_id]);
    }
    "SUCCESS: Payment undone and refunded!".to_string()
}
pub fn get_paid_bills(conn: &Connection) -> Vec<PaidBill> { let mut stmt = conn.prepare("SELECT id, original_bill_id, name, amount, due_date, payday_date, account_id, from_account_id, loan_id, paid_at FROM paid_bills ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(PaidBill { id: row.get(0).unwrap_or(0), original_bill_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), due_date: row.get(4).unwrap_or_else(|_| "".to_string()), payday_date: row.get(5).unwrap_or_else(|_| "".to_string()), account_id: row.get(6).unwrap_or(0), from_account_id: row.get(7).unwrap_or(0), loan_id: row.get(8).unwrap_or(0), paid_at: row.get(9).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut archived = Vec::new(); for b in iter { if let Ok(item) = b { archived.push(item); } } archived }
pub fn get_paid_bills_page(conn: &Connection, offset: i32, limit: i32) -> Vec<PaidBill> { let mut stmt = conn.prepare("SELECT id, original_bill_id, name, amount, due_date, payday_date, account_id, from_account_id, loan_id, paid_at FROM paid_bills ORDER BY id DESC LIMIT ?1 OFFSET ?2").unwrap(); let iter = stmt.query_map(params![limit, offset], |row| { Ok(PaidBill { id: row.get(0).unwrap_or(0), original_bill_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), due_date: row.get(4).unwrap_or_else(|_| "".to_string()), payday_date: row.get(5).unwrap_or_else(|_| "".to_string()), account_id: row.get(6).unwrap_or(0), from_account_id: row.get(7).unwrap_or(0), loan_id: row.get(8).unwrap_or(0), paid_at: row.get(9).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut archived = Vec::new(); for b in iter { if let Ok(item) = b { archived.push(item); } } archived }
pub fn get_archived_loans(conn: &Connection) -> Vec<Loan> { let mut stmt = conn.prepare("SELECT id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM archived_loans ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(Loan { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), principal: row.get(2).unwrap_or(0.0), remaining_balance: row.get(3).unwrap_or(0.0), term: row.get(4).unwrap_or(0), term_type: row.get(5).unwrap_or_else(|_| "".to_string()), frequency: row.get(6).unwrap_or_else(|_| "".to_string()), payment_amount: row.get(7).unwrap_or(0.0), next_due_date: row.get(8).unwrap_or_else(|_| "".to_string()), payday_date: row.get(9).unwrap_or_else(|_| "".to_string()), total_incurred: row.get(10).unwrap_or(0.0), due_structure: row.get(11).unwrap_or_else(|_| "Sinking Fund".to_string()), date_acquired: row.get(12).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut loans = Vec::new(); for l in iter { if let Ok(item) = l { loans.push(item); } } loans }
pub fn get_completed_loans(conn: &Connection) -> Vec<CompletedLoan> { let mut stmt = conn.prepare("SELECT id, original_id, name, principal, term, term_type, frequency, payment_amount, total_incurred, completed_date FROM completed_loans ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(CompletedLoan { id: row.get(0).unwrap_or(0), original_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), principal: row.get(3).unwrap_or(0.0), term: row.get(4).unwrap_or(0), term_type: row.get(5).unwrap_or_else(|_| "".to_string()), frequency: row.get(6).unwrap_or_else(|_| "".to_string()), payment_amount: row.get(7).unwrap_or(0.0), total_incurred: row.get(8).unwrap_or(0.0), completed_date: row.get(9).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut completed = Vec::new(); for l in iter { if let Ok(item) = l { completed.push(item); } } completed }
pub fn undo_delete_loan(conn: &Connection, archive_id: i32, bill_names: Vec<String>, bill_amounts: Vec<f64>, bill_dues: Vec<String>, bill_paydays: Vec<String>) -> String { let mut stmt = conn.prepare("SELECT name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM archived_loans WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![archive_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let principal: f64 = row.get(1).unwrap(); let remaining_balance: f64 = row.get(2).unwrap(); let term: i32 = row.get(3).unwrap(); let term_type: String = row.get(4).unwrap(); let frequency: String = row.get(5).unwrap(); let payment_amount: f64 = row.get(6).unwrap(); let next_due_date: String = row.get(7).unwrap(); let payday_date: String = row.get(8).unwrap(); let total_incurred: f64 = row.get(9).unwrap_or(0.0); let due_structure: String = row.get(10).unwrap_or_else(|_| "Sinking Fund".to_string()); let date_acquired: String = row.get(11).unwrap_or_else(|_| "".to_string()); let _ = conn.execute("INSERT INTO loans (name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired]); let restored_loan_id = conn.last_insert_rowid(); for i in 0..bill_names.len() { let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![bill_names[i], bill_amounts[i], bill_dues[i], bill_paydays[i], restored_loan_id]); } let _ = conn.execute("DELETE FROM archived_loans WHERE id = ?1", params![archive_id]); } "SUCCESS: Loan restored!".to_string() }
pub fn delete_bill(conn: &Connection, bill_id: i32) -> String { let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date, series_id, is_recurring FROM smart_bills WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![bill_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let amount: f64 = row.get(1).unwrap(); let due: String = row.get(2).unwrap(); let payday: String = row.get(3).unwrap(); let series_id: i32 = row.get(4).unwrap_or(0); let is_recurring: i32 = row.get(5).unwrap_or(0); let _ = conn.execute("INSERT INTO archived_smart_bills (original_id, name, amount, due_date, payday_date, series_id, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![bill_id, name, amount, due, payday, series_id, is_recurring]); } let _ = conn.execute("DELETE FROM smart_bills WHERE id = ?1", params![bill_id]); "SUCCESS: Bill moved to Recycle Bin!".to_string() }
pub fn get_archived_bills(conn: &Connection) -> Vec<ArchivedBill> { let mut stmt = conn.prepare("SELECT id, original_id, name, amount, due_date, payday_date FROM archived_smart_bills ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(ArchivedBill { id: row.get(0).unwrap_or(0), original_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), due_date: row.get(4).unwrap_or_else(|_| "".to_string()), payday_date: row.get(5).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut archived = Vec::new(); for b in iter { if let Ok(item) = b { archived.push(item); } } archived }
pub fn undo_delete_bill(conn: &Connection, archive_id: i32) -> String { let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date, series_id, is_recurring FROM archived_smart_bills WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![archive_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let amount: f64 = row.get(1).unwrap(); let due: String = row.get(2).unwrap(); let payday: String = row.get(3).unwrap(); let series_id: i32 = row.get(4).unwrap_or(0); let is_recurring: i32 = row.get(5).unwrap_or(0); let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, series_id, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![name, amount, due, payday, series_id, is_recurring]); let _ = conn.execute("DELETE FROM archived_smart_bills WHERE id = ?1", params![archive_id]); } "SUCCESS: Bill restored to Budget!".to_string() }
pub fn get_sinking_fund_info(conn: &Connection, bill_id: i32) -> SinkingFundInfo {
        get_sinking_fund_info_impl(&conn, bill_id)
}
fn get_sinking_fund_info_impl(conn: &Connection, bill_id: i32) -> SinkingFundInfo {
    let mut stmt = conn.prepare("SELECT due_date, loan_id, series_id FROM smart_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let due: String = row.get(0).unwrap();
        let bill_loan_id: i32 = row.get(1).unwrap_or(0);
        let series_id: i32 = row.get(2).unwrap_or(0);

        // Only Sinking Fund loans save their 1st Half to a fund. A True Bi-Monthly 1st Half is a real payment, not a fund.
        // (Split recurring bills have no loan and are always Sinking Fund, same default pay_bill uses.)
        let due_structure: String = conn.query_row("SELECT due_structure FROM loans WHERE id = ?1", params![bill_loan_id], |r| r.get(0)).unwrap_or_else(|_| "Sinking Fund".to_string());
        if due_structure != "Sinking Fund" {
            return SinkingFundInfo { found: false, account_id: 0, account_name: "".to_string(), amount: 0.0 };
        }

        // Match by loan_id + series_id (never confused by similar names) + the exact 15-day relationship
        let result = conn.query_row(
            "SELECT pb.account_id, a.name, pb.amount FROM paid_bills pb
             JOIN accounts a ON pb.account_id = a.id
             WHERE pb.loan_id = ?1 AND pb.series_id = ?3 AND pb.name LIKE '%(1st Half)%' AND pb.due_date = date(?2, '-15 days')
             ORDER BY pb.id DESC
             LIMIT 1",
            params![bill_loan_id, due, series_id],
            |r| Ok(SinkingFundInfo { found: true, account_id: r.get(0)?, account_name: r.get(1)?, amount: r.get(2)? })
        );

        return result.unwrap_or(SinkingFundInfo { found: false, account_id: 0, account_name: "".to_string(), amount: 0.0 });
    }

    SinkingFundInfo { found: false, account_id: 0, account_name: "".to_string(), amount: 0.0 }
}

pub fn repair_balance_rounding(conn: &Connection) -> String {
        let updated = conn.execute("UPDATE accounts SET balance = ROUND(balance, 2)", []).unwrap_or(0);
    format!("SUCCESS: Cleaned up rounding on {} account(s)!", updated)
}

pub fn repair_loan_ids(conn: &Connection) -> String {
    
    let mut stmt = conn.prepare("SELECT id, name FROM loans").unwrap();
    let loan_iter = stmt.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    }).unwrap();

    let mut total_repaired = 0;

    for loan in loan_iter {
        if let Ok((loan_id, loan_name)) = loan {
            let pattern = format!("{} (%", loan_name);

            let smart_count = conn.execute(
                "UPDATE smart_bills SET loan_id = ?1 WHERE loan_id = 0 AND name LIKE ?2",
                params![loan_id, pattern]
            ).unwrap_or(0);

            let paid_count = conn.execute(
                "UPDATE paid_bills SET loan_id = ?1 WHERE loan_id = 0 AND name LIKE ?2",
                params![loan_id, pattern]
            ).unwrap_or(0);

            total_repaired += smart_count + paid_count;
        }
    }

    format!("SUCCESS: Repaired {} bill(s) across all loans!", total_repaired)
}
pub fn undo_fund_to_savings(conn: &Connection, paid_bill_id: i32) -> String {
        undo_fund_to_savings_impl(&conn, paid_bill_id)
}
fn undo_fund_to_savings_impl(conn: &Connection, paid_bill_id: i32) -> String {
    let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date, account_id, loan_id, from_account_id, series_id, is_recurring FROM paid_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![paid_bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let raw_name: String = row.get(0).unwrap();
        let amount: f64 = round_to_centavo(row.get(1).unwrap());
        let due: String = row.get(2).unwrap();
        let payday: String = row.get(3).unwrap();
        let to_account_id: i32 = row.get(4).unwrap();
        let loan_id: i32 = row.get(5).unwrap_or(0);
        let from_account_id: i32 = row.get(6).unwrap_or(0);
        let series_id: i32 = row.get(7).unwrap_or(0);
        let is_recurring: i32 = row.get(8).unwrap_or(0);

        // Strip off any "(Handled in X)" tag that may have been appended to the name, to restore the original bill name
        let clean_name = if let Some(idx) = raw_name.find(" (Handled in") { raw_name[..idx].to_string() } else { raw_name.clone() };

        // 🛡️ SAFETY CHECK: Can we actually reverse this? The destination account needs enough to give back.
        let dest_balance: f64 = conn.query_row("SELECT balance FROM accounts WHERE id = ?1", params![to_account_id], |r| r.get(0)).unwrap_or(0.0);
        if dest_balance < (amount - 0.01) {
            return format!("ERROR: Cannot undo! The sinking fund account only has ₱{:.2} left (some may have already been used), but ₱{:.2} needs to be reversed.", dest_balance, amount);
        }

        // Reverse the transfer: take it back out of the destination, give it back to the original source
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, to_account_id]);
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, from_account_id]);    

        let undo_desc = format!("Undo Fund Transfer: {}", clean_name);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'EXPENSE', ?2, ?3)", params![to_account_id, amount, undo_desc]);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'INCOME', ?2, ?3)", params![from_account_id, amount, undo_desc]);

        // Restore the original bill (clean name, no "Handled in" tag) back to smart_bills
        let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, series_id, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![clean_name, amount, due, payday, loan_id, series_id, is_recurring]);

        // Remove this receipt
        let _ = conn.execute("DELETE FROM paid_bills WHERE id = ?1", params![paid_bill_id]);
    }
    "SUCCESS: Fund transfer undone!".to_string()
}

pub fn mark_first_half_handled(conn: &Connection, bill_id: i32, account_name: &str) -> String {
        
    let updated_suffix = format!(" (Handled in {})", account_name);
    
    // Append the handled tag to whichever table currently holds the bill
    let _ = conn.execute("UPDATE paid_bills SET name = name || ?1 WHERE original_bill_id = ?2 AND name NOT LIKE '%(Handled in%'", params![updated_suffix, bill_id]);
    let _ = conn.execute("UPDATE smart_bills SET name = name || ?1 WHERE id = ?2 AND name NOT LIKE '%(Handled in%'", params![updated_suffix, bill_id]);
    
    "SUCCESS: 1st Half Sinking Fund Logged!".to_string()
}

// ---- Editing names and details ----
// Some names carry meaning: loan installments are stored as "Loan (1st Half)", split bills as "Bill (2nd Half (Due))", and a
// 1st Half saved to a fund gets " (Handled in Account)". So every rename below changes the part people chose and keeps the rest.

// Text the payment rules look for inside names; a name someone types must not contain it, or a plain bill could be taken for a half
fn has_reserved_text(name: &str) -> bool { ["(1st Half)", "(2nd Half", "(Handled in", "(Due Date)"].iter().any(|t| name.contains(t)) }

fn checked_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() { return Err("ERROR: Please enter a name.".to_string()); }
    if has_reserved_text(name) { return Err("ERROR: Names can't include \"(1st Half)\", \"(2nd Half\", \"(Due Date)\" or \"(Handled in\". Vault adds those itself.".to_string()); }
    Ok(name.to_string())
}

/// Where the part Vault added begins: " (1st Half)…" or " (2nd Half…" for a split bill, or the whole name if there is none
fn split_suffix_at(name: &str) -> usize { [" (1st Half)", " (2nd Half"].iter().filter_map(|t| name.find(t)).min().unwrap_or(name.len()) }

/// Renames an account. Bills saved to it show its name in their "(Handled in …)" tag (in the Budget and the loan schedules), so those follow.
pub fn rename_account(conn: &Connection, account_id: i32, new_name: &str) -> String {
    let new_name = match checked_name(new_name) { Ok(n) => n, Err(e) => return e };
    let Ok(old_name) = conn.query_row("SELECT name FROM accounts WHERE id = ?1", params![account_id], |r| r.get::<_, String>(0)) else { return "ERROR: Could not find that account.".to_string(); };
    if old_name == new_name { return "SUCCESS: Nothing to change.".to_string(); }
    let taken: bool = conn.query_row("SELECT 1 FROM accounts WHERE lower(name) = lower(?1) AND id != ?2", params![new_name, account_id], |_| Ok(true)).unwrap_or(false);
    if taken { return format!("ERROR: You already have an account called '{}'.", new_name); }
    if conn.execute("UPDATE accounts SET name = ?1 WHERE id = ?2", params![new_name, account_id]).is_err() { return "ERROR: Could not rename the account.".to_string(); }
    let (old_tag, new_tag) = (format!(" (Handled in {})", old_name), format!(" (Handled in {})", new_name));
    for table in ["smart_bills", "paid_bills"] {
        let _ = conn.execute(&format!("UPDATE {} SET name = replace(name, ?1, ?2) WHERE instr(name, ?1) > 0", table), params![old_tag, new_tag]);
    }
    format!("SUCCESS: Renamed '{}' to '{}'!", old_name, new_name)
}

/// Edits one unpaid bill that isn't a loan payment: its name, amount and real due date. For a split bill, `amount` is the full
/// bill and `due_date` is the real due date (the 2nd Half's); both unpaid halves of that month change together.
/// A new due date also moves the bill to the paycheck the usual rule picks, since the old one may no longer come before it.
pub fn edit_bill(conn: &Connection, bill_id: i32, name: &str, amount: f64, due_date: &str) -> String {
    let bill: Option<(String, String, String, i32, i32, i32)> = conn.query_row("SELECT name, due_date, payday_date, loan_id, series_id, is_recurring FROM smart_bills WHERE id = ?1", params![bill_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))).ok();
    let Some((old_name, old_due, old_payday, loan_id, series_id, is_recurring)) = bill else { return "ERROR: Could not find that bill. It may have already been paid.".to_string(); };
    if loan_id != 0 { return "ERROR: Loan payments are edited from Active Loans in the Loan Generator.".to_string(); }
    let new_base = match checked_name(name) { Ok(n) => n, Err(e) => return e };
    if !(amount >= 0.0) { return "ERROR: Please enter a valid amount.".to_string(); }
    if amount == 0.0 && is_recurring == 0 { return "ERROR: Only recurring bills can be left as TBD. Please enter an amount.".to_string(); }
    let normalized: Option<String> = conn.query_row("SELECT date(?1)", params![due_date], |r| r.get(0)).unwrap_or(None);
    if normalized.as_deref() != Some(due_date) { return "ERROR: That due date isn't a real calendar date.".to_string(); }

    if series_id == 0 {
        let payday = if due_date == old_due { old_payday.clone() } else { preceding_payday(conn, due_date) };
        if conn.execute("UPDATE smart_bills SET name = ?1, amount = ?2, due_date = ?3, payday_date = ?4 WHERE id = ?5", params![new_base, amount, due_date, payday, bill_id]).is_err() { return "ERROR: Could not save the bill.".to_string(); }
        return edited_message(&old_payday, &payday);
    }

    // A split bill: find this month's pair from its real due date (the 2nd Half's), as update_bill_amount does
    let second_half_due: String = if old_name.contains("(1st Half)") { conn.query_row("SELECT date(?1, '+15 days')", params![old_due], |r| r.get(0)).unwrap_or(old_due.clone()) } else { old_due.clone() };
    let first_half_due: String = conn.query_row("SELECT date(?1, '-15 days')", params![second_half_due], |r| r.get(0)).unwrap_or_default();
    let mut stmt = conn.prepare("SELECT id, name, payday_date FROM smart_bills WHERE series_id = ?1 AND (due_date = ?2 OR due_date = ?3)").unwrap();
    let halves: Vec<(i32, String, String)> = stmt.query_map(params![series_id, second_half_due, first_half_due], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap().filter_map(Result::ok).collect();
    let first_half_saved = !halves.iter().any(|(_, n, _)| n.contains("(1st Half)"));
    // The 2nd Half finds its saved 1st Half by the 15 days between them, so that link must not break
    if due_date != second_half_due && first_half_saved {
        return "ERROR: This bill's 1st Half is already saved to a fund, so its due date can't change this month. You can still change the name and amount.".to_string();
    }
    let new_first_due: String = conn.query_row("SELECT date(?1, '-15 days')", params![due_date], |r| r.get(0)).unwrap_or_default();
    let mut moved_to = old_payday.clone();
    for (id, half_name, payday) in &halves {
        let renamed = format!("{}{}", new_base, &half_name[split_suffix_at(half_name)..]);
        let is_first = half_name.contains("(1st Half)");
        let due = if is_first { new_first_due.clone() } else { due_date.to_string() };
        let new_payday = if due_date == second_half_due { payday.clone() } else { preceding_payday(conn, &due) };
        if *id == bill_id { moved_to = new_payday.clone(); }
        let _ = conn.execute("UPDATE smart_bills SET name = ?1, amount = ?2 / 2.0, due_date = ?3, payday_date = ?4 WHERE id = ?5", params![renamed, amount, due, new_payday, id]);
    }
    // A 1st Half already saved to a fund still belongs to this month's bill: keep its receipt's name in step
    let mut stmt = conn.prepare("SELECT id, name FROM paid_bills WHERE series_id = ?1 AND due_date = ?2 AND name LIKE '%(1st Half)%'").unwrap();
    let receipts: Vec<(i32, String)> = stmt.query_map(params![series_id, first_half_due], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().filter_map(Result::ok).collect();
    for (id, receipt_name) in receipts {
        let _ = conn.execute("UPDATE paid_bills SET name = ?1 WHERE id = ?2", params![format!("{}{}", new_base, &receipt_name[split_suffix_at(&receipt_name)..]), id]);
    }
    edited_message(&old_payday, &moved_to)
}

fn edited_message(old_payday: &str, new_payday: &str) -> String {
    if old_payday == new_payday { "SUCCESS: Bill updated!".to_string() } else { format!("SUCCESS: Bill updated! Its new due date moves it to the {} paycheck.", new_payday) }
}

/// Edits a loan's name and the date it was taken out. Its installments in the Budget and on receipts are named after it, so they follow.
/// (Amounts and due dates aren't editable: they come from the loan's terms, and its payment schedule is worked out from them.)
pub fn edit_loan(conn: &Connection, loan_id: i32, name: &str, date_acquired: &str) -> String {
    let new_name = match checked_name(name) { Ok(n) => n, Err(e) => return e };
    if !date_acquired.is_empty() {
        let normalized: Option<String> = conn.query_row("SELECT date(?1)", params![date_acquired], |r| r.get(0)).unwrap_or(None);
        if normalized.as_deref() != Some(date_acquired) { return "ERROR: That date acquired isn't a real calendar date.".to_string(); }
    }
    let Ok(old_name) = conn.query_row("SELECT name FROM loans WHERE id = ?1", params![loan_id], |r| r.get::<_, String>(0)) else { return "ERROR: Could not find that loan.".to_string(); };
    let taken: bool = conn.query_row("SELECT 1 FROM loans WHERE lower(name) = lower(?1) AND id != ?2", params![new_name, loan_id], |_| Ok(true)).unwrap_or(false);
    if taken { return format!("ERROR: You already have a loan called '{}'.", new_name); }
    if conn.execute("UPDATE loans SET name = ?1, date_acquired = ?2 WHERE id = ?3", params![new_name, date_acquired, loan_id]).is_err() { return "ERROR: Could not save the loan.".to_string(); }
    if old_name != new_name {
        // "Old Loan (1st Half)" -> "New Loan (1st Half)", for this loan's installments only (compared as text, so names with % or _ are safe)
        for table in ["smart_bills", "paid_bills"] {
            let _ = conn.execute(&format!("UPDATE {} SET name = ?1 || substr(name, length(?2) + 1) WHERE loan_id = ?3 AND substr(name, 1, length(?2) + 2) = ?2 || ' ('", table), params![new_name, old_name, loan_id]);
        }
    }
    "SUCCESS: Loan updated!".to_string()
}

// ---------------------------------------------------------------------------------------------------------------------
// One entry point for every command, shared by the desktop app (local vault) and the sync server (remote vault).
// Arguments arrive as a JSON object with camelCase keys, exactly as the frontend has always sent them to Tauri.
// ---------------------------------------------------------------------------------------------------------------------

/// Every command the frontend may call, in the order the dispatcher checks them
pub const COMMANDS: &[&str] = &["add_account", "get_accounts", "delete_account", "log_income", "log_expense", "transfer_money", "undo_transaction", "get_all_transactions", "get_transactions_page", "add_smart_bill", "update_bill_amount", "split_recurring_bill", "move_bill_allocation", "get_smart_bills", "add_loan", "get_loans", "delete_loan", "fund_bill_to_savings", "pay_bill", "undo_bill_payment", "get_paid_bills", "get_paid_bills_page", "get_archived_loans", "get_completed_loans", "undo_delete_loan", "delete_bill", "get_archived_bills", "undo_delete_bill", "get_sinking_fund_info", "repair_balance_rounding", "repair_loan_ids", "undo_fund_to_savings", "mark_first_half_handled", "rename_account", "edit_bill", "edit_loan"];

/// Reads never change the vault, so the sync server only notifies other devices after anything else
pub fn is_mutation(cmd: &str) -> bool { !cmd.starts_with("get_") }

fn arg<T: serde::de::DeserializeOwned>(args: &Value, key: &str) -> Result<T, String> {
    serde_json::from_value(args.get(key).cloned().unwrap_or(Value::Null)).map_err(|e| format!("ERROR: Invalid argument '{}': {}", key, e))
}
fn to_json<T: Serialize>(value: T) -> Result<Value, String> { serde_json::to_value(value).map_err(|e| e.to_string()) }

/// Runs one command as a single all-or-nothing database transaction, so a crash halfway through
/// (or a dropped connection on the server) can never leave a transfer or payment half-applied
pub fn run_command(conn: &Connection, cmd: &str, args: &Value) -> Result<Value, String> {
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| format!("ERROR: Could not start: {}", e))?;
    let result = dispatch(conn, cmd, args);
    match &result {
        Ok(_) => conn.execute_batch("COMMIT").map_err(|e| format!("ERROR: Could not save: {}", e))?,
        Err(_) => { let _ = conn.execute_batch("ROLLBACK"); }
    }
    result
}

/// A consistent copy of the whole vault file, safe to take while it is in use (for uploads and backups)
pub fn snapshot(conn: &Connection, scratch: &Path) -> Result<Vec<u8>, String> {
    let _ = std::fs::remove_file(scratch);
    conn.execute("VACUUM INTO ?1", params![scratch.to_string_lossy()]).map_err(|e| format!("ERROR: Could not copy the vault: {}", e))?;
    let bytes = std::fs::read(scratch).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(scratch);
    bytes
}

// ---------------------------------------------------------------------------------------------------------------------
// Sync replicas. With sync on, every change is added to a shared, numbered list (in Firebase). Each device keeps its
// own copy of the vault and applies that list in order, so every device ends up with exactly the same data.
// That only works if applying a change gives the same result everywhere: the one clock-dependent value (a
// transaction's created_at) is therefore taken from the change itself, not from the device applying it.
// ---------------------------------------------------------------------------------------------------------------------

fn ensure_sync_meta(conn: &Connection) -> Result<(), String> {
    conn.execute("CREATE TABLE IF NOT EXISTS sync_meta (key TEXT PRIMARY KEY, value INTEGER NOT NULL)", []).map(|_| ()).map_err(|e| format!("ERROR: {}", e))
}

/// The number of the last change this replica has applied (0 for a brand-new replica)
pub fn last_applied_seq(conn: &Connection) -> Result<i64, String> {
    ensure_sync_meta(conn)?;
    Ok(conn.query_row("SELECT value FROM sync_meta WHERE key = 'last_seq'", [], |r| r.get(0)).unwrap_or(0))
}

/// Marks the replica as being at change `seq` (after restoring a checkpoint taken at that point)
pub fn set_applied_seq(conn: &Connection, seq: i64) -> Result<(), String> {
    ensure_sync_meta(conn)?;
    conn.execute("INSERT INTO sync_meta (key, value) VALUES ('last_seq', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value", params![seq]).map(|_| ()).map_err(|e| format!("ERROR: {}", e))
}

/// Applies change number `seq` to a replica, all or nothing. `at` is when the change was made ("YYYY-MM-DD HH:MM:SS",
/// the maker's local time). Returns {"result": …} or {"error": …}: a change the rules reject (say, insufficient funds)
/// is still "applied" — it changes nothing, identically on every device.
/// Fails without touching anything if the change is not the very next one, so a replica can never skip or repeat.
pub fn apply_op(conn: &Connection, seq: i64, cmd: &str, args: &Value, at: &str) -> Result<Value, String> {
    let last = last_applied_seq(conn)?;
    if seq != last + 1 { return Err(format!("ERROR: Change {} can't be applied after change {}", seq, last)); }
    let valid_at = at.len() == 19 && at.chars().enumerate().all(|(i, c)| match i { 4 | 7 => c == '-', 10 => c == ' ', 13 | 16 => c == ':', _ => c.is_ascii_digit() });
    if !valid_at { return Err(format!("ERROR: Change {} has an invalid time", seq)); }
    if !is_mutation(cmd) { return Err(format!("ERROR: Change {} is a read, not a change", seq)); }

    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| format!("ERROR: Could not start: {}", e))?;
    let outcome = (|| -> Result<Value, String> {
        let before: i64 = conn.query_row("SELECT COALESCE(MAX(id), 0) FROM transactions", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        let receipts_before: i64 = conn.query_row("SELECT COALESCE(MAX(id), 0) FROM paid_bills", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        let result = match dispatch(conn, cmd, args) { Ok(v) => serde_json::json!({ "result": v }), Err(e) => serde_json::json!({ "error": e }) };
        conn.execute("UPDATE transactions SET created_at = ?1 WHERE id > ?2", params![at, before]).map_err(|e| e.to_string())?;
        conn.execute("UPDATE paid_bills SET paid_at = ?1 WHERE id > ?2", params![at, receipts_before]).map_err(|e| e.to_string())?;
        set_applied_seq(conn, seq)?;
        Ok(result)
    })();
    match outcome {
        Ok(v) => { conn.execute_batch("COMMIT").map_err(|e| format!("ERROR: Could not save: {}", e))?; Ok(v) }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK"); Err(e) }
    }
}

/// Replaces the replica at `path` with a checkpoint (a complete vault file) taken at change `seq`. The new file is
/// written beside the old one and checked before it is swapped in, so a bad download can never damage the replica.
pub fn restore_replica(path: &Path, bytes: &[u8], seq: i64) -> Result<Connection, String> {
    if !bytes.starts_with(b"SQLite format 3\0") { return Err("ERROR: That checkpoint is not a Vault database".into()); }
    let incoming = path.with_extension("incoming");
    std::fs::write(&incoming, bytes).map_err(|e| format!("ERROR: {}", e))?;
    let check = Connection::open(&incoming).and_then(|c| c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0)));
    if !matches!(check.as_deref(), Ok("ok")) {
        let _ = std::fs::remove_file(&incoming);
        return Err("ERROR: That checkpoint is damaged".into());
    }
    std::fs::rename(&incoming, path).map_err(|e| format!("ERROR: {}", e))?;
    let conn = open_vault_file(path).map_err(|e| format!("ERROR: {}", e))?;
    set_applied_seq(&conn, seq)?;
    Ok(conn)
}

// ---------------------------------------------------------------------------------------------------------------------
// Vaults kept in memory. A browser has no files to keep a vault in, so it holds the vault in memory and saves these
// bytes in the browser's own storage instead. The bytes are an ordinary vault file, so checkpoints move freely between
// the browser and the desktop or Android app.
// ---------------------------------------------------------------------------------------------------------------------

/// A complete copy of a vault, as the bytes of a vault file
pub fn vault_bytes(conn: &Connection) -> Result<Vec<u8>, String> {
    conn.serialize(rusqlite::MAIN_DB).map(|data| data.to_vec()).map_err(|e| format!("ERROR: Could not copy the vault: {}", e))
}

/// Opens a vault from the bytes of a vault file into memory, checking it first and bringing its tables up to date
pub fn open_vault_bytes(bytes: &[u8]) -> Result<Connection, String> {
    if !bytes.starts_with(b"SQLite format 3\0") { return Err("ERROR: That checkpoint is not a Vault database".into()); }
    let mut conn = Connection::open_in_memory().map_err(|e| format!("ERROR: {}", e))?;
    conn.deserialize_read_exact(rusqlite::MAIN_DB, bytes, bytes.len(), false).map_err(|_| "ERROR: That checkpoint is damaged".to_string())?;
    let check = conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0));
    if !matches!(check.as_deref(), Ok("ok")) { return Err("ERROR: That checkpoint is damaged".into()); }
    init_schema(&conn);
    Ok(conn)
}

/// The in-memory counterpart of restore_replica: a replica holding the checkpoint taken at change `seq`
pub fn restore_replica_bytes(bytes: &[u8], seq: i64) -> Result<Connection, String> {
    let conn = open_vault_bytes(bytes)?;
    set_applied_seq(&conn, seq)?;
    Ok(conn)
}

pub fn dispatch(conn: &Connection, cmd: &str, args: &Value) -> Result<Value, String> {
    match cmd {
        "add_account" => { let name: String = arg(args, "name")?; to_json(add_account(conn, &name)) }
        "get_accounts" => { to_json(get_accounts(conn)) }
        "delete_account" => { let account_id: i32 = arg(args, "accountId")?; to_json(delete_account(conn, account_id)) }
        "log_income" => { let account_id: i32 = arg(args, "accountId")?; let amount: f64 = arg(args, "amount")?; let description: String = arg(args, "description")?; to_json(log_income(conn, account_id, amount, &description)) }
        "log_expense" => { let account_id: i32 = arg(args, "accountId")?; let amount: f64 = arg(args, "amount")?; let description: String = arg(args, "description")?; to_json(log_expense(conn, account_id, amount, &description)) }
        "transfer_money" => { let from_account_id: i32 = arg(args, "fromAccountId")?; let to_account_id: i32 = arg(args, "toAccountId")?; let amount: f64 = arg(args, "amount")?; let description: String = arg(args, "description")?; to_json(transfer_money(conn, from_account_id, to_account_id, amount, &description)) }
        "undo_transaction" => { let transaction_id: i32 = arg(args, "transactionId")?; to_json(undo_transaction(conn, transaction_id)) }
        "get_all_transactions" => { to_json(get_all_transactions(conn)) }
        "get_transactions_page" => { let offset: i32 = arg(args, "offset")?; let limit: i32 = arg(args, "limit")?; let account_id: Option<i32> = arg(args, "accountId")?; to_json(get_transactions_page(conn, offset, limit, account_id)) }
        "add_smart_bill" => { let name: String = arg(args, "name")?; let amount: f64 = arg(args, "amount")?; let due_date: String = arg(args, "dueDate")?; let payday_date: String = arg(args, "paydayDate")?; let is_recurring: bool = arg(args, "isRecurring")?; let split: Option<bool> = arg(args, "split")?; to_json(add_smart_bill(conn, &name, amount, &due_date, &payday_date, is_recurring, split)) }
        "update_bill_amount" => { let bill_id: i32 = arg(args, "billId")?; let new_amount: f64 = arg(args, "newAmount")?; to_json(update_bill_amount(conn, bill_id, new_amount)) }
        "split_recurring_bill" => { let bill_id: i32 = arg(args, "billId")?; to_json(split_recurring_bill(conn, bill_id)) }
        "move_bill_allocation" => { let bill_id: i32 = arg(args, "billId")?; let new_payday: String = arg(args, "newPayday")?; to_json(move_bill_allocation(conn, bill_id, &new_payday)) }
        "get_smart_bills" => { to_json(get_smart_bills(conn)) }
        "add_loan" => { let name: String = arg(args, "name")?; let principal: f64 = arg(args, "principal")?; let remaining_balance: f64 = arg(args, "remainingBalance")?; let term: i32 = arg(args, "term")?; let term_type: String = arg(args, "termType")?; let frequency: String = arg(args, "frequency")?; let payment_amount: f64 = arg(args, "paymentAmount")?; let due_date: String = arg(args, "dueDate")?; let payday_date: String = arg(args, "paydayDate")?; let total_incurred: f64 = arg(args, "totalIncurred")?; let due_structure: String = arg(args, "dueStructure")?; let date_acquired: String = arg(args, "dateAcquired")?; let bill_names: Vec<String> = arg(args, "billNames")?; let bill_amounts: Vec<f64> = arg(args, "billAmounts")?; let bill_dues: Vec<String> = arg(args, "billDues")?; let bill_paydays: Vec<String> = arg(args, "billPaydays")?; to_json(add_loan(conn, &name, principal, remaining_balance, term, &term_type, &frequency, payment_amount, &due_date, &payday_date, total_incurred, &due_structure, &date_acquired, bill_names, bill_amounts, bill_dues, bill_paydays)) }
        "get_loans" => { to_json(get_loans(conn)) }
        "delete_loan" => { let loan_id: i32 = arg(args, "loanId")?; to_json(delete_loan(conn, loan_id)) }
        "fund_bill_to_savings" => { let bill_id: i32 = arg(args, "billId")?; let from_account_id: i32 = arg(args, "fromAccountId")?; let to_account_id: i32 = arg(args, "toAccountId")?; let amount: f64 = arg(args, "amount")?; let description: String = arg(args, "description")?; to_json(fund_bill_to_savings(conn, bill_id, from_account_id, to_account_id, amount, &description)) }
        "pay_bill" => { let bill_id: i32 = arg(args, "billId")?; let account_id: i32 = arg(args, "accountId")?; let amount: f64 = arg(args, "amount")?; let description: String = arg(args, "description")?; to_json(pay_bill(conn, bill_id, account_id, amount, &description)) }
        "undo_bill_payment" => { let paid_bill_id: i32 = arg(args, "paidBillId")?; to_json(undo_bill_payment(conn, paid_bill_id)) }
        "get_paid_bills" => { to_json(get_paid_bills(conn)) }
        "get_paid_bills_page" => { let offset: i32 = arg(args, "offset")?; let limit: i32 = arg(args, "limit")?; to_json(get_paid_bills_page(conn, offset, limit)) }
        "get_archived_loans" => { to_json(get_archived_loans(conn)) }
        "get_completed_loans" => { to_json(get_completed_loans(conn)) }
        "undo_delete_loan" => { let archive_id: i32 = arg(args, "archiveId")?; let bill_names: Vec<String> = arg(args, "billNames")?; let bill_amounts: Vec<f64> = arg(args, "billAmounts")?; let bill_dues: Vec<String> = arg(args, "billDues")?; let bill_paydays: Vec<String> = arg(args, "billPaydays")?; to_json(undo_delete_loan(conn, archive_id, bill_names, bill_amounts, bill_dues, bill_paydays)) }
        "delete_bill" => { let bill_id: i32 = arg(args, "billId")?; to_json(delete_bill(conn, bill_id)) }
        "get_archived_bills" => { to_json(get_archived_bills(conn)) }
        "undo_delete_bill" => { let archive_id: i32 = arg(args, "archiveId")?; to_json(undo_delete_bill(conn, archive_id)) }
        "get_sinking_fund_info" => { let bill_id: i32 = arg(args, "billId")?; to_json(get_sinking_fund_info(conn, bill_id)) }
        "repair_balance_rounding" => { to_json(repair_balance_rounding(conn)) }
        "repair_loan_ids" => { to_json(repair_loan_ids(conn)) }
        "undo_fund_to_savings" => { let paid_bill_id: i32 = arg(args, "paidBillId")?; to_json(undo_fund_to_savings(conn, paid_bill_id)) }
        "mark_first_half_handled" => { let bill_id: i32 = arg(args, "billId")?; let account_name: String = arg(args, "accountName")?; to_json(mark_first_half_handled(conn, bill_id, &account_name)) }
        "rename_account" => { let account_id: i32 = arg(args, "accountId")?; let name: String = arg(args, "name")?; to_json(rename_account(conn, account_id, &name)) }
        "edit_bill" => { let bill_id: i32 = arg(args, "billId")?; let name: String = arg(args, "name")?; let amount: f64 = arg(args, "amount")?; let due_date: String = arg(args, "dueDate")?; to_json(edit_bill(conn, bill_id, &name, amount, &due_date)) }
        "edit_loan" => { let loan_id: i32 = arg(args, "loanId")?; let name: String = arg(args, "name")?; let date_acquired: String = arg(args, "dateAcquired")?; to_json(edit_loan(conn, loan_id, &name, &date_acquired)) }
        _ => Err(format!("ERROR: Unknown command '{}'", cmd)),
    }
}

// These tests run against a throwaway in-memory database, never a real vault file
#[cfg(test)]
mod tests {
    use super::*;
    // In a browser build the same tests run under wasm-bindgen-test (see vault-web/README.md)
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn);
        conn
    }

    fn add_bill(conn: &Connection, name: &str, amount: f64, due: &str, payday: &str) -> i32 {
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date) VALUES (?1, ?2, ?3, ?4)", params![name, amount, due, payday]).unwrap();
        conn.last_insert_rowid() as i32
    }

    fn payday_of(conn: &Connection, bill_id: i32) -> String {
        conn.query_row("SELECT payday_date FROM smart_bills WHERE id = ?1", params![bill_id], |r| r.get(0)).unwrap()
    }

    #[test]
    fn move_bill_changes_only_that_bill() {
        let conn = test_db();
        let internet = add_bill(&conn, "Internet", 1500.0, "2026-11-20", "2026-11-15");
        let water = add_bill(&conn, "Water", 400.0, "2026-11-22", "2026-11-15");

        let result = move_bill_allocation_impl(&conn, internet, "2026-10-31");

        assert!(result.starts_with("SUCCESS"), "{}", result);
        assert_eq!(payday_of(&conn, internet), "2026-10-31");
        assert_eq!(payday_of(&conn, water), "2026-11-15");
    }

    #[test]
    fn move_bill_rejects_impossible_dates() {
        let conn = test_db();
        let internet = add_bill(&conn, "Internet", 1500.0, "2026-11-20", "2026-11-15");

        for bad in ["2026-02-30", "2026-13-01", "not a date", ""] {
            let result = move_bill_allocation_impl(&conn, internet, bad);
            assert!(result.starts_with("ERROR"), "{} should be rejected", bad);
        }
        assert_eq!(payday_of(&conn, internet), "2026-11-15");
    }

    fn add_account(conn: &Connection, name: &str) -> i32 {
        conn.execute("INSERT INTO accounts (name, balance) VALUES (?1, 0.0)", params![name]).unwrap();
        conn.last_insert_rowid() as i32
    }

    fn add_tx(conn: &Connection, account_id: i32, tx_type: &str, amount: f64, description: &str) {
        conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, ?2, ?3, ?4)", params![account_id, tx_type, amount, description]).unwrap();
    }

    #[test]
    fn transaction_history_can_be_filtered_per_account() {
        let conn = test_db();
        let gcash = add_account(&conn, "GCash");
        let bpi = add_account(&conn, "BPI");
        add_tx(&conn, gcash, "INCOME", 1000.0, "Salary");
        add_tx(&conn, bpi, "EXPENSE", 200.0, "Groceries");
        add_tx(&conn, gcash, "EXPENSE", 50.0, "Load");

        let gcash_only = get_transactions_page_impl(&conn, 0, 50, Some(gcash));
        let descriptions: Vec<&str> = gcash_only.iter().map(|t| t.description.as_str()).collect();
        assert_eq!(descriptions, vec!["Load", "Salary"], "newest first, GCash only");
        assert!(gcash_only.iter().all(|t| t.account_name == "GCash"));

        let everything = get_transactions_page_impl(&conn, 0, 50, None);
        assert_eq!(everything.len(), 3, "None returns the complete history");
    }

    #[test]
    fn per_account_history_pages_within_that_account() {
        let conn = test_db();
        let gcash = add_account(&conn, "GCash");
        let bpi = add_account(&conn, "BPI");
        for i in 0..5 {
            add_tx(&conn, gcash, "INCOME", 1.0, &format!("G{}", i));
            add_tx(&conn, bpi, "INCOME", 1.0, &format!("B{}", i));
        }

        let first = get_transactions_page_impl(&conn, 0, 2, Some(gcash));
        let second = get_transactions_page_impl(&conn, 2, 2, Some(gcash));
        let rest = get_transactions_page_impl(&conn, 4, 2, Some(gcash));

        let names = |page: &Vec<TransactionLog>| page.iter().map(|t| t.description.clone()).collect::<Vec<_>>();
        assert_eq!(names(&first), vec!["G4", "G3"]);
        assert_eq!(names(&second), vec!["G2", "G1"]);
        assert_eq!(names(&rest), vec!["G0"]);
    }

    fn balance_of(conn: &Connection, account_id: i32) -> f64 {
        conn.query_row("SELECT balance FROM accounts WHERE id = ?1", params![account_id], |r| r.get(0)).unwrap()
    }

    fn fund_account(conn: &Connection, name: &str, balance: f64) -> i32 {
        let id = add_account(conn, name);
        conn.execute("UPDATE accounts SET balance = ?1 WHERE id = ?2", params![balance, id]).unwrap();
        id
    }

    // (id, name, amount, due_date, payday_date, is_recurring, series_id), ordered by due date
    fn unpaid_bills(conn: &Connection) -> Vec<(i32, String, f64, String, String, i32, i32)> {
        let mut stmt = conn.prepare("SELECT id, name, amount, due_date, payday_date, is_recurring, series_id FROM smart_bills ORDER BY due_date, id").unwrap();
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    fn bill_id_named(conn: &Connection, name: &str, due: &str) -> i32 {
        conn.query_row("SELECT id FROM smart_bills WHERE name = ?1 AND due_date = ?2", params![name, due], |r| r.get(0)).unwrap()
    }

    fn paid_receipt_id(conn: &Connection, name: &str) -> i32 {
        conn.query_row("SELECT id FROM paid_bills WHERE name LIKE ?1 ORDER BY id DESC LIMIT 1", params![format!("{}%", name)], |r| r.get(0)).unwrap()
    }

    #[test]
    fn a_new_bill_without_a_paycheck_gets_the_usual_one() {
        let conn = test_db();
        assert_eq!(add_smart_bill_impl(&conn, "Internet", 1699.0, "2026-11-20", "", true, false), "SUCCESS!");
        assert_eq!(payday_of(&conn, bill_id_named(&conn, "Internet", "2026-11-20")), "2026-10-31", "latest payday at least 11 days before Nov 20");
        // Split: the 2nd Half gets the usual paycheck too
        add_smart_bill_impl(&conn, "Rent", 9000.0, "2026-12-05", "", true, true);
        assert_eq!(payday_of(&conn, bill_id_named(&conn, "Rent (2nd Half (Due))", "2026-12-05")), "2026-11-15");
        assert_eq!(payday_of(&conn, bill_id_named(&conn, "Rent (1st Half)", "2026-11-20")), "2026-10-31");
        // A paycheck the user picked is kept as is
        add_smart_bill_impl(&conn, "Water", 500.0, "2026-11-20", "2026-11-15", false, false);
        assert_eq!(payday_of(&conn, bill_id_named(&conn, "Water", "2026-11-20")), "2026-11-15");
        assert!(add_smart_bill_impl(&conn, "Bad", 1.0, "2026-02-30", "", false, false).starts_with("ERROR"));
    }

    #[test]
    fn receipts_record_when_they_were_settled() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 5000.0);
        add_smart_bill_impl(&conn, "Internet", 1699.0, "2026-11-20", "2026-10-31", true, false);
        pay_bill_impl(&conn, bill_id_named(&conn, "Internet", "2026-11-20"), bank, 1699.0, "Paid");
        let paid_at: String = conn.query_row("SELECT paid_at FROM paid_bills", [], |r| r.get(0)).unwrap();
        assert_eq!(paid_at.len(), 19, "stamped with the date and time: {}", paid_at);
        assert_eq!(get_paid_bills(&conn)[0].paid_at, paid_at);
    }

    #[test]
    fn synced_receipts_take_their_settled_time_from_the_change() {
        // Every device must show the same settled time, so it comes from the change, not the device's clock
        let changes = [
            ("add_account", serde_json::json!({ "name": "Bank" })),
            ("log_income", serde_json::json!({ "accountId": 1, "amount": 20000.0, "description": "Pay" })),
            ("add_smart_bill", serde_json::json!({ "name": "Rent", "amount": 9000.0, "dueDate": "2026-11-30", "paydayDate": "", "isRecurring": true, "split": true })),
        ];
        let phone = test_db();
        let desktop = test_db();
        for conn in [&phone, &desktop] {
            for (i, (cmd, args)) in changes.iter().enumerate() { apply_op(conn, i as i64 + 1, cmd, args, "2026-10-15 08:00:00").unwrap(); }
            let first = bill_id_named(conn, "Rent (1st Half)", "2026-11-15");
            let second = bill_id_named(conn, "Rent (2nd Half (Due))", "2026-11-30");
            apply_op(conn, 4, "pay_bill", &serde_json::json!({ "billId": second, "accountId": 1, "amount": 4500.0, "description": "Paid" }), "2026-11-29 18:30:00").unwrap();
            assert!(first > 0);
        }
        let times = |conn: &Connection| -> Vec<String> { get_paid_bills(conn).into_iter().map(|b| b.paid_at).collect() };
        assert_eq!(times(&phone), vec!["2026-11-29 18:30:00", "2026-11-29 18:30:00"], "both halves, settled together");
        assert_eq!(times(&phone), times(&desktop));
    }

    #[test]
    fn split_bill_creates_a_linked_pair() {
        let conn = test_db();
        add_smart_bill_impl(&conn, "Internet", 3000.0, "2026-11-20", "2026-10-31", true, true);

        let bills = unpaid_bills(&conn);
        assert_eq!(bills.len(), 2);
        let (_, first_name, first_amount, first_due, first_payday, first_recurring, first_series) = &bills[0];
        let (_, second_name, second_amount, second_due, second_payday, _, second_series) = &bills[1];
        assert_eq!(first_name, "Internet (1st Half)");
        assert_eq!(second_name, "Internet (2nd Half (Due))");
        assert_eq!((*first_amount, *second_amount), (1500.0, 1500.0));
        assert_eq!(first_due, "2026-11-05", "1st Half is due 15 days before the real due date");
        assert_eq!(second_due, "2026-11-20");
        assert_eq!(first_payday, "2026-10-15", "latest payday at least 11 days before Nov 5");
        assert_eq!(second_payday, "2026-10-31", "2nd Half keeps the paycheck the user picked");
        assert_eq!(*first_recurring, 1);
        assert!(*first_series != 0 && first_series == second_series);
    }

    #[test]
    fn split_requires_a_recurring_bill() {
        let conn = test_db();
        let result = add_smart_bill_impl(&conn, "Gift", 500.0, "2026-11-20", "2026-10-31", false, true);
        assert!(result.starts_with("ERROR"), "{}", result);
        assert!(unpaid_bills(&conn).is_empty());
    }

    #[test]
    fn each_split_bill_gets_its_own_series() {
        let conn = test_db();
        add_smart_bill_impl(&conn, "Internet", 3000.0, "2026-11-20", "2026-10-31", true, true);
        add_smart_bill_impl(&conn, "Water", 800.0, "2026-11-20", "2026-10-31", true, true);
        let internet: i32 = conn.query_row("SELECT DISTINCT series_id FROM smart_bills WHERE name LIKE 'Internet%'", [], |r| r.get(0)).unwrap();
        let water: i32 = conn.query_row("SELECT DISTINCT series_id FROM smart_bills WHERE name LIKE 'Water%'", [], |r| r.get(0)).unwrap();
        assert_ne!(internet, water);
    }

    #[test]
    fn entering_a_split_bill_amount_sets_both_halves() {
        let conn = test_db();
        add_smart_bill_impl(&conn, "Electric", 0.0, "2026-11-20", "2026-10-31", true, true);

        // Typing the full amount on either half updates both halves of that month
        let first = bill_id_named(&conn, "Electric (1st Half)", "2026-11-05");
        update_bill_amount_impl(&conn, first, 2400.0);
        assert!(unpaid_bills(&conn).iter().all(|b| b.2 == 1200.0));

        let second = bill_id_named(&conn, "Electric (2nd Half (Due))", "2026-11-20");
        update_bill_amount_impl(&conn, second, 2001.01);
        assert!(unpaid_bills(&conn).iter().all(|b| b.2 == 2001.01 / 2.0));
    }

    #[test]
    fn funding_then_paying_a_split_bill_charges_the_full_amount_and_rolls_to_next_month() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 5000.0);
        let savings = fund_account(&conn, "Savings", 0.0);
        add_smart_bill_impl(&conn, "Internet", 3000.0, "2026-11-20", "2026-10-31", true, true);

        // Save the 1st Half to a fund, like a loan's Sinking Fund
        let first = bill_id_named(&conn, "Internet (1st Half)", "2026-11-05");
        fund_bill_to_savings_impl(&conn, first, bank, savings, 1500.0, "Funded to Savings: Internet (1st Half)");
        assert_eq!((balance_of(&conn, bank), balance_of(&conn, savings)), (3500.0, 1500.0));

        // The 2nd Half can find that fund
        let second = bill_id_named(&conn, "Internet (2nd Half (Due))", "2026-11-20");
        let info = get_sinking_fund_info_impl(&conn, second);
        assert!(info.found);
        assert_eq!((info.account_id, info.amount), (savings, 1500.0));

        // Paying the 2nd Half takes the FULL bill (double the half) from the paying account
        let result = pay_bill_impl(&conn, second, bank, 1500.0, "Paid: Internet (2nd Half (Due))");
        assert!(result.starts_with("SUCCESS"), "{}", result);
        assert_eq!(balance_of(&conn, bank), 500.0);

        // Next month's pair appears as TBD, same series, with the usual paydays
        let next = unpaid_bills(&conn);
        let summary: Vec<(String, f64, String, String)> = next.iter().map(|b| (b.1.clone(), b.2, b.3.clone(), b.4.clone())).collect();
        assert_eq!(summary, vec![
            ("Internet (1st Half)".to_string(), 0.0, "2026-12-05".to_string(), "2026-11-15".to_string()),
            ("Internet (2nd Half (Due))".to_string(), 0.0, "2026-12-20".to_string(), "2026-11-30".to_string()),
        ]);
        let series: i32 = conn.query_row("SELECT series_id FROM paid_bills WHERE name LIKE 'Internet (2nd%'", [], |r| r.get(0)).unwrap();
        assert!(next.iter().all(|b| b.6 == series && b.5 == 1));
    }

    #[test]
    fn paying_one_split_bill_never_touches_another_due_the_same_day() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 10000.0);
        add_smart_bill_impl(&conn, "Internet", 3000.0, "2026-11-20", "2026-10-31", true, true);
        add_smart_bill_impl(&conn, "Water", 800.0, "2026-11-20", "2026-10-31", true, true);

        // Pay Internet's 2nd Half without funding its 1st Half: the full amount is charged and Internet's 1st Half is cleared
        let internet_second = bill_id_named(&conn, "Internet (2nd Half (Due))", "2026-11-20");
        pay_bill_impl(&conn, internet_second, bank, 1500.0, "Paid: Internet (2nd Half (Due))");

        assert_eq!(balance_of(&conn, bank), 7000.0);
        let remaining: Vec<String> = unpaid_bills(&conn).iter().filter(|b| b.3.starts_with("2026-11")).map(|b| b.1.clone()).collect();
        assert_eq!(remaining, vec!["Water (1st Half)", "Water (2nd Half (Due))"], "Water's halves are untouched");
    }

    #[test]
    fn undoing_a_split_payment_restores_the_pair_and_removes_next_month() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 5000.0);
        add_smart_bill_impl(&conn, "Internet", 3000.0, "2026-11-20", "2026-10-31", true, true);
        let before = unpaid_bills(&conn);

        let second = bill_id_named(&conn, "Internet (2nd Half (Due))", "2026-11-20");
        pay_bill_impl(&conn, second, bank, 1500.0, "Paid: Internet (2nd Half (Due))");
        let receipt = paid_receipt_id(&conn, "Internet (2nd Half");
        let result = undo_bill_payment_impl(&conn, receipt);
        assert!(result.starts_with("SUCCESS"), "{}", result);

        assert_eq!(balance_of(&conn, bank), 5000.0, "full amount refunded");
        let after = unpaid_bills(&conn);
        let strip_ids = |bills: &Vec<(i32, String, f64, String, String, i32, i32)>| bills.iter().map(|b| (b.1.clone(), b.2, b.3.clone(), b.4.clone(), b.5, b.6)).collect::<Vec<_>>();
        assert_eq!(strip_ids(&after), strip_ids(&before), "both halves back, still recurring and linked; next month's TBD pair gone");
    }

    #[test]
    fn undoing_a_funded_first_half_keeps_it_linked() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 5000.0);
        let savings = fund_account(&conn, "Savings", 0.0);
        add_smart_bill_impl(&conn, "Internet", 3000.0, "2026-11-20", "2026-10-31", true, true);
        let before = unpaid_bills(&conn);

        let first = bill_id_named(&conn, "Internet (1st Half)", "2026-11-05");
        fund_bill_to_savings_impl(&conn, first, bank, savings, 1500.0, "Funded to Savings: Internet (1st Half)");
        undo_fund_to_savings_impl(&conn, paid_receipt_id(&conn, "Internet (1st Half"));

        assert_eq!((balance_of(&conn, bank), balance_of(&conn, savings)), (5000.0, 0.0));
        let after = unpaid_bills(&conn);
        assert_eq!(after.len(), 2);
        assert!(after.iter().all(|b| b.5 == 1 && b.6 == before[0].6));
    }

    #[test]
    fn existing_recurring_bill_can_be_split() {
        let conn = test_db();
        add_smart_bill_impl(&conn, "Rent", 10000.0, "2026-11-20", "2026-10-31", true, false);
        let rent = bill_id_named(&conn, "Rent", "2026-11-20");

        let result = split_recurring_bill_impl(&conn, rent);
        assert!(result.starts_with("SUCCESS"), "{}", result);

        let names: Vec<(String, f64, String)> = unpaid_bills(&conn).iter().map(|b| (b.1.clone(), b.2, b.4.clone())).collect();
        assert_eq!(names, vec![
            ("Rent (1st Half)".to_string(), 5000.0, "2026-10-15".to_string()),
            ("Rent (2nd Half (Due))".to_string(), 5000.0, "2026-10-31".to_string()),
        ]);

        let again = bill_id_named(&conn, "Rent (2nd Half (Due))", "2026-11-20");
        assert!(split_recurring_bill_impl(&conn, again).starts_with("ERROR"), "can't split twice");
    }

    #[test]
    fn only_recurring_non_loan_bills_can_be_split() {
        let conn = test_db();
        let one_off = add_bill(&conn, "Gift", 500.0, "2026-11-20", "2026-10-31");
        assert!(split_recurring_bill_impl(&conn, one_off).starts_with("ERROR"));

        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, is_recurring) VALUES ('Car (Due Date)', 5000, '2026-11-20', '2026-10-31', 7, 1)", []).unwrap();
        let loan_bill = conn.last_insert_rowid() as i32;
        assert!(split_recurring_bill_impl(&conn, loan_bill).starts_with("ERROR"));
        assert_eq!(unpaid_bills(&conn).len(), 2, "nothing was changed");
    }

    #[test]
    fn regular_recurring_bill_still_rolls_over_as_a_single_bill() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 5000.0);
        add_smart_bill_impl(&conn, "Netflix", 549.0, "2026-11-20", "2026-10-31", true, false);
        let netflix = bill_id_named(&conn, "Netflix", "2026-11-20");

        pay_bill_impl(&conn, netflix, bank, 549.0, "Paid: Netflix");
        assert_eq!(balance_of(&conn, bank), 4451.0, "not doubled");
        let next: Vec<(String, f64, String, String, i32)> = unpaid_bills(&conn).iter().map(|b| (b.1.clone(), b.2, b.3.clone(), b.4.clone(), b.6)).collect();
        assert_eq!(next, vec![("Netflix".to_string(), 0.0, "2026-12-20".to_string(), "2026-11-30".to_string(), 0)]);

        // Undo brings back this month's bill (still recurring) and removes the auto-created next one
        undo_bill_payment_impl(&conn, paid_receipt_id(&conn, "Netflix"));
        let restored: Vec<(String, String, i32)> = unpaid_bills(&conn).iter().map(|b| (b.1.clone(), b.3.clone(), b.5)).collect();
        assert_eq!(restored, vec![("Netflix".to_string(), "2026-11-20".to_string(), 1)]);
    }

    #[test]
    fn loan_sinking_fund_halves_work_as_before() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 10000.0);
        conn.execute("INSERT INTO loans (id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure) VALUES (7, 'Car', 12000, 12000, 6, 'Months', 'Bi-Monthly', 2000, '2026-11-20', '', 12000, 'Sinking Fund')", []).unwrap();
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES ('Car (1st Half)', 1000, '2026-11-05', '2026-10-15', 7)", []).unwrap();
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES ('Car (2nd Half (Due))', 1000, '2026-11-20', '2026-10-31', 7)", []).unwrap();
        let second = bill_id_named(&conn, "Car (2nd Half (Due))", "2026-11-20");

        pay_bill_impl(&conn, second, bank, 1000.0, "Paid: Car (2nd Half (Due))");

        assert_eq!(balance_of(&conn, bank), 8000.0, "full installment charged");
        assert!(unpaid_bills(&conn).is_empty(), "1st Half auto-cleared, no recurring copy for loans");
        let remaining: f64 = conn.query_row("SELECT remaining_balance FROM loans WHERE id = 7", [], |r| r.get(0)).unwrap();
        assert_eq!(remaining, 10000.0);
    }

    fn cents(x: f64) -> i64 { (x * 100.0).round() as i64 }

    // Sinking Fund halves of odd-cent installments are half-cents (e.g. 412.35 / 2 = 206.175).
    // Moving one between accounts must never create or lose a centavo.
    #[test]
    fn funding_half_cent_amounts_never_creates_or_loses_money() {
        let mut failures = Vec::new();
        for start_cents in [100_000_i64, 123_456, 50_001, 99_999, 7_777_777] {
            for odd_installment in (1..400).step_by(2) {
                let conn = test_db();
                let start = start_cents as f64 / 100.0;
                let bank = fund_account(&conn, "Bank", start);
                let savings = fund_account(&conn, "Savings", 0.0);
                let half = (odd_installment as f64 + 41_234.0) / 200.0;
                add_bill(&conn, "Loan (1st Half)", half, "2026-11-05", "2026-10-15");
                let bill = bill_id_named(&conn, "Loan (1st Half)", "2026-11-05");

                fund_bill_to_savings_impl(&conn, bill, bank, savings, half, "Funded");

                let total = cents(balance_of(&conn, bank)) + cents(balance_of(&conn, savings));
                if total != start_cents { failures.push((start, half, total - start_cents)); }
            }
        }
        assert!(failures.is_empty(), "{} transfers changed the total, e.g. (start, amount, centavos created) {:?}", failures.len(), &failures[..failures.len().min(3)]);
    }

    #[test]
    fn full_sinking_fund_cycle_with_odd_centavo_installment_balances_exactly() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 1000.0);
        let savings = fund_account(&conn, "Savings", 0.0);
        conn.execute("INSERT INTO loans (id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure) VALUES (7, 'Gotyme', 412.35, 412.35, 1, 'Months', 'Bi-Monthly', 412.35, '2026-11-20', '', 412.35, 'Sinking Fund')", []).unwrap();
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES ('Gotyme (1st Half)', 206.175, '2026-11-05', '2026-10-15', 7)", []).unwrap();
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES ('Gotyme (2nd Half (Due))', 206.175, '2026-11-20', '2026-10-31', 7)", []).unwrap();

        // Same steps the Budget screen performs
        let first = bill_id_named(&conn, "Gotyme (1st Half)", "2026-11-05");
        fund_bill_to_savings_impl(&conn, first, bank, savings, 206.175, "Funded to Savings: Gotyme (1st Half)");
        let second = bill_id_named(&conn, "Gotyme (2nd Half (Due))", "2026-11-20");
        let fund = get_sinking_fund_info_impl(&conn, second);
        transfer_money_impl(&conn, fund.account_id, bank, fund.amount, "Auto-Transfer: Sinking Fund");
        pay_bill_impl(&conn, second, bank, 206.175, "Paid: Gotyme (2nd Half (Due))");

        assert_eq!(cents(balance_of(&conn, bank)) + cents(balance_of(&conn, savings)), 100_000 - 41_235, "exactly ₱412.35 left the accounts");
        assert_eq!(cents(balance_of(&conn, savings)), 0, "the fund was fully used");
        let completed: i32 = conn.query_row("SELECT COUNT(*) FROM completed_loans WHERE original_id = 7", [], |r| r.get(0)).unwrap();
        assert_eq!(completed, 1, "loan paid off exactly and graduated");
    }

    fn add_loan_with_halves(conn: &Connection, id: i32, name: &str, due_structure: &str) {
        conn.execute("INSERT INTO loans (id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure) VALUES (?1, ?2, 6000, 6000, 3, 'Months', 'Bi-Monthly', 2000, '2026-11-20', '', 6000, ?3)", params![id, name, due_structure]).unwrap();
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, 1000, '2026-11-05', '2026-10-15', ?2)", params![format!("{} (1st Half)", name), id]).unwrap();
        conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, 1000, '2026-11-20', '2026-10-31', ?2)", params![format!("{} (2nd Half (Due))", name), id]).unwrap();
    }

    #[test]
    fn true_bimonthly_second_half_is_not_offered_a_sinking_fund() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 10000.0);
        add_loan_with_halves(&conn, 3, "BILLEASE", "True Bi-Monthly");

        // Both halves of a True Bi-Monthly loan are real payments: pay the 1st Half normally
        let first = bill_id_named(&conn, "BILLEASE (1st Half)", "2026-11-05");
        pay_bill_impl(&conn, first, bank, 1000.0, "Paid: BILLEASE (1st Half)");

        let second = bill_id_named(&conn, "BILLEASE (2nd Half (Due))", "2026-11-20");
        assert!(!get_sinking_fund_info_impl(&conn, second).found, "a paid True Bi-Monthly 1st Half is not a sinking fund");

        // And paying the 2nd Half charges only its own amount
        pay_bill_impl(&conn, second, bank, 1000.0, "Paid: BILLEASE (2nd Half (Due))");
        assert_eq!(balance_of(&conn, bank), 8000.0);
    }

    #[test]
    fn sinking_fund_second_half_still_finds_its_fund() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 10000.0);
        let savings = fund_account(&conn, "Savings", 0.0);
        add_loan_with_halves(&conn, 4, "SLOAN", "Sinking Fund");

        let first = bill_id_named(&conn, "SLOAN (1st Half)", "2026-11-05");
        fund_bill_to_savings_impl(&conn, first, bank, savings, 1000.0, "Funded to Savings: SLOAN (1st Half)");

        let second = bill_id_named(&conn, "SLOAN (2nd Half (Due))", "2026-11-20");
        let info = get_sinking_fund_info_impl(&conn, second);
        assert!(info.found);
        assert_eq!((info.account_id, info.amount), (savings, 1000.0));
    }

    // ---- Editing ----

    fn names_in(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn.prepare(&format!("SELECT name FROM {} ORDER BY due_date, id", table)).unwrap();
        let rows = stmt.query_map([], |r| r.get(0)).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    #[test]
    fn renaming_an_account_updates_the_tags_of_bills_saved_to_it() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 5000.0);
        let fund = add_account(&conn, "Demo Loan sinking Fund");
        add_account(&conn, "GCash");
        add_loan_with_halves(&conn, 4, "SLOAN", "Sinking Fund");
        let first = bill_id_named(&conn, "SLOAN (1st Half)", "2026-11-05");
        fund_bill_to_savings_impl(&conn, first, bank, fund, 1000.0, "Funded");
        mark_first_half_handled(&conn, first, "Demo Loan sinking Fund");

        let result = rename_account(&conn, fund, "  Car Fund ");
        assert!(result.starts_with("SUCCESS"), "{}", result);
        let name: String = conn.query_row("SELECT name FROM accounts WHERE id = ?1", params![fund], |r| r.get(0)).unwrap();
        assert_eq!(name, "Car Fund", "spaces around the name are trimmed");
        assert_eq!(names_in(&conn, "paid_bills"), vec!["SLOAN (1st Half) (Handled in Car Fund)"]);
        // The 2nd Half still finds the money, now under the new name
        let info = get_sinking_fund_info_impl(&conn, bill_id_named(&conn, "SLOAN (2nd Half (Due))", "2026-11-20"));
        assert_eq!((info.found, info.account_name.as_str()), (true, "Car Fund"));

        assert!(rename_account(&conn, fund, "gcash").starts_with("ERROR"), "names stay unique, ignoring case");
        assert!(rename_account(&conn, fund, "   ").starts_with("ERROR"));
        assert!(rename_account(&conn, fund, "Fund (Handled in X)").starts_with("ERROR"));
        assert!(rename_account(&conn, 999, "Ghost").starts_with("ERROR"));
        assert!(rename_account(&conn, fund, "car fund").starts_with("SUCCESS"), "an account can change the case of its own name");
    }

    #[test]
    fn editing_a_plain_bill_changes_it_and_moves_it_to_the_right_paycheck() {
        let conn = test_db();
        add_smart_bill_impl(&conn, "Internet", 1499.0, "2026-11-20", "2026-10-31", true, false);
        let id = bill_id_named(&conn, "Internet", "2026-11-20");

        assert_eq!(edit_bill(&conn, id, "PLDT Fibr", 1699.0, "2026-11-20"), "SUCCESS: Bill updated!");
        assert_eq!(unpaid_bills(&conn), vec![(id, "PLDT Fibr".to_string(), 1699.0, "2026-11-20".to_string(), "2026-10-31".to_string(), 1, 0)], "same due date: the chosen paycheck stays");

        let result = edit_bill(&conn, id, "PLDT Fibr", 1699.0, "2026-12-05");
        assert!(result.contains("2026-11-15"), "{}", result);
        assert_eq!(payday_of(&conn, id), "2026-11-15", "latest payday at least 11 days before Dec 5");

        // Next month's copy keeps the new name
        let bank = fund_account(&conn, "Bank", 5000.0);
        pay_bill_impl(&conn, id, bank, 1699.0, "Paid");
        assert_eq!(names_in(&conn, "smart_bills"), vec!["PLDT Fibr"]);

        assert!(edit_bill(&conn, 999, "X", 1.0, "2026-12-05").starts_with("ERROR"));
        let one_off = add_bill(&conn, "Gift", 500.0, "2026-12-20", "2026-12-15");
        assert!(edit_bill(&conn, one_off, "Gift", 0.0, "2026-12-20").starts_with("ERROR"), "only recurring bills can be TBD");
        assert!(edit_bill(&conn, one_off, "Gift", 500.0, "2026-02-30").starts_with("ERROR"));
        assert!(edit_bill(&conn, one_off, "Gift (1st Half)", 500.0, "2026-12-20").starts_with("ERROR"));
    }

    #[test]
    fn loan_installments_are_not_edited_as_bills() {
        let conn = test_db();
        add_loan_with_halves(&conn, 4, "SLOAN", "Sinking Fund");
        let first = bill_id_named(&conn, "SLOAN (1st Half)", "2026-11-05");
        assert!(edit_bill(&conn, first, "Other", 1.0, "2026-11-05").contains("Active Loans"));
        assert_eq!(names_in(&conn, "smart_bills"), vec!["SLOAN (1st Half)", "SLOAN (2nd Half (Due))"]);
    }

    #[test]
    fn editing_a_split_bill_changes_both_halves() {
        let conn = test_db();
        add_smart_bill_impl(&conn, "Rent", 9000.0, "2026-11-30", "2026-11-15", true, true);
        let second = bill_id_named(&conn, "Rent (2nd Half (Due))", "2026-11-30");

        let first = bill_id_named(&conn, "Rent (1st Half)", "2026-11-15");
        assert!(edit_bill(&conn, first, "House Rent", 10000.0, "2026-12-05").starts_with("SUCCESS"), "either half edits the pair");
        let bills = unpaid_bills(&conn);
        assert_eq!(bills.iter().map(|b| (b.1.as_str(), b.2, b.3.as_str(), b.4.as_str())).collect::<Vec<_>>(), vec![
            ("House Rent (1st Half)", 5000.0, "2026-11-20", "2026-10-31"),
            ("House Rent (2nd Half (Due))", 5000.0, "2026-12-05", "2026-11-15"),
        ]);

        // Paying the month creates next month's pair under the new name
        let bank = fund_account(&conn, "Bank", 20000.0);
        pay_bill_impl(&conn, second, bank, 5000.0, "Paid");
        assert_eq!(names_in(&conn, "smart_bills"), vec!["House Rent (1st Half)", "House Rent (2nd Half (Due))"]);
        assert_eq!(balance_of(&conn, bank), 10000.0, "the 2nd Half charged the full new amount");
    }

    #[test]
    fn a_split_bill_with_its_1st_half_saved_keeps_its_due_date() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 20000.0);
        let savings = add_account(&conn, "Savings");
        add_smart_bill_impl(&conn, "Rent", 9000.0, "2026-11-30", "2026-11-15", true, true);
        let first = bill_id_named(&conn, "Rent (1st Half)", "2026-11-15");
        fund_bill_to_savings_impl(&conn, first, bank, savings, 4500.0, "Funded");
        mark_first_half_handled(&conn, first, "Savings");
        let second = bill_id_named(&conn, "Rent (2nd Half (Due))", "2026-11-30");

        assert!(edit_bill(&conn, second, "Rent", 9000.0, "2026-12-05").starts_with("ERROR"), "moving it would lose the saved 1st Half");
        assert!(edit_bill(&conn, second, "House Rent", 9000.0, "2026-11-30").starts_with("SUCCESS"));
        assert_eq!(names_in(&conn, "smart_bills"), vec!["House Rent (2nd Half (Due))"]);
        assert_eq!(names_in(&conn, "paid_bills"), vec!["House Rent (1st Half) (Handled in Savings)"], "the saved half's receipt follows");
        let info = get_sinking_fund_info_impl(&conn, second);
        assert_eq!((info.found, info.account_id, info.amount), (true, savings, 4500.0));
    }

    #[test]
    fn editing_a_loan_renames_its_installments_only() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 10000.0);
        add_loan_with_halves(&conn, 4, "Car", "Sinking Fund");
        add_loan_with_halves(&conn, 5, "Car Loan 2", "Sinking Fund");
        let first = bill_id_named(&conn, "Car (1st Half)", "2026-11-05");
        let savings = add_account(&conn, "Savings");
        fund_bill_to_savings_impl(&conn, first, bank, savings, 1000.0, "Funded");

        assert!(edit_loan(&conn, 4, "Honda Civic", "2026-01-10").starts_with("SUCCESS"));
        let (name, acquired): (String, String) = conn.query_row("SELECT name, date_acquired FROM loans WHERE id = 4", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((name.as_str(), acquired.as_str()), ("Honda Civic", "2026-01-10"));
        let mut unpaid = names_in(&conn, "smart_bills");
        unpaid.sort();
        assert_eq!(unpaid, vec!["Car Loan 2 (1st Half)", "Car Loan 2 (2nd Half (Due))", "Honda Civic (2nd Half (Due))"]);
        assert_eq!(names_in(&conn, "paid_bills"), vec!["Honda Civic (1st Half)"]);

        assert!(edit_loan(&conn, 4, "car loan 2", "").starts_with("ERROR"), "loan names stay unique");
        assert!(edit_loan(&conn, 4, "Honda", "2026-13-01").starts_with("ERROR"));
        assert!(edit_loan(&conn, 999, "Ghost", "").starts_with("ERROR"));
    }

    #[test]
    fn deleting_a_loan_leaves_loans_with_similar_names_alone() {
        let conn = test_db();
        add_loan_with_halves(&conn, 4, "Car", "Sinking Fund");
        add_loan_with_halves(&conn, 5, "Car Loan 2", "Sinking Fund");
        delete_loan(&conn, 4);
        assert_eq!(names_in(&conn, "smart_bills"), vec!["Car Loan 2 (1st Half)", "Car Loan 2 (2nd Half (Due))"]);
    }

    #[test]
    fn transactions_are_recorded_to_the_centavo() {
        let conn = test_db();
        let bank = fund_account(&conn, "Bank", 1000.0);
        let savings = fund_account(&conn, "Savings", 0.0);
        add_bill(&conn, "Loan (1st Half)", 206.175, "2026-11-05", "2026-10-15");
        let bill = bill_id_named(&conn, "Loan (1st Half)", "2026-11-05");

        fund_bill_to_savings_impl(&conn, bill, bank, savings, 206.175, "Funded");

        let amounts: Vec<f64> = conn.prepare("SELECT amount FROM transactions").unwrap().query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
        assert!(amounts.iter().all(|a| (a * 100.0 - (a * 100.0).round()).abs() < 1e-9), "sub-centavo amount logged: {:?}", amounts);
        assert_eq!(amounts[0], amounts[1], "both sides of a transfer log the same amount");
        assert_eq!(balance_of(&conn, savings), amounts[0], "the logged amount is what actually moved");
    }

    #[test]
    fn move_bill_reports_missing_bill() {
        let conn = test_db();
        let result = move_bill_allocation_impl(&conn, 999, "2026-10-31");
        assert!(result.starts_with("ERROR"), "{}", result);
    }

    #[test]
    fn dispatch_reads_camel_case_arguments_like_tauri() {
        let conn = test_db();
        assert!(dispatch(&conn, "add_account", &serde_json::json!({ "name": "Wallet" })).unwrap().as_str().unwrap().starts_with("SUCCESS"));
        let wallet = conn.query_row("SELECT id FROM accounts WHERE name = 'Wallet'", [], |r| r.get::<_, i32>(0)).unwrap();

        dispatch(&conn, "log_income", &serde_json::json!({ "accountId": wallet, "amount": 1500.255, "description": "Salary" })).unwrap();

        let accounts = dispatch(&conn, "get_accounts", &serde_json::json!({})).unwrap();
        assert_eq!(accounts[0]["name"], "Wallet");
        assert_eq!(accounts[0]["balance"], 1500.26);
    }

    #[test]
    fn dispatch_treats_missing_optional_arguments_as_none() {
        let conn = test_db();
        let page = dispatch(&conn, "get_transactions_page", &serde_json::json!({ "offset": 0, "limit": 10 })).unwrap();
        assert_eq!(page, serde_json::json!([]));
    }

    #[test]
    fn dispatch_rejects_unknown_commands_and_bad_arguments() {
        let conn = test_db();
        assert!(dispatch(&conn, "drop_everything", &serde_json::json!({})).unwrap_err().contains("Unknown command"));
        assert!(dispatch(&conn, "delete_account", &serde_json::json!({ "accountId": "one" })).unwrap_err().contains("accountId"));
    }

    #[test]
    fn every_listed_command_is_dispatchable() {
        let conn = test_db();
        for cmd in COMMANDS {
            if let Err(e) = dispatch(&conn, cmd, &serde_json::json!({})) { assert!(!e.contains("Unknown command"), "{}", cmd); }
        }
    }

    #[test]
    fn only_get_commands_are_reads() {
        assert!(!is_mutation("get_accounts"));
        assert!(is_mutation("pay_bill"));
        assert!(is_mutation("repair_balance_rounding"));
    }

    #[test]
    fn run_command_saves_its_work_and_rolls_back_bad_calls() {
        let conn = test_db();
        run_command(&conn, "add_account", &serde_json::json!({ "name": "Wallet" })).unwrap();
        assert!(run_command(&conn, "delete_account", &serde_json::json!({})).is_err());
        // The connection is usable again after a rejected call, and the earlier work was kept
        run_command(&conn, "add_account", &serde_json::json!({ "name": "Savings" })).unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get::<_, i32>(0)).unwrap(), 2);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))] // a browser has no files; vault_bytes is its snapshot
    fn snapshot_is_a_complete_openable_vault() {
        let conn = test_db();
        add_account(&conn, "Wallet");
        let scratch = std::env::temp_dir().join(format!("vault_snapshot_test_{}.db", std::process::id()));
        let bytes = snapshot(&conn, &scratch).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3\0"));
        assert!(!scratch.exists());
    }

    #[test]
    fn a_vault_survives_the_trip_through_bytes() {
        let conn = test_db();
        let wallet = fund_account(&conn, "Wallet", 1234.56);
        set_applied_seq(&conn, 7).unwrap();
        let bytes = vault_bytes(&conn).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3\0"));

        let copy = restore_replica_bytes(&bytes, 9).unwrap();
        assert_eq!(balance_of(&copy, wallet), 1234.56);
        assert_eq!(last_applied_seq(&copy).unwrap(), 9, "the checkpoint's change number wins");
        // The copy is a normal, writable vault
        log_income(&copy, wallet, 100.0, "More");
        assert_eq!(balance_of(&copy, wallet), 1334.56);
        assert_eq!(balance_of(&conn, wallet), 1234.56, "the original is untouched");

        assert!(open_vault_bytes(b"not a vault").is_err());
        let mut damaged = bytes.clone();
        damaged.truncate(bytes.len() / 2);
        assert!(open_vault_bytes(&damaged).is_err(), "a cut-off download is refused");
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn checkpoints_move_between_file_and_memory_vaults() {
        // A desktop checkpoint (a file copy) opens in memory, and an in-memory copy restores into a file replica
        let conn = test_db();
        let wallet = fund_account(&conn, "Wallet", 500.0);
        let scratch = std::env::temp_dir().join(format!("vault_bytes_test_{}.db", std::process::id()));
        let from_file = snapshot(&conn, &scratch).unwrap();
        assert_eq!(balance_of(&open_vault_bytes(&from_file).unwrap(), wallet), 500.0);

        let replica = std::env::temp_dir().join(format!("vault_bytes_replica_{}.db", std::process::id()));
        let restored = restore_replica(&replica, &vault_bytes(&conn).unwrap(), 3).unwrap();
        assert_eq!(balance_of(&restored, wallet), 500.0);
        drop(restored);
        let _ = std::fs::remove_file(&replica);
    }

    // ---- Sync replicas ----

    fn changes() -> Vec<(&'static str, serde_json::Value)> {
        vec![
            ("add_account", serde_json::json!({ "name": "Wallet" })),
            ("add_account", serde_json::json!({ "name": "GCash" })),
            ("log_income", serde_json::json!({ "accountId": 1, "amount": 1000.0, "description": "Salary" })),
            ("transfer_money", serde_json::json!({ "fromAccountId": 1, "toAccountId": 2, "amount": 250.5, "description": "Load" })),
            ("transfer_money", serde_json::json!({ "fromAccountId": 2, "toAccountId": 1, "amount": 9999.0, "description": "Too much" })),
            ("add_smart_bill", serde_json::json!({ "name": "Internet", "amount": 1500.0, "dueDate": "2026-11-20", "paydayDate": "2026-11-15", "isRecurring": true })),
        ]
    }
    fn state(conn: &Connection) -> String {
        let accounts = dispatch(conn, "get_accounts", &serde_json::json!({})).unwrap();
        let txs = dispatch(conn, "get_all_transactions", &serde_json::json!({})).unwrap();
        let bills = dispatch(conn, "get_smart_bills", &serde_json::json!({})).unwrap();
        format!("{} {} {}", accounts, txs, bills)
    }

    #[test]
    fn replicas_applying_the_same_changes_end_up_identical() {
        let phone = test_db();
        let desktop = test_db();
        for (i, (cmd, args)) in changes().iter().enumerate() {
            let at = format!("2026-10-15 08:00:{:02}", i);
            apply_op(&phone, i as i64 + 1, cmd, args, &at).unwrap();
            apply_op(&desktop, i as i64 + 1, cmd, args, &at).unwrap();
        }
        assert_eq!(state(&phone), state(&desktop));
        assert_eq!(last_applied_seq(&phone).unwrap(), 6);
        // The timestamp comes from the change, not the clock of the device applying it
        assert!(state(&phone).contains("2026-10-15 08:00:02"));
    }

    #[test]
    fn rejected_changes_still_count_and_change_nothing() {
        let conn = test_db();
        for (i, (cmd, args)) in changes().iter().take(4).enumerate() { apply_op(&conn, i as i64 + 1, cmd, args, "2026-10-15 08:00:00").unwrap(); }
        let before = state(&conn);
        let (cmd, args) = &changes()[4];
        let outcome = apply_op(&conn, 5, cmd, args, "2026-10-15 08:00:00").unwrap();
        assert!(outcome["result"].as_str().unwrap().starts_with("ERROR"), "{}", outcome);
        assert_eq!(state(&conn), before);
        assert_eq!(last_applied_seq(&conn).unwrap(), 5);
        // Bad arguments are reported as an error result, and also count
        let outcome = apply_op(&conn, 6, "delete_account", &serde_json::json!({}), "2026-10-15 08:00:00").unwrap();
        assert!(outcome["error"].is_string());
        assert_eq!(last_applied_seq(&conn).unwrap(), 6);
    }

    #[test]
    fn changes_must_be_applied_strictly_in_order() {
        let conn = test_db();
        let add = serde_json::json!({ "name": "Wallet" });
        assert!(apply_op(&conn, 2, "add_account", &add, "2026-10-15 08:00:00").is_err());
        apply_op(&conn, 1, "add_account", &add, "2026-10-15 08:00:00").unwrap();
        assert!(apply_op(&conn, 1, "add_account", &add, "2026-10-15 08:00:00").is_err(), "a change must never be applied twice");
        assert!(apply_op(&conn, 2, "get_accounts", &serde_json::json!({}), "2026-10-15 08:00:00").is_err(), "reads are never changes");
        assert!(apply_op(&conn, 2, "add_account", &add, "yesterday").is_err());
        assert_eq!(last_applied_seq(&conn).unwrap(), 1);
    }
}
