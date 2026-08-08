# INSS Watcher — Minimal-Effort Roadmap

## Guiding rule

You are exiting this project's "active development" phase. From here, every task must directly remove friction between *file appears in folder* and *file gets organized correctly, invisibly*.

If a task doesn't shrink that gap, it goes in "Explicitly Skip." Don't reconsider those unless something forces your hand.

**Target outcome:** binary is on your personal laptop, runs as a service, uses negligible resources, and you stop thinking about it. Total time budget: a few focused sessions, or hand the checklist below to an AI coding agent (Claude Code) and review the diffs.

---

## Phase 0 — Remove the reason it's not running at all (do this first, blocks everything else)

This is the actual root cause: build cost + runtime resource cost. Nothing else matters until this is fixed.

- [x] **Cap OCR to 1 worker thread by default** (config default, override-able). This alone likely fixes most of the CPU/RAM spike.
- [x] **Debounce file processing.** On directory event, wait ~3-5s for more events before triggering OCR, instead of processing each file the instant it appears. Prevents burst load when multiple PDFs land at once.
- [x] **Set `Nice`/`IOSchedulingClass` in the systemd unit** so the daemon never competes with foreground work, regardless of what the OCR engine does internally.
- [x] **Set up a build pipeline (GitHub Actions) that produces release binaries on tag/push.** You should never run `cargo build --release` on the target machine again. Download the binary, done.

**Exit criteria:** you can watch `htop`/Activity Monitor while it processes a batch of PDFs and it doesn't register as "something is happening" on your laptop.

---

## Phase 1 — Deploy once, correctly, so it disappears

- [ ] Install binary + systemd user service on your **personal laptop** (not the old desktop).
- [ ] `loginctl enable-linger $USER` so it survives logout/reboot without you starting it manually.
- [ ] **Automatic rescan on daemon start** — so restarts/crashes/reboots never require you to run `ctl rescan` by hand.

**Exit criteria:** reboot the laptop, do nothing, drop a PDF in the watched folder, it gets organized without you touching a terminal.

---

## Phase 2 — Trust it without checking on it

You said you don't want to invest much more time — this phase is cheap and buys you the ability to *not look*.

- [ ] **`doctor` command**: one command, answers "is everything fine" in one glance (daemon up, config valid, folders exist, OCR available, DB reachable, pending count, recent failures). Run it maybe once a month.
- [ ] **Desktop notification on failure only.** Success stays silent — permanently. Failure pings you once. This replaces "remembering to check logs" entirely.

**Exit criteria:** you get zero signal in a normal month, and if something breaks, you get exactly one notification telling you what to do.

---

## Phase 3 — Only if pending pile actually grows in practice (do NOT do this speculatively)

Wait and see after Phases 0-2 run for a few weeks. Only touch this if you observe real problems.

- [ ] Loosen matching heuristics slightly (prefer pending over wrong match, but reduce unnecessary false negatives).
- [ ] Duplicate detection via SHA-256 hash (protects idempotency, cheap to add, low maintenance once done).

---

## Explicitly Skip (for now — revisit only if circumstances change)

- `setup` command — only useful if you reinstall from scratch on a new machine again; one-off manual config is fine for a single personal deployment.
- Logging overhaul — current logs are enough if Phase 2 notifications work.
- Monthly stats, OCR confidence scoring — polish, not friction removal.
- Everything in the original "Out of Scope" list (GUI, web dashboard, cloud sync, REST API, plugin system, etc.) — still correctly out of scope.

---

## How to execute this with minimal time investment

1. Do **Phase 0** yourself (or with an AI pairing session) — it's the part that most benefits from you actually testing on real hardware and watching resource usage.
2. Hand **Phase 1 + 2** to an AI coding agent (e.g. Claude Code) as a self-contained task list — these are mechanical (systemd unit, one CLI command, one notification call) and reviewable in a single diff.
3. Ignore **Phase 3** until it's actually justified by observed pending-document buildup — don't pre-optimize matching for a problem you haven't seen yet.

Once Phase 2 is done, this roadmap is complete. Anything beyond it is a new project, not this one.
