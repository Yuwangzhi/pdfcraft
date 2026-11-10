# macOS fork integration: PdfCraft 0.20

This branch merges official `v0.20.0` (`2ad7717f87ad6f15b14bdf22cddc1019d6fd0aed`) into the existing 0.19 reading-continuation fork. It retains DeepSeek selection and screenshot translation, locally rendered formulas, Chinese font fallback, default closed right sidebar, native macOS tab gestures, same-path refresh, Finder foregrounding and per-document reading continuation.

The upstream pane-aware tab layout is retained. The main strip continues to remember the original pointer press for native event batches, and sends the resulting drag through the new pane-aware drop handler. Only unused strip space starts native window movement. Translation is settled before document layout, with a single tab strip and mode bar.

Validation targets macOS: the desktop app, CLI, UI, engine and automation packages, formatting, warning-denied Clippy, asset attribution, layers, parity and the WASM compatibility check. Windows installation and execution are outside this update's scope. The installation backup and detailed local validation receipt are kept separately from source control.

The upstream non-UTF-8 output-probe test now checks the macOS filesystem's safe rejection rather than expecting a successful invalid-byte filename. Other Unix targets retain the successful raw-byte-name assertion; every target still verifies that no probe files remain. The native filesystem probe on macOS returned `EILSEQ`.

## Validation result

The completed affected-package run and final UI integration rerun pass 3,894 tests with 10 conditional skips and no remaining failures. The final UI integration run passes 1,660 tests. Formatting, scoped warning-denied Clippy, layers, assets, parity and the 33-crate WASM check pass. Narrow-window menu coverage follows the overflow menu; the rich-copy regression re-aims at the text after the automatic translation sidebar changes the layout.
