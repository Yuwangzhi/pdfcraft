# Upstream integration — 2026-10-15

This independent translation fork integrates upstream `00442a0e9beeef5f2c9ad035df72140e419dbe70`
(0.8.0-era main) with the five published custom commits ending at `f5f7613` and the
previously uncommitted sidebar, tab and explicit-reopen work. The upstream snapshot
is pinned; later upstream commits are not implicitly included.

## Integration decisions

- Retain DeepSeek selection/crop translation, formula typesetting, Chinese-output retry,
  configurable shortcut and persistent sidebar. The new Run/Column selection model
  provides selection identity and crop geometry. Temporarily hide the dock inspector
  while translation is open, preserving its saved layout and restoring it on close.
- Adopt upstream accessible tabs, ordering markers and scrolling, while retaining the
  local separate macOS titlebar and press-position anchoring. The upstream implementation
  still fails the coalesced-press/motion regression on this checkout. Keep native window
  dragging in unused chrome and the `tab-move=from,to` control option.
- Keep the explicit persisted `close_right_panel_on_open` preference, separately from
  upstream's remembered last panel close. Honor PDF page layout, suppress automatic
  navigation panes and retain manual panel access.
- Adopt upstream's file watcher and save-conflict handling. Preserve explicit reopen
  refresh even when watching is disabled, canonical-path alias matching, password reuse,
  and failure/unsaved-work retention. Preserve page, zoom, layout, rotation and continuous
  mode; exact within-page scroll offsets are not preserved on explicit replacement.
- Use upstream's Chinese system-font fallback and retain macOS fractional glyph positioning.
  System fonts and API credentials remain outside the repository and application binary.
- Preserve the existing fork branding policy and the licensed STIX math dependency.

## Compatibility regressions found during integration

- The coalesced tab-drag test fails before the ported press-position fix; the native
  launcher explicitly keeps the content below its titlebar and disables background dragging.
- Recent-file opening now uses the same explicit refresh path as file-open/control.
- Selection gesture tests recalculate screen coordinates after the translation sidebar opens;
  comment actions wait for the sidebar layout to settle. The new docking test fails with the
  inspector-suppression change reverted, then passes with it restored.
- Upstream's search-history unit test now checks the physical destination, leaving page-input
  draft/display assertions to its existing viewer integration tests. Navigation deliberately
  preserves focused drafts; the chrome owns idle label synchronization.
- Rail shortcut tests use the harness operating system, and the persisted sidebar preference
  test scrolls its checkbox into view. The signature fixture satisfies the newer eight-character
  minimum. Missing Lithuanian/Vietnamese labels and duplicate Traditional Chinese keys are fixed.
- Eight upstream parity records contained duplicate TOML fields; combine their evidence
  and notes without dropping either entry, so the parity manifest parses again.
- Signature-label catalog values intentionally keep trailing spaces used when concatenating
  their values; these are data, not accidental source whitespace.

## Build

Use the existing optional font checkout; do not copy system fonts into it:

```sh
PDFCRAFT_BUILD_LABEL=deepseek-upstream-20261015 \
CRAFT_FONTS_DIR=../pdfcraft-build-fonts CRAFT_FONTS_REQUIRED=1 \
CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
cargo build --release --locked -j 2 -p pdfcraft -p pdfcraft-cli
```

The old `PDFCRAFT_UI_FONT` override is superseded by upstream's Chinese fallback.
The API key stays in the existing private per-user configuration; it is not migrated
into the portable test app or committed. This is a local ad-hoc signed build.

## Validation

- Scoped desktop/UI/engine/automation tests: 1,799 passed, 10 explicitly ignored,
  combining the full initial sweep with reruns of every failing target and the added tab tests.
  This is not a full-workspace test claim. The macOS installed-font catalog test passed.
- `cargo fmt --all -- --check`; scoped five-package Clippy with `-D warnings`; layers,
  assets (including generated attribution), parity and all 31 `xtask wasm` libraries passed.
- Negative checks: reverting translation inspector suppression fails its regression;
  upstream tab handling fails the coalesced press/motion regression before its local fix.
- Release desktop/CLI build and ad-hoc signature verification passed. An isolated portable
  app with a separate bundle ID displayed Chinese text correctly, moved the second tab before
  the first through native CUA input, and translated a synthetic English sentence through the
  existing DeepSeek configuration. No personal document was used for network QA.
- In the running app, atomically replacing a two-page synthetic PDF with three pages and
  reopening its path kept two tabs, page 2, 125% zoom and a clean document. The rendered
  AFTER page was inspected. Automated native input does not establish physical-mouse behavior.
- Signed portable QA executable SHA-256: `80591e5f5fd7e2f8026b5a5ac207dec2e85a50b1eef35055e74ba2d3c0440292`.

## Recovery

The original dirty source files, binary patch, Git bundle, SHA-256 manifest, installed app
and settings were saved outside the repository in the local PdfCraftLocalFix backup directory.
The previous implementation notes remain historical records, not descriptions of this build.
