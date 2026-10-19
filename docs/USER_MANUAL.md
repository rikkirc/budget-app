# The Vault: User's Manual

*by RCbreezy ©*

**Version 0.3.0** · The web app for iPhone, Android and computers · Amounts in Philippine pesos (₱) · Paydays: 15th and end of month

**The Vault** ("Vault" for short) is a budgeting app for ordinary paycheck to paycheck people. It plans every bill and loan installment against the paycheck that will cover it and tracks the money in each of your accounts. Use it on your phone, on your computer, or both: sign in to your free account and every change you make on one device appears on the other within seconds.

This manual covers the app from installation to everyday use. A print-ready version is in [Vault-User-Manual.pdf](Vault-User-Manual.pdf). In the app, download it any time from **📄 Manual** in the top bar, or from the welcome, sign-in and unlock screens. It's kept on your device, so it downloads even with no signal.

## Contents

**Part 1 — Get set up**

1. [Installing Vault](#1-installing-vault)
2. [Unlocking your vault](#2-unlocking-your-vault)
3. [Your account & sync](#3-your-account--sync)
4. [Find your way around](#4-find-your-way-around)

**Part 2 — Using Vault**

5. [Key ideas](#5-key-ideas)
6. [Dashboard](#6-dashboard)
7. [Cash Flow](#7-cash-flow)
8. [Bi-Monthly Budget](#8-bi-monthly-budget)
9. [Loan Generator](#9-loan-generator)
10. [Search](#10-search)
11. [Step-by-step guides](#11-step-by-step-guides)

**Part 3 — Reference**

12. [Your data & backups](#12-your-data--backups)
13. [Troubleshooting](#13-troubleshooting)
14. [Glossary](#14-glossary)

[Appendix: Setting up Firebase](#appendix-setting-up-firebase) (only for the person who runs Vault's sync)

---

## 1. Installing Vault

The Vault is a web app: it runs in your browser at **https://thevault-rc.web.app**. There's nothing to download from a file link. You install it from the browser so it gets its own icon, opens full screen like any other app, and updates itself.

You also need the **invite code** from the person who shared Vault with you, to create your account ([section 3](#3-your-account--sync)).

### On an iPhone

1. Open **https://thevault-rc.web.app** in **Safari**.
2. Tap the **Share** button (the square with an arrow pointing up) at the bottom of the screen.
3. Scroll down and tap **Add to Home Screen**, then tap **Add**.
4. Open **Vault** from your home screen. Always open it from this icon: an iPhone may clear the data of websites that are only visited in Safari.

### On an Android phone

1. Open **https://thevault-rc.web.app** in **Chrome**.
2. Tap **⋮** (top right), then **Install app**. On some phones it says **Add to Home screen** instead.
3. Tap **Install**, then open **Vault** from your home screen or app list.

### On a computer

1. Open **https://thevault-rc.web.app** in **Edge** or **Chrome**.
2. Click **Install The Vault** (the ⊕ icon at the right of the address bar), then **Install**.
3. From then on, open **The Vault** from the Start Menu (or the Dock on a Mac). It opens in its own window.

The first screen asks you to sign in or create your account ([section 3](#3-your-account--sync)). After that, Vault opens even with no signal, showing the numbers it last synced.

> [!NOTE]
> Older copies may still use the original link, `https://the-vault-1399a.web.app`. It shows the same app and keeps working, so there's no need to reinstall.

### Try the demo first

Want to look around before creating an account? On the welcome screen, tap **👀 Try the demo (no account needed)**, or open **https://thevault-rc.web.app/?demo**. You don't need an account, an invite code or the terms.

The demo opens a made-up vault, dated around today: four accounts (Payroll Account, GCash, Cash Wallet and Sinking Fund Savings), a salary and some spending, five bills, a split loan paid twice a month, and three receipts. **Electricity** is still TBD and due soon, so the reminder to enter its amount appears as soon as the demo opens.

Everything works as in a real vault: pay bills, log transactions, add loans, undo. But **nothing is saved**: the demo lives only in that browser tab, and nothing is sent to your account or to Firebase. A yellow banner reminds you: **"Demo: nothing here is saved. Create an account to use your own vault."**

To leave, tap **← Back to the welcome screen** (or **🚪 Exit Demo** in the top bar). The demo is thrown away, and the next one starts fresh. To start your own vault, tick the terms and tap **Proceed →** to create your account ([section 3](#3-your-account--sync)).

### Updating

The web app updates itself. When a bar says **A new version of Vault is ready**, tap **Reload**. Tap **Later** to finish what you're doing first.

### Moving from the old Windows or Android app

Earlier versions of The Vault came as a Windows installer (`.exe`) and an Android app (`.apk`). These are retired: they get no more updates, and the web app replaces both. Move to the web app like this, **before** you uninstall anything.

**From the Android app (`.apk`):** your data is already in your account, because the Android app always syncs.

1. Install the web app (above).
2. Sign in with the **same email and password**, and unlock with the **same profile name**. Your data appears.
3. Uninstall the old app: press and hold its icon, then tap **Uninstall**.

**From the Windows app (`.exe`):** first look at the button in the old app's top bar.

- **🟢 Live sync:** your data is already in your account. Install the web app, sign in with the same email and password, and unlock with the same profile name.
- **💻 Local only:** your data is **only on that computer** and must be uploaded first:
  1. In the old app, unlock your vault as usual.
  2. Click **💻 Local only**, then **Create an account** (or **I already have an account** if you have one). Sign in.
  3. When it asks **"Upload this computer's … vault to your account so your phone can use it?"**, click **OK**. Repeat for each profile you use: log out, unlock the next one, and it asks again.
  4. Now install the web app, sign in with the same email and password, and unlock with the same profile name.

Once your data shows in the web app, uninstall the old app: **Start → Settings → Apps → Installed apps → Vault → ⋯ → Uninstall**. Uninstalling never deletes the `Documents\FinancialCommandCenter` folder; keep it as an extra backup.

---

## 2. Unlocking your vault

### The welcome screen

Whenever The Vault opens to the sign-in or unlock screen, it first shows a short welcome: what the app is for, how to get started, a **📄 Download the User's Manual (PDF)** button, the **Terms & Privacy Notice**, and **👀 Try the demo (no account needed)** ([Try the demo first](#try-the-demo-first)). The first time, tick **I have read and agree to the Terms & Privacy Notice**, then tap **Proceed →**. After that, the welcome only needs **Proceed →**, and **Read it again** shows the terms.

### Accounts and profiles

Two different things have names in The Vault:

| | What it is | How many |
|---|---|---|
| **Account** | You, signing in with your **email and password** ([section 3](#3-your-account--sync)). It keeps your data private and in step across your devices. | One per person |
| **Profile** | A **separate budget** inside your account, with its own money accounts, bills and loans. You open one by typing its **profile name**. | As many as you like |

**One account can hold many profiles.** For example, Juan signs in once as `juan@example.com` and keeps three profiles: `Personal`, `Household` and `Sari-sari Store`. Each profile's numbers are completely separate, and every one of them syncs to all of Juan's devices. Nobody else can open Juan's profiles, even if they type the same name: profiles always belong to the account that's signed in.

Most people need just one profile, for example `Juan Dela Cruz`.

### Open a profile

After you sign in ([section 3](#3-your-account--sync)), Vault asks for your profile name.

1. Type your profile name.
2. Leave **Open this profile automatically next time** ticked if this is your own device.
3. Tap **🔐 Unlock Vault** or press **Enter**. A new name creates a new, empty vault; a name you've used before opens that vault.
4. If any recurring bills due in the next 10 days still have no amount, a reminder lists them so you can enter the amounts from your billing statements.

> [!CAUTION]
> **Use exactly the same profile name on every device.** Capitals don't matter and spaces become underscores, so `Juan Dela Cruz` and `juan dela cruz` open the same vault. But a typo like `Juan Dela Curz` quietly creates a brand-new empty vault. If your data seems to be missing, log out and check the name.

**Opening automatically.** With **Open this profile automatically next time** ticked (and **Keep me signed in**), The Vault skips the welcome, sign-in and unlock screens from then on and opens straight to your profile. Pressing **🔒 Log Out** turns this off, so the next person on that device sees the unlock screen; tick the box again when you unlock to turn it back on.

> [!CAUTION]
> **Not a password.** The profile name only picks which vault to open. Your vaults are protected by your account password; also protect your phone with its screen lock and your computer with its sign-in.

Profile names may only contain letters, numbers, spaces, `-` and `_`.

---

## 3. Your account & sync

Your account keeps your phone and computer on the same data. It's free, and each person's account is private: nobody else can see your vaults.

### Create your account (once)

You need the **invite code** from the person who shared Vault with you.

1. Open Vault. After the welcome screen, the **Sign in** screen appears.
2. Tap **Create an account**. Enter your email, a password (at least 8 characters) and the invite code (in the **Family invite code** box), then tap **Create account** (or press **Enter**).
3. Leave **Keep me signed in on this device** ticked on your own device; untick it on a shared or borrowed one, and you'll be signed out when the app closes.
4. Unlock with your profile name ([section 2](#2-unlocking-your-vault)).

### Sign in on your other devices

1. Open Vault and choose **I already have an account**.
2. Enter the same email and password and tap **Sign in** (or press **Enter**).
3. Unlock with the **same profile name**. Your data appears.

### Live sync

The button in the top bar shows how sync is doing:

| Shows | Meaning |
|---|---|
| **🟢 Live sync** | Connected. Your changes reach your other devices within a second or two, and theirs reach you. |
| **🟠 Reconnecting…** | The connection dropped. Vault reconnects by itself as soon as it can. |

When a change arrives from another device, the screen refreshes by itself. If you're halfway through typing, Vault shows **🔄 Your vault was updated on another device. Tap here to refresh** instead, so your half-filled form isn't wiped. Save what you're doing first, then tap it.

### Using Vault offline

Every device keeps its own full copy of your vault, so with no signal you can still open Vault and see all your balances, bills and loans.

**Changes need a connection.** If you try to save one offline, Vault shows **"Couldn't reach the sync service, so this change may not have been saved"**. Once you're back online, press **🔄 Refresh** to check whether it went through *before* trying again. Vault never retries by itself, so a bill can never be paid twice.

### Passwords and lost phones

- **Forgot your password?** On the sign-in screen, tap **Forgot password?**, enter your email, and tap **Email me a reset link**. Check your inbox (and spam folder).
- **Change your password:** tap **🟢 Live sync → Change password (lost a phone? this signs it out)**, and enter your current and new password.
- **Lost a phone?** Change your password from another device. Every other device is signed out (within about an hour) and needs the new password. Your data stays safe in your account.

### Signing out

Tap **🟢 Live sync → Sign out**. The vault locks; sign in again any time and your data is still there.

---

## 4. Find your way around

The top of every screen shows **The Vault by RCbreezy ©** with today's date and the time. Below it, a bar holds the five sections of the app, your vault name, and a few controls.

| Button | What it does |
|---|---|
| **🏠 Dashboard** | Your totals and what's coming up next. See [section 6](#6-dashboard). |
| **💰 Cash Flow** | Your accounts, income, expenses, transfers, and transaction history. See [section 7](#7-cash-flow). |
| **📅 Bi-Monthly Budget** | Every unpaid bill, grouped by the paycheck that pays it. See [section 8](#8-bi-monthly-budget). |
| **📉 Loan Generator** | Add loans and generate their payment schedules. See [section 9](#9-loan-generator). |
| **🔍 Search** | Find any bill, loan, or amount. See [section 10](#10-search). |
| **🟢 Live sync** | Your account and sync status. See [section 3](#3-your-account--sync). |
| **📄 Manual** | Downloads this User's Manual (PDF), on any screen. It works offline too. |
| **🔄 Refresh** | Reloads the current screen with the latest numbers. |
| **🔒 Log Out** | Locks the vault and returns to the profile screen. It also stops this profile from opening by itself next time. |

### On the phone

Every screen and button works the same as on a computer. A few things are arranged differently to fit:

- **Swipe the row of tabs** left and right to reach all five sections.
- Your vault name, sync status, **Refresh** and **Log Out** sit in a compact row under the tabs.
- **Wide tables** (like loan schedules and the budget summary) scroll sideways inside their card: swipe across the table.
- Questions and confirmations appear as pop-ups; tap **OK** or **Cancel**.

---

## 5. Key ideas

Vault is built around a few ideas. Knowing them makes every screen easier to understand.

### Paychecks and allocations

Vault assumes you're paid on the **15th** and at the **end of the month**. Every unpaid bill is **allocated** to one paycheck: the paycheck whose money will pay that bill. On the Bi-Monthly Budget screen, each paycheck gets its own tab listing its bills and their total.

### The 10-day safety gap

When Vault picks a paycheck for you, it chooses the **latest payday that lands at least 11 days before the due date**. That leaves a 10-day cushion between getting paid and the bill being due. A bill due on November 20 is allocated to the October 31 paycheck, not November 15, because November 15 is only 5 days early.

You can always override the choice: when previewing a loan's schedule, or later with the **📅 Move** button on any bill.

### Bi-monthly halves: Sinking Fund vs True Bi-Monthly

A monthly payment can be spread over both paychecks as a **1st Half** and a **2nd Half (Due)**. The 2nd Half falls on the real due date; the 1st Half is 15 days earlier. Vault supports two kinds:

| | Sinking Fund | True Bi-Monthly |
|---|---|---|
| **Real payments** | One per month, on the 2nd Half's date | Two per month, one per half |
| **1st Half** | Saved into a savings account ("fund") so the money is set aside | Paid to the lender like any bill |
| **2nd Half** | Pays the *full* monthly amount, using the saved 1st Half plus this half | Pays only its own half |
| **Use it for** | Lenders that charge once a month, when you want to save half from each paycheck | Lenders that actually collect twice a month |

Here is one month of a Sinking Fund loan with a ₱2,000.00 monthly payment due on November 20:

| When | What you do | Money |
|---|---|---|
| **Oct 15 pay** | Handle **1st Half** → **📥 Save to Fund**. Moves ₱1,000 from your main account into your savings account. | main −1,000.00 · savings +1,000.00 |
| **Oct 31 pay** | Handle **2nd Half (Due)** → **🏦 Pay Bank**, click OK to auto-transfer. Vault moves the saved ₱1,000 back, then pays the full ₱2,000 to the lender. | savings −1,000.00 · main −1,000.00 |
| **Nov 20** | Due date. Already paid; the loan's remaining balance went down by ₱2,000. | loan −2,000.00 |

### Recurring bills and TBD amounts

A **recurring** bill (marked 🔁) comes back every month on the same day. When you pay it, next month's copy is created automatically with the amount set to **TBD**, because utility bills change every month. Enter the amount when the statement arrives; a bill can't be handled until it has one.

### Receipts, Recycle Bins, and Undo

Very little in Vault is permanent. Paid bills become **receipts** you can undo; deleted bills, loans, and accounts go to a **Recycle Bin** you can restore from; and ordinary transactions can be undone from the history. To prevent accidents, only the 3 most recent items show an **↩️ Undo** button right away; older ones need **Show undo** first.

---

## 6. Dashboard

The Dashboard is a read-only overview. Nothing here can be changed; use it to see where you stand.

| Panel | What it shows |
|---|---|
| **Total Liquid Cash** | The combined balance of all your accounts. |
| **Total Remaining Debt** | What's still owed across all active loans. |
| **Next 5 Upcoming Bills** | The five unpaid bills with the earliest due dates, with their paycheck allocations and a **total**. Bills still TBD show "TBD" and are counted separately, e.g. "+ 1 TBD bill not yet included". |
| **Bi-Monthly Budget Summary** | Every unpaid bill in a single table, sorted by due date. |
| **Active Loans Summary** | Each active loan's term, frequency, payment, and remaining balance. |

> [!NOTE]
> **About halves.** Upcoming bills list each half separately at its listed amount, which is the money to set aside from that paycheck. A Sinking Fund 2nd Half actually pays double its listed amount, but half of that comes from the fund you already saved.

---

## 7. Cash Flow

Cash Flow is where your accounts live: bank accounts, e-wallets, cash, and savings funds. Every peso Vault moves goes through one of them.

### Add an account

1. Under **Add New Account**, type a name such as `GCash`, `BPI`, or `Emergency Fund`.
2. Click **+ Add**. The account starts at ₱0.00. Each name can only be used once.
3. Log an **Income** into it to set its starting balance.

> [!TIP]
> Make a separate account for each savings fund you use with Sinking Fund loans. It keeps set-aside money from being spent by accident.

### Log a transaction

Use the **Log Transaction** bar. Pick the type, the account and the amount; the **description is optional**.

| Type | Effect |
|---|---|
| **Income (+)** | Adds money to the account. Use it for salary, freelance pay, refunds, gifts. |
| **Expense (-)** | Takes money out. Use it for spending that isn't a bill, like groceries or load. |
| **Transfer (↔)** | Moves money from one account to another. Pick the receiving account in the second dropdown. Vault refuses the transfer if the source account doesn't have enough. |

The amount box adds thousands separators as you type, so `12500` becomes `12,500`. A short description (e.g. `Salary`, `Groceries`) is optional, but it makes the entry easier to find later in Search.

### Transaction history

History shows **one account at a time**, starting with your first account. To switch, click an **account card** (the selected one is outlined in green) or one of the buttons above the list. Click **📚 All Accounts** for the complete history across every account.

- Newest entries are at the top, with the date and time each was recorded.
- Transfers show both accounts, e.g. **🔁 BPI ➡ GCash**.
- Click **Load More Transactions** to see entries beyond the first 50.

### Undo a transaction

Income, expenses, and transfers can be undone from the history. Undoing a transfer reverses both accounts. Undone entries stay in the list, faded and marked **(Undone)**, so you have a full record.

> [!CAUTION]
> **Bill payments are undone elsewhere.** Entries created by paying or funding bills (starting with `Paid:`, `Funded to Savings:`, and their refunds) have no Undo button here. Undo them from **Receipts** on the Bi-Monthly Budget screen, which also puts the bill back.

### Rename an account

Click the ✏️ on an account card, type the new name, and click ✓ (or press Enter). Its balance and history stay as they are. Bills saved to the account follow the new name too: a 1st Half marked *Handled in* that account shows the new name in the Budget, on its receipt, and in the loan's payment schedule. Two accounts can't share a name.

### Delete an account

Click the 🗑️ on an account card. Vault only deletes accounts that are empty; transfer out any remaining balance first.

---

## 8. Bi-Monthly Budget

This is the heart of Vault: every unpaid bill and loan installment, organized by the paycheck that will pay it.

### Log a new bill

| Field | What to enter |
|---|---|
| **Bill Name** | e.g. `Internet`, `Water`, `Rent`. |
| **Amount (₱)** | The bill amount. Optional for recurring bills: leave it blank and it shows as TBD. |
| **Actual Due Date** | The date the biller expects payment. |
| **Paycheck Allocation** | **🤖 Let the tracker decide** (the default) picks the paycheck the same way it does for loans: the latest payday (15th or end of month) at least 11 days before the due date. A line underneath shows which one, e.g. **→ October 31, 2026 paycheck**. Choose **✋ I'll pick the paycheck** to enter a payday yourself. |
| **🔁 Recurring Bill** | Tick for bills that repeat on the same day every month. |
| **✂️ Split into Bi-Monthly Halves** | Appears for recurring bills. Turns the bill into a Sinking Fund pair. See [Split a recurring bill](#split-a-recurring-bill). |

Click **Save Bill**. Vault confirms which paycheck the bill is on, and rejects dates that don't exist on the calendar, like February 30.

### Paycheck tabs

Each tab, such as **💼 October 31, 2026 BILLS**, holds the bills allocated to that paycheck, nearest due date first. **Total Tab Due** at the top is how much that paycheck needs to cover.

### The controls on each bill

| Control | What it does |
|---|---|
| **⚠️ TBD — Click to Enter Amount** | Appears when a recurring bill has no amount yet. Type the amount and click ✓. |
| **✏️ Edit** | Changes the bill's name, amount, and due date. See [Edit a bill](#edit-a-bill). Loan installments have no Edit button: edit the loan in **Active Loans** instead. |
| **✂️ Split** | Splits a recurring bill into a 1st Half and 2nd Half (Due). |
| **🗑️ Delete** | Moves the bill to the Recycle Bin at the bottom of the screen. |
| **📅 Move** | Reassigns the bill to a different paycheck. |
| **Handle Bill** | Pays the bill or saves it to a fund. Greyed out while the amount is TBD. |

### Edit a bill

Click **✏️ Edit** on any bill that isn't a loan installment, change what you need, and click **Save Changes**.

- **Bill Name, Amount, and Due Date** can all change. Recurring bills can leave the amount blank to make it TBD.
- **A new due date moves the bill** to the latest paycheck at least 11 days before it (the usual safety gap). Use **📅 Move** afterwards to pick a different paycheck.
- **Split bills** are edited as one bill: type the base name (e.g. `Rent`), the **full** amount, and the real due date (the 2nd Half's). Both halves of that month change together, and next month's pair uses the new name.
- If a split bill's 1st Half is already saved to a fund, its due date can't change until that month is paid; the name and amount still can.

### Handle a bill

Click **Handle Bill** and choose one of two actions:

- **🏦 Pay Bank (Lowers Debt)**: pays the bill from the account you pick. For a loan installment, the loan's remaining balance goes down by the same amount.
- **📥 Save to Fund (Moves Cash)**: moves the amount from one account into a savings account, setting it aside without paying anyone yet. A Sinking Fund **1st Half** can only be saved to a fund.

Choose the account under **Take money from...** (and, for a fund, **Move to Savings...**), then click **Confirm Payment** or **Confirm Transfer**. If the account doesn't hold enough, Vault stops and tells you how much it needs.

When you pay a Sinking Fund **2nd Half (Due)** whose 1st Half was saved, Vault asks whether to auto-transfer the saved money first:

- **OK** moves the saved 1st Half from the fund into the account you're paying from, then pays the full amount.
- **Cancel** pays the full amount from your selected account and leaves the fund untouched.

True Bi-Monthly loans never show this prompt; each half pays only itself.

### Split a recurring bill

Splitting lets you save for a big monthly bill, like rent or tuition, from both paychecks. It works exactly like a Sinking Fund loan:

- **Name — 1st Half** is due 15 days before the real due date and is saved to a fund.
- **Name — 2nd Half (Due)** is due on the real due date and pays the full bill.

Split a new bill by ticking **🔁 Recurring** and then **✂️ Split into Bi-Monthly Halves**. The form then asks for the **Full Amount** and the **2nd Half Paycheck**; Vault picks the 1st Half's paycheck with the 10-day rule. To split an existing recurring bill, click its **✂️ Split** button.

For split bills, always type the **full bill amount**, both when entering a TBD amount and when editing. Vault puts half on each half, and shows "half of ₱X" under each. Every future month is split too: paying the 2nd Half creates next month's pair as TBD.

### Move a bill to another paycheck

Click **📅 Move**, then either pick an existing paycheck from the list or type any date, and click **Confirm Move**. Vault warns you if the new paycheck lands after the bill's due date. Only that occurrence moves; next month's copy of a recurring bill uses the normal 10-day rule again.

### Receipts

Paid and funded bills appear under **Receipts (Paid & Settled Bills)**, newest first. Each shows its original due date and **✅ Settled** (or **✅ Saved to fund**) with the date and time it was handled. Receipts from before this was recorded show only the due date. Undoing a receipt:

- **A payment** refunds the full amount to the account it came from, puts the bill back, restores a loan's balance (reopening it if that payment had completed it), and removes next month's TBD copy if it was created automatically.
- **A Save to Fund** moves the money back from the savings account and puts the bill back. If some of the fund was already spent, Vault refuses and tells you how much is missing.

### Recycle Bin

Deleted bills wait in **Recycle Bin (Deleted Bills)**. Click **↩️ Restore** to put one back in the Budget.

---

## 9. Loan Generator

Enter a loan once and Vault creates every installment as a bill in the Budget, already allocated to paychecks.

### Choose an entry mode

- **🆕 New Loan (Auto-Calculate)**: for a fresh loan. Enter the principal, interest rate, and duration, and Vault calculates the total and the monthly payment.
- **📋 Existing Loan (Manual Entry)**: for a loan you're already paying. Type the remaining balance and the monthly payment exactly as your lender shows them.

### How interest is calculated

| Interest type | Total interest | ₱10,000 at 3% for 6 months |
|---|---|---:|
| **Flat (one-time)** | Principal × rate | ₱300.00 |
| **Per Month** | Principal × rate × months | ₱1,800.00 |
| **Per Year** | Principal × rate × months ÷ 12 | ₱150.00 |

**Total Loan Incurred** = principal + interest, and the monthly payment = total ÷ number of months. A green panel previews Total Interest, Total Loan Incurred, Per Month, and Per Bi-Monthly Half as you type.

### Fill in the schedule

| Field | Meaning |
|---|---|
| **Duration** | How many months or years the loan runs. |
| **Preferred Schedule** | **Monthly** (one installment per month) or **Bi-Monthly** (two halves per month). |
| **Bi-Monthly Type** | **Sinking Fund** or **True Bi-Monthly**. See [Key ideas](#5-key-ideas). |
| **Set Your Paycheck Routine** | Your two paydays as days of the month. Defaults are 15 and 30; use 30 or 31 for end of month. |
| **Next Due Date** | The next date the lender expects payment. For bi-monthly loans this is the 2nd Half's date. |
| **Date Acquired** | Optional. When you took out the loan, for your records. |

### Preview, adjust, and save

1. Click **+ Preview Draft Schedule**. Vault lists every installment with its due date, amount, and suggested paycheck.
2. Change any **Allocate to Paycheck** date you want to adjust.
3. Click **✅ Confirm & Save Loan**. Every installment appears in the Bi-Monthly Budget, named like **Loan — 1st Half**, **Loan — 2nd Half (Due)**, or **Loan — Due Date**.

### Active, completed, and deleted loans

- **Active Loans** show each loan's principal and remaining balance. Click **View** for the full payment schedule; each installment is marked **✅ Paid**, **📥 Saved** (a 1st Half sitting in a fund), or *unpaid*.
- When a payment brings the remaining balance to zero, the loan moves to **✅ Completed Loans** automatically and any leftover installments are cleared.
- **✏️ Edit** changes a loan's name and its date acquired. Its installments in the Budget and on its receipts are renamed to match. Amounts and due dates can't be edited, because they set the loan's whole payment schedule; to change them, delete the loan and log it again.
- **🗑️ Delete** moves a loan to the **Loan Recycle Bin** and removes its unpaid installments from the Budget. **↩️ Undo & Restore** brings it back and rebuilds its schedule using the paycheck routine currently entered on this screen.

> [!NOTE]
> **Loan names are unique.** Two active loans can't share a name, so their installments are never mixed up. Deleting a loan removes only its own installments.

---

## 10. Search

Pick what you're looking for first, then type. Results appear as you type, grouped by where they live.

| Category | Searches |
|---|---|
| **🔎 Everything** | Bill, receipt, and loan names; transaction descriptions and account names; and amounts, when you type a number. |
| **📅 Bills** | Unpaid bills and paid receipts, by name. |
| **📉 Loans** | Active and completed loans, by name. |
| **💵 Amount** | Bill amounts (including a half's full installment), loan figures (remaining, payment, per half, principal, total), and transaction amounts. |

- **Whole numbers match any centavos.** `1500` finds everything from ₱1,500.00 to ₱1,500.99; `1500.50` finds exactly ₱1,500.50. Peso signs and commas are fine.
- **Word order doesn't matter.** `tuition loan` finds "Loan – Tuition". Capitals don't matter either.
- For amount matches, the result says *which* amount matched, e.g. "matched remaining" on a loan.
- Each group has an **Open ›** button that takes you to that screen.

Deleted items in the Recycle Bins aren't searched.

---

## 11. Step-by-step guides

### Set up Vault for the first time

1. Unlock a vault with your profile name.
2. In **Cash Flow**, add each account you use and log an Income for its current balance (description: `Opening balance`).
3. Add a savings account to hold Sinking Fund money, e.g. `Loan Fund`.
4. In **Loan Generator**, add each loan. Use **Existing Loan (Manual Entry)** for loans you're already paying.
5. In **Bi-Monthly Budget**, add your bills, ticking **Recurring** for monthly ones.
6. Open the **Dashboard** and check that Total Liquid Cash matches your real balances.

### Payday routine

1. In **Cash Flow**, log your salary as Income.
2. In **Bi-Monthly Budget**, open this paycheck's tab and compare **Total Tab Due** with your pay.
3. Enter any TBD amounts from your billing statements.
4. Handle each bill: **Pay Bank** for bills you're paying now, **Save to Fund** for Sinking Fund 1st Halves.
5. If the tab is short, use **📅 Move** to shift a bill to the next paycheck, as long as it's still before the due date.

### Pay a utility bill whose amount changes

1. Add it once as a **Recurring** bill, leaving the amount blank.
2. When the statement arrives, click **⚠️ TBD — Click to Enter Amount** and type it.
3. Handle the bill with **Pay Bank**. Next month's copy appears as TBD, ready for the next statement.

### Fix a mistake

- **Wrong income, expense, or transfer:** undo it from the history in Cash Flow, then log it correctly.
- **Paid the wrong bill or from the wrong account:** undo the receipt in the Budget, then handle the bill again.
- **Typo in a name, or the wrong amount or due date:** use **✏️ Edit** on the account, bill, or loan.
- **Deleted something by accident:** restore it from the matching Recycle Bin.

### Start using Vault on a new phone or computer

1. Install the web app ([section 1](#1-installing-vault)).
2. Open it, choose **I already have an account**, and sign in with your email and password.
3. Unlock with the **same profile name** you use on your other devices. Your accounts, bills and loans download onto the new device.

### Everyday use on two devices

- Use whichever device is in your hand: changes made on one appear on the other within a second or two.
- If a **"Your vault was updated on another device"** banner appears while you're typing, finish and save what you're doing, then tap the banner.
- Before paying a bill on the go, glance at the top bar: **🟢 Live sync** means the payment will reach your other devices straight away.

---

## 12. Your data & backups

Your vaults are stored in your account (on Google's Firebase service), and each device you sign in on keeps a full copy in its browser's own storage, kept in step through your account.

- **Lost or broken phone?** Nothing is lost: install the web app on the new one and sign in.
- **Browser data cleared?** If a device's website data is ever cleared (for example when you clear Safari's or Chrome's website data), just sign in again: everything comes back from your account.
- **Old Windows app data:** if you used the retired Windows app, its files are still in `Documents\FinancialCommandCenter` (one `.db` file per profile). They're only a backup now; move their data into your account as described in [Moving from the old Windows or Android app](#moving-from-the-old-windows-or-android-app).

### Privacy

- Your data travels encrypted to the Firebase project that runs Vault. Each person can only ever see their own vaults, and nothing is shared, sold or used for anything else.
- The stored data is **not end-to-end encrypted**: the administrator of that Firebase project (RCbreezy) has technical access to it, as the owner of any app's database does, but does not look at it. This is stated in the **Terms & Privacy Notice** you agree to on the welcome screen.
- Don't store passwords, PINs, card numbers or full account numbers in The Vault.
- Passwords are handled entirely by Firebase; Vault never stores them.
- You can ask RCbreezy to delete your account and data at any time.

---

## 13. Troubleshooting

### Using Vault

**My vault is empty after logging in**
You most likely typed a different profile name, which created a new empty vault. Click **🔒 Log Out** and unlock with the exact name you used before. Also check you're signed in to the right account (tap **🟢 Live sync** to see its email).

**A balance is off by ₱0.01**
Versions before 0.2.0 could create or lose a centavo when moving Sinking Fund halves of odd-centavo installments (for example ₱412.35 ÷ 2 = ₱206.175). Version 0.2.0 fixes this for all new transactions.
Older differences can't be corrected automatically, because Vault can't know which side was right. Compare each account with your bank or e-wallet and log a small Income or Expense to match, with a description like `Rounding correction`.

**"Insufficient funds!" when paying or transferring**
The account you picked holds less than the payment. The message shows both amounts. Transfer money in from another account, or pick a different account. For a Sinking Fund 2nd Half, remember the payment is the full monthly amount.

**The Handle Bill button is greyed out**
The bill's amount is still TBD. Enter the amount first.

**I can't delete an account**
Accounts must be empty before they can be deleted. Transfer the remaining balance to another account first.

**A transaction has no Undo button**
Only the 3 most recent undoable entries show Undo; click **Show undo** on older ones. Bill payments and fund transfers are undone from Receipts in the Bi-Monthly Budget instead.

**Undoing a Save to Fund says "Cannot undo!"**
Some of the money has already left the savings account, often because it was auto-transferred to pay the 2nd Half. Undo that 2nd Half payment first, or move money back into the fund.

**Numbers on a screen look out of date**
Click **🔄 Refresh** in the top bar.

**What are the "advanced tools"?**
Hidden maintenance buttons for data from older versions of the app. **Fix Stray Rounding** (Cash Flow) rounds every balance to whole centavos. **Repair Loan Links** (Loan Generator) reconnects old installments to their loans. Use them only once, after moving data from an old version.

### Phone, accounts & sync

**"That family invite code isn't right"**
Check the code with the person who shared Vault with you; it must match exactly, including dashes.

**"Wrong email or password"**
Check for typos, or tap **Forgot password?** to get a reset email. After several wrong tries, Vault asks you to wait a few minutes.

**The web app shows an empty vault after moving from the old Windows app**
Either the profile name is different (log out and check the spelling), or the computer's data was never uploaded. In the old Windows app, check the top bar: if it shows **💻 Local only**, follow [Moving from the old Windows or Android app](#moving-from-the-old-windows-or-android-app). If you chose **Cancel** at the upload question earlier, sign out and sign in again in the old app to be asked again.

**"OFFLINE: Couldn't reach the sync service" when unlocking**
You're seeing the last data this device saved. Everything is viewable; changes wait until you're back online.

**"Couldn't reach the sync service, so this change may not have been saved"**
Reconnect, press **🔄 Refresh**, and check whether the change is there before entering it again.

**The top bar stays on 🟠 Reconnecting…**
Check the device's internet connection. If it stays orange on a good connection for a long time, Vault may have reached Firebase's free daily limit, which almost never happens; it resets at about **3 PM Philippine time**. Nothing is ever charged.

**I was signed out by myself**
Your password was changed on another device (or reset by email). Sign in with the new password.

**"To sync, profile names may only use letters, numbers, spaces, - and _"**
Your old Windows app profile name has other characters, like a period, so it can't be uploaded. Create a new vault with a plain name, or ask the person who shared Vault with you to help move the data.

**The web app doesn't show the latest version**
Close it and open it again; when the **A new version of Vault is ready** bar appears, tap **Reload**. The web app checks for a new version each time it opens, and every hour while it stays open.

**There's no "Add to Home Screen" or "Install app"**
On an iPhone, open the link in **Safari** (not another app's built-in browser, like Messenger's). On Android, open it in **Chrome**.

**The web app asks me to sign in again and downloads my vault again**
The phone cleared its website data. Nothing is lost: once you sign in, everything comes back from your account. On an iPhone, open Vault from its home-screen icon rather than from Safari to avoid this.

**The old Android or Windows app shows an error or looks out of date**
Those apps are retired and no longer updated. Switch to the web app ([section 1](#1-installing-vault)); your synced data is safe in your account.

---

## 14. Glossary

| Term | Meaning |
|---|---|
| **Allocation** | The paycheck a bill is assigned to. |
| **Paycheck tab** | A tab in the Bi-Monthly Budget listing one paycheck's bills. |
| **10-day safety gap** | Vault's rule of allocating a bill to the latest payday at least 11 days before its due date. |
| **1st Half / 2nd Half (Due)** | The two parts of a monthly payment split across both paychecks. The 2nd Half falls on the real due date. |
| **Sinking Fund** | A bi-monthly loan or split bill where the 1st Half is saved and the 2nd Half pays the full amount. |
| **True Bi-Monthly** | A bi-monthly loan where each half is a real payment to the lender. |
| **Fund** | A savings account holding a 1st Half until the 2nd Half is paid. |
| **Recurring** | A bill that comes back on the same day every month. |
| **Split bill** | A recurring bill turned into a Sinking Fund pair. |
| **TBD** | A recurring bill whose amount hasn't been entered yet. |
| **Receipt** | The record of a paid or funded bill, which can be undone. |
| **Total Loan Incurred** | Principal plus all interest. |
| **Vault** | One profile's data file, or the app itself. |
| **Account** | Your email-and-password sign-in. It keeps your vaults in step across your devices, and can hold many profiles. |
| **Profile** | One separate budget inside your account, opened by typing its profile name. See [Accounts and profiles](#accounts-and-profiles). |
| **Invite code** | The code needed to create an account (the **Family invite code** box), given to you by the person who shared Vault with you. |
| **Live sync** | The connection that sends changes between your devices as they happen. |
| **Local only** | The retired Windows app, used without signing in; its data stays on that computer until uploaded. |
| **Upload** | Copying the retired Windows app's vault into your account, the first time you sign in there. |
| **Firebase** | Google's free service that stores your vaults, passes changes between your devices, and hosts the web app. |
| **Demo** | A made-up vault for trying The Vault without an account. It lives only in the browser tab and is never saved. |
| **Web app** | The Vault itself: an app that runs in your browser and installs from it, at **https://thevault-rc.web.app**. |

---

## Appendix: Setting up Firebase

This is done **once**, by the person who runs Vault, before anyone creates an account. Everyone else can skip it. It takes about 10 minutes and is free: Firebase's **Spark** plan needs no credit card and can't charge you. Never click **Upgrade** to the Blaze plan.

1. **Create the project.** Go to **console.firebase.google.com**, click **Create a project**, name it (e.g. `vault-family`), and turn **Google Analytics off**.
2. **Turn on email sign-in.** **Build → Authentication → Get started → Sign-in method → Email/Password**, switch on the first toggle, **Save**.
3. **Create the database.** **Build → Firestore Database → Create database**. Location **asia-southeast1 (Singapore)** (it can't be changed later), **Start in production mode**, **Create**.
4. **Add the security rules.** In Firestore's **Rules** tab, replace everything with the contents of the `firestore.rules` file from Vault's source code and click **Publish**.
5. **Create the invite code.** In Firestore's **Data** tab: **+ Start collection**, ID `invites`, **Next**. As the **Document ID**, type your code, long and hard to guess (e.g. `adobo-sinigang-mango-2026`). Add a field `note` = `family` and **Save**. To stop new sign-ups later, delete that document.
6. **Connect the app.** **⚙️ Project settings → General → Your apps → Web (`</>`)**, register an app named `Vault`, and copy the `firebaseConfig` values into `src/firebaseConfig.js` in the app's source code, then publish the web app (next step).
7. **Publish the web app.** Turn on **Build → Hosting**, then click **Add another site** and name it `thevault-rc`, so the app is at the short link **https://thevault-rc.web.app** as well as the original one. Create a deploy key and add it to GitHub as described in [SYNC_AND_MOBILE.md](SYNC_AND_MOBILE.md) ("Publish the web app"). From then on, every update is published automatically and every device picks it up by itself.

The full technical guide, including free-plan limits and how sync works inside, is in [SYNC_AND_MOBILE.md](SYNC_AND_MOBILE.md).

---

*The Vault 0.3.0 · This manual describes the web app as of October 2026. The Windows and Android apps are retired.*
