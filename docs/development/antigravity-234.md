# Antigravity 2.34 compatibility and upstream review

Validated on Windows on 2026-11-24. Branch `fix/antigravity-234-quota`, based on the user's custom `4e891910` tree.

## Repair

Backport of the discovery fix from upstream `93ac21c9e89e5ae2e055968c65ef5ba1c4ba26ac` (#139, Jazzen Chen with Claude Fable 5.2), using the existing Antigravity storage descriptors as current upstream #170 does. Antigravity 2.34 does not expose its CSRF token on the language server command line. Read its small daemon registration JSON, match its PID to a running recognized language server, and retain the existing loopback GetUserStatus authentication handshake. An explicit command-line token takes precedence. Invalid, oversized and control-character entries are skipped. Existing descriptors preserve GEMINI_CLI_HOME and desktop/IDE/CLI paths. Credentials are not logged.

Only service discovery and its regression tests changed. The user's HUD settings and visual implementation stay on the custom baseline.

## Verification and deployment

- Service tests including the real signed-in Windows account test: 10 passed.
- Platform tests: 209 passed, 1 intentionally ignored. The ignored live test was run separately above.
- Existing TypeScript tests: 189 passed, 1 skipped.
- cargo fmt, git diff whitespace and source size checks passed. The platform-boundary script still flags three pre-existing conditions in deepseek_plugin; no Antigravity condition was added.
- Produced the normal NSIS package `target/release/bundle/nsis/Agent HUD_0.6.1-local.7_x64-setup.exe`.
- Backed up the four E-drive executables and settings/presentation to `work/before-antigravity-234-20261124`, then ran the NSIS installer with destination `E:\use\Agent HUD` and restarted it.
- Verified installed engine and MCP bytes against packaged resources. Tauri restores the development output's `TAURI_BUNDLE_TYPE_VAR_UNK` marker after bundling; the installed executable has `TAURI_BUNDLE_TYPE_VAR_NSS`. Those three marker bytes are the only main-executable difference. The verification manifest is `work/antigravity-234-install-verification.json`.
- Read the installed app's published snapshot via its official `--mcp` entry. Antigravity became `ready`, notice null, checkedAt `2026-11-24T09:18:42.528882700Z`; all four quota rows had numeric values and valid status. Old transport connections can close during the necessary HUD restart; reconnect the client when needed.
- The build machine's Visual Studio developer shell corrupted the inherited Chinese profile/temp paths. Build-only environment restoration and an ASCII temporary directory resolved this. Bun also needed its executable under the ASCII build directory. Helpers and logs are in the workspace `work` directory, not in source control.

## Upstream changes worth considering next

Review target upstream main `1a8fd246`. The custom branch and that main differ across 988 files with 298 upstream-only commits. Apply focused backports with dependency review.

| Priority | Upstream change | Benefit and compatibility review |
| --- | --- | --- |
| Done | #139 `93ac21c9`, descriptor resolution from #170 | Current Antigravity 2.34 quota discovery, verified above. |
| High | #136 `4b17525a` | Prevent a stale CLI report from hiding a fresh service reading for the same account. Review old account IDs and merge logic before porting. |
| High | #100 `3ee58dbc` | Refresh Codex/Claude MCP registration to the running installation. This machine still has a C-drive MCP copy alongside the E-drive install. Reconcile with the custom conservative-path policy. |
| High | `5af5ebf3` and `cde284fa` | Installer stops its own engine/MCP helpers, handles held files and restores them if install fails. Useful for updates while AI clients hold relays. Review full installer hook dependency and both installation paths. |
| High | #165 `23efc1c6` and #168 `2aa82444` | Live DeepSeek sessions outside the 24-hour snapshot, compressed log recognition, Windows discovery and accurate hookless-client status. Review messaging/listing dependencies. |
| Medium | #135 `0791f7af` and cadence `3edb1df6` | Show last quota reading time and offer an actual refresh action; refresh enabled providers on an interval and after work ends. These improve observability and freshness, but do not prove a fix for the separate Codex HTTP transport failure. |
| Medium | #166 `2e0ee550`, #167 `032bc4c2`, #142 queue fallback | Better current ChatGPT Work/Codex originator resolution, opening a closed thread, reply ownership and persisted reasoning effort. Port the corresponding queue/identity tests together. |
| Conditional | `b61b9502` | Windows full-canvas behavior avoids jumping/blinking with right/bottom HUD edges. Check whether this older custom host has the newer dynamic-depth mechanism before applying. |

Latest upstream also changed the messaging contract: `72930c5f` removed get_message/reply_hint, while #151 adds list_sent_messages and #150 checks conflicting to/reply_to targets. A messaging update therefore needs matching MCP schemas, plugin code and the user's cross-chat communication guidance. Keep these as a separately reviewed migration.

Auto-hide, edge glow and additional appearance changes affect the user's reviewed visual design. The user has requested an assessment of those updates, not their installation in this repair.

Source commits can be reviewed at `https://github.com/jazzenchen/agent-hud-open/commit/<full-or-short-commit>` and the numbered PRs in the same repository. Initial baseline, repair source and installation backup are retained for rollback. Reinstalling the old program is separate from reverting source and should preserve current user data.
