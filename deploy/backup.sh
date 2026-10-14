#!/bin/sh
# Copies every account and vault into a dated backup folder, keeping the last 30. Safe to run while Vault is in use.
# Schedule it daily with:  crontab -e  then add:  0 3 * * * /home/ubuntu/budget-app/deploy/backup.sh
set -e
cd "$(dirname "$0")"
dest="backups/$(date +%Y-%m-%d)"
mkdir -p "$dest"
find vault-data -name '*.db' ! -name '.*' | while read -r db; do
  out="$dest/${db#vault-data/}"
  mkdir -p "$(dirname "$out")"
  rm -f "$out"
  sqlite3 "$db" "VACUUM INTO '$out'"
done
ls -1d backups/*/ | head -n -30 | xargs -r rm -rf
echo "Backed up to $dest"
