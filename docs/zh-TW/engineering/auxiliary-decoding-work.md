<a name="xdzip-算法辅助工作计量"></a>

# 演算法輔助工作計量

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/auxiliary-decoding-work.md) | [한국어](../../ko-KR/engineering/auxiliary-decoding-work.md) | [Français](../../fr-FR/engineering/auxiliary-decoding-work.md) | [Deutsch](../../de-DE/engineering/auxiliary-decoding-work.md) | [Español](../../es-ES/engineering/auxiliary-decoding-work.md) | [Português (Brasil)](../../pt-BR/engineering/auxiliary-decoding-work.md) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

解壓和檢查分別累計檔案解碼位元組、實際輸出與演算法輔助工作。PPM 模型初始化、容量重啟、控制及引數解碼等輔助工作不是檔案內容，不能增加 `decoded_bytes`、`written_bytes`、檔案進度或壓縮膨脹比，也不能產生預計費交付額度。任務快照不新增面向使用者的輔助計數字段。

`DecodedSink::charge_auxiliary(bytes)` 只報告已經執行的輔助工作，預設實現不改變任何檔案計數。演算法先用自己的剩餘預算準入未來工作，執行後再結算實際數量，包括失敗和取消前已經完成的部分。未執行的預留不報告為實際工作，已執行工作不因錯誤、模型清除或密碼重試退款。RAR 的私有總工作預算繼續約束檔案解碼與輔助工作之和。

<a name="xdzip-隔离边界"></a>

## 隔離邊界

- worker 協議 `AUX_WORK=24` 攜帶唯一小端 `u64` 數量，不附帶內容。每包必須為 1..65,536，幀必須精確結束，宿主成功計量後回覆空 `ACK`。空數量、截斷、尾隨欄位、超量或非法操作階段均拒絕。
- 該訊息僅允許 RAR 的負載解碼，不能用於後設資料掃描、建立、修改或其他格式。CXX 同步傳遞數量，不保留 Rust 指標；仍有的源和輸出切片只在呼叫期間借用。C++ 異常在回撥內對映為穩定錯誤，Rust unwind 不跨 CXX，abort 或程序退出使未完成輸出失敗。
- Native 負載例項單獨累計最多 20 GiB 輔助工作，跨條目、worker 重啟和密碼更換保留。它在呼叫宿主 sink 前記錄已執行數量；sink 拒絕不會使後續重試獲得退款。`AUX_WORK` 不增加 `WorkerCredit`。
- runtime 解壓和檢查任務再獨立累計最多 20 GiB 輔助工作，計量物件位於條目及密碼重試迴圈外。此額度與實際檔案位元組和輸出限額分別檢查。超限或取消返回失敗，不出版未完成的檔案，也不依靠更改公開計數來規避限制。
- `WORK=17` 繼續表示已經解碼的檔案位元組。只有其成功准入能為同一個有效工作程序的 `PRECHARGED_DATA=23` 提供一次性交付額度，交付前扣減，重啟即清零。RAR、7z 與 CAB 使用該路徑，掃描與建立不能使用。輔助數量不能用於支付這類檔案輸出。

高壓縮率准入使用凍結的宣告尺寸與 packed/整包尺寸，不引用輔助計量。執行期檔案解碼和輸出限額仍獨立成立。一個 87 位元組檔案執行數 MiB 模型初始化後，檔案計數仍只反映其實際 87 位元組內容。

<a name="xdzip-必须保持的验收条件"></a>

## 必須保持的驗收條件

- 正常小檔案在 64 KiB 以上輔助工作及模型初始化後可以完成解壓和檢查，檔案計數精確，最終內容校驗成立。
- Native 和 runtime 分別驗證實際重複輔助包到達 20 GiB 後拒絕；密碼重試和 worker 更換不重置累計成本，不出現未完成檔案的提交。
- 偽 worker 的其他格式、後設資料階段、空包、超包、尾隨欄位和僅以輔助工作申請預計費輸出均失敗。
- 正常 `WORK` 可支付同一 worker 的 RAR 預計費輸出，額度只能消費一次；錯誤或重啟後不能複用。
- 協議夾具只驗證狀態機和計量。完整 Rust RAR worker、runtime 業務鏈、三平臺和桌面驗收分別儲存實際證據，不由協議夾具成功推定。

協議夾具僅編譯專案自有 `tools/native-codec/main.cpp` 與自有 stub，不含壓縮演算法：

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

將 `XDZIP_NATIVE_CODEC_WORKER` 指向該夾具，並設定 `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` 後，顯式執行 `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`。普通定向迴歸使用 `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`，通用預設 sink 的相容行為由 codec API 測試核驗。生成證據只進入 build 或 CI，任務結果和剩餘項由[遷移任務 #51](https://github.com/xiaodaozhi/xdzip/issues/51)登記。
