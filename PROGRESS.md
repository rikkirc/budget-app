# Progress log

What's done, what's next, and what's blocked. Newest entries first.

## Current state (2026-10-15) — v0.2.0 + layout fixes (#18)

- **Platforms:** Windows desktop (Tauri) and Android.
- **Sync:** optional real-time sync between devices through Firebase (free Spark plan), with family accounts gated by an invite code. Without sync, each vault stays on the device.
- **Features:** Dashboard, Cash Flow with per-account history, Bi-Monthly Budget, Loan Generator, Search, smart bill reminders.
- **Docs:** README, User's Manual (Markdown + PDF), sync and mobile setup guide.
- **Tests:** frontend search and sync engine, Rust backend tests (run on Linux CI), Firebase security-rule tests on the emulators.

## Log

### 2026-10-15 (evening)
- Fixed all five layout bugs from the list and updated the User's Manual to the new name format (#18). CI green: tests, Android APK, Windows installer.

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

- [x] Add screenshots (desktop Dashboard, phone Dashboard, Loan Generator) to the README.
- [x] Add a Bi-Monthly Budget screenshot.
- [x] Retake the Dashboard pair (desktop + phone) with the updated demo data.
- [x] Record a 60-second demo: pay a bill on the phone, watch the desktop update (linked in the README).
- [x] Write a half-page case study for the portfolio (docs/CASE_STUDY.md).

### Planned features
- [ ] Edit and rename accounts in Cash Flow (e.g. rename "Demo Loan sinking Fund"). The new name should also show in loan schedules and Receipts.

### Bugs fixed (#18)
- [x] Phone: bill rows on Bi-Monthly Budget wrap their buttons below the name, so nothing runs off the screen. Receipts and Recycle Bin too.
- [x] Phone: the page keeps at least 64px clear at the bottom, since Android's WebView reports the safe-area inset as 0. **Still needs checking on a real phone with the new APK.**
- [x] Phone: Budget Summary and Active Loans tables fit the screen; Amount and Remaining stay visible.
- [x] Split-loan names display as "Car Loan — 2nd Half (Due)" instead of nesting brackets (display only; stored names unchanged). `src/billNames.js`, with tests.
- [x] Desktop: a single loan card fills the full width.

### Next up
- [ ] Install the new APK and confirm nothing sits under the navigation bar.
- [ ] Retake the Bi-Monthly Budget screenshot so the README shows the new "— 2nd Half (Due)" name format.

## Known limits

- Synced data is stored unencrypted in Firestore; anyone who controls the Firebase project can read it. Fine for a family, not something to promise a client.
- No iOS build.
