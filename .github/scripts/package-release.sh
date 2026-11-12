#!/usr/bin/env bash
# Packs one release archive: the backend binary, the built frontend, .env.example and RUN.md.
#
# Usage: package-release.sh <version> <suffix> <binary> <public-dir> <env-example> <out-dir>
#
#   suffix   linux-*, macos-* or windows-*; picks the archive format and the RUN.md notes.
#
# Writes <out-dir>/hypecheck-<version>-<suffix>.tar.gz (.zip for Windows), unpacking to a
# hypecheck-<version>-<suffix>/ folder, and prints the archive's path.
set -euo pipefail

version="$1"
suffix="$2"
binary="$3"
public_dir="$4"
env_example="$5"
out_dir="$6"

name="hypecheck-$version-$suffix"
os="${suffix%%-*}"
case "$os" in
  linux | macos) exe="hypecheck_api" ;;
  windows) exe="hypecheck_api.exe" ;;
  *)
    echo "unknown OS in suffix '$suffix'" >&2
    exit 1
    ;;
esac

stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
dir="$stage/$name"
mkdir -p "$dir"

cp "$binary" "$dir/$exe"
# Workflow artifacts don't keep the executable bit.
chmod +x "$dir/$exe"
cp -R "$public_dir" "$dir/public"
cp "$env_example" "$dir/.env.example"

# RUN.md: shared setup, then the notes for this OS.
{
  cat <<EOF
# Hype Check $version

This folder holds the whole app: the \`$exe\` server and the built frontend in \`public/\`.
The server serves both on one port, so there is nothing else to install.

## Configure

The server reads its settings from environment variables, or from a \`.env\` file in the
folder you start it from. Copy \`.env.example\` to \`.env\` and set:

- \`STATIC_DIR=./public\` — serve the frontend from this folder. Without it the server only
  answers \`/api\`. The path is resolved from the folder you start the server in.
- \`BIND_ADDR\` — the address and port to listen on. Default: \`127.0.0.1:8080\`.
- \`IGDB_CLIENT_ID\` and \`IGDB_CLIENT_SECRET\` — your Twitch application's credentials (see the
  project README). Without them the server shows a few demo events instead of IGDB data.
- \`SESSION_SECRET\` — a long random string.

The SQLite database goes in \`data/\` next to where you start the server, unless you change
\`DATABASE_URL\`.

EOF
  case "$os" in
    linux)
      cat <<'EOF'
## Run

```bash
cp .env.example .env   # then edit .env as above
./hypecheck_api
```

Open http://127.0.0.1:8080 (or the address you set in `BIND_ADDR`).
EOF
      ;;
    macos)
      cat <<'EOF'
## Run

```bash
cp .env.example .env   # then edit .env as above
./hypecheck_api
```

Open http://127.0.0.1:8080 (or the address you set in `BIND_ADDR`).

## Unsigned binary

The binary is not signed or notarized, so macOS quarantines it when it comes from a browser
download and refuses to open it. Clear the quarantine flag once before the first run:

```bash
xattr -d com.apple.quarantine hypecheck_api
```
EOF
      ;;
    windows)
      cat <<'EOF'
## Run

In PowerShell, from this folder:

```powershell
Copy-Item .env.example .env   # then edit .env as above
.\hypecheck_api.exe
```

Or set the variables in the shell instead of in `.env`:

```powershell
$env:STATIC_DIR = ".\public"
$env:BIND_ADDR = "127.0.0.1:8080"
$env:IGDB_CLIENT_ID = "your-client-id"
$env:IGDB_CLIENT_SECRET = "your-client-secret"
.\hypecheck_api.exe
```

Open http://127.0.0.1:8080 (or the address you set in `BIND_ADDR`).

## Unsigned binary

The binary is not signed, so Windows SmartScreen may warn that it is from an unknown
publisher. Click **More info**, then **Run anyway**.
EOF
      ;;
  esac
} >"$dir/RUN.md"

mkdir -p "$out_dir"
out_dir="$(cd "$out_dir" && pwd)"
if [[ "$os" == "windows" ]]; then
  archive="$out_dir/$name.zip"
  (cd "$stage" && zip -qr "$archive" "$name")
else
  archive="$out_dir/$name.tar.gz"
  tar -czf "$archive" -C "$stage" "$name"
fi
echo "$archive"
