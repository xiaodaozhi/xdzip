# 安全政策

当前最新稳定版为 [`v0.1.0`](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.0)，支持范围以[系统支持与维护政策](docs/engineering/support-policy.md)为准：macOS 14 及以上 Apple Silicon arm64、仍受 Microsoft 维护的 Windows 11 x86_64、Ubuntu 24.04 LTS x86_64 X11。安全维护优先覆盖最新稳定版；alpha、beta、rc、开发构建和超出支持矩阵的系统不承诺单独维护。个人项目不承诺固定响应时限或 SLA。

不要在公开 issue 上传漏洞利用归档、密码、机密路径或用户文件。优先使用仓库 Security 页的 [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new)；入口不可用时通过已有私密渠道联系仓库所有者，不虚构安全邮箱。不要把公开 issue、普通 PR 或 Release 评论当作私密披露渠道。

报告应包括受影响提交/版本、平台、最小复现步骤、预期与实际结果、影响和可再分发的最小样本。可提供脱敏日志、崩溃栈和最小化输入，但不要附带用户文件或凭据。维护者会在可行时确认可复现性、评估影响、修复并补回归测试，再协调披露；报告提交不代表一定接受、一定修复或承诺公开时间。

重点边界：路径逃逸、链接/重解析点竞态、归档炸弹、原生解析器、Qt/Rust 所有权、扩展 IPC、密码生命周期、更新清单与签名、安装助手和第三方依赖。详情见 [文件安全设计](docs/design/07-file-security.md)、[主动更新契约](docs/engineering/software-update.md)和[依赖准入](docs/engineering/dependencies.md)。
