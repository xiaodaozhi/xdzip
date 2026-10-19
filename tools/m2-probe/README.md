# M2 元数据与查询测量

先用 `python3 scripts/generate-m2-fixture.py --output build/m2-flat.zip` 生成十万条项目自有 ZIP64，再执行 `cargo run --release -p xdzip-m2-probe -- build/m2-flat.zip`。

探针读取实际归档、建立真实索引、排序并取完所有传输批次、搜索末尾记录；不修改输入，不构造假元数据替代扫描。JSON 记录实际耗时、批次数和 Linux 进程峰值 RSS；其他平台的 RSS 留空，不伪造测量。共享 CI 耗时仅作证据，不作为稳定参考机性能承诺。
