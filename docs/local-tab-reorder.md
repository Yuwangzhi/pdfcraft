# Local document-tab reorder — 2026-10-14

> Historical implementation and validation record. The 2026-10-15 integration
> supersedes parts of this implementation; see [current integration notes](local-upstream-sync-20261015.md).

Installed version: `0.5.0-local-font1-deepseek6-sidebar2-tabs3` (`0.5.0.13`).

Hold the left mouse button on a PDF tab in the row below the native titlebar
and drag horizontally. The accent line
marks the insertion position; release to move the tab. Escape cancels, and
holding near either edge scrolls an overflowing tab strip. The document views
move intact, preserving the selected document, zoom and reading state. Home and
Combine files retain their fixed positions. The close button retains its normal
behaviour.

`PdfCraftApp::move_tab(from, to)` and `ui.set` with `key=tab-move` and
`value=from,to` expose the same operation with zero-based final positions.

The first local implementation passed pointer-event tests but failed in the
native app: macOS can batch the left-button press and first pointer movement
before a repaint, so egui's current-position drag hit-test chose another tab.
The corrected implementation locks the source document to the raw left-button
press position and excludes the close button.

The regression must populate `Harness::input_mut().events` directly:
`Harness::event` processes each queued event in a separate frame and therefore
does not reproduce this native batching. The batched-input regression failed
before the fix and passed afterwards.

Version `.12` passed automated native drag checks but the user still observed
whole-window movement with a physical mouse. The macOS launcher placed the
tabs inside a transparent native titlebar (`fullsize_content_view=true`), which
still receives native window dragging. Version `.13` uses a normal, separate
titlebar and explicitly disables full-size content and background dragging.
The application tab strip therefore begins below the native titlebar.

The previous egui window-drag response also covered the entire tab strip,
including the margins above and between tabs. It is now restricted to the
empty area between Open and the right-hand controls. A regression test
reproduced `StartDrag` when pressing above a tab before the fix; the same test
now checks that this margin is inert and the actual empty area remains
draggable. A launcher test checks the native viewport configuration.

Apple's documentation describes the full-size content option and its titlebar
layout implications:
https://developer.apple.com/documentation/appkit/nswindow/stylemask-swift.struct/fullsizecontentview

Validation:

- 118 integration tests passed across ui, control, viewer and pickers, plus the
  desktop titlebar configuration regression.
- Formatting, diff whitespace, and UI/desktop Clippy with warnings denied passed.
- Release desktop and CLI builds passed; installed bundle signature verified.
- CUA automated native left-button drags on the installed `.13` app moved the
  second PDF to the first position and back. The selected document and page
  remained unchanged; screenshots show tabs below the separate native titlebar.
- The user's current three-PDF order, selected review PDF at page 4, closed
  side panels and startup preference were restored after verification. The
  restored fit zoom is 117% with the separate titlebar, previously 120%.
- Physical-mouse confirmation was requested after installation; automated input
  alone is not claimed as confirmation of the user's physical-mouse result.

Installed/staged executable SHA-256:
`9f16a507063af37406833dc05aec96a27ba0166afa83c90d57c7c42a676e162c`.

The pre-change application is preserved at
`~/Library/Application Support/PdfCraftLocalFix/backups/PdfCraft-0.5.0.10-before-tab-reorder.app`.
The preceding `.12` build is also preserved as
`PdfCraft-0.5.0.12-before-native-titlebar-fix.app` in the same backup directory.
This remains a local ad-hoc signed build. Diagnostic event logging is removed.
