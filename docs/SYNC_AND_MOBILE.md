# Vault on your phone: family accounts and sync

Vault can keep each family member's phone and computer on the same data in real time. Pay a bill on your phone and
your desktop shows it within a second, and the other way round.

## How it works

```
  Juan's PC ─┐                                      ┌─ Juan's phone
             ├──►  Vault sync server (free Oracle  ◄──┤
  Maria's PC ┘     Cloud computer, always online)    └─ Maria's phone
```

- **One server for the whole family**, running on a free Oracle Cloud computer. You set it up once.
- **Everyone makes their own account** with an email, a password and the **family invite code** you choose.
  You never have to create accounts for anyone. Strangers who find the server can't sign up without the code.
- **Each account is private.** Juan can't see Maria's vaults, even if they both use the same profile name.
- **Forgot a password?** At sign-up, everyone gets a **recovery code** to write down. It resets their password
  without email and without asking you.
- **Live updates:** when you change something, your other signed-in devices refresh right away. If you're halfway
  through typing, the app shows *"Your vault was updated on another device"* instead, so your form isn't wiped.
- **Offline:** each device keeps the last data it loaded, so you can still look at your balances and bills with no
  signal. Changes need a connection. Vault tells you when a change couldn't reach the server, and never quietly
  retries it, so a payment can't be recorded twice.
- **"This computer only"** still works exactly as before. Signing in is optional on the desktop.

---

## Part 1: Set up the server (once, by one person, about 1 hour)

Everything below is free. You need a credit or debit card for Oracle's identity check; Always Free resources are
not charged.

### 1. Create the free Oracle Cloud computer

1. Sign up at https://www.oracle.com/cloud/free/. When asked for a **Home Region**, choose **Singapore**. It's
   closest to the Philippines, and you can't change it later.
2. In the Oracle Cloud console, go to **Compute → Instances → Create instance**:
   - **Image:** Canonical **Ubuntu** (22.04 or 24.04).
   - **Shape:** **Ampere VM.Standard.A1.Flex** with 1 OCPU and 6 GB memory. It's labelled *Always Free-eligible*.
     If Oracle says it's *out of capacity*, try again later, or pick **VM.Standard.E2.1.Micro** (also free; see
     the note in step 4).
   - **SSH keys:** choose *Generate a key pair* and **download the private key**. You need it to log in.
3. Click **Create**. When it's running, copy its **Public IP address**.
4. Open the web ports. On the instance page, click the **subnet**, then its **Default Security List**, then
   **Add Ingress Rules**:
   - Source CIDR `0.0.0.0/0`, IP protocol TCP, destination port **80**
   - Source CIDR `0.0.0.0/0`, IP protocol TCP, destination port **443**

### 2. Get a free web address (DuckDNS)

1. Go to https://www.duckdns.org and sign in (Google or GitHub works).
2. Add a subdomain, e.g. `delacruz-vault`, and set its **current ip** to your instance's public IP.
3. Your server address is now `https://delacruz-vault.duckdns.org`.

### 3. Log in to the server and install Docker

From your PC, in a terminal (PowerShell on Windows), log in with the key you downloaded:

```bash
ssh -i path/to/ssh-key.key ubuntu@YOUR_PUBLIC_IP
```

Then, on the server:

```bash
# Oracle's Ubuntu images block web traffic with their own firewall, so let ports 80 and 443 through
sudo iptables -I INPUT 6 -p tcp -m multiport --dports 80,443 -j ACCEPT
sudo netfilter-persistent save

# Install Docker, git and sqlite3 (for backups)
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker ubuntu
sudo apt-get install -y git sqlite3
exit
```

Log in again with the same `ssh` command, so the Docker permission takes effect.

> **Using the E2.1.Micro shape (1 GB memory)?** Building the server needs more memory than that, so add swap
> first: `sudo fallocate -l 4G /swapfile && sudo chmod 600 /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile`

### 4. Start Vault

```bash
git clone https://github.com/rikkirc/budget-app.git
cd budget-app/deploy
cp .env.example .env
nano .env
```

Your repository is private, so `git clone` asks for your GitHub username and a **personal access token** in place
of a password. Create one at https://github.com/settings/tokens with read access to the repository.

In `.env`, set:

- `VAULT_DOMAIN`: your DuckDNS address, without `https://` (e.g. `delacruz-vault.duckdns.org`)
- `VAULT_INVITE_CODE`: make up a family code, at least 8 characters (e.g. `adobo-sinigang-2026`)

Save with **Ctrl+O**, **Enter**, then **Ctrl+X**. Then start everything:

```bash
docker compose up -d --build
```

The first build takes about 5–10 minutes. When it's done, open `https://delacruz-vault.duckdns.org/health` in a
browser. It should say `ok`. The HTTPS certificate is created and renewed automatically.

### 5. Turn on daily backups

```bash
crontab -e
```

Add this line, then save:

```
0 3 * * * /home/ubuntu/budget-app/deploy/backup.sh >> /home/ubuntu/backup.log 2>&1
```

Every night at 3 AM this copies all accounts and vaults into `deploy/backups/<date>/` and keeps the last 30 days.
Now and then, also copy the backups **off** the server to your PC:

```bash
scp -i path/to/ssh-key.key -r ubuntu@YOUR_PUBLIC_IP:budget-app/deploy/backups ./vault-backups
```

