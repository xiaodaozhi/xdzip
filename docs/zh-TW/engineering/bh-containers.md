<a name="xdzip-bh-容器与准入的-blakhole-配置"></a>

# BH 容器與准入的 BlakHole 配置

[English](../../engineering/bh-containers.md) | [简体中文](../../zh-CN/engineering/bh-containers.md) | 繁體中文（台灣）

`xdzip-codec-bh` 從主機提供的源讀取順序 BlakHole（`.bh`）成員。產品只准入保守配置：要求 `BH 05 07` 字首、實測固定欄位、普通檔案後設資料、準確的壓縮和解碼長度，以及成員宣告的 IEEE CRC-32。成員按物理順序保留，重複名稱仍是不同記錄，凍結行身份獨立於顯示名稱解碼。

目前唯一準入的載荷方法是方法 8，即 raw RFC 1951 Deflate。共享有界 Deflate 讀取器要求碼流完整結束並準確消費壓縮區間；尾隨位元組、截斷、解碼長度不符和 CRC 不符都會使載荷任務失敗。未知方法保留為帶警告的可見記錄，不能解壓，也不能計入已驗證內容。

目前准入的頭部佈局是五位元組固定字首，隨後依次為小端 16 位頭部長度、常量位元組 `25 d8 0c 00`、方法、四位元組 DOS 形狀時間戳、壓縮長度、解碼長度、CRC-32、屬性 `0x20`、五個零保留位元組、四位元組檔名長度和檔名。頭部長度必須等於 `37 + 文件名长度`；成員連續排列，最後一個載荷必須準確結束於物理 EOF。不接受歸檔尾部、目錄記錄、自解壓字首、重新同步、填充或鄰接檔案掃描。

BH 沒有公開且持續維護的正式規範。格式觀察固定使用 `ashang/libxad` 的 `support/formats/BLAKHOLE.TXT`（revision `dbe67a865207e49cd917ad9170a11b9eaa83998a`，SHA-256 `a77d570bba1ceafc443fa2404dfaccc8d8872a6a`）和 Hawkynt/CompressionWorkbench 的 `docs/BLAKHOLE-ON-DISK.md`（commit `ab61c59f789d82c628d3c5b4bac264afbdc0c7a7`，SHA-256 `2b7572396683e96f8b5a47ff0b3f0dde2de4b839`）。後者只有一個帶來源記錄的五成員樣本，不能證明 Store、加密、目錄、固實或分卷行為。

IZArc Command Line Add-On v1.1 作為僅構建期參考儲存。官方下載公佈的 MD5 是 `9002c7634768e5336244f9accb2223b0`，本地 `IZArcCL.exe` 的 SHA-256 是 `959050AFC513C64FD61F9EE15837054D0FAEE5EFB98C9107B4DC5D9A528A62D4`。目前環境中，解包得到的 `IZARCC.exe` 和 `IZARCE.exe` 都在產出結果前以 Windows `0xC0000005` 退出，因此 Store、壓縮級別、密碼或提取能力均不以該工具作為證據。參考工具只保留在 `build/toolchain`，不進入產品依賴。

BH 僅支援讀取，不支援建立、修改、加密、分卷，也不宣告專用 Linux MIME。格式登記和[歸檔檢查契約](archive-checking.md)描述這條精確邊界，`.bh` 字尾只是入口候選，完整內容校驗仍以實際位元組為準。
