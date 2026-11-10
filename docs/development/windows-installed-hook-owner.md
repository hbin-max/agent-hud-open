# Windows 安装版的固定回调程序

## 原因与接入

开源桌面入口原先调用默认 HostOptions，没有指定 canonical_executable，
所以回调状态为 installable=false。正常安装也不能启用回调。

桌面入口现在显式选择 HostOptions::installed()。Windows 适配器只读取当前用户
NSIS 卸载记录 Software\Microsoft\Windows\CurrentVersion\Uninstall\Agent HUD。
目录可由安装器自定义；不从 LOCALAPPDATA、当前工作目录或外部回调输入推断。

只有以下条件同时满足时才向现有回调模块委托固定程序：

- DisplayName 是 Agent HUD，安装目录为绝对路径。
- 当前运行路径就是登记目录内的 Agent HUD.exe。
- DisplayIcon 和 UninstallString 分别指向该程序与该目录的 uninstall.exe。
- 主程序、卸载程序、engine 和 MCP 配套文件都存在。
- 登记路径及解析后的路径均不在 target 编译目录中，大小写变体也拒绝。

没有修改现有回调程序校验、客户端协议或权限判定。macOS/Linux 仍保留原先由
具体产品显式委托固定路径的策略。移动或复制程序、残缺安装或登记失效时关闭入口，
应重新安装或从已登记的安装目录启动；不会自动把副本登记为可信程序。

本接入不主动打开 clientHooks 开关。用户已经保存的开关状态沿用原有运行逻辑。
它也不会代替客户端自身的回调批准步骤，例如 Codex 的“审查钩子”。

## 验证

- 自定义且含中文/空格的登记目录可以通过。
- 副本、陈旧登记、缺少资源、target 目录及畸形路径被拒绝。
- 隔离 home 中生成并写入 Codex 权限回调配置，保留原有无关字段。
- Host 测试全量通过（34 项），原有回调安装测试通过（2 项），Host Clippy 通过。
- 本机安装 0.6.1-local.3 后，设置页的客户端回调开关已可操作，未出现指定安装位置的限制。
- 保留用户关闭状态，“未安装”指回调尚未写入客户端，并不表示软件未安装。
- 隔离配置写入和实际界面验证通过；尚未验证真实 AI 会话的权限请求与回答全链路。

## 独立保存与回退

本改动单独保存于 fix/windows-installed-hook-owner，基于界面方案的 d5f0fbc，
不改动原始 main 或此前的 feat/compact-status-bar。
后续提交给原作者时可独立挑选。撤销对应提交可恢复此前默认不委托的启动策略；
若已经安装了客户端回调，应先在应用中关闭回调，再卸载或迁移程序。
