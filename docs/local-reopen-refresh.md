# Local PDF reopen refresh — 2026-10-15

> Historical implementation and validation record. The 2026-10-15 integration
> supersedes parts of this implementation; see [current integration notes](local-upstream-sync-20261015.md).

Installed at `/Applications/PdfCraft.app`, version
`0.5.0-local-font1-deepseek6-sidebar2-tabs3-reopen1` / build `0.5.0.14`.

Opening an already-open filesystem path now focuses its existing tab. If the
bytes changed, it parses the new document before replacing that tab, drops stale
render/text state, and preserves page, zoom/fit, layout and rotation. Pages past
the new end are clamped. The page is restored at its top; exact within-page
scroll offsets are not retained. This is reopen-triggered refresh, not a watcher.

Desktop canonical paths also match symlink/relative aliases. Identical filenames
in different directories remain independent. Unchanged bytes keep the current
view and document id. Unsaved edits block replacement with a translated notice.
Missing/broken replacements retain the old document. Encrypted replacements
reuse the known password; changed passwords prompt while retaining the old tab,
and cancelling keeps it. The same behavior serves Finder OS events, Open, recent
files, drop and existing `ui.open`; control-channel failures now return their
actual error without inferring success from an increased tab count.

Validation:

- 209 tests passed across control, ui, editing, commands, pickers and viewer,
  including new reopen, atomic replacement, missing/broken file, dirty work,
  alias, same-name/different-path and encryption regressions.
- Release app/CLI builds, formatting, whitespace check and UI/desktop Clippy
  (`--all-targets -- -D warnings`) passed.
- wasm32 checks passed for 30 crates; existing web-only dead-code warnings remain.
- The installed application was launched and a synthetic test PDF double-clicked
  in Finder. Its blue BEFORE page was atomically replaced on disk by a green
  AFTER page, then double-clicked again. The real app showed AFTER with exactly
  one document tab and the same 154% zoom. Both states were visually inspected.
- Closed the test tab and restored the user's two PDFs in their original order:
  advisor comments at page 7 and PeakFocus at page 1 (active), side panels closed.
  Reading zoom is 150% for both, approximately the earlier 146% and 151%.
- No research PDF was modified. No Git push or commit was performed.

Installed/staged executable SHA-256:
`f4c1536800331a72229e547f132d3a94af05ea070476fec914f62448fe2b5f6b`.
Both binaries match and the installed ad-hoc bundle signature verifies.

Backup:
`~/Library/Application Support/PdfCraftLocalFix/backups/PdfCraft-0.5.0.13-before-reopen-refresh.app`.
Pre-change settings and source snapshots are in the same backup directory.
Logs and the synthetic fixture are under
`~/Library/Application Support/PdfCraftLocalFix/reopen-refresh-20261015/`.

The existing local font, translation, sidebar and native titlebar patches are
retained. This remains a local build; an official update may overwrite it.
