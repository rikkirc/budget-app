# Case study: Vault, a budget app that runs on payday

**Built by:** Rikki Cabanlet · **Built with:** AI-assisted coding (Claude) in VS Code · **Stack:** Tauri, React, Rust, SQLite, Firebase · **Time:** at least four days, 18 merged pull requests

## The problem

Most budgeting apps think in calendar months. Many Filipino workers, especially in government, are paid twice a month: on the 15th and at the end of the month. The real question each payday is not "what do I owe this month?" but "which bills does *this* paycheck have to cover?" I built Vault for myself: I wanted to stop worrying about my bills every time a paycheck came in, and I wanted my loan payments worked out for me instead of by hand, already split across the paychecks that will pay them.

## What I built

A desktop and Android app where every bill is assigned to the paycheck that will pay it. Each paycheck gets its own tab with its own total. Bills whose amount isn't known yet, like electricity, are flagged as TBD so they aren't forgotten. A loan generator turns a loan into a full bi-monthly payment schedule and drops each payment into the right paycheck.

## The hard part: syncing money safely

A household needs the same data on more than one device. I added optional real-time sync through Firebase's free plan, with no server to rent. Money made this harder than ordinary syncing: a payment recorded twice is a real error, not a cosmetic one. So every change is stored once, in order, in a log that can't be edited, and the app never retries a failed change on its own. It tells the user instead. Each device also keeps a full offline copy. Access is limited by an invite code, and each account can only read its own data.

## Result

- Windows and Android apps, with changes showing on the other device within about a second ([demo video](https://drive.google.com/file/d/1LszPGQpBaUTLtbLXiTfI56x02FueR9nd/view))
- Automated tests for the backend, the sync engine and the database security rules
- A user's manual in Markdown and PDF

## What I'd do differently

Synced data is stored in Firebase without its own encryption, which is fine for a family but not for sensitive client records. For an app holding other people's data I would add encryption and check compliance with the Data Privacy Act of 2012 from the start. I'd also test on a real phone earlier: my first phone build had screens wider than the display, which I only caught from screenshots and fixed afterwards.
