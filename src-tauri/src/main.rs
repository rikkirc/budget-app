#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

// 🚨 NEW: The Receptionist's memory! This holds the name of the currently logged-in user's safe.
struct AppState { 
    db_path: Mutex<String> 
}

#[derive(Serialize)] struct Account { id: i32, name: String, balance: f64 }
#[derive(Serialize)] struct TransactionLog { id: i32, account_name: String, tx_type: String, amount: f64, description: String, is_reversed: bool, created_at: String, linked_account_name: String }
#[derive(Serialize)] struct SmartBill { id: i32, name: String, amount: f64, due_date: String, payday_date: String, is_recurring: bool, loan_id: i32 }
#[derive(Serialize)] struct Loan { id: i32, name: String, principal: f64, remaining_balance: f64, term: i32, term_type: String, frequency: String, payment_amount: f64, next_due_date: String, payday_date: String, total_incurred: f64, due_structure: String, date_acquired: String }
#[derive(Serialize)] struct PaidBill { id: i32, original_bill_id: i32, name: String, amount: f64, due_date: String, payday_date: String, account_id: i32, from_account_id: i32, loan_id: i32 }
#[derive(Serialize)] struct CompletedLoan { id: i32, original_id: i32, name: String, principal: f64, term: i32, term_type: String, frequency: String, payment_amount: f64, total_incurred: f64, completed_date: String }
#[derive(Serialize)] struct SinkingFundInfo { found: bool, account_id: i32, account_name: String, amount: f64 }
#[derive(Serialize)] struct ArchivedBill { id: i32, original_id: i32, name: String, amount: f64, due_date: String, payday_date: String }

// 🚨 NEW: The Front Door Login Command. It creates a brand new safe if it doesn't exist!
#[tauri::command]
fn login_user(profile: &str, state: State<'_, AppState>) -> String {
    // Turn "John Doe" into "john_doe_finance_data.db"
    let db_filename = format!("{}_finance_data.db", profile.to_lowercase().replace(" ", "_"));

    // 🚨 NEW: Always save to Documents\FinancialCommandCenter\ — a fixed, permanent home
    // that every future version of the app will automatically find, regardless of where the .exe itself sits.
    let mut app_folder = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    app_folder.push("FinancialCommandCenter");
    let _ = std::fs::create_dir_all(&app_folder); // Creates the folder if it doesn't exist yet, safely does nothing if it already does

    let db_path_buf = app_folder.join(&db_filename);
    let db_name = db_path_buf.to_string_lossy().to_string();
    
    // Tell the Receptionist to remember this safe for all future commands
    *state.db_path.lock().unwrap() = db_name.clone();
    
    let conn = Connection::open(&db_name).unwrap();
    init_schema(&conn);
    format!("SUCCESS: Logged in as {}", profile)
}

// Open the safe and build the empty drawers if this is a brand-new user!
fn init_schema(conn: &Connection) {
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
    conn.execute("CREATE TABLE IF NOT EXISTS archived_smart_bills (id INTEGER PRIMARY KEY, original_id INTEGER, name TEXT NOT NULL, amount REAL NOT NULL, due_date TEXT NOT NULL, payday_date TEXT NOT NULL)", []).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS completed_loans (id INTEGER PRIMARY KEY, original_id INTEGER, name TEXT NOT NULL, principal REAL NOT NULL, term INTEGER NOT NULL, term_type TEXT NOT NULL, frequency TEXT NOT NULL, payment_amount REAL NOT NULL, total_incurred REAL NOT NULL, completed_date TEXT NOT NULL)", []).unwrap();
    // Date a loan was acquired/taken out (optional; blank for older loans)
    let _ = conn.execute("ALTER TABLE loans ADD COLUMN date_acquired TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE archived_loans ADD COLUMN date_acquired TEXT NOT NULL DEFAULT ''", []);
    // Date & time each transaction was recorded (stamped automatically by the database, in local time)
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN created_at TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute_batch("CREATE TRIGGER IF NOT EXISTS set_tx_timestamp AFTER INSERT ON transactions WHEN NEW.created_at = '' BEGIN UPDATE transactions SET created_at = datetime('now','localtime') WHERE id = NEW.id; END;");
}

