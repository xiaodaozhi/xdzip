<a name="xdzip-lz4-压缩流契约"></a>

# LZ4 壓縮流契約

[English](../../engineering/lz4-streams.md) | [简体中文](../../zh-CN/engineering/lz4-streams.md) | 繁體中文 | [日本語](../../ja-JP/engineering/lz4-streams.md) | [한국어](../../ko-KR/engineering/lz4-streams.md) | [Français](../../fr-FR/engineering/lz4-streams.md) | [Deutsch](../../de-DE/engineering/lz4-streams.md) | [Español](../../es-ES/engineering/lz4-streams.md) | [Português (Brasil)](../../pt-BR/engineering/lz4-streams.md) | [Русский](../../ru-RU/engineering/lz4-streams.md)

標準幀配置依據作者的[幀格式 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md)、[塊格式，2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md)及 [XXH32 演算法規格](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md)。產品演算法使用獨立 Rust 實現，參考工具和執行證據留在本地 `build/`，只對自有、可重現的輸入模式進行版本管理。宿主授權和提交邊界見[單檔案壓縮流](single-file-streams.md)。

<a name="xdzip-准入与帧结构"></a>

## 准入與幀結構

標準版本 1 幀驗證保留位、BD 取值 4–7、頭部 XXH32、可選內容尺寸、宣告的逐塊及內容 XXH32，以及結束標記。塊資料長度不得超過宣告的 64 KiB–4 MiB 上限。獨立塊重置歷史，關聯塊僅保留前 64 KiB。原始塊和零長度原始塊同樣執行校驗與計量。外部字典配置和舊式幀不支援，字典宣告不授權讀取其他檔案。

串聯標準幀及中間的 skippable 幀受到物理輸入、任務預算、取消和十萬幀／塊策略的約束。成功的獨立檔案必須包含至少一個標準資料幀，任意尾隨內容和僅有 skippable 幀的輸入不能成功。發現階段最多實際讀取 8 MiB 壓縮輸入以產生 512 位元組內容字首。強識別或已識別的 TAR 損壞不能因失敗降級為其他格式，完整掃描驗證全部幀後才能釋出實際大小和虛擬條目。

<a name="xdzip-块处理与创建"></a>

## 塊處理與建立

長度運算須檢查溢位。匹配偏移必須非零且位於有效歷史內，重疊位元組按遞增順序複製，並滿足宣告的輸出上限。最後一個序列只有字面量，其未使用的匹配半位元組忽略。含匹配的流至少保留五個末尾字面量，最後一次匹配須在塊的解碼末尾前至少十二位元組開始。讀取按有界塊交付，失敗後不可繼續；僅精確驗證塊 EOF 後允許移交歷史。宿主 I/O、取消、超限和不支援配置須保留不同錯誤類別。

已准入寫入器須產生真實 LZ4 匹配，僅在壓縮不能減小時使用原始塊，並提供逐塊與內容校驗和。提供的級別選擇有界匹配工作量，每個級別均須用獨立工具驗證多塊內容。分配前准入幀與匹配緩衝，建立仍執行凍結輸入精確尺寸、宿主暫存、完整驗證及最終源和輸出檢查。普通 LZ4 僅接受一個普通輸入檔案；TAR 組合只有在共享 TAR 解析、寫入及相應自動化覆蓋後才能登記。字尾登記不授予修改、加密或分卷能力。

檢查使用 `format_declared_checks`，因為輸入可省略內容校驗和。不得把無內容校驗和的幀描述為具備內容驗證保證。資源限制、實際平臺驗收、格式有效性和正式發行許可須分別記錄。
