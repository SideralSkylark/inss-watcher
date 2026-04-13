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

## Next Steps & Improvements

### Architecture & Concurrency
- [ ] **Transition to Async (Tokio)**: Migrate the synchronous `mpsc` and `std::thread` implementation to `tokio`. This includes using `tokio::fs`, `tokio::sync::mpsc`, and `tokio::spawn` for better resource management.
- [ ] **Robust Orchestrator Loop**: Implement a proper control loop for the daemon that handles signals (SIGINT, SIGTERM) gracefully and allows for runtime commands (pause, resume, reload).
- [ ] **Database Migrations**: Replace the manual `schema.sql` initialization with a robust migration tool like `rusqlite_migration` or `refinery`.

### Core Logic & Robustness
- [ ] **Advanced Error Handling**: Move away from using `anyhow` everywhere. Implement custom error types using `thiserror` for better error categorization (e.g., `ParsingError`, `StorageError`).
- [ ] **Precision Money Handling**: Refactor `Money` parsing to avoid `f64` and use cents (integers) or a dedicated decimal crate to prevent precision issues.
- [ ] **Improved PDF & OCR**: 
    - Add startup checks for external dependencies (`pdftoppm`, `tesseract`).
    - Explore native Rust crates for OCR or more robust PDF text extraction.
- [ ] **Configurable Policies**: Move hardcoded paths and business rules (quarantine logic, retention periods) into the configuration system.

### CLI & User Experience
- [ ] **Comprehensive CLI**: Build a multi-command CLI using `clap` (e.g., `inss-watcher start`, `inss-watcher status`, `inss-watcher rescan`).
- [ ] **Dry-Run Mode**: Implement a flag to simulate file movements and matching without making actual changes to the filesystem or database.
- [ ] **Enhanced Configuration**: Use the `config` crate to support merging settings from `config.toml`, environment variables, and CLI arguments.

### Quality & Observability
- [ ] **Testing Strategy**:
    - **Unit Tests**: Add tests for domain logic (parsing, matching) with various mock inputs.
    - **Integration Tests**: Use temporary directories and in-memory databases to test the full pipeline.
- [ ] **Structured Logging**: Refine `tracing` usage to include more contextual metadata in logs and support different output formats (JSON for production, pretty for development).
- [ ] **CI/CD Pipeline**: Set up GitHub Actions for automated linting (`clippy`), formatting (`fmt`), and test execution.
- [ ] **Documentation**: Improve code documentation with `rustdoc` comments and generate a project site.

