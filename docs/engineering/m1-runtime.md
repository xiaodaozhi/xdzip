# M1 产品运行时与 Qt 生命周期

M1 范围以 design/12 为准：产品骨架、双句柄、事件/背压、关闭、三平台最小构建。M2 十万条索引、M3 安全解压和 M5 私测发行门槛不随此阶段完成。

## 已落地的服务

`xdzip-runtime::service` 是产品实现，诊断探针继续独立保留。GUI 只持有 `CommandClient`，专用 Qt 接收线程独占 `EventReceiver`；两个句柄各取一次，cxx 生成 ABI 胶水是唯一 unsafe 边界。源文件与归档快照由单个 Rust 工作线程拥有，不再向 Qt 暴露 Archive 句柄或同步扫描入口。

M1 命令为 Open、Browse、Preflight、CloseSession、Snapshot；现有真实浏览和预检全部接入。每个 accepted 普通请求分配不复用 ID，返回至多一个有界数据事件，再返回恰好一次 Finished；错误直接 Finished。页面携带会话、请求和偏移身份；本阶段单会话、串行 GUI pending 请求，完整 M2 查询代际协议没有伪装成已实现。跨会话请求返回 stale_session。

GUI 普通准入使用 try_lock，不等待队列空间，校验路径 4 KiB、搜索 1 KiB、选择 128 项。拒绝不消耗 ID，Busy 可重试。取消有独立最多 128 项控制通道和自己的 Finished 确认；确认不保证目标尚未完成。关闭保留独立、幂等 RequestId，预留最后一个可用 ID，即使正常 ID 耗尽或队列满也可关闭。

## 有界传输

- 普通 outstanding 最多 128，直到 receiver 取走 Finished 才释放名额。每项预留两个事件位置，终态不被数据挤掉。
- Rust 数据事件累计字符串预算 8 MiB，超过预算暂停发布；取消/关闭可唤醒等待并改发 cancelled 终态。单事件超限返回 resource_limit。
- Qt mailbox 最多 256 个事件、8 MiB 字符串，其中普通数据最多 128 个位置，其余为终态/控制保留；不会为每条 Rust 事件生成一个 Qt callback。
- 一个通知链按每轮最多 32 个事件、约 4 ms 的时间预算 drain，未结束则投递下一轮；单事件处理不可抢占。队列空时不再调度，接收线程由条件变量阻塞。
- 字符串预算不等于 RSS：容器、快照、转换中的单个事件另有空间，但条目和页大小始终有界。当前页仍为 128 项、归档扫描仍为 10,000 项。

控制与普通事件交替取出，避免持续取消确认使普通终态饥饿。递增 sequence 在交付时分配；Qt 丢弃旧序号，控制确认和过期 request_id 不修改当前视图。

## 关闭、销毁与故障

原生窗口关闭和应用 Quit（含 macOS Cmd+Q）进入同一异步关闭路径。停止准入，取消已接收工作，继续 drain 终态，Rust 释放会话后标记工作结束；接收线程 join Rust 工作线程，再收到 ShutdownReady 和 EOF。Qt 接收线程结束且 mailbox 排空后，GUI 才允许退出。GUI 从不 join 等待事件/文件工作的线程。

取消/关闭控制使用只含有界状态操作的短临界区，不受数据队列容量约束。协作取消不能打断操作系统阻塞读，这不是硬实时退出保证。M1 没有输出事务，因此不存在关闭时未提交文件的恢复承诺。

QObject 提前销毁时，关闭 mailbox、唤醒接收线程、停止服务；后台仅持有共享队列和 Rust 接收句柄，不捕获控制器指针。Qt 自动撤销针对已销毁对象的通知。事件消费回调若同步销毁连接也有 QPointer 生命周期防护。放弃接收时不在析构函数 join，工作协作结束；正常 ShutdownReady 才是已 join 的证明。

只读工作处理函数发生可展开 panic 时，边界捕获后废弃整个服务，发送 RuntimeFault，并将未完成请求终结为 runtime_unavailable，不继续使用该会话。不恢复半完成事务；目前没有文件写入。分配失败、进程 abort、poisoned 内部锁等不可恢复错误不承诺发送终态。Qt 对意外 EOF/连接失败统一结束 pending 状态，防止永久忙碌。

## 验证与构建

见 [M1 验收说明](../qa/m1-acceptance.md)。Rust 核心测试不需要 Qt；Qt 窗口、模型、压力和关闭用例在三平台原生构建运行。macOS 构建明确验证 arm64 并部署为含 Qt 的开发 app；Windows 生成部署目录，Linux 验收二进制需同版 Qt。它们不是签名公证或完整首版发行。
