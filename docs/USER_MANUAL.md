# Vault — Financial Command Center: User's Manual

**Version 0.2.0** · Windows desktop and Android phone · Amounts in Philippine pesos (₱) · Paydays: 15th and end of month

Vault is a budgeting app for people paid twice a month. It plans every bill and loan installment against the paycheck that will cover it and tracks the money in each of your accounts. Use it on your Windows computer, on your Android phone, or both: sign in to a free family account and every change you make on one device appears on the other within seconds.

This manual covers both apps from installation to everyday use. A print-ready version is in [Vault-User-Manual.pdf](Vault-User-Manual.pdf).

## Contents

**Part 1 — Get set up**

1. [Installing Vault](#1-installing-vault)
2. [Unlocking your vault](#2-unlocking-your-vault)
3. [Family accounts & sync](#3-family-accounts--sync)
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

You'll receive Vault as a **Google Drive link** from the person who shared it with you. There are two files; install the one for your device:

| Device | File |
|---|---|
| **Windows computer** | `Vault_0.2.0_x64-setup.exe` |
| **Android phone** | A file ending in `.apk` |

### Install on a Windows computer

1. **Back up first if you already use Vault:** copy the folder `Documents\FinancialCommandCenter` to your Desktop or a USB drive.
2. **If an older Vault is installed,** remove it: **Start → Settings → Apps → Installed apps → Vault → ⋯ → Uninstall**. Your data stays in `Documents\FinancialCommandCenter`; uninstalling never deletes it.
3. Open the Google Drive link and click **Download** (⬇). If Drive or your browser warns that the file might be harmful or can't be scanned, choose **Download anyway** or **Keep**.
4. Open your **Downloads** folder and double-click `Vault_0.2.0_x64-setup.exe`.
5. If Windows shows **Windows protected your PC**, click **More info**, then **Run anyway**. This appears because the installer isn't signed with a paid certificate, which is normal for a family-built app.
6. Click **Next → Install → Finish**, then open **Vault** from the Start Menu. The window is titled **Financial Command Center**.

### Install on an Android phone

1. Open the Google Drive link on your phone and tap **Download** (⬇). Confirm if your phone asks.
2. When the download finishes, tap the notification (or open the **Files** app → **Downloads**) and tap the `.apk` file.
3. If Android says your browser or Drive **isn't allowed to install unknown apps**, tap **Settings**, switch on **Allow from this source**, then go back and tap **Install**.
4. If **Google Play Protect** warns about an unrecognized app, tap **More details → Install anyway**. This appears because Vault isn't from the Play Store.
5. Open **Vault** from your apps.

### Updating

When you're sent a new version, install it the same way. On Android, open the new `.apk` and tap **Update**; there's no need to uninstall first. Your data is kept on every device, and everything synced is also safe in your family account.

---

## 2. Unlocking your vault

Each **profile name** is its own separate vault, with its own accounts, bills and loans. Most people need just one, for example `Juan Dela Cruz`.

**On the computer**, the first screen asks for your profile name. **On the phone**, you first sign in to your family account ([section 3](#3-family-accounts--sync)), then see the same screen.

1. Type your profile name.
2. Tap or click **🔐 Unlock Vault**. A new name creates a new, empty vault; a name you've used before opens that vault.
3. If any recurring bills due in the next 10 days still have no amount, a reminder lists them so you can enter the amounts from your billing statements.

> [!CAUTION]
> **Use exactly the same profile name on every device.** Capitals don't matter and spaces become underscores, so `Juan Dela Cruz` and `juan dela cruz` open the same vault. But a typo like `Juan Dela Curz` quietly creates a brand-new empty vault. If your data seems to be missing, log out and check the name.

> [!CAUTION]
> **Not a password.** The profile name only picks which vault to open. Protect your computer with your Windows sign-in and your phone with its screen lock; synced vaults are also protected by your account password.

For syncing, profile names may only contain letters, numbers, spaces, `-` and `_`.

---

## 3. Family accounts & sync

Signing in to a family account keeps your phone and computer on the same data. It's free, needs no server, and each family member's account is private: nobody else can see your vaults.

You need two things from the person who shared Vault with you: the app (section 1) and the **family invite code**.

> [!NOTE]
> On the **computer**, signing in is optional. Without it, Vault works exactly as before and keeps everything on that one computer ("This computer only"). On the **phone**, you always sign in, because the phone keeps its data in step with your account.

### Create your account (once)

Do this on whichever device you set up first. **If you already have data on your computer, start there**, so it can be uploaded for your phone.

**On the computer:**

1. Unlock your vault as usual (section 2).
2. In the top bar, click **💻 Local only**.
3. Click **Create an account**. Enter your email, a password (at least 8 characters) and the family invite code, then click **Create account**.
4. Vault asks **"Upload this computer's … vault to your account so your phone can use it?"** Click **OK**. All your accounts, bills and loans are copied into your account.
5. The top bar now shows **🟢 Live sync**.

**On the phone:**

1. Open Vault. The **Sign in** screen appears.
2. Tap **Create an account**, enter your email, a password and the family invite code, then tap **Create account**.
3. Unlock with your profile name.

### Sign in on your other devices

1. Open Vault and choose **I already have an account** (on the computer, first click **💻 Local only** in the top bar).
2. Enter the same email and password and click **Sign in**.
3. Unlock with the **same profile name**. Your data appears.

> [!CAUTION]
> **If Vault says your account already has data for this vault**, you've used that profile name on another device already. **OK** replaces the account's copy with this computer's copy on *every* device; anything entered only on the other device is lost. **Cancel** keeps the account's copy and shows it here instead. When in doubt, choose **Cancel**.

### Live sync

The button in the top bar shows how sync is doing:

| Shows | Meaning |
|---|---|
| **🟢 Live sync** | Connected. Your changes reach your other devices within a second or two, and theirs reach you. |
| **🟠 Reconnecting…** | The connection dropped. Vault reconnects by itself as soon as it can. |
| **💻 Local only** | (Computer only) Not signed in. This computer keeps its own separate data. |

When a change arrives from another device, the screen refreshes by itself. If you're halfway through typing, Vault shows **🔄 Your vault was updated on another device. Tap here to refresh** instead, so your half-filled form isn't wiped. Save what you're doing first, then tap it.

### Using Vault offline

Every device keeps its own full copy of your vault, so with no signal you can still open Vault and see all your balances, bills and loans.

**Changes need a connection.** If you try to save one offline, Vault shows **"Couldn't reach the sync service, so this change may not have been saved"**. Once you're back online, press **🔄 Refresh** to check whether it went through *before* trying again. Vault never retries by itself, so a bill can never be paid twice.

### Passwords and lost phones

- **Forgot your password?** On the sign-in screen, tap **Forgot password?**, enter your email, and tap **Email me a reset link**. Check your inbox (and spam folder).
- **Change your password:** click **🟢 Live sync → Change password (lost a phone? this signs it out)**, enter your current and new password.
- **Lost a phone?** Change your password from another device. Every other device is signed out (within about an hour) and needs the new password. Your data stays safe in your account.

### Signing out

Click **🟢 Live sync → Sign out**.

- **On the phone,** the vault locks; sign in again any time.
- **On the computer,** Vault goes back to "This computer only" and shows the computer's own old data, which **doesn't include** changes made while you were signed in. Unless you mean to stop syncing, stay signed in.

---

## 4. Find your way around

The bar at the top of every screen holds the five sections of the app, your vault name, and a few controls.

| Button | What it does |
|---|---|
| **🏠 Dashboard** | Your totals and what's coming up next. See [section 6](#6-dashboard). |
| **💰 Cash Flow** | Your accounts, income, expenses, transfers, and transaction history. See [section 7](#7-cash-flow). |
| **📅 Bi-Monthly Budget** | Every unpaid bill, grouped by the paycheck that pays it. See [section 8](#8-bi-monthly-budget). |
| **📉 Loan Generator** | Add loans and generate their payment schedules. See [section 9](#9-loan-generator). |
| **🔍 Search** | Find any bill, loan, or amount. See [section 10](#10-search). |
| **🟢 Live sync / 💻 Local only** | Your account and sync status. See [section 3](#3-family-accounts--sync). |
| **🔄 Refresh** | Reloads the current screen with the latest numbers. |
| **🔒 Log Out** | Locks the vault and returns to the profile screen. |

### On the phone

Every screen and button works the same as on the computer. A few things are arranged differently to fit:

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

Use the **Log Transaction** bar. Every field is required.

| Type | Effect |
|---|---|
| **Income (+)** | Adds money to the account. Use it for salary, freelance pay, refunds, gifts. |
| **Expense (-)** | Takes money out. Use it for spending that isn't a bill, like groceries or load. |
| **Transfer (↔)** | Moves money from one account to another. Pick the receiving account in the second dropdown. Vault refuses the transfer if the source account doesn't have enough. |

The amount box adds thousands separators as you type, so `12500` becomes `12,500`. Write a short description (e.g. `Salary`, `Groceries`) so you can find it later in Search.

### Transaction history

History shows **one account at a time**, starting with your first account. To switch, click an **account card** (the selected one is outlined in green) or one of the buttons above the list. Click **📚 All Accounts** for the complete history across every account.

- Newest entries are at the top, with the date and time each was recorded.
- Transfers show both accounts, e.g. **🔁 BPI ➡ GCash**.
- Click **Load More Transactions** to see entries beyond the first 50.

### Undo a transaction

Income, expenses, and transfers can be undone from the history. Undoing a transfer reverses both accounts. Undone entries stay in the list, faded and marked **(Undone)**, so you have a full record.

> [!CAUTION]
> **Bill payments are undone elsewhere.** Entries created by paying or funding bills (starting with `Paid:`, `Funded to Savings:`, and their refunds) have no Undo button here. Undo them from **Receipts** on the Bi-Monthly Budget screen, which also puts the bill back.

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
| **Paycheck Allocation** | The payday whose money will pay it, e.g. Oct 31. |
| **🔁 Recurring Bill** | Tick for bills that repeat on the same day every month. |
| **✂️ Split into Bi-Monthly Halves** | Appears for recurring bills. Turns the bill into a Sinking Fund pair. See [Split a recurring bill](#split-a-recurring-bill). |

Click **Save Bill**. Vault rejects dates that don't exist on the calendar, like February 30.

### Paycheck tabs

Each tab, such as **💼 October 31, 2026 BILLS**, holds the bills allocated to that paycheck, nearest due date first. **Total Tab Due** at the top is how much that paycheck needs to cover.

### The controls on each bill

| Control | What it does |
|---|---|
| **⚠️ TBD — Click to Enter Amount** | Appears when a recurring bill has no amount yet. Type the amount and click ✓. |
| **✏️ Edit** | Changes a recurring bill's amount. |
| **✂️ Split** | Splits a recurring bill into a 1st Half and 2nd Half (Due). |
| **🗑️ Delete** | Moves the bill to the Recycle Bin at the bottom of the screen. |
| **📅 Move** | Reassigns the bill to a different paycheck. |
| **Handle Bill** | Pays the bill or saves it to a fund. Greyed out while the amount is TBD. |

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

Paid and funded bills appear under **Receipts (Paid & Settled Bills)**, newest first. Undoing a receipt:

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
- **🗑️ Delete** moves a loan to the **Loan Recycle Bin** and removes its unpaid installments from the Budget. **↩️ Undo & Restore** brings it back and rebuilds its schedule using the paycheck routine currently entered on this screen.

> [!CAUTION]
> **Give loans distinct names.** Deleting a loan removes unpaid bills whose names *start with* the loan's name. A loan called `Car` would also clear a bill called `Car Insurance`. Use names that don't overlap, like `Car Loan`.

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
- **Deleted something by accident:** restore it from the matching Recycle Bin.

### Start using Vault on a new phone

1. Install the Android app ([section 1](#1-installing-vault)).
2. Open it, choose **I already have an account**, and sign in with your email and password.
3. Unlock with the **same profile name** you use on your computer. Your accounts, bills and loans download onto the phone.

### Everyday use on two devices

- Use whichever device is in your hand: changes made on one appear on the other within a second or two.
- If a **"Your vault was updated on another device"** banner appears while you're typing, finish and save what you're doing, then tap the banner.
- Before paying a bill on the go, glance at the top bar: **🟢 Live sync** means the payment will reach your other devices straight away.

---

## 12. Your data & backups

Where your data lives depends on whether you're signed in.

| | Not signed in (computer only) | Signed in to a family account |
|---|---|---|
| **Where** | One file per vault on this computer | A full copy on each of your devices, kept in step through your account |
| **Shared with your phone?** | No | Yes, in real time |
| **Survives losing a device?** | Only if you have a backup | Yes: sign in on a new device and it downloads everything |

### Computer-only vaults

Each vault is one file:

```
Documents\FinancialCommandCenter\<profile_name>_finance_data.db
```

For example, the profile `Juan Dela Cruz` is saved as `juan_dela_cruz_finance_data.db`.

**Back up:** close Vault, then copy the whole `Documents\FinancialCommandCenter` folder to a USB drive or cloud storage. Do this before installing a new version.

**Restore or move to a new computer:** install Vault, copy your backed-up `.db` file into `Documents\FinancialCommandCenter` (create the folder if needed), and unlock with the same profile name.

### Synced vaults

Your synced vaults are stored in your family account (on Google's Firebase service) and copied onto each device you sign in on. If a phone breaks or is lost, nothing is lost: sign in on the new one.

For an extra copy, back up the computer's synced vaults folder now and then. Each file in it is a normal Vault database:

```
%APPDATA%\com.rikki.budget-app\synced\
```

(Paste that into the File Explorer address bar to open it.)

### Privacy

- On the computer without signing in, Vault never sends your data anywhere.
- When signed in, your data travels encrypted to your family's Firebase project. Each person can only ever see their own vaults. Whoever manages the Firebase project could technically look at the stored data, as with any family-run service.
- Passwords are handled entirely by Firebase; Vault never stores them.

---

## 13. Troubleshooting

### Using Vault

**My vault is empty after logging in**
You most likely typed a different profile name, which created a new empty vault. Click **Log Out** and unlock with the exact name you used before. Your data is still in its own file in `Documents\FinancialCommandCenter`.

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

**The phone says "Sync isn't set up yet"**
That copy of the app is out of date. Ask for the latest version and install it (section 1).

**"That family invite code isn't right"**
Check the code with the person who shared Vault with you; it must match exactly, including dashes.

**"Wrong email or password"**
Check for typos, or tap **Forgot password?** to get a reset email. After several wrong tries, Vault asks you to wait a few minutes.

**My phone shows an empty vault**
Either the profile name differs from the computer's (log out and check the spelling), or the computer's data was never uploaded. On the computer, make sure you're signed in to the same account; if you chose **Cancel** at the upload question earlier, sign out and sign in again on the computer to be asked again.

**"OFFLINE: Couldn't reach the sync service" when unlocking**
You're seeing the last data this device saved. Everything is viewable; changes wait until you're back online.

**"Couldn't reach the sync service, so this change may not have been saved"**
Reconnect, press **🔄 Refresh**, and check whether the change is there before entering it again.

**The top bar stays on 🟠 Reconnecting…**
Check the device's internet connection. If it stays orange on a good connection for a long time, the family may have reached Firebase's free daily limit, which almost never happens; it resets at about **3 PM Philippine time**. Nothing is ever charged.

**I was signed out by myself**
Your password was changed on another device (or reset by email). Sign in with the new password.

**"To sync, profile names may only use letters, numbers, spaces, - and _"**
Your computer profile name has other characters, like a period. Create a new vault with a plain name, or ask the person who shared Vault with you to help move the data.

**After signing out on the computer, my recent changes are gone**
They aren't gone: they're in your account. The computer is now showing its old offline copy. Sign in again to see your synced data.

**The phone shows "ERROR: Expected the checkpoint's bytes"**
This was a bug in an early phone app. Ask for the latest version and install it over the old one (section 1); your data is safe.

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
| **Family account** | Your email-and-password sign-in. It keeps your vaults in step across your devices. |
| **Family invite code** | The code needed to create an account, given to you by the person who shared Vault with you. |
| **Live sync** | The connection that sends changes between your devices as they happen. |
| **Local only** | Using Vault on a computer without signing in; its data stays on that computer. |
| **Upload** | Copying a computer's existing vault into your account the first time you sign in. |
| **Firebase** | Google's free service that stores and passes on your synced vaults. |

---

## Appendix: Setting up Firebase

This is done **once**, by the person who runs Vault's sync for the family, before anyone creates an account. Family members can skip it. It takes about 10 minutes and is free: Firebase's **Spark** plan needs no credit card and can't charge you. Never click **Upgrade** to the Blaze plan.

1. **Create the project.** Go to **console.firebase.google.com**, click **Create a project**, name it (e.g. `vault-family`), and turn **Google Analytics off**.
2. **Turn on email sign-in.** **Build → Authentication → Get started → Sign-in method → Email/Password**, switch on the first toggle, **Save**.
3. **Create the database.** **Build → Firestore Database → Create database**. Location **asia-southeast1 (Singapore)** (it can't be changed later), **Start in production mode**, **Create**.
4. **Add the security rules.** In Firestore's **Rules** tab, replace everything with the contents of the `firestore.rules` file from Vault's source code and click **Publish**.
5. **Create the family invite code.** In Firestore's **Data** tab: **+ Start collection**, ID `invites`, **Next**. As the **Document ID**, type your code, long and hard to guess (e.g. `adobo-sinigang-mango-2026`). Add a field `note` = `family` and **Save**. To stop new sign-ups later, delete that document.
6. **Connect the app.** **⚙️ Project settings → General → Your apps → Web (`</>`)**, register an app named `Vault`, and copy the `firebaseConfig` values into `src/firebaseConfig.js` in the app's source code, then rebuild the apps and share the new installers with everyone.

The full technical guide, including free-plan limits and how sync works inside, is in [SYNC_AND_MOBILE.md](SYNC_AND_MOBILE.md).

---

*Vault 0.2.0 · This manual describes the Windows desktop app and the Android app as of October 2026.*
