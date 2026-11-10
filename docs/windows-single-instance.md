# Windows desktop instance ownership

## Scope

Normal Windows desktops share a per-user kernel presence object and local
show-window endpoint, regardless of executable path, `LOCALAPPDATA` redirection,
or `AGENTHUD_DATA_DIR`. Both identities are versioned protocol names containing
the Windows user SID. The mutex uses the global namespace: different logon
sessions for the same user also share the guard. Different users are independent.

The guard is claimed before Tauri, data migration, collection or HUD creation.
The first process keeps its handle until shutdown. A second process retries the
show request for up to ten seconds while the owner prepares its UI, then exits
after acknowledgement. Errors fail closed; an unresponsive owner never causes
an unprotected second desktop. If the owner disappears during startup, a waiting
launch can claim the released object. Windows releases handles after a crash.

Existing collector/data and hook locks remain in place. The new guard owns the
desktop, not the database. Existing OS adapters and collector behavior on macOS
and Linux are unchanged. Hook/CLI commands are dispatched before desktop claim;
explicit `--demo` keeps its existing exemption and does not collect real data.
Changing the data directory alone no longer opts a normal Windows UI into multi-open.

The IPC endpoint accepts only a bounded `"show"` request, uses the existing
current-user/SYSTEM ACL, and rejects remote clients. No model messages, client
approvals, credentials or config changes are accepted through this endpoint.
Acknowledgement means a request to show Settings was dispatched, not a guarantee
that Windows permitted foreground focus across sessions or integrity levels.

## Compatibility and rollout

This protocol coordinates **updated builds**. Older binaries do not acquire it;
they can still coexist when their data paths differ. Before first using an
updated build, exit all older copies and point shortcuts to one installation.
Installer cleanup, migration of old shortcuts, and diagnostic UI are follow-up
work, not silently performed by this patch. No installed application is replaced
by building or committing this branch.

## Verification

Run on Windows:

```powershell
cargo test -p agent-hud-platform instance::desktop::tests -- --nocapture
cargo test -p agent-hud-host instance::tests
cargo check -p agent-hud-host
node src/tooling/check-platform-boundary.mjs
node src/tooling/check-file-size.mjs
node src/tooling/check-test-paths.mjs
```

The native tests cover exclusivity, simultaneous acquisition, release, startup
handover, an unresponsive owner, user identities, and a copied test executable
with different AppData and explicit data-directory settings. The copied process
must receive a handover acknowledgement, and the original process must observe
the show callback. A separate child is terminated to test crash recovery.

Local validation on Windows, 2026-11-10 (Rust 1.99.1):

| Check | Result |
| --- | --- |
| `cargo test -p agent-hud-platform --locked` | 206 passed, 1 ignored (existing signed-in Antigravity live test) |
| `cargo test -p agent-hud-platform instance::desktop::tests --locked -- --nocapture` | 10 passed, including child helper entry points |
| `cargo test -p agent-hud-host instance::tests -- --nocapture` | 2 passed; desktop host compiled |
| `cargo clippy -p agent-hud-platform -p agent-hud-host --tests --locked -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| Platform, package, file-size and test-path boundary scripts | Passed |

The branch also includes a Windows GitHub Actions workflow for native instance
regressions. Local automated checks do not certify the full packaged GUI,
elevation/session combinations, or macOS/Linux behavior; the release acceptance
checks below remain necessary before distributing an installer.

For release acceptance, use two **updated application builds** in different
folders. Launch via Explorer, shortcut and command line, including rapid
concurrent launches. Expect one normal desktop process, one HUD, and a show
request to the original Settings window. Repeat with redirected AppData,
separate data directories, a minimized window, normal/elevated launches and two
Windows logon sessions. Kill the first process, then verify a fresh launch works.
Verify hook/CLI commands and explicit demos still work. In the single real HUD,
switch Simple → Notch → Simple and check there is no duplicate surface.

## Git rollback

Development is on `fix/windows-single-instance`, based on `b4fcb0e`.
`main` is left unchanged. To return to the original source, first save any local
work and switch to `main`. For a shared branch, use `git revert` on the repair
commit rather than rewriting remote history. Restoring installed binaries is a
separate operation; this patch does not uninstall or overwrite them.

API reference: [CreateMutexW](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw).
