# 实现前接口契约审查

权威接口在 design/03、05、06、08；本页定义冻结检查，不复制另一套签名。M1 内部桥接、双句柄和生命周期已实现，具体边界见 [M1 运行时契约](m1-runtime.md)。查询代际、来源版本检查、创建与解压任务已分别落地，见[浏览契约](readonly-browser.md)、[安全解压](m3-extraction.md)及[创建契约](m4-creation.md)。第三方进程协议已有独立[只读 SDK 试验](sdk-trial.md)，尚未作为稳定 SDK 发布或接入产品；下表规定的是接入门槛。

| 边界 | 开工前确认 | 必测失败行为 |
| --- | --- | --- |
| UI → bridge | 句柄所有权、RequestId、非阻塞提交、关闭时拒绝 | 队列满、重复关闭、对象已销毁 |
| bridge → UI | session/query/generation/revision、批量页面、事件序号 | 旧响应丢弃、整数窄化失败、终态不丢失 |
| runtime → codec | 能力而非后缀、输入 reader、输出 sink、取消令牌 | 不支持特性、CRC 错误、短读/短写 |
| runtime → 安全 I/O | 授权根、映射、暂存、提交许可、提交回执 | 链接竞态、磁盘满、覆盖目标已变化 |
| plugin → host | 握手版本、帧上限、能力、会话生命周期 | 畸形/超长帧、超时、乱序、拒绝取消 |

每个接口说明输入有效性、所有权、线程、资源预算、错误种类、幂等性和取消边界。未知值与零分开；TaskId 与 EntryId 不可混用；选择身份绑定条目而非当前行号。

CLI 与 GUI 共享 Rust 服务，Catalog 提供最多 128 条物理元数据，SelectMembers 在同一不可变会话内解析完整有效路径与 EntryId，再提交现有任务。列表 ID 只属于该次扫描，归档变化后不能作为持久身份。submit_wait 只等待短准入锁，队列容量及关闭拒绝规则不变。GUI 不开放这些内部命令的 QML 编码，CLI 不使用转义展示文本写出文件。

固定宿主元数据使用 open_regular_nofollow，拒绝叶符号链接和重解析点，并与目录授权、大小及身份检查组合。既有用户所选归档读取仍保留自己的路径别名语义。桌面 CLI 请求通过同用户单实例的可选 cli_protocol=1 扩展传送有界请求目录，旧打开协议仍为 1；没有该扩展的实例不能确认 CLI 交接。具体结果、退出码与安装边界见[命令行契约](command-line.md)。

English: CLI catalog/selection commands bind physical IDs and effective paths to one immutable scan, with 128-entry pages and unchanged admission limits. IDs are not durable across archive changes. Fixed host metadata uses no-follow leaf reads plus authorization/size/identity checks, without changing user-selected archive alias behavior. Same-user desktop CLI intents require the optional cli_protocol=1 capability; legacy file-open protocol 1 remains valid and cannot falsely acknowledge unsupported CLI handoffs.

M1 冻结内部首个最小切片；随实现可通过同提交更新内部契约。第三方线协议须两个独立实现互通并完成兼容性套件后才能称 v1 stable；版本号未知必须明确拒绝或按协商规则降级。
