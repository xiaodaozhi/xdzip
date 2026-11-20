<a name="xdzip-自有-windows-zip-自解压包"></a>

# 自有 Windows ZIP 自解壓包

[English](../../engineering/sfx-creation.md) | [简体中文](../../zh-CN/engineering/sfx-creation.md) | 繁體中文 | [日本語](../../ja-JP/engineering/sfx-creation.md) | [한국어](../../ko-KR/engineering/sfx-creation.md) | [Français](../../fr-FR/engineering/sfx-creation.md) | [Deutsch](../../de-DE/engineering/sfx-creation.md) | [Español](../../es-ES/engineering/sfx-creation.md) | [Português (Brasil)](../../pt-BR/engineering/sfx-creation.md) | [Русский](../../ru-RU/engineering/sfx-creation.md)

首版 SFX 配置僅支援 Windows x64，使用明確的 `.sfx.exe` 建立字尾。它把無加密、單卷 Store/Deflate ZIP 與專案編譯的 `xdzip-sfx.exe` 解壓程式封裝。建立時絕不執行該程式。通用 `.exe` 不進入建立、拖放或系統關聯登記。

宿主僅從應用目錄，或私有 `cli` 目錄的共享父目錄讀取解壓程式。不跟隨連結，限制普通檔案大小，並驗證 PE32+、x64、節表範圍和自有版本標記。ZIP 結束記錄的註釋包含 64 位元組版本化封裝，記錄解壓程式長度、ZIP 長度及前面所有位元組的 SHA-256。執行時先驗證封裝，再使用共享 Rust ZIP 與提取契約。這能發現損壞和錯誤封裝，不證明發布者身份，也不認證任意可執行程式碼。

獨立解壓程式不依賴 Qt 或已安裝的 xdzip。Windows 系統對話方塊選擇已有目標目錄和明確的衝突策略，隨後顯示進度與取消。歸檔操作仍由 Rust 在介面執行緒外執行，複用已有授權、凍結身份、資源預算、逐檔案校驗、暫存、衝突處理和清理。取消保留已提交檔案，清理未完成暫存。成功或失敗均不開啟或執行解壓檔案，不呼叫指令碼，不安裝軟體，不提權，不修改關聯。

程式另提供 `--test`（只驗證、不寫入）和 `--extract-to <已有目录> --conflict skip|rename|overwrite` 自動化入口。錯誤封裝和不支援或加密的 ZIP 配置在輸出前失敗。不支援載荷密碼、分卷、自定義啟動命令、靜默安裝或自動執行可執行檔案。生成檔案未簽名，簽名和釋出者信譽屬於獨立發行事項，遵循 [PE/Authenticode 規則](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)。

桌面繼續使用 Qt Quick。獨立 SFX 的系統對話方塊屬於 [ADR-013](../design/architecture-decisions.md#xdzip-adr-013-%E7%8B%AC%E7%AB%8B-windows-sfx-%E5%AE%BF%E4%B8%BB) 登記的有限架構例外。
