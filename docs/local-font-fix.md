# Local macOS font repair

> Historical implementation and validation record. The 2026-10-15 integration
> supersedes parts of this implementation; see [current integration notes](local-upstream-sync-20261015.md).

Base: PdfCraft v0.5.0, ef9b553f5505bcd4954ba01d6a896ea2a37efe06.

This is a local modified build, not an official upstream release.

- `PDFCRAFT_UI_FONT` selects an already installed Chinese UI font at runtime.
- `PDFCRAFT_UI_FONT_INDEX` selects its TTC face. This Mac uses PingFangSC-Regular, index 3.
- The face is checked for Latin and missing Simplified Chinese glyphs before use.
- Latin fonts retain their regular/medium/semibold weights. The Chinese fallback precedes Japanese faces in every UI font family.
- macOS restores fractional glyph placement for small Latin text.
- ArtCraft trademark artwork is excluded from this modified binary.
- System font bytes are not copied into this repository, app bundle, or PDFs.

Build input: craft-fonts abb83316d96aa59c1cf64784289e378fe9fa5695 (same as the official release).

The installed bundle sets the two font variables in Info.plist `LSEnvironment`, so Finder launches preserve the repair.

The official signed application is backed up under:
`~/Library/Application Support/PdfCraftLocalFix/backups/PdfCraft-0.5.0-official.app`.

This build is locally ad-hoc signed. It does not retain the official developer signature or Apple notarization. Installing a later official update will replace the patch.

Verified on this Mac (2026-10-11):

- Release app and CLI build completed successfully.
- Font integration suite: 8 reported passed (unavailable optional scripts return early); the new full Simplified Chinese ideograph and Latin coverage test executed and passed.
- Theme integration suite: 3 passed.
- Stock fallback test, without the override: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- Installed binary equals the tested bundle binary, SHA256 `9d60a37228bdb866967fbd9fea86e21ddc0a59cc3bae48eee9c4d3dabcc7fa6f`.
- Local signature verified. Launched through macOS application services, inspected Home and Preferences visually: missing boxes resolved; Latin, numbers and Chinese render correctly. Relaunched from `/Applications/PdfCraft.app` and rechecked.

Build commands (from the source checkout):

```sh
export CRAFT_FONTS_DIR="$(cd ../pdfcraft-build-fonts && pwd)"
export CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
~/.cargo/bin/cargo build --release --locked -p pdfcraft -p pdfcraft-cli
# For patched coverage tests, set PDFCRAFT_UI_FONT and PDFCRAFT_UI_FONT_INDEX
# to the values in the installed app's Info.plist LSEnvironment.
~/.cargo/bin/cargo test --release --locked -p pdfcraft-ui-egui --test fonts --test theme
```

To restore: quit PdfCraft, then replace `/Applications/PdfCraft.app` with the backed-up official app. The user settings are in `~/Library/Application Support/PdfCraft/` and need not be deleted.
