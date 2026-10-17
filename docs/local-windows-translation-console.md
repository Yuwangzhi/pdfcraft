# Windows translation console window

Selecting a screenshot translation region in the Windows GUI could open a separate
black command-prompt window until the translation request finished. The page's own
selection rectangle was rendering correctly: the external curl worker was a console
executable launched without Windows console suppression.

## Fix

Translation requests now construct their worker through `background_command` in
`crates/engine/src/translation.rs`. On Windows it uses the standard library's safe
`CommandExt::creation_flags` wrapper with `CREATE_NO_WINDOW` (`0x08000000`).
Both selected-text and screenshot translation use this path. The curl executable,
request limits, cancellation/error handling and piped credential/response transport
remain as before. Other desktop platforms retain their existing command construction.

This follows the documented [Windows process creation flag](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
and [Rust CommandExt API](https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html#tymethod.creation_flags).
It adds no unsafe Rust, dependency, credential copy or new network destination.

## Regression

`translation::tests::translation_worker_has_no_console_and_keeps_stdout` runs on
Windows. The test first starts its own test executable with `CREATE_NEW_CONSOLE`.
A child PowerShell process then queries its actual `GetConsoleWindow` handle and
writes a marker through the response pipe. Both console inheritance and suppression
are therefore observable on service runners. The negative control removes the
creation flag and must observe a console; the normal worker must return handle zero.
A failure to execute the probe is an error, rather than a successful no-window result.

On the user's Windows machine, an independent process-creation probe returned a
nonzero console handle for ordinary startup and zero with console-free startup,
with successful stdout in both cases. A real screenshot-translation request in
the old build also created a curl child followed by its own conhost process and
a new OpenConsole process in the user's interactive session. Screenshots of that GUI
showed the normal blue crop outline, confirming that GUI-only screenshots do not
capture this separate console window. Only synthetic PDF content was used.

Local engine tests after the fix: 291 passed, 0 failed, 3 ignored across 7 targets.
The upstream integration has a separate 1,953-pass scoped macOS baseline. The
Windows-only console regression is exercised by Windows CI.

The packaged Windows app must also complete real selected-text and screenshot
translation before installation. Native app screenshots, build hashes and test
receipts are retained locally outside the repository.
