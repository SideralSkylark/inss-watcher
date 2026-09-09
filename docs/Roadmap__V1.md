# INSS Watcher V1 — Usability & Maintenance Roadmap

## Status

The original implementation roadmap is complete.

INSS Watcher now reliably watches for INSS documents, processes them in the background, matches guides and payments, organizes files, and reports failures without requiring manual intervention.

This roadmap is a new phase focused on making the existing tool easier to **understand, manage, maintain, and clean up**.

The goal is not to turn INSS Watcher into a large application. It should remain a small background utility that happens to have a convenient way to inspect and control what it is doing.

---

## Guiding rule

Every new feature should make the existing workflow easier without substantially increasing the cognitive or operational burden of using the tool.

The daemon should remain invisible during normal operation.

When the user wants to know what happened, change a setting, inspect unmatched documents, or clean things up, there should be **one obvious interface** for doing so.

Do not solve every new requirement by adding another CLI command.

CLI commands may remain as implementation and maintenance interfaces, but day-to-day interaction should increasingly happen through a single visual control surface.

---

# Phase 3 — Feedback & visibility

## Match notifications

* [ ] Notify when a guide/payment pair is successfully matched.
* [ ] Keep unmatched documents silent.
* [ ] Keep existing failure notifications.
* [ ] Allow match notifications to be disabled.
* [ ] Include useful context such as contributor, reference period, and matched documents.

Example:

```text
INSS Watcher

Payment matched
Contributor: 123456789
Reference period: August 2026

2 documents organized
```

The notification system should remain deliberately quiet. Detection, OCR, successful individual processing, and "still unmatched" states should not generate notifications.

## Visual control surface

Introduce a small local interface for inspecting and controlling the daemon.

This does **not** need to be a traditional desktop GUI.

Possible implementations:

* TUI
* static local HTML interface
* tiny local web server
* another equally small local presentation layer

Choose the simplest implementation that provides a good user experience.

The interface should present, in one place:

### Overview

```text
INSS Watcher

● Running

Today
  Processed     14
  Matched       11
  Unmatched      2
  Duplicates     1
  Failures       0

Last activity
  17:31 — Payment matched
```

### Recent activity

Show a small, useful history of what recently happened.

For example:

```text
17:31  Matched    123456789   08/2026
16:42  Matched    987654321   08/2026
15:18  Unmatched  123123123   07/2026
14:03  Duplicate  document.pdf
```

### Controls

Provide obvious controls for the operations a normal user might actually need:

```text
[ Rescan ] [ Pause ] [ Resume ]
```

Do not expose every internal operation as a button.

### Configuration

Expose the small set of settings that are useful to change without editing a TOML file manually.

For example:

```text
Watch directory
Output directory

OCR workers
Processing debounce

Match notifications
Failure notifications
```

Configuration should remain editable by file for advanced users.

---

# Phase 4 — Database as a useful source of information

The database already contains useful information, but currently it primarily exists to support the daemon.

Make that information accessible through the visual interface rather than requiring the user to learn increasingly specific commands.

## Documents view

Provide a simple searchable/filterable view of processed documents.

Useful filters:

* Contributor
* Reference period
* Status
* Date

Possible statuses:

```text
Pending
Matched
Unmatched
Duplicate
Failed
```

A typical view might look like:

```text
Documents

Filter: [ Unmatched ▼ ]   Contributor: [           ]

Date       Contributor    Period      Status
------------------------------------------------
09/09/26   123456789      08/2026     Unmatched
07/09/26   987654321      08/2026     Unmatched
03/09/26   123456789      07/2026     Matched
```

The important property is that questions such as:

> "How many unmatched guides does contributor 123456789 have?"

should be answerable by looking at the interface and applying filters, rather than remembering a command.

## Document details

Selecting a document should provide useful information such as:

```text
Contributor:       123456789
Reference period:  August 2026
Status:             Matched
Detected:           09/09/2026 17:30
Processed:          09/09/2026 17:31

Guide
  original filename
  final location

Payment
  original filename
  final location
```

