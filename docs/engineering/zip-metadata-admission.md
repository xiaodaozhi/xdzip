# ZIP 元数据依赖准入

日期：2026-10-11。限于首个只读目录扫描，不批准解压、创建或第三方插件执行。

采用 [rawzip 0.5.2](https://github.com/nickbabcock/rawzip)，精确版本和校验和见[锁定清单](zip-metadata-dependencies.json)及根目录 Cargo.lock。关闭默认特性，显式启用 std；无运行时依赖、无 build.rs。上游 MIT 许可全文保存在 [third_party/rawzip/LICENSE](../../third_party/rawzip/LICENSE)，随使用该依赖的发行产物保留。

审阅 README、Cargo 配置和目录读取相关源码。选择理由：可对借用的 Read + Seek 输入逐条读取中央目录，保留物理顺序和同名条目，宿主可限制读取字节及累计名称大小。评估过 zip 9.0.1，其按名称建立索引的读取结构不适合此阶段保留物理重复记录的契约，未引入该依赖。

RustSec 数据库快照 `7eebec69c352c7191b1f13eb95dd510eeca5d1de` 中未找到 rawzip 公告。cxx 的 RUSTSEC-2026-0202 影响低于 1.0.195 的 let_cxx_string!；现有锁定 1.0.205 已包含修复。这里记录的是定向公告核对，不是 cargo-audit 全依赖审计或独立安全审查。

没有启用解码器，没有验证文件内容、CRC 或本地头与中央目录的一致性。未知压缩方法只显示编号；“能够列出”不代表能够解压。单卷 ZIP64 元数据已按 M2 增补准入，验证十万条记录与超过 2^53 的精确声明大小；分卷、编码协商与内容解码仍待后续准入。限额和失败行为见[实现契约](readonly-browser.md)。

[M2 验收（本地／CI 工件）](../qa/README.md)包含真实 ZIP64、非法 locator／尾记录、ZIP64 扩展起始磁盘、名称／显示／读取预算、取消及源变化用例。解析仍由固定 rawzip 0.5.2 承担，没有新增解码或原生库依赖。

M3 负载解码与宿主输出另行登记于 [解压依赖准入](zip-extraction-admission.md)；本页保留 M2 只读阶段的历史范围。
