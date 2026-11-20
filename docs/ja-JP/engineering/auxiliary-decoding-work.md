<a name="xdzip-auxiliary-algorithm-work-accounting"></a>

# アルゴリズムの補助処理量の計上

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | [繁體中文（台灣）](../../zh-TW/engineering/auxiliary-decoding-work.md) | 日本語 | [한국어](../../ko-KR/engineering/auxiliary-decoding-work.md) | [Français](../../fr-FR/engineering/auxiliary-decoding-work.md) | [Deutsch](../../de-DE/engineering/auxiliary-decoding-work.md) | [Español](../../es-ES/engineering/auxiliary-decoding-work.md) | [Português (Brasil)](../../pt-BR/engineering/auxiliary-decoding-work.md) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

展開と検査では、ファイルのデコード済みバイト、実際の出力、アルゴリズムの補助処理量を別々に計上します。PPM モデルの初期化、容量上限による再初期化、制御情報やパラメーターのデコードはファイル内容ではありません。これらで `decoded_bytes`、`written_bytes`、ファイルの進捗、展開比率を増やしたり、事前計上による出力許容量を発行したりしてはいけません。タスクスナップショットにはユーザー向けの補助カウンターを追加しません。

`DecodedSink::charge_auxiliary(bytes)` は実行済みの補助処理だけを報告します。既定の実装はファイルカウンターを変えません。アルゴリズムはまず残りの予算で予定の処理を許可し、実行後に実処理量を計上します。失敗やキャンセル前に完了した処理も含めます。未実行の予約は実処理として報告しません。エラー、モデルのリセット、パスワードの再試行でも実行済み処理の許容量は戻りません。RAR の内部の総処理上限はファイルのデコードと補助処理の合計を引き続き制限します。

<a name="xdzip-isolation-boundary"></a>

## 隔離の境界

- `AUX_WORK=24` はリトルエンディアンの `u64` カウントだけを転送します。内容は含みません。各パケットの値は 1 から 65,536 とし、フレームはその直後に終わる必要があります。計上に成功するとホストは空の `ACK` を返します。ゼロ、切り詰め、余分な末尾フィールド、上限超過、不正な処理段階を拒否します。
- このメッセージは RAR データのデコード時だけ許可します。メタデータ走査、作成、変更、他の形式では使いません。CXX はカウントを同期的に渡し、Rust ポインターを保持しません。入力元と出力のスライスは呼び出し中だけ借用します。C++ の例外はコールバック内で安定したエラーへ変換します。Rust のアンワインドは CXX を越えず、異常停止やプロセス終了は未完了の出力を失敗させます。
- ネイティブのデコードインスタンスは補助処理を最大 20 GiB まで累積し、項目の切り替え、プロセスの再起動、パスワードの変更をまたいで保持します。ホストのシンクを呼ぶ前に完了した処理を記録します。シンクが拒否しても再試行時に許容量は戻りません。`AUX_WORK` は `WorkerCredit` を増やしません。
- ランタイムの展開タスクと検査タスクは、補助処理を独立に最大 20 GiB まで累積します。計上オブジェクトは項目とパスワード再試行のループ外に置きます。実ファイルのバイト数と出力制限とは別に確認します。上限超過やキャンセルは失敗を返し、未完了ファイルを確定しません。公開カウンターを変更して制限を回避することはできません。
- `WORK=17` は引き続きデコード済みのファイルバイトを表します。受け入れに成功したときだけ、同じ有効なプロセスに `PRECHARGED_DATA=23` 用の一回限りの許容量が与えられます。転送前に許容量を差し引き、再起動で初期化します。RAR、7z、CAB はこの経路を使います。走査と作成では使いません。補助処理をファイル出力の許容量に振り替えることはできません。

高圧縮率の受け入れは固定された宣言サイズ、圧縮サイズ、またはアーカイブ全体のサイズを使います。ランタイムのファイルデコードと出力の制限は独立に適用します。87 バイトのファイルがモデル初期化に数 MiB の処理を必要としても、ファイルカウンターは実際の 87 バイトだけを示します。

<a name="xdzip-required-acceptance-conditions"></a>

## 受け入れ条件

- 通常の小さなファイルは、64 KiB を超える補助処理とモデル初期化の後でも展開および検査を完了できます。ファイルの計上は正確で、最終内容検証も成功します。
- 実際の補助パケットの累積が 20 GiB に達した場合の拒否をネイティブとランタイムでそれぞれ検証します。パスワードの再試行やプロセスの交換で累積処理量は初期化されず、未完了ファイルは確定しません。
- 模擬プロセスからの他形式やメタデータ段階の要求、空または過大なパケット、末尾の余分なフィールド、補助処理だけで許可しようとする事前計上出力はすべて失敗します。
- 通常の `WORK` は同じプロセスの事前計上済み RAR 出力に使えます。許容量は一度だけ消費され、エラー後や再起動後には再利用できません。
- プロトコルのフィクスチャーは状態機械と計上だけを確認します。Rust の完全な RAR プロセス、ランタイムの実処理経路、三つのプラットフォーム、デスクトップの受け入れには個別の計測結果が必要です。

プロトコルフィクスチャーはプロジェクトの `tools/native-codec/main.cpp` とスタブだけをコンパイルします。圧縮アルゴリズムは含まれません。

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

`XDZIP_NATIVE_CODEC_WORKER` でフィクスチャーを指定し、`XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` を設定して、`cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored` を明示的に実行します。通常の対象限定の回帰検査は `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary` を使います。コーデック API テストは既定のシンクとの互換性を確認します。生成した検証記録は build または CI に置きます。結果と残りの項目は[移行タスク #51](https://github.com/xiaodaozhi/xdzip/issues/51)に記録します。
