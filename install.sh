#!/usr/bin/env bash
set -euo pipefail

# --- Edit these if they don't match your setup ---
REPO="SideralSkylark/inss-watcher"     # owner/repo for GitHub Releases
BIN_NAME="inss-watcher"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/inss-watcher"
CONFIG_FILE="${CONFIG_DIR}/config.toml"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_FILE="${SERVICE_DIR}/inss-watcher.service"
# ---------------------------------------------------

log() { printf '\033[1;32m==>\033[0m %s\n' "$1"; }
die() { printf '\033[1;31mError:\033[0m %s\n' "$1" >&2; exit 1; }

command -v systemctl >/dev/null 2>&1 || die "systemd not found — this script targets Linux with a user systemd instance (macOS/Windows need a different service setup)."
command -v curl >/dev/null 2>&1 || die "curl is required."

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ASSET_ARCH="x86_64" ;;
  aarch64|arm64) ASSET_ARCH="aarch64" ;;
  *) die "Unsupported architecture: $ARCH" ;;
esac

log "Fetching latest release info for ${REPO}..."
RELEASE_JSON="$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest")" \
  || die "Could not reach GitHub API for ${REPO}. Check the REPO variable at the top of this script."

ASSET_URL="$(printf '%s' "$RELEASE_JSON" \
  | grep -oE '"browser_download_url":\s*"[^"]*linux[^"]*'"${ASSET_ARCH}"'[^"]*"' \
  | head -n1 \
  | sed -E 's/.*"(https[^"]+)"/\1/')"

if [[ -z "$ASSET_URL" ]]; then
  die "No matching release asset found for linux/${ASSET_ARCH}. Check your release naming convention in GitHub Actions, or set ASSET_URL manually."
fi

log "Downloading binary from: $ASSET_URL"
mkdir -p "$INSTALL_DIR"
TMP_FILE="$(mktemp)"
curl -sfL "$ASSET_URL" -o "$TMP_FILE" || die "Download failed."
chmod +x "$TMP_FILE"
mv "$TMP_FILE" "${INSTALL_DIR}/${BIN_NAME}"
log "Installed binary to ${INSTALL_DIR}/${BIN_NAME}"

if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  log "NOTE: ${INSTALL_DIR} is not on your PATH. Add this to your shell rc:"
  echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

mkdir -p "$CONFIG_DIR"
if [[ ! -f "$CONFIG_FILE" ]]; then
  log "No existing config found — creating a default at ${CONFIG_FILE}"
  cat > "$CONFIG_FILE" <<'EOF'
# inss-watcher default config
# Edit paths below before first run.

[watch]
directory = "~/Documents/INSS/incoming"

[organize]
output_directory = "~/Documents/INSS/organized"

[ocr]
worker_threads = 1
EOF
  log "Edit ${CONFIG_FILE} to set your real watch/output directories before first run."
else
  log "Existing config found at ${CONFIG_FILE} — leaving it untouched."
fi

log "Installing systemd user service..."
mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=INSS Watcher daemon
After=default.target

[Service]
ExecStart=${INSTALL_DIR}/${BIN_NAME} start
Restart=on-failure
RestartSec=5
Nice=10
IOSchedulingClass=best-effort
IOSchedulingPriority=7

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now "${BIN_NAME}"

log "Enabling linger so the service survives logout/reboot..."
loginctl enable-linger "$USER" 2>/dev/null || log "Could not enable linger automatically — run manually: loginctl enable-linger \$USER"

sleep 1
log "Status check:"
systemctl --user status "${BIN_NAME}" --no-pager -l || true

echo
log "Done. Day-to-day commands:"
echo "    systemctl --user status ${BIN_NAME}"
echo "    journalctl --user -u ${BIN_NAME} -f"
echo "    ${BIN_NAME} ctl rescan"
