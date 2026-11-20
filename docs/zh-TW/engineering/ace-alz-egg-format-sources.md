<a name="xdzip-acealz-与-egg-的资料和许可门槛"></a>

# ACE、ALZ 與 EGG 的資料和許可門檻

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

本文說明 xdzip 對 ACE、ALZ 和 EGG 格式的支援範圍，以及實現所參考的公開技術資料與許可限制。參考資料保留在 `build/toolchain/references`，不作為產品依賴。

<a name="xdzip-ace"></a>

## ACE

- 技術說明：Marcel Lemke，《Technical information of the archiver ACE v1.2》，來自 `wdlkmpx/unace`，SHA-256 為 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`。
- 獨立實現：`droe/acefile`，提交 `11ed076ae19e8228148731cfbc05f019a395b3b9`，BSD-2-Clause 實現許可 SHA-256 為 `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`，實現檔案 SHA-256 為 `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`。
- 該實現說明它能讀取 ACE 1.0 和 2.0，包括 LZ77、DELTA、EXE、PIC、SOUND、密碼和分卷。許可只覆蓋 Python 實現，並明確不對 ACE 技術本身主張權利。

技術資料足以定義塊頭、CRC、儲存成員和舊 LZ77 家族，但不能直接等同於 ACE 技術的商業許可。實現還引用 WinAce 與 unace，不能把引用實現本身當作獨立互操作樣本。產品決定現已永久放棄 ACE，不實現或登記 ACE，也不增加 ACE 字尾、探測器、MIME 或關聯入口。

<a name="xdzip-alz"></a>

## ALZ

- 格式規範：`alkegi/docs`，提交 `36230c34d3056604cc63519e9855a79e894496b7`，CC0 許可 SHA-256 為 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`。
- `alz.md` SHA-256 為 `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`。
- 資料定義 ALZ 和本地檔案簽名、可變寬度尺寸、CRC-32、UTF-8/CP949 名稱、Store、原始 RFC 1951 Deflate 和 ALZ 改造的 BZip2 流，也定義 PKWARE 傳統加密和分卷佈局。

ALZ 已在 xdzip 中准入保守的只讀產品切片：普通單卷、無加密的 Store 和原始 Deflate，要求準確 CRC、尺寸和物理結束。codec、runtime、CLI 和關聯入口均有自動化驗證。改造 BZip2、ZipCrypto 和分卷需要獨立完整樣本以及單獨的資源和安全檢查。CC0 文件本身不能證明所有實現細節都沒有第三方專利或商標義務。

<a name="xdzip-egg"></a>

## EGG

- 公開格式資料：`alkegi/docs`，同為 `36230c34d3056604cc63519e9855a79e894496b7`，使用上面的 CC0 許可。
- `egg.md` SHA-256 為 `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`。
- 參考模組：`dterracino/UnEgg`，提交 `f21e653b02f7ab467bbc16af879e68a48dabcb2b`，EGG 規範 PDF SHA-256 為 `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`。
- UnEGG 原始碼包許可 SHA-256 為 `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6`，模組包許可 SHA-256 為 `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0`。

UnEGG 許可寫明商業使用需要 ESTsoft 批准，模組可在非商業條件下使用和分發，並禁止用模組開發 EGG 壓縮器，同時保留 ESTsoft 的全部權利。因此只能把它作為測試參考，不能複製、連結或作為產品依賴。

線格式與受限模組在技術上可以分離。xdzip 現在已獨立實現並准入無加密、非固實普通單卷 Store/raw Deflate 切片，codec、runtime、CLI 和關聯測試覆蓋準確塊尺寸與 CRC-32。標準 BZip2 和文件定義的 LZMA 封裝仍需獨立完整樣本和複核。AZO、AES/LEA、ZipCrypto、分卷和固實配置繼續排除在目前產品邊界外。

<a name="xdzip-必要实现门槛"></a>

## 必要實現門檻

1. 資料繼續放在產品樹外，使用前核對固定雜湊。
2. 每個准入方法都要有獨立完整樣本。只有頭部或 Store 樣本不能證明方法互操作。
3. 容器、runtime、CLI 和桌面入口必須一起接入，未知方法可見但不能獲得載荷許可權。
4. 密碼、分卷、固實和特殊物件必須分別完成資料、安全與取消測試後才可啟用。
5. 在改變公開能力登記前，把法律結論和樣本身份記錄到 issue。
