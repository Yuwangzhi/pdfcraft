# Local fork: upstream 0.19 and reading continuation

This integration merges the official `v0.19.0` tag (`8c572792d6760bdc903d6f9a032e3e38e8f71cb8`) into the existing fork. Selection and screenshot translation, local math rendering, sidebar defaults, native tab gestures, font fallback and same-path reload are retained across upstream's module and translation-catalog refactors.

## Reading continuation

Preferences offers **再次打开文件时，提示继续上次阅读** (`prompt-last-view` in UI automation). Together with the existing last-view preference, it displays a non-modal **上次读到第 N 页** banner when a path-backed document has a saved position different from its opening page. **继续阅读** restores the page, zoom and display. **从当前页阅读** dismisses the prompt and starts a new position.

The last 100 paths are remembered locally. Pending prompts survive regular preference saves without overwriting the previous reading position. New files have no prompt, each tab owns its prompt, canonical aliases share their history, and a shorter replacement clamps the saved page. The feature stores no changes in the PDF itself. It remembers a page, not an offset within the page.

`ui.set key=prompt-last-view value=true` enables the prompt. `ui.command id=view.resume_reading` and `ui.command id=view.dismiss_reading_prompt` resolve it; `ui.state` includes `reading_resume_page` (one-based or null).

## Build and validation

Use the pinned craft-i18n revision in `craft-i18n.rev` via `CRAFT_I18N_DIR`, and the craft-fonts revision in the release workflow via `CRAFT_FONTS_DIR`. Release validation sets both `CRAFT_I18N_REQUIRED=1` and `CRAFT_FONTS_REQUIRED=1`. Existing fork translation preference strings use the same build-time catalogs; no font or translation credentials are committed here.

The integration is validated with the five affected packages (`pdfcraft`, `pdfcraft-cli`, `pdfcraft-ui-egui`, `pdfcraft-engine`, `pdfcraft-automation`), formatting, warning-denied Clippy, assets, layers, parity and WASM checks. The reading-prompt tests cover a real UI click, persistence across restart, pending-prompt autosave, dismissal, changed page counts and independent tabs. Installed-app checks use synthetic PDFs and the opt-in local control channel; original user documents are hash-checked and reading tabs restored.

## Source verification result

The affected-package validation completed with 3,442 passed, 0 failed and 10 conditional skips. The final UI integration rerun passed 1,499 tests; it updates the platform-specific macOS Settings menu label and the new last settings card in existing expectations. Warning-denied Clippy, formatting, assets, layers, parity and the 33-crate WASM check passed. Disabling the new prompt makes all four continuation regressions fail; the production logic was restored before final validation. Installed-app verification is recorded separately in the local release receipt.
