# Windows DeepSeek Harness 插件加载

Windows 桌面版 Harness 的 Node 模块加载器不能直接以盘符路径导入插件。
Agent HUD 原先将 `C:\...\agent-hud-*.mjs` 写入插件登记，导致文件存在但
插件没有运行，设置页面仍可能显示已安装。Windows 现在将绝对模块路径编码成
`file:///` URL，正确转义空格和非 ASCII 字符；其他系统仍保持原路径格式。
Windows 登记的 id 同时改变，使现有 Harness 主机重新加载修正后的条目。

首次启动会复用已有安装检查，在此前用户已安装插件时更新登记；不会主动安装
未被用户启用的插件。设置页保留安装和移除行为，通信仍由用户的设置控制。

本机验证：在正常 Windows 桌面版中，旧登记下心跳目录为空；在文件真实可见后，
将插件入口改成文件 URL 并更改 id，会话心跳文件开始出现。平台插件测试和
Clippy 通过。此验证不代表已完成跨智能体的消息往返。

本次修复独立保存在 `fix/windows-deepseek-plugin-url`，基于
`fix/windows-installed-hook-owner`；损坏数据库和迁移前两个环境的数据各自备份。
