# Vault on your phone: sync setup

Vault can keep your desktop and your phone on the same data in real time. When you pay a bill on your phone, the
desktop shows it within a second, and the other way round.

## How it works

```
  Desktop app  ──┐                          ┌──  Phone app (Android, iOS later)
                 ├──►  Vault sync server  ◄──┤
  (any PC)     ──┘    (always online, holds   └──  (any phone)
                       the one shared copy)
```

- The **sync server** is a small program you run on a server you control. It keeps the shared copy of your vaults
  and runs every command (pay bill, transfer, undo…) with exactly the same logic as the desktop app.
- Every device connects with the server's **address** and a **sync token** (a password you choose).
- When one device changes something, the server tells all the others over a live connection, and their screens
  reload. If you're halfway through typing, the app shows *"Your vault was updated on another device"* instead,
  so your form isn't wiped.
- **Offline:** each device keeps the last data it loaded, so you can still look at your balances and bills with no
  signal. Changes need a connection. Vault tells you when a change couldn't reach the server, and never quietly
  retries it, so a payment can't be recorded twice.
- **"This computer only"** still works exactly as before. Sync is optional, and you can turn it off any time.

## Step 1: Run the sync server

You need the server running somewhere that is always on. Pick **one** of these:

### Option A: Fly.io (recommended, about US$2–5/month)

1. Install `flyctl` from https://fly.io/docs/flyctl/install/ and sign up with `fly auth signup`.
2. In `fly.toml`, change `app = "CHANGE-ME-vault-sync"` to a unique name, e.g. `juan-vault-sync`.
3. From the repository folder, run:

   ```bash
   fly launch --copy-config --no-deploy
   fly volumes create vault_data --size 1 --region sin
   fly secrets set VAULT_TOKEN="paste-a-long-random-password-here"
   fly deploy
   ```

   Make up a long random sync token (a password manager can generate one) and save it there. Fly.io never shows it
   again, and you'll type it into each device.
4. Your server address is `https://<your-app-name>.fly.dev`. Open `https://<your-app-name>.fly.dev/health` in a
   browser: it should say `ok`.

### Option B: Any Linux server or VPS with Docker

```bash
docker build -f server/Dockerfile -t vault-server .
docker run -d --name vault-server --restart unless-stopped \
  -e VAULT_TOKEN="a-long-random-password" \
  -v vault-data:/data -p 8080:8080 vault-server
```

Put it behind HTTPS (for example with Caddy or a Cloudflare Tunnel). **The phone app only connects over `https://`.**

### Server settings

| Variable         | Meaning                                                        |
|------------------|----------------------------------------------------------------|
| `VAULT_TOKEN`    | Required. The sync token every device must use (16+ characters). |
| `VAULT_DATA_DIR` | Folder for the vault files (default `data`, `/data` in Docker). |
| `PORT`           | Port to listen on (default `8080`).                             |

## Step 2: Connect the desktop app

1. Open Vault and unlock your vault as usual.
2. Click **💻 Local only** in the top bar (it opens **☁️ Sync**).
3. Enter the server address and your sync token, then click **🔗 Connect**.
4. Vault asks whether to **upload this computer's vault** to the server. Click **OK** the first time, so the server
   starts with all your existing accounts, bills and loans.
   - If the server already has data for that profile, Vault asks whether to replace it or use the server's copy.
     It never overwrites without asking.
5. The button now shows **🟢 Live sync**.

## Step 3: Install the phone app

### Android

1. On GitHub, open the repository's **Actions** tab, then the **Android app** workflow, then the latest successful run.
2. Download **vault-android-apk** from *Artifacts* and unzip it to get the `.apk` file.
3. Copy it to your phone and open it. Allow *Install unknown apps* when Android asks.
4. Open Vault, enter the **same server address and sync token**, then unlock with the **same profile name** you use
   on the desktop.

To build it yourself instead, install Android Studio and the NDK (see
https://v2.tauri.app/start/prerequisites/#android), then run `npm run tauri android init` once and
`npm run tauri android build --apk`.

### iOS (later)

The app is already set up for iOS as well. Building it needs a Mac with Xcode and an Apple Developer account
(US$99/year): `npm run tauri ios init`, then `npm run tauri ios build`.

## Backups

Your vault now lives on the server, so back it up from there. Download a complete copy any time:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" -o juan_backup.db \
  "https://<your-app-name>.fly.dev/api/vault/Juan%20Dela%20Cruz"
```

The file is a normal Vault database. Keep a few dated copies somewhere safe.

## Turning sync off

Click **🟢 Live sync**, then **Turn off sync**. The desktop goes back to its own vault file, which **doesn't include**
changes made since you turned sync on. To keep them, download a backup from the server first (see above), close
Vault, and put it in `Documents\FinancialCommandCenter\` with the vault's file name (e.g.
`juan_dela_cruz_finance_data.db`).

## Security notes

- Anyone with your server address **and** token can read and change your vaults. Use a long random token and keep
  it secret, the same way you would a bank password.
- All traffic is encrypted with HTTPS. The vault files on the server are not encrypted separately, so pick a host
  you trust (your own Fly.io account or VPS).
- If the token leaks, set a new one on the server (`fly secrets set VAULT_TOKEN=...`) and enter it on each device.

## For developers

```
vault-core/   All business logic (every command), shared by everything below. `cargo test` here.
server/       The sync server (axum): HTTP API + WebSocket change feed. `cargo test` here.
src-tauri/    The desktop and mobile app shell; runs vault-core locally when sync is off.
src/api.js    The frontend's single door to the data: local Tauri backend or the sync server.
```

Server API (every `/api` route needs `Authorization: Bearer <token>`):

| Route                       | Purpose                                                              |
|-----------------------------|----------------------------------------------------------------------|
| `GET  /health`              | Liveness check (no token).                                           |
| `GET  /api/ping`            | Checks the token.                                                    |
| `POST /api/login`           | `{ profile }`: opens (or creates) a vault.                           |
| `POST /api/call`            | `{ profile, cmd, args, clientId }`: runs any app command.            |
| `GET  /api/vault/{profile}` | Downloads a complete copy of the vault.                              |
| `PUT  /api/vault/{profile}` | Uploads a vault. Refuses to overwrite data unless `?replace=true`.   |
| `GET  /api/events?token=…`  | WebSocket: `{"type":"changed","profile":…,"clientId":…}` after every change. |
