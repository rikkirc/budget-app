# Progress log

What's done, what's next, and what's blocked. Newest entries first.

## Current state (2026-10-19) — v0.3.0

- **Platforms:** the web app (an installable PWA) on iPhone and Android, and on computers through Edge or Chrome, at https://thevault-rc.web.app. It updates itself and opens offline. The Windows desktop (Tauri) and Android apps are retired (#25): their code is kept, but they're no longer built.
- **Sync:** real-time sync between devices through Firebase (free Spark plan), with accounts gated by an invite code. The web app is published by Firebase Hosting on every merge to `main`.
- **Try the demo:** a made-up vault that opens from the welcome screen (or a `?demo` link) with no account, kept in the browser tab only and never saved.
- **Features:** Dashboard, Cash Flow with per-account history, Bi-Monthly Budget, Loan Generator, Search, smart bill reminders, editing for accounts, bills and loans, a welcome screen with Terms & Privacy, stay signed in, and receipts with the date each bill was settled.
- **Docs:** README, User's Manual (Markdown + PDF, also downloadable from the welcome screen), sync setup guide, case study.
- **Tests:** 44 Rust backend tests, run natively and again inside WebAssembly; a check that the browser and desktop builds turn the same changes into identical vaults; 27 frontend tests (search, bill names, loan schedules, sync engine, demo vault); and Firebase security-rule tests on the emulators.

## Log

### 2026-10-19 — #27
- Manual: new "Try the demo first" section and a glossary entry; PDF rebuilt (27 pages), served by the app's download button.

### 2026-10-19 — #26
- **Try the demo:** a button on the welcome screen or a `?demo` link opens a made-up vault dated around today (four accounts, salary and spending, five bills with one TBD, a split loan, three receipts). No account or invite code; it lives in the browser tab only and is never saved or sent to Firebase. A yellow banner says so, with a way back. It runs on the real engine.
- Docs: live and demo links at the top of the README; PROGRESS current state (v0.3.0) and log for #20–#24; case study updated (stack, results, one Rust engine natively and in the browser).
- App tests up from 19 to 27 (loan schedules and the demo, including month ends, Feb 29 and Dec 31). Checked in a real browser at desktop and iPhone sizes, including that nothing is stored and no Firebase database or sign-in requests are made.

### 2026-10-19 — #25
- Retired the Windows and Android apps: the `.exe` and `.apk` no longer build in CI. Their code is kept and can still be built by hand. The last builds are archived as release [v0.3.0](https://github.com/rikkirc/budget-app/releases/tag/v0.3.0).
- The manual covers only the web app, with a section on moving from the old apps (Android users are already synced and just sign in; Windows "Local only" users upload their data once before uninstalling).

### 2026-10-18 (night) — #24
- The manual's opening line matches the welcome screen ("A budgeting app for ordinary paycheck to paycheck people"), and the PDF is rebuilt.
- The website now republishes when only the manual PDF changes, so the welcome screen's download is never out of date.

### 2026-10-18 (night) — #23
- Welcome screen wording: who the app is for, and "Sign in with your email, or create an account with the invite code." The invite code stays required.

### 2026-10-18 (evening) — #22
- v0.3.0. A welcome screen ("The Vault by RCbreezy ©") with how to start, the manual PDF to download, and honest Terms & Privacy to accept.
- Stay signed in, and open a remembered profile automatically. Enter submits sign-in.
- Receipts show the date each bill was settled. A transaction's description is optional. New bills can let the tracker pick the paycheck.
- The header shows the app name with the date and time. The short link thevault-rc.web.app. The manual explains one account, many profiles.

### 2026-10-18 (afternoon) — #21
- The web app became installable (home-screen icon, full screen), opens offline, and offers an update banner when a new version is out.
- Published to Firebase Hosting automatically by GitHub Actions.
- Fixed: opening a vault right after creating an account could fail with "permission denied".

### 2026-10-18 (morning) — #20
- vault-core compiled to WebAssembly, so the browser runs the same Rust rules as the desktop and Android apps. The browser keeps its copy of each vault in IndexedDB.
- CI runs vault-core's tests natively and inside WebAssembly, and checks that both builds turn the same list of changes into identical vaults.

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

### Next up (web app)
- [ ] Replace the README screenshots: they still show the retired Windows and Android apps. Take new ones from the demo (https://thevault-rc.web.app/?demo) on a phone and a computer.
- [ ] Re-record the sync demo video with the web app, or label the current one as the earlier desktop/Android version.

### Earlier (desktop and Android)
- [x] Test the edit buttons on a real PC and phone.
- Skipped: renaming "Demo Loan sinking Fund" in the demo vault and retaking the loan screenshot (minor detail).
- [x] Decide whether the green "Total Liquid Cash" and Completed Loans cards should be toned down too.
- [x] Install the new APK and confirm nothing sits under the navigation bar.
- Skipped: retaking the Bi-Monthly Budget screenshot for the new "— 2nd Half (Due)" name format (minor detail).

## Known limits

- Synced data is stored unencrypted in Firestore; anyone who controls the Firebase project can read it. Fine for a family, not something to promise a client.
- Every synced device must run the same version: an older app can't apply a rename and keeps showing the old name.
