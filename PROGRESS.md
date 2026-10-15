# Progress log

What's done, what's next, and what's blocked. Newest entries first.

## Current state (2026-10-15) — v0.2.0

- **Platforms:** Windows desktop (Tauri) and Android.
- **Sync:** optional real-time sync between devices through Firebase (free Spark plan), with family accounts gated by an invite code. Without sync, each vault stays on the device.
- **Features:** Dashboard, Cash Flow with per-account history, Bi-Monthly Budget, Loan Generator, Search, smart bill reminders.
- **Docs:** README, User's Manual (Markdown + PDF), sync and mobile setup guide.
- **Tests:** frontend search and sync engine, Rust backend tests (run on Linux CI), Firebase security-rule tests on the emulators.

## Log

### 2026-10-15
- Merged the Android app and real-time Firebase sync (#17).
- Fixed synced vaults failing to open on Android.
- Added Firestore security rules: invite-code sign-up, private data per account, append-only change log.
- Added the User's Manual (#16) and bumped to v0.2.0 (#15).
- Added Search (#14), the next-5-bills total on the Dashboard (#13), per-account transaction history (#9).
- Bi-monthly budgeting: move bills between paychecks (#8), split bills into halves (#10), fixed ₱0.01 rounding errors (#11, #12).

### 2026-10-14
- First commit. Cleaned out the Tauri template, wrote the README, named the app Vault with a new logo (#1–#7).

## Next

- [ ] Add screenshots (desktop Dashboard, phone view, Budget) to the README.
- [ ] Record a 60-second demo: pay a bill on the phone, watch the desktop update.
- [ ] Write a half-page case study for the portfolio.

## Known limits

- Synced data is stored unencrypted in Firestore; anyone who controls the Firebase project can read it. Fine for a family, not something to promise a client.
- No iOS build.
