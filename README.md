# A Rust daemon to organize my guides and payments

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
- NF01: **Reliability**: Must not lose or corrupt files during processing
- NF02: **Idempotency**: Processing same file multiple times should be safe
- NF03: **Performance**: Process files within 5 seconds of appearing
- NF04: **Resource Usage**: Use <100MB RAM and minimal CPU when idle
- NF05: **Cross-Platform**: Work on macOS, Linux, and Windows
- NF06: **Observability**: Clear logs showing what happened to each file

---

## Installation & Running

### Build

```bash
cargo build --release
# binary at: target/release/inss-watcher
```

### Running manually (foreground)

```bash
inss-watcher start          # starts the daemon, blocks the terminal
inss-watcher ctl rescan     # send commands from another shell
inss-watcher ctl pause
inss-watcher ctl resume
inss-watcher ctl stop
```

### Running as a systemd user service (recommended)

Create the service file:

```bash
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/inss-watcher.service << 'EOF'
[Unit]
Description=INSS Watcher daemon
After=default.target

[Service]
ExecStart=%h/.cargo/bin/inss-watcher start
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
```

Install the binary and enable the service:

```bash
cargo install --path .

systemctl --user daemon-reload
systemctl --user enable --now inss-watcher
```

Day-to-day commands:

```bash
systemctl --user status inss-watcher        # check it is running
journalctl --user -u inss-watcher -f        # follow logs
systemctl --user stop inss-watcher          # stop
systemctl --user start inss-watcher         # start
inss-watcher ctl rescan                     # re-scan watched dirs
```

> **Note:** by default user services only run while you are logged in.
> To keep the daemon running on a headless or server machine:
> ```bash
> loginctl enable-linger $USER
> ```
