# INSS Watcher Roadmap 

Merges the closed Minimal-Effort Roadmap (Phases 0–2) and the V1 Usability & Maintenance Roadmap, and adds printing. Last updated 2026-10-23.

## Status

Phases 0–2 are complete and the tool is in daily use: it watches a folder, OCRs and classifies INSS guides and payment receipts, matches them, organizes them, and notifies on failure. Idle footprint is \~9–11 MB RAM.

Since the last roadmap, a period-parsing bug (months 11 and 12 were read as 1 and 2 because of a greedy regex) was found and fixed, and matching now works on real documents.

This document continues from there. It is about making the tool **safe to update, easy to inspect, able to print what needs printing, and tidy over time**, without turning it into a large application.

## Guiding rules

1. The daemon stays invisible in normal operation. It watches, processes, matches and organizes. Nothing else.
2. One obvious interface for everything a person does by hand: look, audit, print, clean up. Not one new CLI command per need.
3. The interface is a convenience layer, not a second application. It holds no business logic and talks to the daemon through the existing `ctl` channel and a read-only view of the database.
4. Only three things ever touch user files: organizing (daemon), printing, and purging. Printing and purging happen only from the interface, only after a preview, and never automatically.
5. Prefer pending over a wrong match. Prefer a loud failure over a silent wrong value.
6. Every parsing rule gets a test built from a real (anonymized) document, not a hand-typed string.

## Decisions recorded

- A **local-only** interface (127.0.0.1, short-lived, no accounts) is now in scope. This supersedes the old "GUI / web dashboard" skip. A remote or public dashboard stays out of scope.
- **Printing is in scope.** The tool is used each month to print other people's payment receipts (and the occasional guide) to hand over together with the physical guide they bring. Their guides are never digitized, and their payments are not kept long term. Only the user's own documents are retained.
- Document state stays minimal: `pending`, `matched`, `unmatched`, `duplicate`, `failed`. Printing adds a `printed_at` timestamp, not a new state.

## Done (Phases 0–2)

- [x] OCR capped to one worker thread, debounced file events, `Nice`/IO scheduling in the systemd unit
- [x] Release binaries via GitHub Actions; SQLite bundled into the binary
- [x] systemd user service, linger, automatic rescan on start, `install.sh` / `uninstall.sh`
- [x] `doctor` command; failure-only desktop notifications
- [x] Period parsing fixed for all twelve months, with an every-month regression test

---

## Phase 3 — Make the current build safe to update

The installer is the first priority because every later phase ships through it.

- [ ] `install.sh`: use `restart` instead of `enable --now`, so an update replaces the running code
- [ ] `install.sh`: create the download temp file inside the install directory so the final `mv` is an atomic rename
- [ ] `install.sh`: fix the wrong paths; add dependency checks (`pdftoppm`, OCR tool) with install hints; run `doctor` at the end
- [ ] `uninstall.sh`: review for the same path problems; it must ask before removing config or database
- [ ] Confirm the deployed binary actually contains the parser fix (binary modification time vs. service start time)
- [ ] Smoke-test parsing against every real guide on hand; keep two or three anonymized PDFs as permanent fixtures
- [ ] Verify order independence: receipt before guide, a bulk dump of 10 pairs, and a rescan. Write a failing test first if anything is stranded
- [ ] Decide what happens to a receipt whose guide has not arrived yet: it should wait as `pending`, not go to quarantine
- [ ] Anchor the receipt's payment date on the `Data processamento` label instead of taking the first date in the text
- [ ] Compile regexes once as static values that fail at startup or in tests, instead of `.ok()?` turning a bad pattern into "missing field"
- [ ] Use unique temp names in `pdf_to_img` so same-stem files in a bulk dump cannot overwrite each other

**Exit criteria:** re-running `install.sh` on a working install updates the binary, restarts the daemon and ends with a clean `doctor`.

---

## Phase 4 — Local interface, read-only first

The current web server is buggy. Rebuild it small and verify each step before adding the next.

**Design**

- `inss-watcher ui` starts a separate, short-lived process, opens the browser, and exits after an idle timeout or when the launching command is interrupted
- Bind to `127.0.0.1` on a random free port
- Generate a random token at startup, include it in the opened URL, require it on every request, and check the `Host` header. This protects against other websites in the browser reaching the local server
- Read the database through a separate read-only connection, so the interface can never disturb the daemon
- One HTML page embedded in the binary (`include_str!`) that fetches JSON and renders it. Keep the HTML dumb
- Run the server in the foreground with verbose logging while debugging

**Steps**

