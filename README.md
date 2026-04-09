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
- [x] RF9: Store metadata of processed guides
- [x] RF10: Match incoming payments with existing guides
- [x] RF11: Move matched pairs to final organized location
- [] RF12: Retain unmatched guides for future matching (configurable period)
- [] RF13: Clean up old unmatched records

### Monitoring & Control
- [] RF14: Log all actions (file processed, moved, matched)
- [] RF15: Provide CLI for manual operations
- [] RF16: Export list of unmatched documents
- [] RF17: Configuration via file or CLI arguments

## Non-Functional Requirements
- NF01: **Reliability**: Must not lose or corrupt files during processing
- NF02: **Idempotency**: Processing same file multiple times should be safe
- NF03: **Performance**: Process files within 5 seconds of appearing
- NF04: **Resource Usage**: Use <100MB RAM and minimal CPU when idle
- NF05: **Cross-Platform**: Work on macOS, Linux, and Windows
- NF06: **Observability**: Clear logs showing what happened to each file

## User Stories
**As a user, I want to:**
- [] US01: Drop a PDF in Downloads and have it automatically organized
- [] US02: See what happened to my files via logs
- [] US03: Manually trigger processing of old files
- [] US04: See unmatched guides waiting for payments
- [] US05: Fix incorrect matches manually
- [] US06: Configure watched directories
- [] US07: Pause/stop the daemon cleanly

## Refactoring & Improvement TODOs

### Concurrency & Performance
- [ ] **Non-blocking Orchestrator**: Move `processor::process_file` to a worker thread or thread pool (e.g., using `rayon` or simple `thread::spawn`) to prevent the orchestrator's command loop from blocking during slow OCR operations.
- [ ] **Asynchronous Stability Checks**: Move the `wait_until_stable` logic from the main watcher loop into the processing task so the watcher can continue detecting new files immediately.

### Configuration & Flexibility
- [ ] **Configurable Storage Paths**: Relocate hardcoded path logic from `infra/fs.rs` (like "INSS" and "quarentine" folders) into the `Settings` struct.
- [ ] **Environment Variable Support**: Allow overriding configuration via environment variables (e.g., `INSS_WATCH_DIRS`).

### Core Logic Improvements
- [ ] **Robust Money Parsing**: Refactor `Money::from_str` in `domain/money.rs` to parse cents directly from strings, avoiding potential `f64` precision issues.
- [ ] **Implement Rescan**: Complete the `orchestrator::rescan` function to scan and process all existing files in watched directories upon request or startup.
- [ ] **Robust Startup Error Handling**: Correctly handle and log errors from `watch::start` in `orchestrator::start`, ensuring the daemon doesn't continue silently if the watcher fails.
- [ ] **Improved Dependency Checks**: Add startup checks to ensure required external binaries (`pdftoppm`, `tesseract`) are available in the system PATH.

### Monitoring & Control
- [ ] **CLI Interface**: Implement a basic CLI (using `clap`) to send commands like `stop`, `rescan`, and `status` to the running daemon (e.g., via a local socket).
- [ ] **Enhanced Observability**: Add a "dry-run" mode to see what files would be moved without actually moving them.

---
Errors:

TODOS:
1. configs(multiple dirs, policies and parameters)
2. main.rs should run on threads with a controll loop
3. cli manual operations for rescan statuses pause continue etc.
4. review logging

