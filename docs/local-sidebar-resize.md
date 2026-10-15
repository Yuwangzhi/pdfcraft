# Local bookmark sidebar resize repair — 2026-10-11

> Historical implementation and validation record. The 2026-10-15 integration
> supersedes parts of this implementation; see [current integration notes](local-upstream-sync-20261015.md).

Installed version: `0.5.0-local-font1-deepseek6-sidebar1` (`0.5.0.9`).

The bookmark search row reserved 32 points for a 26-point clear button plus
an 8-point widget gap. Its 2-point overflow expanded the panel on every frame,
overriding a user's drag until the panel reached its maximum width. The search
field now reserves the button size plus the actual configured spacing.

Validation:

- A kittest pointer-drag regression failed before the change: the left edge
  moved from 980 to 849 after release and 80 more frames.
- The same regression passes in English and Simplified Chinese after the fix.
- All 7 bookmark integration tests pass; formatting and diff whitespace checks pass.
- Pre-push validation (2026-10-12): 1,757 workspace tests pass, 7 are ignored;
  workspace Clippy passes with warnings denied, and all 30 library crates pass
  the WebAssembly check.
- The release desktop build succeeds with the pinned craft-fonts input required.
- The installed application launches, opens PDFs, and displays Chinese/Latin text.
- Native CUA drag attempts did not produce a measurable resize; drag stability
  is verified by the egui pointer-event regression above.

The existing font and translation changes remain in this build. The application
is locally ad-hoc signed, as before. The pre-repair bundle is preserved at
`~/Library/Application Support/PdfCraftLocalFix/backups/PdfCraft-0.5.0-local-font1-deepseek6-before-sidebar-resize.app`.
Quit PdfCraft before restoring that bundle. No PDF contents or private provider
configuration are changed by this repair.
