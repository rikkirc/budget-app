# Progress log

What's done, what's next, and what's blocked. Newest entries first.

## Current state (2026-10-15) — v0.2.0 + layout fixes (#18) + edit buttons (#19)

- **Platforms:** Windows desktop (Tauri) and Android.
- **Sync:** optional real-time sync between devices through Firebase (free Spark plan), with family accounts gated by an invite code. Without sync, each vault stays on the device.
- **Features:** Dashboard, Cash Flow with per-account history, Bi-Monthly Budget, Loan Generator, Search, smart bill reminders, and editing for accounts, bills and loans.
- **Docs:** README, User's Manual (Markdown + PDF), sync and mobile setup guide.
- **Tests:** 39 Rust backend tests (run on Linux CI), 16 frontend tests (search, bill names, sync engine), and Firebase security-rule tests on the emulators.

## Log

### 2026-10-15 (late evening) — #19
- Edit buttons: rename accounts (Cash Flow), edit bills (name, amount, due date; split bills edit as one), edit loans (name and date acquired). Renames carry through to bills, receipts and loan schedules.
- Fixed: deleting a loan also deleted the payments of any loan whose name started the same way ("Car" wiped "Car Loan 2"). Active loans can no longer share a name.
- Fixed: bright near-white and mint boxes on desktop toned down to grey.
- 8 new backend tests for the edits. Manual and PDF updated.

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
- [x] Edit and rename accounts in Cash Flow. The new name also shows in loan schedules and Receipts (#19).

### Bugs fixed (#18)
- [x] Phone: bill rows on Bi-Monthly Budget wrap their buttons below the name, so nothing runs off the screen. Receipts and Recycle Bin too.
- [x] Phone: the page keeps at least 64px clear at the bottom, since Android's WebView reports the safe-area inset as 0. Checked on a real phone.
- [x] Phone: Budget Summary and Active Loans tables fit the screen; Amount and Remaining stay visible.
- [x] Split-loan names display as "Car Loan — 2nd Half (Due)" instead of nesting brackets (display only; stored names unchanged). `src/billNames.js`, with tests.
- [x] Desktop: a single loan card fills the full width.

### Bugs fixed (#19)
- [x] Desktop: bright near-white and mint boxes (account cards, Add New Account, bill rows, Log a New Loan) now use the calmer grey of the Log Transaction box.
- [x] Deleting a loan removed other loans' payments when one name started with the other. Now it removes only its own payments.

### Next up
- [x] Test the edit buttons on a real PC and phone.
- Skipped: renaming "Demo Loan sinking Fund" in the demo vault and retaking the loan screenshot (minor detail).
- [x] Decide whether the green "Total Liquid Cash" and Completed Loans cards should be toned down too.
- [x] Install the new APK and confirm nothing sits under the navigation bar.
- Skipped: retaking the Bi-Monthly Budget screenshot for the new "— 2nd Half (Due)" name format (minor detail).

## Known limits

- Synced data is stored unencrypted in Firestore; anyone who controls the Firebase project can read it. Fine for a family, not something to promise a client.
- No iOS build.
- Every synced device must run the same version: an older app can't apply a rename and keeps showing the old name.