> **Why copy backups off the server?** Oracle may reclaim Always Free computers that sit almost idle for a long
> time, and a family budget app is very idle. A copy on your PC means you never depend on the server alone.

### 6. Send your family the details

Give each family member:

1. The **Android app** (see Part 2)
2. The **server address**, e.g. `https://delacruz-vault.duckdns.org`
3. The **family invite code**

That's it: they create their own accounts.

### Updating the server later

```bash
cd ~/budget-app && git pull && cd deploy && docker compose up -d --build
```

---

## Part 2: Install the app

### Android phone

1. On GitHub, open the repository's **Actions** tab, then the **Android app** workflow, then the latest
   successful run.
2. Download **vault-android-apk** from *Artifacts* and unzip it to get the `.apk` file.
3. Copy it to the phone and open it. Allow *Install unknown apps* when Android asks.

### iOS (later)

The app is already set up for iOS. Building it needs a Mac with Xcode and an Apple Developer account
(US$99/year): `npm run tauri ios init`, then `npm run tauri ios build`.

## Part 3: Create your account (each family member)

1. Open Vault. On the phone, the account screen appears first. On the desktop, unlock your vault first if you
   already use Vault there, then click **💻 Local only** in the top bar.
2. Choose **Create an account**. Enter the server address, your email, a password (at least 8 characters) and
   the family invite code.
3. **Write down your recovery code** and keep it safe. It's the only way back in if you forget your password.
4. **On a desktop that already has your data:** Vault offers to upload this computer's vault. Click **OK** so your
   phone starts with all your accounts, bills and loans. It never overwrites data already in your account without
   asking.
5. Unlock with your profile name, e.g. `Juan Dela Cruz`. Use the **same profile name** on every device.
6. On your other devices, choose **I already have an account** and sign in with the same email and password.

The top bar shows **🟢 Live sync** when changes are flowing, or **🟠 Reconnecting…** if the connection dropped
(Vault reconnects by itself).

### Forgot your password?

On the sign-in screen, tap **Forgot password?**, then enter your email, your recovery code and a new password.
You get a **new** recovery code (the old one stops working), and you're signed out on your other devices.

### Lost your phone?

On another device, click **🟢 Live sync**, then **Lost a phone? Sign out on every device**, and sign in again on
the devices you still have.

### Signing out on the desktop

Signing out takes the desktop back to its own vault file, which **doesn't include** changes made while you were
signed in. Unless you want that old copy, stay signed in.

---

## Security notes

- Passwords and recovery codes are stored only as one-way **Argon2 hashes**, and sign-in sessions only as
  **SHA-256 hashes**. Even someone holding a copy of the server's files can't sign in as anyone.
- After **5 wrong passwords** (or 5 wrong invite codes), further tries are refused for 15 minutes.
- All traffic is encrypted with HTTPS. The vault files themselves aren't encrypted on the server, so whoever
  controls the Oracle account (you) could read them. Family members should know this.
- To stop new sign-ups, change `VAULT_INVITE_CODE` in `deploy/.env` and run `docker compose up -d`. Existing
  accounts keep working.
- A device that hasn't been used for 180 days has to sign in again.

## For developers

```
vault-core/   All business logic (every command), shared by everything below. `cargo test` here.
server/       The sync server (axum): accounts, HTTP API and WebSocket change feed. `cargo test` here.
deploy/       docker-compose (server + Caddy for HTTPS), Caddyfile, nightly backup script.
src-tauri/    The desktop and mobile app shell; runs vault-core locally when signed out.
src/api.js    The frontend's single door to the data: the local Tauri backend or the sync server.
```

Server configuration: `VAULT_INVITE_CODE` (required, 8+ characters), `VAULT_DATA_DIR` (default `data`),
`PORT` (default `8080`). Accounts live in `<data>/accounts.db`, and each account's vaults in `<data>/users/<id>/`.

| Route                              | Auth    | Purpose                                                        |
|------------------------------------|---------|----------------------------------------------------------------|
| `GET  /health`                     | none    | Liveness check.                                                |
| `POST /api/sign-up`                | none    | `{ email, password, inviteCode }` → `{ token, email, recoveryCode }` |
| `POST /api/sign-in`                | none    | `{ email, password }` → `{ token, email }`                     |
| `POST /api/reset-password`         | none    | `{ email, recoveryCode, newPassword }` → `{ token, email, recoveryCode }` |
| `GET  /api/me`                     | session | The signed-in email.                                           |
| `POST /api/change-password`        | session | `{ currentPassword, newPassword }`                             |
| `POST /api/sign-out`               | session | Ends this session.                                             |
| `POST /api/sign-out-everywhere`    | session | Ends every session of this account.                            |
| `POST /api/login`                  | session | `{ profile }`: opens (or creates) one of your vaults.          |
| `POST /api/call`                   | session | `{ profile, cmd, args, clientId }`: runs any app command.      |
| `GET  /api/vault/{profile}`        | session | Downloads a complete copy of one of your vaults.               |
| `PUT  /api/vault/{profile}`        | session | Uploads a vault. Refuses to overwrite data unless `?replace=true`. |
| `GET  /api/events?token=…`         | session | WebSocket: `{"type":"changed","profile":…,"clientId":…}` after each change to your vaults. |

"session" means `Authorization: Bearer <token>`, or `?token=` for the WebSocket.
