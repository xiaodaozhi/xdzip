# ZIP 元数据依赖准入

本页保留 2026-10-11 首个只读目录扫描的历史准入，不批准解压、创建或第三方插件执行。当前 ZIP 元数据已迁移至[独立 Rust 记录层](independent-rust-codecs.md)，rawzip 仅作为开发测试参照；以下历史选择不代表当前生产依赖。

采用 [rawzip 0.5.2](https://github.com/nickbabcock/rawzip)，精确版本和校验和见[锁定清单](zip-metadata-dependencies.json)及根目录 Cargo.lock。关闭默认特性，显式启用 std；无运行时依赖、无 build.rs。上游 MIT 许可全文保存在 [third_party/rawzip/LICENSE](../../third_party/rawzip/LICENSE)，随使用该依赖的发行产物保留。

历史准入审阅过 README、Cargo 配置和目录读取相关源码。选择理由：可对借用的 Read + Seek 输入逐条读取中央目录，保留物理顺序和同名条目，宿主可限制读取字节及累计名称大小。只读扫描选型时评估过 zip 9.0.1，其按名称建立索引的读取结构不适合保留物理重复记录。后续曾另行引入 zip-rs 9.0.3 用于 AES 写入，未以该写入依赖替换只读物理索引。当前独立实现继续保留这些物理索引契约。

RustSec 数据库快照 `7eebec69c352c7191b1f13eb95dd510eeca5d1de` 中未找到 rawzip 公告。cxx 的 RUSTSEC-2026-0202 影响低于 1.0.195 的 let_cxx_string!；现有锁定 1.0.205 已包含修复。这里记录的是定向公告核对，不是 cargo-audit 全依赖审计。

本元数据适配器不启用解码器，不验证文件内容、CRC 或本地头与中央目录的一致性。未知压缩方法只显示编号；“能够列出”不代表能够解压。单卷 ZIP64 元数据已按 M2 增补准入，验证十万条记录与超过 2^53 的精确声明大小；[分卷主机](archive-opening.md)将受支持卷组转换为单卷逻辑视图，适配器仍拒绝未经转换的多磁盘记录；文件名的 Unicode 扩展及旧编码解析按[独立契约](filename-encoding.md)处理；实际内容解码按[解压依赖准入](zip-extraction-admission.md)及[安全解压契约](m3-extraction.md)独立处理。限额和失败行为见[实现契约](readonly-browser.md)。

元数据回归须包含真实 ZIP64、非法 locator／尾记录、ZIP64 扩展起始磁盘、名称／显示／读取预算、取消及源变化用例。独立实现额外覆盖自解压偏移、尾记录歧义及有限固定种子变异；格式库退出由生产依赖闭包检查保证，不以锁文件中仍有测试依赖判断迁移失败。

M3 负载解码与宿主输出另行登记于 [解压依赖准入](zip-extraction-admission.md)；本页保留 M2 只读阶段的历史范围。
