# Vault on your phone: family accounts and sync

Vault can keep each family member's phone and computer on the same data in real time. Pay a bill on your phone and
your desktop shows it within a second, and the other way round. It runs on **Firebase's free Spark plan**: no
server to rent, no credit card, and no bill, ever.

## How it works

```
  Juan's phone ─┐                                   ┌─ Maria's phone
                ├──►  Firebase (Google, free plan)  ◄─┤
  Juan's PC ────┘     Singapore data centre          └─ Maria's PC
```

- **Everyone makes their own account** with an email, a password and the **family invite code** you choose.
  You never create accounts for anyone. Without the code, nobody can store anything.
- **Each account is private.** Juan can't see Maria's vaults, even if they both use the same profile name.
- **Forgot a password?** Tap *Forgot password?* and Firebase emails a reset link.
- **Lost a phone?** Change your password on another device. That signs out every other device.
- **Live updates:** a change on one device shows up on your others within about a second. If you're halfway
  through typing, the app shows *"Your vault was updated on another device"* instead, so your form isn't wiped.
- **Offline:** every device keeps its own full copy of your vault, so you can always see your balances and bills.
  Changes need a connection. Vault tells you if a change couldn't be sent, and never quietly retries it, so a
  payment can't be recorded twice.
- **"This computer only"** still works exactly as before. Signing in is optional on the desktop.

### Is it really free?

Yes. The Spark plan has no payment method, so it **cannot** charge you. Its daily limits are far above what a family
budget uses:

| Spark limit (per day, whole family) | Typical family use                                    |
|-------------------------------------|-------------------------------------------------------|
| 20,000 changes saved                | A few dozen (one per payment, transfer, bill…)        |
| 50,000 reads                        | A few hundred (each change is read once per device)   |
| 1 GB stored in total                | A few MB, even after years                            |

If a limit were ever reached, syncing would pause until it resets at midnight US Pacific time (about 3 PM
Philippine time), and you'd see an error, never a charge. **Never click "Upgrade" to the Blaze plan**; nothing in
Vault needs it.

---

## Part 1: Set up Firebase (once, by one person, about 10 minutes)

### 1. Create the project

1. Go to https://console.firebase.google.com and sign in with a Google account.
2. Click **Create a project** (or *Add project*), name it e.g. `vault-family`, and continue.
3. When asked about **Google Analytics**, turn it **off**. Vault doesn't need it. Click **Create project**.

### 2. Turn on email sign-in

1. In the left menu: **Build → Authentication → Get started**.
2. Under **Sign-in method**, click **Email/Password**, switch on the first toggle (*Email/Password*), and **Save**.

### 3. Create the database

1. Left menu: **Build → Firestore Database → Create database**.
2. **Location:** choose **asia-southeast1 (Singapore)**. It's closest to the Philippines, and you can't change it
   later.
3. Choose **Start in production mode**, then **Create**.
4. Open the **Rules** tab, delete everything there, paste the whole contents of
   [`firestore.rules`](../firestore.rules) from this repository, and click **Publish**.

### 4. Create the family invite code

1. Still in Firestore, open the **Data** tab and click **+ Start collection**.
2. **Collection ID:** `invites`, then **Next**.
3. **Document ID:** type your family invite code, e.g. `adobo-sinigang-mango-2026`. Make it long and hard to
   guess: three or four random words plus a number is ideal. The code is what keeps strangers out.
4. Add one field so the console lets you save: field `note`, type *string*, value `family`. Click **Save**.

To stop new sign-ups later, delete that document. To let someone in again, create a new code the same way.
Existing accounts keep working either way.

### 5. Connect the app to your project

1. Click the ⚙️ gear next to *Project Overview*, then **Project settings**.
2. Under **Your apps**, click the **web** icon (`</>`), name it `Vault`, leave *Firebase Hosting* unticked, and
   click **Register app**.
3. Firebase shows a block of code with `const firebaseConfig = { apiKey: "…", authDomain: "…", … }`.
4. Open [`src/firebaseConfig.js`](../src/firebaseConfig.js) in this repository (on GitHub you can click the ✏️
   pencil to edit it in the browser) and replace the placeholder values with yours. Commit the change.

These values are **not secret**. Every Firebase app contains them. What protects your data is `firestore.rules`.

Committing the file starts a new Android build on GitHub automatically (see Part 2). For the desktop app, build
it again (`npm run tauri build`) or run it with `npm run tauri dev`.

### 6. Publish the web app (once)

The web app is published to **Firebase Hosting**, which the free Spark plan includes (10 GB of storage and 360 MB
of downloads a day; Vault is about 1 MB once compressed, and each phone downloads it once per update). GitHub
publishes it automatically every time `main` changes, using a key you create once:

