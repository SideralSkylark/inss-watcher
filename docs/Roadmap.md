# INSS Watcher — Minimal-Effort Roadmap

## Status: Closed (Phases 0-2 complete, in use daily)

Per this roadmap's own closing rule: once Phase 2 is done, the roadmap is complete
and anything past it is a new project, not this one. That point has been reached.
This doc is kept as a record of what was done and why — not an active task list.

**Observed result:** ~9-11MB RAM idle, brief spikes (~150MB during OCR) that clear
in under a second, no perceptible impact on a modern machine even under load.
Matches the Phase 0 exit criteria exactly.

---

## Guiding rule

You are exiting this project's "active development" phase. From here, every task must directly remove friction between *file appears in folder* and *file gets organized correctly, invisibly*.

If a task doesn't shrink that gap, it goes in "Explicitly Skip." Don't reconsider those unless something forces your hand.

**Target outcome:** binary is on your personal laptop, runs as a service, uses negligible resources, and you stop thinking about it. Total time budget: a few focused sessions, or hand the checklist below to an AI coding agent (Claude Code) and review the diffs.

---

## Phase 0 — Remove the reason it's not running at all ✅ Done

- [x] **Cap OCR to 1 worker thread by default** (config default, override-able).
- [x] **Debounce file processing.** ~3-5s wait for more events before triggering OCR.
- [x] **Set `Nice`/`IOSchedulingClass` in the systemd unit.**
- [x] **Set up a build pipeline (GitHub Actions) that produces release binaries on tag/push.**

**Exit criteria met:** confirmed via direct observation — idle and under-load resource
usage is imperceptible on both desktop and laptop.

*Note: this phase also surfaced two build-portability issues not in the original
plan, both fixed along the way — worth recording since they'll bite again if the
release pipeline changes:*
- *SQLite was dynamically linked (`rusqlite` without the `bundled` feature),
  which broke on any machine without a matching system `libsqlite3` — including,
  non-obviously, segfaulting rather than failing cleanly when a mismatched version
  was found. Fixed by enabling `features = ["chrono", "bundled"]`, which compiles
  SQLite directly into the binary.*
- *`install.sh`/`uninstall.sh` were added on top of the GitHub Actions binary to
  make Phase 1 installs fully scripted (see below) — not in the original Phase 0
  scope but a natural extension of "you should never run `cargo build --release`
  on the target machine again."*

---

## Phase 1 — Deploy once, correctly, so it disappears ✅ Done

- [x] Install binary + systemd user service on your **personal laptop**.
- [x] `loginctl enable-linger $USER`.
- [x] **Automatic rescan on daemon start.**
- [x] *(added)* `install.sh` / `uninstall.sh` scripts — download the release binary,
  install to `~/.local/bin`, write and enable the systemd unit, create a default
  config, and enable linger, in one command. Makes "deploy on a new machine" a
  single script run instead of the manual steps below.

**Exit criteria met:** reboot, do nothing, drop a PDF in the watched folder — it
gets organized without touching a terminal.

---

## Phase 2 — Trust it without checking on it ✅ Done

- [x] **`doctor` command** — daemon up, config valid, folders exist, OCR available,
  DB reachable, pending count, recent failures, all in one glance.
- [x] **Desktop notification on failure only.** Success stays silent, permanently.

**Exit criteria met:** zero signal in normal operation; a failure produces exactly
one notification.

---

## Phase 3 — Only if pending pile actually grows in practice (still wait-and-see)

Not started. Per the original rule: do NOT start this speculatively. Revisit only
if real usage over the coming weeks shows an actual problem.

- [ ] Loosen matching heuristics slightly (prefer pending over wrong match, but reduce unnecessary false negatives).
- [ ] Duplicate detection via SHA-256 hash (protects idempotency, cheap to add, low maintenance once done).

---

## Explicitly Skip (for now — revisit only if circumstances change)

- `setup` command beyond `install.sh`/`uninstall.sh` above.
- Logging overhaul — current logs + Phase 2 notifications are enough.
- Monthly stats, OCR confidence scoring — polish, not friction removal.
- Everything in the original "Out of Scope" list (GUI, web dashboard, cloud sync, REST API, plugin system, etc.) — still correctly out of scope.
- **Any broader expansion of the project's scope** (generalizing beyond INSS
  guides/receipts, new platforms, etc.) — deliberately deferred. If it happens,
  it's a new project with its own roadmap, not an amendment to this one.

---

## Closing note

This roadmap is done. The only open thread is Phase 3, and it stays closed until
observed pending-document buildup justifies it — not before. Check back in a few
weeks with real usage data, not on a schedule.