Provide practical actions where appropriate, such as opening/revealing the associated file.

---

# Phase 5 — Document hygiene

The quarantine should remain a **safe holding area**, not an ever-growing garbage pile.

## Explicit document states

Represent document state clearly in the database:

```text
pending
matched
unmatched
duplicate
failed
```

Avoid creating additional states unless a real use case appears.

## Duplicate detection

Add SHA-256 based duplicate detection.

The purpose is to establish a reliable definition of an exact duplicate rather than continually expanding heuristic duplicate rules.

Duplicates should be recorded rather than silently disappearing.

For example:

```text
Status: Duplicate
Duplicate of: <document>
Detected: ...
```

This also makes later cleanup safer.

## Cleanup

Provide cleanup through the visual interface.

The user should be able to see:

```text
Cleanup

Unmatched documents: 12
Duplicates:            5

Oldest unmatched:  03/11/2025
```

Allow cleanup to be filtered by:

* Status
* Age
* Contributor
* Reference period

Before destructive operations, show exactly what will be removed.

For example:

```text
You are about to remove:

8 unmatched documents
older than 90 days

[ Cancel ] [ Review ] [ Remove ]
```

A dry-run concept should exist internally, but the normal user should not need to understand command-line flags to perform cleanup safely.

Automatic destructive deletion should remain disabled by default.

---

# Phase 6 — Installation, updates & repair

The installer should become the complete lifecycle mechanism for the application.

A user should not need to understand systemd, binary replacement, database migrations, or configuration preservation.

## Installation

The existing installer should continue to support a clean installation:

```bash
./install.sh
```

## Existing installation detection

When an existing installation is found, detect that situation automatically.

The installer should be able to:

* Update the binary
* Preserve configuration
* Preserve the database
* Update the systemd service
* Apply required database migrations
* Restart the daemon
* Run a health check

## Repair

Provide a straightforward repair path for broken installations.

Repair should be able to detect and restore expected infrastructure such as:

* Missing binary
* Missing service definition
* Disabled service
* Missing application directories
* Invalid or missing configuration
* Database access problems where safely repairable

Repair must never silently delete or reorganize user documents.

## Database migrations

Track the database schema version.

When a newer application requires a newer schema:

```text
Database schema: 3
Required schema: 4

Applying migration...
Done.
```

Back up the database before migrations where practical.

---

# Phase 7 — Polish based on actual use

This phase exists only for problems that appear during real usage of the new interface.

Possible work includes:

* Improving matching heuristics if unmatched documents accumulate.
* Improving duplicate handling if duplicate documents prove common.
* Improving filtering/search.
* Improving notification usefulness.
* Improving the visual layout.
* Adding a small amount of history or statistics if they prove genuinely useful.

Do not add features simply because the interface makes them possible.

---

# Implementation constraint

The visual interface should remain intentionally small.

There is no requirement for:

* A large desktop GUI
* A frontend framework unless it provides a clear benefit
* A public web server
* Remote access
* User accounts
* Authentication
* Cloud services
* A REST API
* Multi-user support

A local-only interface backed by the existing daemon and SQLite database is sufficient.

The interface should be a **convenience layer over the existing system**, not a second application with its own business logic.

The daemon remains responsible for watching, processing, matching, and organizing documents.

The interface is responsible for making that state understandable and providing safe control over it.

---

# Explicitly Skip

* Cloud synchronization
* Remote/web dashboard
* Accounts or authentication
* REST API for external consumers
* Plugin architecture
* Multi-user functionality
* General-purpose document management
* Complex analytics
* OCR confidence dashboards
* Automatic destructive cleanup
* Expanding beyond the existing INSS document workflow without treating it as a separate project

---

# Closing rule

This roadmap is complete when:

> INSS Watcher remains invisible during normal operation, notifies the user when something meaningful happens, provides a simple visual way to understand and manage its state, keeps accumulated documents under control, and can be installed, updated, or repaired without manual system administration.

At that point, stop.

Anything beyond this should be evaluated as a new project rather than another incremental expansion of INSS Watcher.
