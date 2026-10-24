# Upstream integration and macOS external opens — 2026-10-24

This personal fork integrates upstream `c894928e` (0.13.0) into fork main
`314abe03`: 1,349 upstream commits since the previous integration. The backup ref
`backup/pre-upstream-20261024` retains the prior source.

## Opening PDFs

Upstream PR #2461 (`9e4031cd`, issue #1928) already fixes external opening in a
running macOS app: an OS open event restores a minimized window and requests
focus. Keep that implementation and its existing regression instead of adding a
competing activation mechanism.

Keep the fork's canonical-path refresh: unchanged files focus their existing tab;
changed files replace it only after successful parsing, retaining page and view
settings. Unsaved work and password/parse failures keep the previous readable
copy. Adapt this to upstream's `OpenOutcome`: a failed refresh returns `Failed`,
so `ui.open` reports its error instead of falsely reporting successful focus.
A new OS-event regression checks foreground commands, selection of the existing
tab, no duplicate tab and preservation of an unsaved bookmark together.

## Integration choices

- Retain automatic selection/screenshot translation, local formula rendering,
  configurable shortcut, docked translation sidebar and Windows console suppression.
- Retain runtime Chinese font fallback, closed-on-open sidebar preference and
  native titlebar separation for tab dragging. Honor upstream's remembered panel closure too.
- Adopt upstream raster hold/release tests and the consolidated `tests/it` layout;
  retain local regressions and adapt bookmark edits to optional page coordinates.
- Keep upstream catalog wording plus fork-only entries, keyed by context and source.
  Adopt the new attribution policy without catalog hashes and keep trademark artwork excluded.
- Restore the production PNG dependency needed by the translation-layout tool.
- Settle automatic translation before canvas layout, so opening its sidebar does
  not leave selection-action hit targets at the prior frame's coordinates. The
  new upstream copy/comment/markup/replace regressions exposed this conflict.
  Wait until pointer movement and the entire click gesture finish before expanding
  the sidebar, including when a press and release arrive in one frame.
- Keep the readable tab and report the failed refresh when its file disappears.
  Adapt the upstream focus-only assertion to the fork's explicit refresh contract.

## Validation

The release app and CLI build and all-target compile checks passed on Apple Silicon.
A negative control removes the OS-open restoration/focus commands; the new
OS-reopen regression then fails at its foreground-command assertion. The exact
source is restored before the scoped suite runs.

Installed native checks use synthetic PDFs in Finder and observe macOS state
before any app interaction can bring its window forward:

- A hidden running app becomes visible and frontmost after double-clicking a PDF.
- A minimized window is restored and the newly opened PDF is the active tab.
- A full-screen window hidden away returns to the foreground, stays full screen,
  and selects an already-open PDF without creating a duplicate tab.
- LaunchServices resolves the PDF to `/Applications/PdfCraft.app`.
- The installed ad-hoc signature verifies and the installed executable matches
  the staged build. The synthetic PDF rendering and Chinese interface were
  visually checked. Existing reading tabs, page numbers and zoom were restored.

The five-package release suite (`pdfcraft`, `pdfcraft-cli`, `pdfcraft-ui-egui`,
`pdfcraft-engine`, `pdfcraft-automation`) passed **2,331 tests, 0 failures, 11 ignored**,
including the selection-action regressions and three UI performance tests.
Scoped all-target Clippy with warnings denied, formatting, assets, layers, parity,
and the 31-crate wasm check passed. The wasm check retains existing target-specific
unused-code warnings; this is not a full workspace or cross-platform runtime run.
Logs,
read-only native-state receipts and synthetic fixtures stay in the local
PdfCraftLocalFix support folder outside the repository. This record does not
claim Windows installation or a remote push.
