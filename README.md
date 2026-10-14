# Financial Command Center

A desktop budgeting app built with [Tauri](https://tauri.app/), React, and Vite. Each profile gets its own local SQLite "vault", so your data stays on your machine.

## Features

- **Dashboard**: an overview of your accounts and finances
- **Cash Flow**: track money coming in and going out
- **Bi-Monthly Budget**: plan spending across twice-monthly pay periods
- **Loan Generator**: model debt and loan payoff
- **Smart bills**: reminders at login for recurring bills due within 10 days that still need an amount

## Getting started

Prerequisites: [Node.js](https://nodejs.org/) and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) (Rust and the WebView2 runtime on Windows).

```bash
npm install
npm run tauri dev
```

To build an installer:

```bash
npm run tauri build
```

## Your data

When you unlock a vault, the app creates `<profile_name>_finance_data.db` in `src-tauri/`. These files hold your personal financial data and are excluded from git by `.gitignore`. Back them up separately and never commit them.

## Recommended IDE setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
