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
- [ ] RF12: Retain unmatched guides for future matching (configurable period)
- [ ] RF13: Clean up old unmatched records

### Monitoring & Control
- [ ] RF14: Log all actions (file processed, moved, matched)
- [x] RF15: Provide CLI for manual operations
- [ ] RF16: Export list of unmatched documents
- [x] RF17: Configuration via file or CLI arguments

## Non-Functional Requirements
- NF01: **Reliability**: Must not lose or corrupt files during processing
- NF02: **Idempotency**: Processing same file multiple times should be safe
- NF03: **Performance**: Process files within 5 seconds of appearing
- NF04: **Resource Usage**: Use <100MB RAM and minimal CPU when idle
- NF05: **Cross-Platform**: Work on macOS, Linux, and Windows
- NF06: **Observability**: Clear logs showing what happened to each file

## User Stories
- [ ] US01: Drop a PDF in Downloads and have it automatically organized
- [ ] US02: See what happened to my files via logs
- [ ] US03: Manually trigger processing of old files
- [ ] US04: See unmatched guides waiting for payments
- [ ] US05: Fix incorrect matches manually
- [x] US06: Configure watched directories
- [x] US07: Pause/stop the daemon cleanly

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

---

## Roadmap

### Phase 1 — Finish the daemon skeleton 

- [x] **IPC socket + JSON-lines protocol** — Unix socket listener thread, `{"command":"rescan"}` in, `{"status":"ok"}` out. *(RF15, US07)*
- [x] **Signal handling** — Send `Command::Stop` on SIGINT/SIGTERM for clean shutdown. Use the `ctrlc` crate. *(US07)*
- [x] **Implement `rescan()`** — Walk watched dirs with `walkdir`, push every PDF into the work queue. *(US03, RF12)*
- [x] **`inss-ctl` binary** — Second binary that connects to the socket, sends a command, prints the reply. *(RF15)*

### Phase 2 — Correctness 

- [x] **Unit tests for parsing and matching** — Test `parse_guide`, `parse_receipt`, `classify_doc` with fixture strings. No filesystem, no DB. *(NF02)*
- [x] **Custom error types with `thiserror`** — Replace `anyhow` in the domain layer (`ParseError`, `StorageError`). Keep `anyhow` in the application layer. *(NF01)*
- [x] **Fix money precision** — Store values as integer cents instead of `f64`. *(NF01)*

### Phase 3 — CLI and observability

- [x] **Full `clap` CLI** — Subcommands: `start`, `ctl stop`, `ctl rescan`, `ctl status`. One binary instead of two. *(RF15, RF17, US03)*
- [x] **Startup dependency checks** — Check for `pdftoppm` and `tesseract` on startup, fail loudly if missing. Add JSON log mode for production. *(NF06, RF14)* **← do this next**
- [ ] **`status` command + unmatched export** — Query the DB, return a JSON blob: queue depth, matched count, unmatched list. *(RF16, US04)* **← then this**
- [ ] **Dry-run mode** — Flag through `Settings` to log what would happen without moving files or writing to DB. *(NF02)*

### Known Issues & Bugs

- [x] **Fix typos in logs and comments** — Correct `unavalible`, `sucessfull`, `insuported_type`, `resouce`, and "failed to updated".
- [x] **Improve matching logic** — `within_period` only checks the deadline; add a lower bound to prevent matching with very old receipts.
- [ ] **Implement `queue_depth`** — The status command currently returns a placeholder `0` for the work queue depth.
- [ ] **Configurable output directory** — Move the hardcoded `~/Documents/INSS` path into the `Settings` struct.
- [ ] **Temporary directory cleanup** — The `inss_watcher` temp directory is created but never removed.
- [ ] **Robust error handling** — Replace `unwrap()` calls in `persistence.rs` with proper error propagation to prevent daemon panics.
- [ ] **Non-blocking work queue** — The orchestrator currently blocks when the work queue is full, which can make the daemon unresponsive to commands.

### Phase 4 — Later

Only worth doing once the synchronous version is stable and tested.

- [ ] **Retention + cleanup** — Configurable quarantine period, periodic cleanup job. Document options in `config.toml`. *(RF12, RF13)*
- [ ] **CI/CD + rustdoc** — GitHub Actions: `clippy`, `fmt`, `cargo test`. Write module-level docs incrementally.
