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

# $USER isn't guaranteed to be set (some minimal shells/containers don't export
# it), and this script runs under `set -u`, so fall back to `whoami` rather
# than dying on an unbound variable later at the loginctl step.
USER="${USER:-$(whoami)}"

log "Fetching most recent release info for ${REPO}..."
# Note: /releases/latest deliberately excludes pre-releases, and your current
# release(s) are marked pre-release — so we pull the release list and take
# the first (most recent, GitHub returns them newest-first) instead.
RELEASE_JSON="$(curl -sf "https://api.github.com/repos/${REPO}/releases")" \
  || die "Could not reach GitHub API for ${REPO}. Check the REPO variable at the top of this script."

if [[ "$(printf '%s' "$RELEASE_JSON" | tr -d '[:space:]')" == "[]" ]]; then
  die "No releases found for ${REPO} at all (not even pre-releases)."
fi

# Asset name matching: your build currently publishes a single asset named
# exactly "${BIN_NAME}" with no OS/arch suffix, so match on that. If you later
# start publishing per-platform assets (e.g. inss-watcher-linux-x86_64), add
# an arch suffix here and to your release workflow to keep them in sync.
# NOTE: every extraction below ends in "|| true". Without it, a failure deep
# inside these pipelines (e.g. jq erroring on unexpected JSON) counts as the
# whole "VAR=$(...)" assignment failing, and under `set -e` that kills the
# script immediately -- silently, before the diagnostic check below ever runs.
# "|| true" lets extraction fail soft so we can report *why* it's empty.
if command -v jq >/dev/null 2>&1; then
  log "(using jq for JSON parsing)"
  ASSET_URL="$(printf '%s' "$RELEASE_JSON" \
    | jq -r --arg name "$BIN_NAME" '[.[] | .assets[]? | select(.name == $name)][0].browser_download_url // empty' 2>/tmp/inss_jq_err \
    | head -n1 || true)"
  if [[ -z "$ASSET_URL" && -s /tmp/inss_jq_err ]]; then
    log "jq reported an error (falling back to plain-text parsing): $(cat /tmp/inss_jq_err)"
  fi
fi

if [[ -z "${ASSET_URL:-}" ]]; then
  # Fallback (also used if jq is absent, or errored above): find the JSON
  # block for the first asset object whose "name" matches BIN_NAME, then pull
  # the browser_download_url from that block.
  ASSET_URL="$(printf '%s' "$RELEASE_JSON" \
    | tr ',' '\n' \
    | grep -A5 "\"name\": *\"${BIN_NAME}\"" \
    | grep '"browser_download_url"' \
    | head -n1 \
    | sed -E 's/.*"browser_download_url": *"([^"]+)".*/\1/' || true)"
fi

if [[ -z "${ASSET_URL:-}" ]]; then
  log "Could not auto-extract an asset URL. Raw release JSON for debugging:"
  printf '%s\n' "$RELEASE_JSON" | head -c 2000
  echo
  die "No asset named '${BIN_NAME}' found on any release. Check the asset name in your GitHub Actions release step, or set ASSET_URL manually and skip this block."
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
