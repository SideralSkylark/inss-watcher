#!/usr/bin/env bash
set -euo pipefail

# --- Must match install.sh ---
BIN_NAME="inss-watcher"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/inss-watcher"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_FILE="${SERVICE_DIR}/inss-watcher.service"
# -------------------------------

log() { printf '\033[1;32m==>\033[0m %s\n' "$1"; }

log "Stopping and disabling service..."
systemctl --user stop "${BIN_NAME}" 2>/dev/null || true
systemctl --user disable "${BIN_NAME}" 2>/dev/null || true

if [[ -f "$SERVICE_FILE" ]]; then
  rm -f "$SERVICE_FILE"
  log "Removed ${SERVICE_FILE}"
fi
systemctl --user daemon-reload

if [[ -f "${INSTALL_DIR}/${BIN_NAME}" ]]; then
  rm -f "${INSTALL_DIR}/${BIN_NAME}"
  log "Removed binary: ${INSTALL_DIR}/${BIN_NAME}"
fi

echo
read -r -p "Also delete config at ${CONFIG_DIR}? This includes your watch/output paths and processed-guide database. [y/N] " REPLY
if [[ "$REPLY" =~ ^[Yy]$ ]]; then
  rm -rf "$CONFIG_DIR"
  log "Removed ${CONFIG_DIR}"
else
  log "Kept config at ${CONFIG_DIR} — delete manually later if needed."
fi

echo
log "Uninstall complete."
log "Note: linger (loginctl enable-linger \$USER) was left enabled since other user services may depend on it."
echo "    To disable it manually: loginctl disable-linger \$USER"