// 🚨 UPDATED: Every single command now asks the Receptionist which safe to open!
#[tauri::command] fn add_account(name: &str, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); match conn.execute("INSERT INTO accounts (name, balance) VALUES (?1, 0.0)", params![name]) { Ok(_) => format!("SUCCESS: Created new account '{}'!", name), Err(_) => "ERROR: Account might already exist.".to_string() } }
#[tauri::command] fn get_accounts(state: State<'_, AppState>) -> Vec<Account> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, name, balance FROM accounts").unwrap(); let iter = stmt.query_map([], |row| { Ok(Account { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), balance: row.get(2).unwrap_or(0.0) }) }).unwrap(); let mut accounts = Vec::new(); for a in iter { if let Ok(acc) = a { accounts.push(acc); } } accounts }
#[tauri::command] fn delete_account(account_id: i32, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let balance: f64 = conn.query_row("SELECT balance FROM accounts WHERE id = ?1", params![account_id], |row| row.get(0)).unwrap_or(0.0); if balance > 0.0 { return format!("ERROR: You cannot delete an account that still holds money. Please transfer the remaining ₱{:.2} first!", balance); } let name: String = conn.query_row("SELECT name FROM accounts WHERE id = ?1", params![account_id], |row| row.get(0)).unwrap_or_else(|_| "".to_string()); let _ = conn.execute("INSERT INTO archived_accounts (original_id, name) VALUES (?1, ?2)", params![account_id, name]); let _ = conn.execute("DELETE FROM accounts WHERE id = ?1", params![account_id]); "SUCCESS: Empty account safely moved to the Recycle Bin!".to_string() }
#[tauri::command] fn log_income(account_id: i32, amount: f64, description: &str, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, account_id]); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'INCOME', ?2, ?3)", params![account_id, amount, description]); "SUCCESS!".to_string() }
#[tauri::command] fn log_expense(account_id: i32, amount: f64, description: &str, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, account_id]); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'EXPENSE', ?2, ?3)", params![account_id, amount, description]); "SUCCESS!".to_string() }
#[tauri::command] fn transfer_money(from_account_id: i32, to_account_id: i32, amount: f64, description: &str, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let current_balance: f64 = conn.query_row("SELECT balance FROM accounts WHERE id = ?1", params![from_account_id], |r| r.get(0)).unwrap_or(0.0); if current_balance < (amount - 0.01) { return format!("ERROR: Insufficient funds! Source account only has ₱{:.2}, but the transfer needs ₱{:.2}.", current_balance, amount); } let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![amount, from_account_id]); let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![amount, to_account_id]); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'TRANSFER_OUT', ?2, ?3)", params![from_account_id, amount, description]); let out_id = conn.last_insert_rowid(); let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description, linked_id) VALUES (?1, 'TRANSFER_IN', ?2, ?3, ?4)", params![to_account_id, amount, description, out_id]); let in_id = conn.last_insert_rowid(); let _ = conn.execute("UPDATE transactions SET linked_id = ?1 WHERE id = ?2", params![in_id, out_id]); "SUCCESS!".to_string() }
#[tauri::command]
fn undo_transaction(transaction_id: i32, state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();

    let mut stmt = conn.prepare("SELECT account_id, tx_type, amount, is_reversed, linked_id FROM transactions WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![transaction_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let account_id: i32 = row.get(0).unwrap();
        let tx_type: String = row.get(1).unwrap();
        let amount: f64 = row.get(2).unwrap();
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
                let p_amount: f64 = p_row.get(2).unwrap();
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
#[tauri::command] fn get_all_transactions(state: State<'_, AppState>) -> Vec<TransactionLog> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT t.id, a.name, t.tx_type, t.amount, t.description, t.is_reversed, t.created_at, COALESCE(pa.name, '') FROM transactions t JOIN accounts a ON t.account_id = a.id LEFT JOIN transactions p ON t.linked_id = p.id LEFT JOIN accounts pa ON p.account_id = pa.id ORDER BY t.id DESC").unwrap(); let iter = stmt.query_map([], |row| { let reversed_int: i32 = row.get(5).unwrap_or(0); Ok(TransactionLog { id: row.get(0).unwrap_or(0), account_name: row.get(1).unwrap_or_else(|_| "".to_string()), tx_type: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), description: row.get(4).unwrap_or_else(|_| "".to_string()), is_reversed: reversed_int != 0, created_at: row.get(6).unwrap_or_else(|_| "".to_string()), linked_account_name: row.get(7).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut logs = Vec::new(); for t in iter { if let Ok(l) = t { logs.push(l); } } logs }
// account_id = None returns the complete history across all accounts; Some(id) returns only that account's entries
#[tauri::command] fn get_transactions_page(offset: i32, limit: i32, account_id: Option<i32>, state: State<'_, AppState>) -> Vec<TransactionLog> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); get_transactions_page_impl(&conn, offset, limit, account_id) }
fn get_transactions_page_impl(conn: &Connection, offset: i32, limit: i32, account_id: Option<i32>) -> Vec<TransactionLog> { let mut stmt = conn.prepare("SELECT t.id, a.name, t.tx_type, t.amount, t.description, t.is_reversed, t.created_at, COALESCE(pa.name, '') FROM transactions t JOIN accounts a ON t.account_id = a.id LEFT JOIN transactions p ON t.linked_id = p.id LEFT JOIN accounts pa ON p.account_id = pa.id WHERE (?3 IS NULL OR t.account_id = ?3) ORDER BY t.id DESC LIMIT ?1 OFFSET ?2").unwrap(); let iter = stmt.query_map(params![limit, offset, account_id], |row| { let reversed_int: i32 = row.get(5).unwrap_or(0); Ok(TransactionLog { id: row.get(0).unwrap_or(0), account_name: row.get(1).unwrap_or_else(|_| "".to_string()), tx_type: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), description: row.get(4).unwrap_or_else(|_| "".to_string()), is_reversed: reversed_int != 0, created_at: row.get(6).unwrap_or_else(|_| "".to_string()), linked_account_name: row.get(7).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut logs = Vec::new(); for t in iter { if let Ok(l) = t { logs.push(l); } } logs }
#[tauri::command] fn add_smart_bill(name: &str, amount: f64, due_date: &str, payday_date: &str, is_recurring: bool, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); match conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, is_recurring) VALUES (?1, ?2, ?3, ?4, ?5)", params![name, amount, due_date, payday_date, is_recurring as i32]) { Ok(_) => "SUCCESS!".to_string(), Err(_) => "ERROR".to_string() } }
#[tauri::command] fn update_bill_amount(bill_id: i32, new_amount: f64, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); match conn.execute("UPDATE smart_bills SET amount = ?1 WHERE id = ?2", params![new_amount, bill_id]) { Ok(_) => "SUCCESS: Amount updated!".to_string(), Err(_) => "ERROR: Could not update amount".to_string() } }
// Reassign one unpaid bill to a different paycheck allocation (only this occurrence; next month's recurring copy still uses the normal payday rule)
#[tauri::command] fn move_bill_allocation(bill_id: i32, new_payday: &str, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); move_bill_allocation_impl(&conn, bill_id, new_payday) }
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
#[tauri::command] fn get_smart_bills(state: State<'_, AppState>) -> Vec<SmartBill> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, name, amount, due_date, payday_date, is_recurring, loan_id FROM smart_bills").unwrap(); let iter = stmt.query_map([], |row| { let recurring_int: i32 = row.get(5).unwrap_or(0); Ok(SmartBill { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), amount: row.get(2).unwrap_or(0.0), due_date: row.get(3).unwrap_or_else(|_| "".to_string()), payday_date: row.get(4).unwrap_or_else(|_| "".to_string()), is_recurring: recurring_int != 0, loan_id: row.get(6).unwrap_or(0) }) }).unwrap(); let mut bills = Vec::new(); for b in iter { if let Ok(item) = b { bills.push(item); } } bills }
#[tauri::command] fn add_loan(name: &str, principal: f64, remaining_balance: f64, term: i32, term_type: &str, frequency: &str, payment_amount: f64, due_date: &str, payday_date: &str, total_incurred: f64, due_structure: &str, date_acquired: &str, bill_names: Vec<String>, bill_amounts: Vec<f64>, bill_dues: Vec<String>, bill_paydays: Vec<String>, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let _ = conn.execute("INSERT INTO loans (name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![name, principal, remaining_balance, term, term_type, frequency, payment_amount, due_date, payday_date, total_incurred, due_structure, date_acquired]); let new_loan_id = conn.last_insert_rowid(); for i in 0..bill_names.len() { let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![bill_names[i], bill_amounts[i], bill_dues[i], bill_paydays[i], new_loan_id]); } "SUCCESS: Loan generated!".to_string() }
#[tauri::command] fn get_loans(state: State<'_, AppState>) -> Vec<Loan> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM loans").unwrap(); let iter = stmt.query_map([], |row| { Ok(Loan { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), principal: row.get(2).unwrap_or(0.0), remaining_balance: row.get(3).unwrap_or(0.0), term: row.get(4).unwrap_or(0), term_type: row.get(5).unwrap_or_else(|_| "".to_string()), frequency: row.get(6).unwrap_or_else(|_| "".to_string()), payment_amount: row.get(7).unwrap_or(0.0), next_due_date: row.get(8).unwrap_or_else(|_| "".to_string()), payday_date: row.get(9).unwrap_or_else(|_| "".to_string()), total_incurred: row.get(10).unwrap_or(0.0), due_structure: row.get(11).unwrap_or_else(|_| "Sinking Fund".to_string()), date_acquired: row.get(12).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut loans = Vec::new(); for l in iter { if let Ok(item) = l { loans.push(item); } } loans }
#[tauri::command] fn delete_loan(loan_id: i32, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM loans WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![loan_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let principal: f64 = row.get(1).unwrap(); let remaining_balance: f64 = row.get(2).unwrap(); let term: i32 = row.get(3).unwrap(); let term_type: String = row.get(4).unwrap(); let frequency: String = row.get(5).unwrap(); let payment_amount: f64 = row.get(6).unwrap(); let next_due_date: String = row.get(7).unwrap(); let payday_date: String = row.get(8).unwrap(); let total_incurred: f64 = row.get(9).unwrap_or(0.0); let due_structure: String = row.get(10).unwrap_or_else(|_| "Sinking Fund".to_string()); let date_acquired: String = row.get(11).unwrap_or_else(|_| "".to_string()); let _ = conn.execute("INSERT INTO archived_loans (original_id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", params![loan_id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired]); let search_pattern = format!("{}%", name); let _ = conn.execute("DELETE FROM smart_bills WHERE name LIKE ?1", params![search_pattern]); } let _ = conn.execute("DELETE FROM loans WHERE id = ?1", params![loan_id]); "SUCCESS: Loan moved to Recycle Bin!".to_string() }

#[tauri::command]  
fn fund_bill_to_savings(bill_id: i32, from_account_id: i32, to_account_id: i32, amount: f64, description: &str, state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();
    
    // 1. Get bill details (now also grabbing loan_id!)
    let mut stmt = conn.prepare("SELECT name, due_date, payday_date, loan_id FROM smart_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![bill_id]).unwrap();
    
    if let Some(row) = rows.next().unwrap() {
        let bill_name: String = row.get(0).unwrap();
        let due: String = row.get(1).unwrap();
        let payday: String = row.get(2).unwrap();
        let bill_loan_id: i32 = row.get(3).unwrap_or(0);

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
        let _ = conn.execute("INSERT INTO paid_bills (original_bill_id, name, amount, due_date, payday_date, account_id, loan_id, from_account_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![bill_id, bill_name, amount, due, payday, to_account_id, bill_loan_id, from_account_id]);
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
#[tauri::command]
fn pay_bill(bill_id: i32, account_id: i32, amount: f64, description: &str, state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();
    
    // 1. Get details of the bill being paid
    let mut stmt = conn.prepare("SELECT name, due_date, payday_date, loan_id, is_recurring FROM smart_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![bill_id]).unwrap();
    
    if let Some(row) = rows.next().unwrap() {
        let bill_name: String = row.get(0).unwrap();
        let due: String = row.get(1).unwrap();
        let payday: String = row.get(2).unwrap();
        let bill_loan_id: i32 = row.get(3).unwrap_or(0);
        let is_recurring_int: i32 = row.get(4).unwrap_or(0);
        let is_recurring = is_recurring_int != 0;

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
        let real_amount: f64 = if is_sinking_fund_second_half { amount * 2.0 } else { amount };

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
        let _ = conn.execute("INSERT INTO paid_bills (original_bill_id, name, amount, due_date, payday_date, account_id, loan_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![bill_id, bill_name, amount, due, payday, account_id, bill_loan_id]);
        let _ = conn.execute("DELETE FROM smart_bills WHERE id = ?1", params![bill_id]);

        // 🔁 RECURRING BILL: automatically create next month's occurrence, amount reset to TBD (0)
        if is_recurring {
            // Next month's due date, same day (e.g. Sep 20 -> Oct 20)
            let next_due: String = conn.query_row("SELECT date(?1, '+1 month')", params![due], |r| r.get(0)).unwrap_or_else(|_| due.clone());
            // Same 10-day rule as loans: the paycheck must land at least 11 days before the due date
            let gap_date: String = conn.query_row("SELECT date(?1, '-11 days')", params![next_due], |r| r.get(0)).unwrap_or_else(|_| next_due.clone());
            // Paydays are the 15th and the end of the month: pick the latest one on or before the gap date
            let next_payday: String = conn.query_row(
                "SELECT MAX(c) FROM (
                    SELECT date(?1, 'start of month', '-1 month', '+14 days') AS c
                    UNION ALL SELECT date(?1, 'start of month', '-1 day')
                    UNION ALL SELECT date(?1, 'start of month', '+14 days')
                    UNION ALL SELECT date(?1, 'start of month', '+1 month', '-1 day')
                 ) WHERE c <= ?1",
                params![gap_date],
                |r| r.get(0)
            ).unwrap_or_else(|_| payday.clone());
            let _ = conn.execute(
                "INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id, is_recurring) VALUES (?1, 0, ?2, ?3, 0, 1)",
                params![bill_name, next_due, next_payday]
            );
        }
        
        // Deduct the REAL full amount from the account and log it
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance - ?1, 2) WHERE id = ?2", params![real_amount, account_id]);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'EXPENSE', ?2, ?3)", params![account_id, real_amount, description]);

        if is_sinking_fund_second_half {
            // Match the 1st Half by loan_id + exact 15-day relationship — no more name/date guesswork
            let _ = conn.execute(
                "INSERT INTO paid_bills (original_bill_id, name, amount, due_date, payday_date, account_id, loan_id) 
                 SELECT id, name, amount, due_date, payday_date, ?1, loan_id FROM smart_bills 
                 WHERE loan_id = ?2 AND name LIKE '%(1st Half)%' AND due_date = date(?3, '-15 days')",
                params![account_id, bill_loan_id, due]
            );
            let _ = conn.execute(
                "DELETE FROM smart_bills WHERE loan_id = ?1 AND name LIKE '%(1st Half)%' AND due_date = date(?2, '-15 days')",
                params![bill_loan_id, due]
            );
        }

        // Loan's remaining balance drops by the same REAL amount that left the account — matched by ID, not name
        let _ = conn.execute("UPDATE loans SET remaining_balance = remaining_balance - ?1 WHERE id = ?2", params![real_amount, bill_loan_id]);

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
#[tauri::command]
fn undo_bill_payment(paid_bill_id: i32, state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();

    let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date, account_id, loan_id FROM paid_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![paid_bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let name: String = row.get(0).unwrap();
        let amount: f64 = row.get(1).unwrap();
        let due: String = row.get(2).unwrap();
        let payday: String = row.get(3).unwrap();
        let account_id: i32 = row.get(4).unwrap();
        let loan_id: i32 = row.get(5).unwrap_or(0);

        // Figure out if this was a Sinking Fund 2nd Half (meaning the REAL amount taken was double the listed amount)
        let due_structure: String = conn.query_row(
            "SELECT due_structure FROM loans WHERE id = ?1",
            params![loan_id],
            |r| r.get(0)
        ).unwrap_or_else(|_| "Sinking Fund".to_string());

        let is_sinking_fund_second_half = due_structure == "Sinking Fund" && (name.contains("(2nd Half (Due))") || name.contains("(2nd Half)"));
        let real_amount: f64 = if is_sinking_fund_second_half { amount * 2.0 } else { amount };

        // 1. Refund the REAL amount back to the account it was taken from
        let _ = conn.execute("UPDATE accounts SET balance = ROUND(balance + ?1, 2) WHERE id = ?2", params![real_amount, account_id]);
        let refund_desc = format!("REFUND (Undo): {}", name);
        let _ = conn.execute("INSERT INTO transactions (account_id, tx_type, amount, description) VALUES (?1, 'INCOME', ?2, ?3)", params![account_id, real_amount, refund_desc]);

        // 2. If this loan still exists (wasn't graduated to Completed), restore its remaining_balance
        let loan_exists: bool = conn.query_row("SELECT 1 FROM loans WHERE id = ?1", params![loan_id], |_| Ok(true)).unwrap_or(false);
        if loan_exists {
            let _ = conn.execute("UPDATE loans SET remaining_balance = remaining_balance + ?1 WHERE id = ?2", params![real_amount, loan_id]);
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

        // 3. Restore this bill to smart_bills (with its correct loan_id)
        let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![name, amount, due, payday, loan_id]);

        // 4. If it was a Sinking Fund 2nd Half, also restore the matching 1st Half that got auto-cleared
        if is_sinking_fund_second_half {
            let _ = conn.execute(
                "INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id)
                 SELECT name, amount, due_date, payday_date, loan_id FROM paid_bills
                 WHERE loan_id = ?1 AND name LIKE '%(1st Half)%' AND due_date = date(?2, '-15 days')
                 ORDER BY id DESC LIMIT 1",
                params![loan_id, due]
            );
            let _ = conn.execute(
                "DELETE FROM paid_bills WHERE id = (SELECT id FROM paid_bills WHERE loan_id = ?1 AND name LIKE '%(1st Half)%' AND due_date = date(?2, '-15 days') ORDER BY id DESC LIMIT 1)",
                params![loan_id, due]
            );
        }

        // 5. Remove the auto-generated NEXT recurring bill this payment may have created (only if it's still untouched/TBD)
        let _ = conn.execute(
            "DELETE FROM smart_bills WHERE name = ?1 AND loan_id = 0 AND is_recurring = 1 AND amount = 0 AND due_date = date(?2, '+1 month')",
            params![name, due]
        );

        // 6. Remove this paid receipt
        let _ = conn.execute("DELETE FROM paid_bills WHERE id = ?1", params![paid_bill_id]);
    }
    "SUCCESS: Payment undone and refunded!".to_string()
}
#[tauri::command] fn get_paid_bills(state: State<'_, AppState>) -> Vec<PaidBill> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, original_bill_id, name, amount, due_date, payday_date, account_id, from_account_id, loan_id FROM paid_bills ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(PaidBill { id: row.get(0).unwrap_or(0), original_bill_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), due_date: row.get(4).unwrap_or_else(|_| "".to_string()), payday_date: row.get(5).unwrap_or_else(|_| "".to_string()), account_id: row.get(6).unwrap_or(0), from_account_id: row.get(7).unwrap_or(0), loan_id: row.get(8).unwrap_or(0) }) }).unwrap(); let mut archived = Vec::new(); for b in iter { if let Ok(item) = b { archived.push(item); } } archived }
#[tauri::command] fn get_paid_bills_page(offset: i32, limit: i32, state: State<'_, AppState>) -> Vec<PaidBill> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, original_bill_id, name, amount, due_date, payday_date, account_id, from_account_id, loan_id FROM paid_bills ORDER BY id DESC LIMIT ?1 OFFSET ?2").unwrap(); let iter = stmt.query_map(params![limit, offset], |row| { Ok(PaidBill { id: row.get(0).unwrap_or(0), original_bill_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), due_date: row.get(4).unwrap_or_else(|_| "".to_string()), payday_date: row.get(5).unwrap_or_else(|_| "".to_string()), account_id: row.get(6).unwrap_or(0), from_account_id: row.get(7).unwrap_or(0), loan_id: row.get(8).unwrap_or(0) }) }).unwrap(); let mut archived = Vec::new(); for b in iter { if let Ok(item) = b { archived.push(item); } } archived }
#[tauri::command] fn get_archived_loans(state: State<'_, AppState>) -> Vec<Loan> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM archived_loans ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(Loan { id: row.get(0).unwrap_or(0), name: row.get(1).unwrap_or_else(|_| "".to_string()), principal: row.get(2).unwrap_or(0.0), remaining_balance: row.get(3).unwrap_or(0.0), term: row.get(4).unwrap_or(0), term_type: row.get(5).unwrap_or_else(|_| "".to_string()), frequency: row.get(6).unwrap_or_else(|_| "".to_string()), payment_amount: row.get(7).unwrap_or(0.0), next_due_date: row.get(8).unwrap_or_else(|_| "".to_string()), payday_date: row.get(9).unwrap_or_else(|_| "".to_string()), total_incurred: row.get(10).unwrap_or(0.0), due_structure: row.get(11).unwrap_or_else(|_| "Sinking Fund".to_string()), date_acquired: row.get(12).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut loans = Vec::new(); for l in iter { if let Ok(item) = l { loans.push(item); } } loans }
#[tauri::command] fn get_completed_loans(state: State<'_, AppState>) -> Vec<CompletedLoan> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, original_id, name, principal, term, term_type, frequency, payment_amount, total_incurred, completed_date FROM completed_loans ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(CompletedLoan { id: row.get(0).unwrap_or(0), original_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), principal: row.get(3).unwrap_or(0.0), term: row.get(4).unwrap_or(0), term_type: row.get(5).unwrap_or_else(|_| "".to_string()), frequency: row.get(6).unwrap_or_else(|_| "".to_string()), payment_amount: row.get(7).unwrap_or(0.0), total_incurred: row.get(8).unwrap_or(0.0), completed_date: row.get(9).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut completed = Vec::new(); for l in iter { if let Ok(item) = l { completed.push(item); } } completed }
#[tauri::command] fn undo_delete_loan(archive_id: i32, bill_names: Vec<String>, bill_amounts: Vec<f64>, bill_dues: Vec<String>, bill_paydays: Vec<String>, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired FROM archived_loans WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![archive_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let principal: f64 = row.get(1).unwrap(); let remaining_balance: f64 = row.get(2).unwrap(); let term: i32 = row.get(3).unwrap(); let term_type: String = row.get(4).unwrap(); let frequency: String = row.get(5).unwrap(); let payment_amount: f64 = row.get(6).unwrap(); let next_due_date: String = row.get(7).unwrap(); let payday_date: String = row.get(8).unwrap(); let total_incurred: f64 = row.get(9).unwrap_or(0.0); let due_structure: String = row.get(10).unwrap_or_else(|_| "Sinking Fund".to_string()); let date_acquired: String = row.get(11).unwrap_or_else(|_| "".to_string()); let _ = conn.execute("INSERT INTO loans (name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![name, principal, remaining_balance, term, term_type, frequency, payment_amount, next_due_date, payday_date, total_incurred, due_structure, date_acquired]); let restored_loan_id = conn.last_insert_rowid(); for i in 0..bill_names.len() { let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![bill_names[i], bill_amounts[i], bill_dues[i], bill_paydays[i], restored_loan_id]); } let _ = conn.execute("DELETE FROM archived_loans WHERE id = ?1", params![archive_id]); } "SUCCESS: Loan restored!".to_string() }
#[tauri::command] fn delete_bill(bill_id: i32, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date FROM smart_bills WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![bill_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let amount: f64 = row.get(1).unwrap(); let due: String = row.get(2).unwrap(); let payday: String = row.get(3).unwrap(); let _ = conn.execute("INSERT INTO archived_smart_bills (original_id, name, amount, due_date, payday_date) VALUES (?1, ?2, ?3, ?4, ?5)", params![bill_id, name, amount, due, payday]); } let _ = conn.execute("DELETE FROM smart_bills WHERE id = ?1", params![bill_id]); "SUCCESS: Bill moved to Recycle Bin!".to_string() }
#[tauri::command] fn get_archived_bills(state: State<'_, AppState>) -> Vec<ArchivedBill> { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT id, original_id, name, amount, due_date, payday_date FROM archived_smart_bills ORDER BY id DESC").unwrap(); let iter = stmt.query_map([], |row| { Ok(ArchivedBill { id: row.get(0).unwrap_or(0), original_id: row.get(1).unwrap_or(0), name: row.get(2).unwrap_or_else(|_| "".to_string()), amount: row.get(3).unwrap_or(0.0), due_date: row.get(4).unwrap_or_else(|_| "".to_string()), payday_date: row.get(5).unwrap_or_else(|_| "".to_string()) }) }).unwrap(); let mut archived = Vec::new(); for b in iter { if let Ok(item) = b { archived.push(item); } } archived }
#[tauri::command] fn undo_delete_bill(archive_id: i32, state: State<'_, AppState>) -> String { let db = state.db_path.lock().unwrap().clone(); let conn = Connection::open(&db).unwrap(); let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date FROM archived_smart_bills WHERE id = ?1").unwrap(); let mut rows = stmt.query(params![archive_id]).unwrap(); if let Some(row) = rows.next().unwrap() { let name: String = row.get(0).unwrap(); let amount: f64 = row.get(1).unwrap(); let due: String = row.get(2).unwrap(); let payday: String = row.get(3).unwrap(); let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date) VALUES (?1, ?2, ?3, ?4)", params![name, amount, due, payday]); let _ = conn.execute("DELETE FROM archived_smart_bills WHERE id = ?1", params![archive_id]); } "SUCCESS: Bill restored to Budget!".to_string() }
#[tauri::command]
fn get_sinking_fund_info(bill_id: i32, state: State<'_, AppState>) -> SinkingFundInfo {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();

    let mut stmt = conn.prepare("SELECT due_date, loan_id FROM smart_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let due: String = row.get(0).unwrap();
        let bill_loan_id: i32 = row.get(1).unwrap_or(0);

        // Match by loan_id (never confused by similar names) + the exact 15-day relationship
        let result = conn.query_row(
            "SELECT pb.account_id, a.name, pb.amount FROM paid_bills pb
             JOIN accounts a ON pb.account_id = a.id
             WHERE pb.loan_id = ?1 AND pb.name LIKE '%(1st Half)%' AND pb.due_date = date(?2, '-15 days')
             ORDER BY pb.id DESC
             LIMIT 1",
            params![bill_loan_id, due],
            |r| Ok(SinkingFundInfo { found: true, account_id: r.get(0)?, account_name: r.get(1)?, amount: r.get(2)? })
        );

        return result.unwrap_or(SinkingFundInfo { found: false, account_id: 0, account_name: "".to_string(), amount: 0.0 });
    }

    SinkingFundInfo { found: false, account_id: 0, account_name: "".to_string(), amount: 0.0 }
}

#[tauri::command]
fn repair_balance_rounding(state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();
    let updated = conn.execute("UPDATE accounts SET balance = ROUND(balance, 2)", []).unwrap_or(0);
    format!("SUCCESS: Cleaned up rounding on {} account(s)!", updated)
}

#[tauri::command]
fn repair_loan_ids(state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();

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
#[tauri::command]
fn undo_fund_to_savings(paid_bill_id: i32, state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();

    let mut stmt = conn.prepare("SELECT name, amount, due_date, payday_date, account_id, loan_id, from_account_id FROM paid_bills WHERE id = ?1").unwrap();
    let mut rows = stmt.query(params![paid_bill_id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let raw_name: String = row.get(0).unwrap();
        let amount: f64 = row.get(1).unwrap();
        let due: String = row.get(2).unwrap();
        let payday: String = row.get(3).unwrap();
        let to_account_id: i32 = row.get(4).unwrap();
        let loan_id: i32 = row.get(5).unwrap_or(0);
        let from_account_id: i32 = row.get(6).unwrap_or(0);

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
        let _ = conn.execute("INSERT INTO smart_bills (name, amount, due_date, payday_date, loan_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![clean_name, amount, due, payday, loan_id]);

        // Remove this receipt
        let _ = conn.execute("DELETE FROM paid_bills WHERE id = ?1", params![paid_bill_id]);
    }
    "SUCCESS: Fund transfer undone!".to_string()
}

#[tauri::command]
fn mark_first_half_handled(bill_id: i32, account_name: &str, state: State<'_, AppState>) -> String {
    let db = state.db_path.lock().unwrap().clone();
    let conn = Connection::open(&db).unwrap();
    
    let updated_suffix = format!(" (Handled in {})", account_name);
    
    // Append the handled tag to whichever table currently holds the bill
    let _ = conn.execute("UPDATE paid_bills SET name = name || ?1 WHERE original_bill_id = ?2 AND name NOT LIKE '%(Handled in%'", params![updated_suffix, bill_id]);
    let _ = conn.execute("UPDATE smart_bills SET name = name || ?1 WHERE id = ?2 AND name NOT LIKE '%(Handled in%'", params![updated_suffix, bill_id]);
    
    "SUCCESS: 1st Half Sinking Fund Logged!".to_string()
}
fn main() {
    tauri::Builder::default()
        // Initialize the Receptionist memory with a blank slate!
        .manage(AppState { db_path: Mutex::new("".to_string()) })
        .invoke_handler(tauri::generate_handler![
            login_user, // 🚨 The new Front Door command!
            add_account, get_accounts, delete_account, mark_first_half_handled, log_income, fund_bill_to_savings, log_expense, transfer_money, get_all_transactions,
            add_smart_bill, get_smart_bills, add_loan, get_loans, delete_loan, pay_bill, get_paid_bills, undo_bill_payment,
            get_archived_loans, undo_delete_loan, delete_bill, get_archived_bills, undo_delete_bill, get_completed_loans, get_sinking_fund_info, repair_loan_ids, update_bill_amount,
            undo_fund_to_savings, undo_transaction, repair_balance_rounding, get_transactions_page, get_paid_bills_page,
            move_bill_allocation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// These tests run against a throwaway in-memory database, never a real vault file
#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn move_bill_reports_missing_bill() {
        let conn = test_db();
        let result = move_bill_allocation_impl(&conn, 999, "2026-10-31");
        assert!(result.starts_with("ERROR"), "{}", result);
    }
}