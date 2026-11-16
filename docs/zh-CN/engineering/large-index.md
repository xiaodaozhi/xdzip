# 大归档索引与按需浏览

普通单文件 ZIP/ZIP64 的源码能力上限为一百万条物理记录。其他格式和分卷路径保持各自现有的十万条准入；格式名称不表示所有组合都已扩大上限。目录、重复名称和待检查记录均计入物理数量，隐式导航目录只增加展示行。

## 扫描与存储

ZIP 逐条扫描复用普通 ZIP 解析器，先完成有界的整包旧编码名称采样，再以同一编码决定执行第二遍。两遍合计读取最多 512 MiB，中央目录最多 256 MiB，原始名称累计最多 128 MiB，解码和显示名称合计最多 128 MiB，每个原始或解码名称仍限 4 KiB。损坏尾部、取消或来源变化使整次结果失败，不发布局部索引。

热元数据到达 64 MiB 或超过十万条后，逐条迁入私有磁盘存储，释放已有名称和记录，不同时保留两份完整索引。物理位置表最多一百万个 u64 偏移。磁盘元数据最多 512 MiB，文件名称与内容均不进入诊断字符串。目录树仍受 32768 节点、64 层和 4 MiB 名称预算限制，超过时整棵树不发布，根目录和物理搜索仍可用。

临时文件位于宿主临时目录下随机命名、仅当前用户可读写的 `xdzip-index-*` 目录，通过持有的目录和普通文件句柄操作。正常关闭、查询替换、失败和取消清理已知文件及核对身份后的空目录。崩溃遗留使用与目录随机身份一致的 16 字节标记及排他租约识别；后续创建时有界扫描，七天以上且未使用的已知文件才可清理。链接、身份失配或未知对象保留，不递归删除陌生内容，不恢复或跨启动复用索引。

## 查询与选择

搜索与排序生成一个不可变的完整行序，按每段最多 8 MiB 的排序键写入私有归并段，最多 128 段，临时排序存储最多 512 MiB。终态查询只存行令牌，不保留整套名称副本。比较仍使用原始整数、现有名称／类型规则和稳定物理身份；未知大小排在已知项后，不因降序改变重复项身份顺序。取消检查覆盖扫描、排序段前后和归并，不在比较器内改变顺序关系。

桥接每批最多 128 行、64 KiB 名称。最多 1024 行的查询保留原有完整提交方式，更大的 Qt 模型保留全部逻辑行数，仅按可见位置加载，驻留缓存最多 1024 行。未载入行不能提交条目动作。每个延迟页绑定会话、查询、代际、修订及请求偏移，旧页不能进入新结果。

跨页选择以当前不可变查询中的有界区间表达，最多 4096 段；全选覆盖整个当前查询。离开缓存不改变选择。解压确认前由 Rust 将区间冻结为物理／导航身份，查询变化后仍使用已冻结的身份。超过任务十万条上限的选择明确拒绝，不自动缩小范围。未驻留选择的总大小暂不显示；超出驻留内容的整批剪贴板复制给出资源限制，不隐式复制不完整清单。

## 任务边界

磁盘索引中的普通文件选择可经完整源复核、安全计划、整条目校验和既有授权提交解压。单次冻结计划仍限十万条及 64 MiB 元数据，文件和实际解码字节沿用[解压契约](m3-extraction.md)。目录扩展不能跳过未解析名称；所选链接在此索引配置中拒绝。ZIP 读取器重新检查全部中央目录物理偏移，只保留所选负载定位，保留重叠、局部头、尺寸、CRC、认证及取消检查。

进入磁盘索引的归档暂不支持整体修改或整包测试，不能把空的内存投影当成完整归档。创建、其他格式、分卷与归档重写的原有上限不变。预览另见[内容预览](content-preview.md)。CLI 的物理列举与选择复用同一索引，使用[显式分页参数](command-line.md#归档行为)取得大包的完整物理范围；不分页的响应保留十万条及 16MiB 序列化数据／文本预算，超限整体失败。

## 验证

`scripts/probe-large-index.py` 在 `build/` 生成含中文、重复名和嵌套目录的真实 ZIP64，再调用实际 `xdzip-m2-probe`。探针读完所有查询批次，并单独记录扫描、排序、检索、磁盘字节与平台可用的内存指标。样本生成器不在内存构造百万条文件名数组。范围、失败、取消、清理、过期页和实际所选文件输出按[验收要求](../qa/acceptance.md)执行，结果保留在 issue／本地或 CI 工件。

English: Ordinary single-file ZIP/ZIP64 supports up to one million physical records. Other profiles retain their existing limits. Two streaming passes preserve the archive-wide filename decision. Metadata spills privately at 64 MiB or beyond 100,000 records; bounded external sorting retains only row tokens. Qt preserves the logical count with at most 1,024 cached rows. Pages and range selections bind the full query identity; extraction freezes selectors before confirmation and retains the independent 100,000-record task bound. Disk-index profiles reject whole-archive editing/checking and selected links. Private leased scratch storage is reclaimed on teardown and by bounded, identity-checked seven-day cleanup. Measurements are evidence for their actual machine and workload, not universal performance guarantees.