- [ ] 4a — Overview: daemon running or not, today's counts (processed, matched, unmatched, duplicates, failures), last activity
- [ ] 4b — Documents view: filter by contributor, period, status, date; detail panel with original filename, final location and timestamps; open or reveal the file
- [ ] 4c — Controls: Rescan, Pause, Resume, sent through the existing `ctl` channel
- [ ] 4d — Configuration: the small set of settings worth editing without opening the TOML (directories, OCR workers, debounce, notification toggles). The file remains the source of truth

**Exit criteria:** a bad match can be diagnosed from the page alone, without the sqlite shell or the logs.

---

## Phase 5 — Printing from the interface

Every month the user prints other people's payment receipts (and occasionally a guide) and hands them over with the physical guide. Those documents do not match anything, so they sit in quarantine. The print flow should work from exactly that pile.

**Print queue**

- [ ] A Print view listing quarantined receipts and guides that are eligible
- [ ] Show, per item: type, reference number, amount, payment date, and for guides the contributor number and period
- [ ] Show the owner or payer if the receipt text contains one (see open questions)
- [ ] Do not offer items younger than a short grace period, so a receipt waiting for its guide is not picked up by mistake. The length is configurable
- [ ] Never offer documents that are `matched`: those are the user's own

**Printing**

- [ ] Select items, preview the batch, then merge the selection into one PDF in a temporary folder and open it in the browser's print dialog. This needs no printer configuration on any machine
- [ ] Optional later: one-click printing through `lp` if CUPS is available
- [ ] After printing, set `printed_at` on the items and show them as printed

**Purge**

- [ ] Printed items older than a configurable retention period appear in Cleanup, never in an automatic job
- [ ] Removal always shows exactly what will be deleted and requires confirmation

**Exit criteria:** at month end, open the interface, review the queue, print, and clear it in under a minute, with nothing of the user's own deleted.

---

## Phase 6 — Notifications and overdue alerts

- [ ] Notification when a guide and payment are matched (contributor, period, documents organized). Can be disabled
- [ ] Keep failure notifications; keep unmatched documents silent
- [ ] Configure the user's own contributor numbers (`own_contributors` in the config)
- [ ] Overdue alert: a guide for an own contributor, past its deadline (the 10th of the following month) with no payment. This is the one unmatched state that matters. Guides for other people are excluded, since they are expected to stay unmatched
- [ ] Optional monthly digest that also works as a heartbeat. If the digest stops arriving, the daemon is down or notifications are broken

---

## Phase 7 — Document hygiene

- [ ] Explicit states in the database (`pending`, `matched`, `unmatched`, `duplicate`, `failed`) plus the `printed_at` timestamp
- [ ] SHA-256 duplicate detection. Duplicates are recorded with a reference to the original, not silently dropped
- [ ] Replace "rename on conflict" with hash-aware handling, so exact copies stop piling up
- [ ] Cleanup view: counts by status, oldest unmatched, filters by status, age, contributor and period, a preview of exactly what will be removed, and an internal dry-run. Automatic deletion stays off by default
- [ ] Make state rebuildable: back up the database, or make sure it can be reconstructed by rescanning the organized folders
- [ ] Extra rules for unmatched guides stay small and driven by real data. Loosen matching only if the unmatched export shows false negatives

---

## Phase 8 — Install, update and repair

- [ ] Detect an existing installation and update in place: binary, service file, restart, health check. Preserve config and database
- [ ] Track the database schema version; back up before migrating and report the migration
- [ ] `repair` path that restores a missing binary, service definition, enabled state, directories or config
- [ ] Checksum for the release binary, verified before installing
- [ ] Repair never deletes or reorganizes user documents
- [ ] Optional NixOS / Home Manager module on top of the existing flake for machines that should be fully declarative. On NixOS, runtime dependencies (poppler, OCR) come from Nix

---

## Phase 9 — Polish from real use

Only problems that show up in actual use: matching tweaks, duplicate handling, filtering, notification usefulness, layout. Do not add features just because the interface makes them possible.

---

## Open questions

- What identifying information does a receipt PDF contain beyond reference, date and amount (payer name, contributor number)? This decides what the print queue can show as "whose it is". Check with `pdftotext -layout receipt.pdf -`
- Default grace period before an unmatched item is eligible for printing, and default retention for printed items
- Whether the user's own guides always carry one of a small set of contributor numbers (needed for the overdue alert)

## Explicitly skip

- Cloud sync, remote or public dashboard, accounts or authentication, REST API for external consumers, plugin system, multi-user support
- General-purpose document management
- Complex analytics, OCR confidence dashboards
- Automatic destructive cleanup
- Anything beyond the INSS guide and receipt workflow. That would be a separate project

## Closing rule

This roadmap is complete when INSS Watcher stays invisible in normal operation, tells the user when something meaningful happens, gives one simple place to inspect, print and clean up, keeps accumulated documents under control, and can be updated or repaired without manual system administration.

At that point, stop. Anything beyond it is a new project.
