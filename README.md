# A Rust daemon to organize my guides and payments

## Functional Requirements

### Detection & Parsing
- [x] RF01: Detect PDF files in watched directories
- [] RF02: Extract text from PDF files (handle scanned/OCR)
- [] RF03: Identify INSS guides vs payment receipts vs other files [high priority]
- [x] RF04: Extract reference period (month/year) from documents
- [] RF05: Extract contributor/reference number (if available)

### File Organization
- [] RF06: Generate destination path based on date + contributor [high priority]
- [x] RF07: Move files to appropriate directory
- [] RF08: Handle naming conflicts (rename if file exists) [high priority]
- [] RF09: Maintain symbolic link or record in original location (optional)

### Matching System
- [] RF10: Store metadata of processed guides
- [] RF11: Match incoming payments with existing guides
- [] RF12: Move matched pairs to final organized location
- [] RF13: Retain unmatched guides for future matching (configurable period)
- [] RF14: Clean up old unmatched records

### Monitoring & Control
- [] RF15: Log all actions (file processed, moved, matched)
- [] RF16: Provide CLI for manual operations
- [] RF17: Export list of unmatched documents
- [] RF18: Configuration via file or CLI arguments

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
