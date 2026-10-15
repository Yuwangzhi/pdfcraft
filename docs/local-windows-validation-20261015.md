# Windows native delivery and CI follow-up — 2026-10-15

The existing PR integrates this fork's work into `Yuwangzhi/pdfcraft:main`.
It does not submit the local translation build to `storytold/pdfcraft`.

## Changes

- Context-only translation tests now explicitly clear `FullOutput.textures_delta` after
  inspecting a frame. There is no renderer in these tests. Dropping unapplied deltas
  triggers egui 0.36's debug assertion, which reproduced in Linux, Windows and FreeBSD
  CI even though the earlier local release-profile tests passed. Layout assertions stay.
- A second complete CI sweep exposed an upstream inertia bug: a fresh swipe at the page
  edge turned the page, but the client-side fling then moved the new page away from its top.
  The pager now owns both the consumed gesture's deltas and residual inertia. A real-frame
  regression forces client-side inertia on every host, including macOS, and reproduces
  the same -195-point offset before the fix. Normal continuous scrolling still glides.
- The Windows dropped-file regression waits for the native open worker to finish instead
  of assuming it completes within three synthetic frames. It still checks both the
  original document and the exact text of the new tab; production loading stays asynchronous.
- Translation credentials follow native per-user paths: macOS retains its existing
  Application Support path; Windows uses `%APPDATA%\PdfCraft\private\deepseek.json`
  (with a USERPROFILE fallback); Linux uses XDG_CONFIG_HOME or `~/.config`. Tests inject
  environment lookups instead of changing the process-global environment. Removing the
  Windows branch makes the configuration-path regression fail.
- The Windows x64 workflow builds the desktop app, CLI and an offline UI test executable
  from the exact PR head. It bundles licence notices and SHA-256 hashes. These are unsigned
  local-fork artifacts, not upstream signed releases. Runtime tests on a user's Windows
  machine can use the test executable without installing Rust or compiler tools.

## Informational dependency maintenance exception

[RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) classifies
`ttf-parser` as unmaintained, not as a reported vulnerability, and lists no patched
version. The pinned `latex-rust 2.1.1` still uses it. This fork reaches that dependency
only in `translation_math.rs` through `MathFont::stix_two_math()`, which parses the
compiled-in OFL STIX Two Math bytes. Uploaded PDFs, installed fonts and user-supplied
font files never enter this font parser. The font hash and licence are in the existing
asset manifest.

`deny.toml` records an exception for that advisory ID with this reason, following the
existing policy's narrow informational exceptions. Other advisory, licence and source
checks remain enabled. This is maintenance debt, not a claim that the dependency can
never have vulnerabilities. Revisit when the math library provides a maintained font
backend; review any new vulnerability advisory separately.

## Native installation and validation procedure

1. Hash-verify the build manifest and the downloaded archive. Preserve any existing app,
   configuration and shortcuts before replacement.
2. Run the bundled offline UI tests on the target Windows computer. Test a portable
   staging copy in the user's interactive session with synthetic PDFs only.
3. Use the opt-in authenticated app control channel to inspect state, render a screenshot,
   select text, verify translation, reorder tabs, replace a PDF at the same path, and
   confirm page/zoom retention. Keep screenshots and receipts outside Git.
4. Install the verified app under the user's local Programs directory; retain a rollback
   copy, create the normal launch shortcut, and verify the installed executable/version.
   Store credentials only in the user's private configuration directory with restricted ACLs.
5. Merge the existing PR only after its required CI jobs pass, then verify remote and local
   refs. Actual machine results belong in the PR verification record and local delivery
   receipt, rather than being inferred from a successful compiler exit.
