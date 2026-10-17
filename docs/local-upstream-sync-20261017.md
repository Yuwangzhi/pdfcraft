# Upstream integration — 2026-10-17

This personal translation fork merges upstream
`924c4da4ac2879ac9107c6c82e20ac6be5570f0a` into fork main
`916d3d8f34523b6db2eedd4fd25e777e7464a02a`. The snapshot adds 56 upstream
commits, including tests, documentation and merge commits. It remains based on
the 0.8.0 release family; it is not a new official PdfCraft release.

## Resulting behavior

- Adopt upstream macOS quit protection, including Cmd+Q save confirmation and
  the Touch Bar close handling, while keeping the fork's separate native titlebar.
- Adopt upstream text-box growth, editing of existing text, preserved paragraph
  line breaks, form/page-editing fixes and rendering cache improvements.
- Keep automatic DeepSeek selection translation, screenshot translation,
  formula typesetting, the configurable screenshot shortcut and translation sidebar.
- Keep the persisted closed-sidebar preference, coalesced native tab drag handling,
  canonical-path reopen refresh and protection of unsaved work.
- Keep API credentials, system fonts and personal documents outside the repository.

## Merge decisions

- The page-turning inertia changes are equivalent in both branches; retain the
  upstream naming and the fork's existing behavioral regression.
- Menu shortcut rendering keeps both the configurable translation shortcut and
  upstream's permission-aware selected-text Copy shortcut.
- Use upstream Spanish, Lithuanian and Vietnamese catalog wording, then retain
  every additional fork-only translation/sidebar/reopen key. Recalculate catalog
  hashes in ATTRIBUTION.toml and regenerate ATTRIBUTION.md.
- Take corrected upstream parity records, retaining the local sidebar preference
  evidence and notes. Keep both upstream and fork progress history.
- Keep the newer upstream test coverage for asynchronous opening, focused page
  input drafts, OS-specific shortcut labels and signature-password length.

## Validation

- `cargo fmt --all -- --check`: passed.
- Scoped Clippy with `-D warnings` for the desktop app, CLI, UI, engine and
  automation packages, including all their targets: passed.
- Tests for those five packages: 1,953 passed, 0 failed, 10 ignored across
  89 test and documentation-test targets. This is a scoped macOS run.
- Existing regressions for automatic translation, docked translation layout,
  screenshot shortcut changes, default sidebar behavior and same-path reopen
  all passed. The native titlebar regression passed as well.
- Layering: 37 crates passed. Assets: 263 repository assets, 7 dependency-bundled
  assets and 22 fetched build inputs passed. Parity validation passed.
- `cargo xtask wasm`: all 31 checked library crates passed.

The local checks use the existing optional font input and two Cargo build jobs.
GitHub CI checks the full workspace on the pull request. This source integration
does not claim a new native Windows UI session or a replacement installed Mac app.

## Recovery

The original checkout is kept intact while the merge is prepared in a separate
worktree. The pre-integration source is retained as `backup/pre-upstream-20261017`.
No installed application or personal settings are replaced by this source sync.
