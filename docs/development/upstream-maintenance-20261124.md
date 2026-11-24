# 定制版维护更新移植记录（2026-11-24）

本次从 `bffb3861`（已修复 Antigravity 2.34 的定制版）建立独立分支
`feat/upstream-maintenance-20261124`。核对的上游快照是 `1a8fd246`。
采用分项移植与旧接口适配，没有合并上游全部代码。

## 移植范围及来源

| 项目 | 上游提交 | 当前版本的接入 |
| --- | --- | --- |
| Antigravity 旧 CLI 额度让位于实时服务 | `4b17525a`（#136） | 同账户、同邮箱的过期 CLI 记录，仅在服务成功提供额度后移除；其他账户和服务失败时的缓存仍保留。 |
| Codex / Claude MCP 路径跟随当前安装 | `3ee58dbc`（#100） | 仅针对已登记 Agent HUD 的客户端；用客户端自己的 CLI 重新登记。没有 CLI 时保留旧配置并记录错误。原有 DeepSeek 标记块与文件型客户端的保守修复策略保留。 |
| Windows 更新时的占用处理 | `5af5ebf3`, `cde284fa` | NSIS 只停止当前安装目录的 engine、MCP 和 `--mcp` 中继；被占用的辅助程序先改名，安装失败或取消时尝试放回。 |
| DeepSeek 实时会话与 Windows 发现 | `23efc1c6`（#165）, `2aa82444`（#168） | 将插件已报到但尚未进入快照的会话加入可寻址列表；保留 `session-` 前缀；兼容 DSH_HOME 的大小写、空白和反斜杠形式的 `~`，发现 `.jsonl.zstd` 日志；无 hooks 客户端不会因 hooks 缺失误报。 |
| 额度更新时间与刷新 | `0791f7af`（#135）, `3edb1df6` | 展开面板页脚显示最近读取时间，并通过现有 `refresh` 请求重新读取。调度为空闲 180 秒、工作中 120 秒，各加 0–60 秒分散；结束回合后合并等待 5–15 秒，保留最小 60 秒间隔、失败退避与 Retry-After。 |
| Codex Work 身份、回复与推理强度 | `2e0ee550`（#166）, `032bc4c2`（#167）, `d3ca858b`, `4004bd8a`（#142） | 解析字符串或对象形式的回合元数据；查找快照外仍打开的线程；拒绝向已归档、删除或子代理线程误回复；保存新线程有效推理强度；stdin EOF 后等待 app-server 刷盘；Windows 队列可唤醒桌面线程。Unix 桌面入口明确不可用时才回退队列，不对超时或不确定投递重发。 |

## 保持兼容的内容

- 保留定制悬浮栏的整体缩放、高度、圆角、AI 显示选择、灰度数字和透明度、短周期优先及低长周期轮显；本次未修改 Bot 动画和轮廓灯。
- 保留 `get_message`、现有 `send_message` 的 `state` 回执结构和权限开关；未引入上游删除旧接口或模型列表体系的改动。
- 回复必须属于接收该消息的客户端和会话；`to` 与 `reply_to` 指向不同对象时明确拒绝，避免旧版允许的错投递。
- 刷新按钮表示已请求读取；不能把按钮停止旋转或已有缓存误认作所有供应商都已成功读取。
- 当前使用原来的中英文文本体系，没有随页脚移植整个新版多语言架构。

## 验证及边界

Windows x64 执行：完整 Rust 工作区 Release 测试、TypeScript 类型检查与测试、MCP 中继测试、安装器脚本测试、DeepSeek 插件测试，以及文件大小和包边界检查。
详细最终结果与安装后核验记录保存在本机 `work/maintenance-*.log`、`work/maintenance-install-verification.json`。
真实跨 AI 模型往返、macOS / Linux 平台构建，以及所有第三方客户端版本组合不因夹具测试通过而视为已验证。

原分支已存在的 3 条平台边界检查告警位于 `deepseek_plugin/mod.rs` 和 `deepseek_plugin/tests.rs`，本次未增加这类条件。
`check:clients` 输出的是原项目累计硬编码计数报告，不代表每条都是本次新增问题。

## 构建、安装与回滚

使用项目原有的 Tauri → NSIS 流程，版本配置见同目录的 JSON 文件。Windows 构建需要 VS C++ 工具链、Rust、Node/pnpm 和 Bun；Unicode 用户目录下 Bun 的编译器可使用 ASCII 路径的同一程序副本。

```powershell
pnpm install --frozen-lockfile
pnpm --filter @agent-hud/desktop exec tauri build --bundles nsis --config '<本文件目录>/upstream-maintenance-20261124.json'
```

本机目标安装目录为 `E:\use\Agent HUD`；源代码位于
`C:\Users\黄斌\Documents\Codex\2026-11-10\b\worktrees\upstream-maintenance-20261124`。
覆盖前备份二进制、显示设置和 Codex / Claude MCP 配置；原用量数据库不删除、不合并。
本机备份目录由 `work/maintenance-backup-path.txt` 记录，不含在 Git 仓库内。

代码可回到 `bffb3861` 分支或回滚本分支各项提交。软件可重新运行保留的 `0.6.1-local.7` 安装包，并在关闭 HUD 后按备份清单恢复配置。
回滚前保留更新后产生的数据；不要使用 `git reset --hard` 清除尚未提交的用户修改。
