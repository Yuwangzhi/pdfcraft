# Local default sidebar preference — 2026-10-13

> Historical implementation and validation record. The 2026-10-15 integration
> supersedes parts of this implementation; see [current integration notes](local-upstream-sync-20261015.md).

Installed version: `0.5.0-local-font1-deepseek6-sidebar2` (`0.5.0.10`).

Preferences → Documents and view now includes “Keep the right sidebar closed
when opening PDFs” (打开 PDF 时默认关闭右侧栏). It is enabled on this Mac.
The persistent preference suppresses automatic comment/bookmark panels and PDF
Initial View navigation panels on each newly activated file. It keeps layout,
zoom and page metadata intact, and users can still open panels manually.

The setting is available through `ui.set` as
`close-right-panel-on-open=true|false`, and through `ui.state` as
`close_right_panel_on_open`. Existing settings without this key keep their old
behaviour. The font, translation and bookmark-resize fixes remain included.

Validation: 143 integration tests passed across ui, control, viewer, pickers,
bookmarks and comments; formatting, diff whitespace and UI-crate Clippy with
warnings denied passed. Release app and CLI builds passed. The installed bundle
signature verifies. The real app checkbox was enabled, a PDF was opened with the
right panel closed, and both the enabled preference and closed panel were
rechecked after quitting and relaunching.

Backup: `~/Library/Application Support/PdfCraftLocalFix/backups/PdfCraft-0.5.0.9-before-default-sidebar.app`.
The old app settings are preserved beside it as
`app-before-default-sidebar-20261013.ron`. This is a local ad-hoc signed build.
