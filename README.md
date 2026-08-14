# INSS Watcher

A Rust daemon that watches a folder, identifies INSS payment guides vs. payment receipts,
extracts their reference period and contributor number, and organizes/matches them
automatically — no manual sorting.

## Functional Requirements

### Detection & Parsing
- [x] RF01: Detect PDF files in watched directories
- [x] RF02: Extract text from PDF files (handle scanned/OCR)
- [x] RF03: Identify INSS guides vs payment receipts vs other files
- [x] RF04: Extract reference period (month/year) from documents
- [x] RF05: Extract contributor number/reference number

### File Organization
- [x] RF06: Generate destination path based on date + contributor
- [x] RF07: Move files to appropriate directory
- [x] RF08: Handle naming conflicts (rename if file exists)

### Matching System
- [x] RF09: Store metadata of processed guides
- [x] RF10: Match incoming payments with existing guides
- [x] RF11: Move matched pairs to final organized location
- [x] RF12: Retain unmatched guides

### Monitoring & Control
- [x] RF13: Log all actions (file processed, moved, matched)
- [x] RF14: Provide CLI for manual operations
- [x] RF15: Export list of unmatched documents
- [x] RF16: Configuration via file or CLI arguments

## Non-Functional Requirements

- NF01: **Reliability** — must not lose or corrupt files during processing
- NF02: **Idempotency** — processing the same file multiple times is safe
- NF03: **Performance** — files are processed within 5 seconds of appearing
- NF04: **Resource Usage** — <100MB RAM and negligible CPU when idle (in practice:
  ~9-11MB idle, brief spikes to ~150MB during OCR that clear in under a second)
- NF05: **Cross-Platform** — Linux is the actively supported/tested platform;
  macOS/Windows are not currently exercised
- NF06: **Observability** — clear logs per file, plus a `doctor` command and
  failure-only desktop notifications (see below)

---

## Installation (recommended: install script)

The fastest path on a new machine — downloads the latest release binary, installs
it, sets up the systemd user service, and enables linger so it survives reboots:

```bash
curl -fsSL <raw-github-url-to-install.sh> -o install.sh
chmod +x install.sh
./install.sh
```

This handles:
- Downloading the release binary (no local `cargo build` needed)
- Installing to `~/.local/bin`
- Writing and enabling the systemd user service
- Creating a default config at `~/.config/inss-watcher/config.toml` if one doesn't exist
- `loginctl enable-linger` so the daemon survives logout/reboot

**Edit `~/.config/inss-watcher/config.toml`** after first install to point at your
real watch/output directories.

To uninstall:
```bash
./uninstall.sh
```
Stops and removes the service and binary; asks before deleting your config/database.

## Installation (manual / building from source)

```bash
cargo build --release
# binary at: target/release/inss-watcher
```

Running manually (foreground):
```bash
inss-watcher start          # starts the daemon, blocks the terminal
inss-watcher ctl rescan     # send commands from another shell
inss-watcher ctl pause
inss-watcher ctl resume
inss-watcher ctl stop
inss-watcher doctor         # health check: daemon up, config valid, folders exist,
                             # OCR available, DB reachable, pending count, recent failures
```

Setting up the systemd user service manually:
```bash
mkdir -p ~/.config/systemd/user
cp inss-watcher.service.example ~/.config/systemd/user/inss-watcher.service
# edit ExecStart to point at your binary path, then:
systemctl --user daemon-reload
systemctl --user enable --now inss-watcher
loginctl enable-linger $USER
```

Day-to-day commands:
```bash
systemctl --user status inss-watcher        # check it is running
journalctl --user -u inss-watcher -f        # follow logs
systemctl --user stop inss-watcher          # stop
systemctl --user start inss-watcher         # start
inss-watcher ctl rescan                     # re-scan watched dirs (also happens
                                             # automatically on every daemon start)
```

---

## Monitoring & Trust

You shouldn't need to check on this daemon. Two things make that possible:

- **`inss-watcher doctor`** — one command, one glance: daemon up, config valid,
  folders exist, OCR available, DB reachable, pending count, recent failures.
  Run it occasionally if you want reassurance; not required.
- **Desktop notifications on failure only.** Success is silent, permanently.
  If something breaks, you get exactly one notification telling you what to do.

---

## Project status

The original build-out roadmap (Phase 0–2) is complete:
- Resource usage tuned (capped OCR threads, debounced file events, `Nice`/IO
  scheduling in the systemd unit)
- Release binaries built via GitHub Actions — no local compilation needed to install
- Deployed as a systemd user service with linger enabled and automatic rescan on start
- `doctor` command and failure-only notifications in place

**Currently in "use it, don't touch it" mode.** Any further work — loosened
matching heuristics, SHA-256 duplicate detection, or anything broader — is
deliberately deferred until real usage over the coming weeks shows it's actually
needed. See the project roadmap doc for the full reasoning.

## Explicitly out of scope (for now)

- GUI, web dashboard, cloud sync, REST API, plugin system
- `setup` command beyond the install script above
- Logging overhaul (current logs + notifications are sufficient)
- Monthly stats, OCR confidence scoring
