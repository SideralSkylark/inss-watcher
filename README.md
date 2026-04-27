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
- [ ] RF15: Provide CLI for manual operations
- [ ] RF16: Export list of unmatched documents
- [ ] RF17: Configuration via file or CLI arguments

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
- [ ] US06: Configure watched directories
- [ ] US07: Pause/stop the daemon cleanly

---

## Roadmap

### Phase 1 — Finish the daemon skeleton

Get the process running reliably end-to-end before adding features.

- [ ] **IPC socket + JSON-lines protocol** — Unix socket listener thread, `{"command":"rescan"}` in, `{"status":"ok"}` out. *(RF15, US07)*
- [ ] **Signal handling** — Send `Command::Stop` on SIGINT/SIGTERM for clean shutdown. Use the `ctrlc` crate. *(US07)*
- [ ] **Implement `rescan()`** — Walk watched dirs with `walkdir`, push every PDF into the work queue. *(US03, RF12)*
- [ ] **`inss-ctl` binary** — Second binary that connects to the socket, sends a command, prints the reply. *(RF15)*

### Phase 2 — Correctness

Before adding features, trust that what you have is right.

- [ ] **Unit tests for parsing and matching** — Test `parse_guide`, `parse_receipt`, `classify_doc` with fixture strings. No filesystem, no DB. *(NF02)*
- [ ] **Custom error types with `thiserror`** — Replace `anyhow` in the domain layer (`ParseError`, `StorageError`). Keep `anyhow` in the application layer. *(NF01)*
- [ ] **Fix money precision** — Store values as integer cents instead of `f64`. *(NF01)*
- [ ] **DB migrations with `rusqlite_migration`** — Replace the raw `schema.sql` init with versioned migrations. *(NF01)*

### Phase 3 — CLI and observability

The daemon works. Now make it usable and debuggable.

- [ ] **Full `clap` CLI** — Subcommands: `start`, `stop`, `rescan`, `status`. One binary instead of two. *(RF15, RF17, US03)*
- [ ] **`status` command + unmatched export** — Query the DB, return a JSON blob: queue depth, matched count, unmatched list. *(RF16, US04)*
- [ ] **Startup dependency checks** — Check for `pdftoppm` and `tesseract` on startup, fail loudly if missing. Add JSON log mode for production. *(NF06, RF14)*
- [ ] **Dry-run mode** — Flag through `Settings` to log what would happen without moving files or writing to DB. *(NF02)*

### Phase 4 — Later

Only worth doing once the synchronous version is stable and tested.

- [ ] **Retention + cleanup** — Configurable quarantine period, periodic cleanup job. Document options in `config.toml`. *(RF12, RF13)*
- [ ] **CI/CD + rustdoc** — GitHub Actions: `clippy`, `fmt`, `cargo test`. Write module-level docs incrementally.
