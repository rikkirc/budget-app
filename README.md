# The Vault

🌐 **Live: [thevault-rc.web.app](https://thevault-rc.web.app)** · 👀 **[Try the demo (no account needed)](https://thevault-rc.web.app/?demo)**

A budgeting app for ordinary paycheck to paycheck people: an installable web app (PWA) for iPhone, Android and computers, built with React, Vite and Rust compiled to WebAssembly. Each profile gets its own SQLite "vault", synced in real time between your devices through Firebase.

The earlier Windows (`.exe`) and Android (`.apk`) apps, built with [Tauri](https://tauri.app/), are retired: their code is still here, but they're no longer built or updated.

▶️ **[Watch the sync demo](https://drive.google.com/file/d/1LszPGQpBaUTLtbLXiTfI56x02FueR9nd/view)**: a bill paid on the phone shows up on the desktop within a second.

📄 **[Read the case study](docs/CASE_STUDY.md)**: why I built it, how sync avoids recording a payment twice, and what I'd do differently.

## Screenshots

The same demo vault on the desktop and on an Android phone, kept in step by live sync.

<p>
  <img src="docs/screenshots/desktop-dashboard.png" alt="Desktop dashboard showing total cash, total debt, and the next 5 upcoming bills" width="640">
  <img src="docs/screenshots/phone-dashboard.jpg" alt="The same dashboard on an Android phone" width="200">
</p>

<img src="docs/screenshots/desktop-budget-summary.png" alt="Bi-Monthly Budget summary: each bill with its due date, amount, and the paycheck that pays it" width="720">

*Bi-Monthly Budget: every bill is assigned to the paycheck that will cover it. Electricity is still TBD, so it's flagged until the amount comes in.*

<img src="docs/screenshots/desktop-loan.png" alt="Loan Generator: a 12-month loan with its full bi-monthly payment schedule" width="480">

*Loan Generator: a ₱10,000 loan turned into a 24-payment bi-monthly schedule, each payment assigned to a paycheck.*

## Features

- **Dashboard**: an overview of your accounts and finances, with the total of the next 5 upcoming bills
- **Cash Flow**: track money coming in and going out, with transaction history per account or across all accounts
- **Bi-Monthly Budget**: plan spending across twice-monthly pay periods; move bills between paychecks and split recurring bills into halves
- **Loan Generator**: model debt and loan payoff, with Sinking Fund or True Bi-Monthly schedules
- **Search**: find any bill, loan, or amount
- **Smart bills**: reminders at login for recurring bills due within 10 days that still need an amount
- **Web app for any device**: installable from the browser, working offline and updating itself. The Rust business logic runs in the browser as WebAssembly, so every device computes identical balances
- **Real-time sync**: people create their own accounts (with an invite code) on a free Firebase project (no server, no card), and every change on one device shows up on their others within a second. See [Accounts and sync](docs/SYNC_AND_MOBILE.md)

## User's Manual

New to the app? Read the [User's Manual](docs/USER_MANUAL.md), or the print-ready [PDF version](docs/Vault-User-Manual.pdf).

## Getting started

Prerequisites: [Node.js](https://nodejs.org/), Rust, and the WebAssembly tools in [vault-web/README.md](vault-web/README.md).

```bash
npm install
npm run dev:web     # local dev server
npm run build:web   # build into dist-web/, published to Firebase Hosting by .github/workflows/web.yml
```

Every merge to `main` that changes the app (or the manual PDF) publishes it automatically.

The retired desktop and Android apps can still be run with `npm run tauri dev` (needs the [Tauri prerequisites](https://tauri.app/start/prerequisites/)), or built by hand from **Actions → Windows app / Android app → Run workflow**.

## Running tests

```bash
npm run build:wasm && npm test
```

runs the frontend tests (search, and the sync engine against real vault-core replicas, native and WebAssembly). The backend tests run against throwaway databases, never your real vaults:

```bash
cd vault-core && cargo test   # the business logic shared by every app
npm run test:firebase         # sync and security rules on the Firebase emulators (needs Java)
```

## Project layout

- `src/`: the React frontend. `src/api.js` sends each command through the sync engine in `src/sync/` (or, in the retired desktop app, to the local vault).
- `vault-core/`: all the business logic (accounts, bills, loans…), and the rules for applying synced changes.
- `src-tauri/`: the retired desktop/Android app shell (no longer built automatically).
- `vault-web/`: vault-core compiled to WebAssembly for the web app.
- `src/firebaseConfig.js` and `firestore.rules`: the optional Firebase sync setup; see [docs/SYNC_AND_MOBILE.md](docs/SYNC_AND_MOBILE.md).

## Your data

Your vaults are stored in your Firebase account, and each device keeps its own copy in the browser's storage, kept in step through Firebase (see [backups](docs/SYNC_AND_MOBILE.md#backups)). The retired desktop app kept unsynced vaults as `<profile_name>_finance_data.db` in `Documents\FinancialCommandCenter\`; those files hold personal financial data and live outside the repo. `.gitignore` also excludes any stray `*.db` files so they are never committed.

## Recommended IDE setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