1. **Turn on Hosting.** In the Firebase console, left menu **Build → Hosting → Get started**. Click **Next** through
   the steps (you don't need to run the commands it shows) and **Continue to console**.
2. **Create the deploy key.** Open https://console.cloud.google.com/iam-admin/serviceaccounts, sign in with the
   same Google account and pick your Firebase project at the top. Click **+ Create service account**, name it
   `github-deploy`, and click **Create and continue**. Add these three roles, then click **Done**:
   **Firebase Hosting Admin**, **API Keys Viewer**, **Cloud Run Viewer**.
3. Click the new `github-deploy` account, open the **Keys** tab, then **Add key → Create new key → JSON → Create**.
   A `.json` file downloads. It can publish your website, so keep it private.
4. **Give it to GitHub.** In the repository on GitHub: **Settings → Secrets and variables → Actions → New repository
   secret**. Name: `FIREBASE_SERVICE_ACCOUNT`. Secret: open the `.json` file in a text editor and paste all of it.
   Click **Add secret**, then delete the downloaded file.
5. **Publish.** **Actions → Web app → Run workflow**. When the run is green, the app is live at
   `https://<your-project-id>.web.app` (for this repository, https://the-vault-1399a.web.app).

If your project ID is different, change `projectId` in `.github/workflows/web.yml` to match. Until the secret
exists, the workflow still builds the web app and attaches it to the run, and skips publishing with a note.

---

## Part 2: Install the app

### Phones: the web app (iPhone and Android)

Send everyone the link, `https://<your-project-id>.web.app`. On an iPhone they open it in **Safari** and choose
**Share → Add to Home Screen**; on Android, they open it in **Chrome** and choose **⋮ → Install app**. The web app
updates itself whenever a new version is published, so there's nothing to resend. Step-by-step instructions for
family members are in the User's Manual, section 1.

It runs the same vault-core as the other apps (compiled to WebAssembly), keeps its copy of the vault in the
browser's storage, and opens offline. Like the Android app, it always works through a family account.

### Android phone (.apk)

The web app above is simpler; the `.apk` remains available.

1. On GitHub, open the repository's **Actions** tab, then the **Android app** workflow, then the latest
   successful run **after** you committed `firebaseConfig.js`.
2. Download **vault-android-apk** from *Artifacts* and unzip it to get the `.apk` file.
3. Send it to each family member's phone and open it. Allow *Install unknown apps* when Android asks.

### iOS

iPhones use the web app above. A native iOS app is also possible (the Tauri project is set up for it), but it needs
a Mac with Xcode and an Apple Developer account (US$99/year): `npm run tauri ios init`, then
`npm run tauri ios build`.

## Part 3: Create your account (each family member)

1. Open Vault.
   - **On a phone,** the account screen appears first.
   - **On a desktop that already has your data,** unlock your vault as usual first, then click **💻 Local only** in
     the top bar.
2. Choose **Create an account**. Enter your email, a password (at least 8 characters) and the family invite code.
3. **On a desktop that already has your data:** Vault offers to upload this computer's vault. Click **OK** so your
   phone starts with all your accounts, bills and loans. It never overwrites data already in your account without
   asking.
4. Unlock with your profile name, e.g. `Juan Dela Cruz`. Use the **same profile name** on every device.
5. On your other devices, choose **I already have an account** and sign in with the same email and password.

The top bar shows **🟢 Live sync** when changes are flowing, or **🟠 Reconnecting…** if the connection dropped
(Vault reconnects by itself).

### Signing out on the desktop

Signing out takes the desktop back to its own vault file, which **doesn't include** changes made while you were
signed in. Unless you want that old copy, stay signed in.

---

## Backups

Every device holds a complete copy of each synced vault, so a phone or PC breaking loses nothing. For an extra
copy, back up the desktop's synced vaults folder now and then:

- **Windows:** `%APPDATA%\com.rikki.budget-app\synced\`
- **macOS:** `~/Library/Application Support/com.rikki.budget-app/synced/`
- **Linux:** `~/.local/share/com.rikki.budget-app/synced/`

Each file in it is a normal Vault database.

## Security notes

- Passwords are handled entirely by Firebase Authentication; Vault never stores or sees them.
- `firestore.rules` makes sure that:
  - only accounts that joined with a valid invite code can store anything;
  - each person can only ever read and change their own vaults;
  - invite codes can't be read or listed by anyone;
  - the list of changes is append-only, so no change can be edited or deleted afterwards.
- All traffic is encrypted. As the owner of the Firebase project, you could technically look at family members'
  data in the Firebase console, the same as with any family-run server. Family members should know this.

---

## For developers

```
vault-core/        All business logic (every command), plus apply_op: applying the numbered list of changes.
vault-web/         vault-core compiled to WebAssembly for the web app (see vault-web/README.md).
src-tauri/         The desktop and mobile app shell: local vaults, and this device's sync replicas.
src/api.js         The single door to the data: the local vault, or the synced vault via the sync engine.
src/sync/
  engine.js          Keeps a replica in step with the shared list of changes; checkpoints; offline handling.
  firestoreRemote.js The shared list of changes in Cloud Firestore.
  account.js         Family accounts on Firebase Authentication.
  tauriReplica.js    The replica, run by vault-core through Tauri commands (desktop and Android).
  webReplica.js      The web app's replica: vault-core in WebAssembly, saved to IndexedDB (browserStore.js).
src/UpdateBanner.jsx The web app's "new version ready" bar (the service worker comes from vite-plugin-pwa).
firestore.rules    The security rules (invite codes, privacy, append-only changes).
```

**How sync works.** Each vault has a numbered, append-only list of changes in Firestore
(`users/{uid}/vaults/{vault}/ops`). A change (e.g. `pay_bill` with its arguments) takes the next number in a
transaction, and every device applies the list in order to its own SQLite replica using the same vault-core code.
Every device therefore computes identical results. Transaction timestamps come from the change itself, not the
applying device's clock. Every 200 changes, a device uploads a checkpoint (the whole vault, in ≤ 900 KB Firestore
documents), so a new device starts from the latest checkpoint instead of replaying everything. An uploaded desktop
vault is a special `__restore__` change pointing at a checkpoint.

Tests:

```bash
cd vault-core && cargo test   # business logic and replica rules
npm run build:wasm && npm test  # sync engine against real vault-core replicas, native and WebAssembly
npm run test:firebase         # security rules + sync over the Firestore & Auth emulators (needs Java)
```

The web app: `npm run dev:web` serves it locally, `npm run build:web` builds it into `dist-web/`. Both need the
WebAssembly tools listed in `vault-web/README.md`.
