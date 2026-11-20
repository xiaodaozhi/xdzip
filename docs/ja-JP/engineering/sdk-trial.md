<a name="xdzip-external-read-only-sdk-trial"></a>

# 外部読み取り専用 SDK の試験

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | [繁體中文](../../zh-TW/engineering/sdk-trial.md) | 日本語 | [한국어](../../ko-KR/engineering/sdk-trial.md) | [Français](../../fr-FR/engineering/sdk-trial.md) | [Deutsch](../../de-DE/engineering/sdk-trial.md) | [Español](../../es-ES/engineering/sdk-trial.md) | [Português (Brasil)](../../pt-BR/engineering/sdk-trial.md) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol` と `xdzip-plugin-host` は[形式拡張の設計](../design/codec-extension-sdk.md)に記載された独立した読み取り専用試験を実装します。デスクトップと CLI の形式検出、インストール、パッケージング、自動起動からは除外されています。ユーザーが試験コマンドを明示的に実行し、パッケージとソースを選びます。製品内のサードパーティー拡張は無効のままです。交渉するのは `xdp1` のみで、作成、パスワード、分割、リンク、GUI、任意のホストパスへの権限はありません。完全なプラグインエコシステムでも SDK リリースでもありません。

<a name="xdzip-reproducible-entry-points"></a>

## 再現可能な入口

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

Windows では `python` と `.exe` を使用します。`test` はネイティブ参照プログラムをビルドし、Rust フレーミング回帰、独立した Python 相互運用、悪意あるプロトコル、キャンセル、ソース変更、競合、カーネル権限の回帰を実行します。Python はテスト用にコンパイルされた信頼できる適合性検査入口からのみ起動します。配布されるホストはインタープリター、引数、シェル、サンドボックスなしの実行スイッチを受け付けません。例は `build/` 内に自前のパッケージ、マニフェスト、XDP1 のみを生成し、リリースや第三者からのダウンロードを発生させません。ログはワークスペースとユーザーホームのパスを除き、JSON はコミット、変更状態、バイナリハッシュを結び付けます。どちらもローカルまたは CI の成果物です。同じネイティブ Foundation マトリクスがこの入口を呼び出します。別の [SDK 試験ワークフロー](../../../.github/workflows/sdk-trial.yml)は無関係な形式・Qt パイプラインを繰り返さず、まずプラットフォームのサンドボックス互換性を確認します。Foundation ワークフローも `sdk_only=true` で再利用可能な試験を呼び出せますが、完全な Foundation 検査を省略し、完全な Foundation 結果を証明しません。Windows では実行拒否を検証します。

<a name="xdzip-package-admission"></a>

## パッケージの受理

厳格な JSON マニフェストは `deny_unknown_fields` を使用します。フィールドは `manifest_version=1`、`plugin_id`、`version`、`author`、`protocol_majors=[1]`、`targets`、パッケージ内の `entrypoint`、小文字の 16 進数 `sha256`、`formats=["xdp1"]`、`permissions`、`resources`、`license_files`、HTTPS の `source_url` です。上記の `example` が生成する参照マニフェストは現在のスキーマで実行できる入口です。

`permissions` は正確に `broker_source_read=true`、`broker_output_write=true`、`network=false`、`resources` は正確に `memory_bytes=536870912`、`threads=1` です。マニフェストは最大 32 KiB、ネイティブ実行ファイルは最大 64 MiB、ライセンスファイルは最大 4 個で各最大 128 KiB です。入口とライセンスファイルは絶対パス、親ディレクトリへの移動、リンクや再解析ポイントによる脱出を許しません。各パッケージメンバーは保持した cap-std ディレクトリからリンクをたどらず要素ごとに開き、検証後に元の文字列から開き直しません。ホストは保持した通常ファイルからコピーし、実際のバイト数、SHA-256、ファイルバージョンをコピー前後で検証して、非公開の一時コピーのみを実行し、プロセスと IPC の終了まで保持します。宣言されたハッシュは選択したパッケージ内容の整合性を示しますが、発行元の認証やコードの安全性は証明しません。

<a name="xdzip-wire-protocol-and-state"></a>

## 通信プロトコルと状態

自前の制限付き CBOR サブセットは [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) に従い、第三者 CBOR 実装を使いません。Rust の参照相手はフレーミング crate を再利用し、[Python の相手](../../../examples/sdk/reference.py)は Rust や生成済みプロトコルコードをインポートせず、エンコード、デコード、XDP1 解析を独立に実装します。相互運用では実内容の複数ブロックとホストのダイジェストを検証します。

フレームは `u32 LE length + CBOR` で、割り当て前に長さを確認します。フレームは最大 1 MiB、個々の bytes/text は最大 256 KiB、深さは最大 16、配列は最大 128、マップは最大 32、デコード済みノードは最大 8192 です。UInt、bytes、厳格な UTF-8 テキスト、配列、UInt キーのマップ、bool のみ受け入れます。最短でない整数、重複キー、不定長、負の整数、タグ、浮動小数点、末尾データは拒否します。エンベロープマップのキー `1..5` は kind、request、operation、sequence、payload です。sequence は各方向で 1 から連続して増加し、request は呼び出しごとに増え、operation はホストのランダムな非ゼロ識別子です。各応答は現在の識別情報に完全一致する必要があります。不明な payload キー `<128` は拒否し、`>=128` は無視可能な任意フィールドです。

| kind | メッセージ | payload の数値キー |
| --- | --- | --- |
| 1 | Hello | 1 最小 major、2 最大 major、3 パッケージ ID、4 `xdp1` |
| 2 | Welcome | 1 major、2 ブロック上限、3 単一ブロッククレジット、4 総読み取り予算、5 出力予算 |
| 10 | Probe | 1 ソーストークン、2 ソース長 |
| 11 | Open | 1 ソーストークン |
| 12 | Scan | 1 セッショントークン |
| 13 | ReadEntry | 1 セッション、2 項目、3 出力先、4 クレジット数 1、5 出力予算、6 初回ブロックのクレジットトークン |
| 14 | Close | 1 セッション |
| 20, 21 | ReadAt, ReadData | 1 ソーストークン、2 オフセット、3 長さまたはバイト |
| 30 | EntryBatch | 1 セッション、2 項目。各レコードは 1 トークン、2 生パス、3 種別 0 ファイル / 1 ディレクトリ、4 既知サイズ |
| 31 | ScanFinished | 1 実際のレコード数 |
| 40 | WriteData | 1 出力先、2 0 から始まるブロック順序、3 バイト、4 現在のクレジットトークン |
| 41 | FinishOutput | 1 出力先、2 宣言した出力数、3 SHA-256 バイト |
| 42 | WriteAck | 1 出力先、2 次のブロック順序、3 新しいランダムクレジットトークン |
| 50, 51, 52 | Cancel, CancelAck, Shutdown | 空のマップ |
| 61 | OperationFinished | 1 結果、0 成功 / 1 キャンセル。Probe は 2 形式、Open は 2 セッションも持つ |

フレーミングが認識しても、試験の状態機械は他の kind を拒否します。最初に Hello が到着し、双方が major 1 を選ぶ必要があります。ハンドシェイクの識別情報はゼロ、その後の呼び出しは非ゼロです。ScanFinished の後も一意の成功終端が必要です。一度に実行する操作は一つで、バックグラウンドの拡張再試行はありません。現在の単一ブロッククレジットは設計上の最大 4 より厳しく、各出力ブロックの実際の受理後に予測不能なトークンを更新し、事前支払いやリプレイはできません。

<a name="xdzip-broker-budgets-and-publication"></a>

## ブローカー、予算、公開

ソーストークン、出力先、ブロックごとのクレジットはランダムな 128 ビット値で、操作間で再利用できません。相手にはアーカイブや保存先のパスを渡しません。ホストは許可されたソース範囲内で読み取りごとに最大 256 KiB を返します。オフセットのオーバーフロー、EOF 超過、長さゼロ、累積読み取り 64 MiB 超過は失敗します。保持したオブジェクトと対応パスの `FileVersion` を各読み取りの前後および最終公開前に検査します。既存の識別情報・時刻・長さのフィンガープリントを再利用するもので、同一アカウントの攻撃者がすべてのメタデータを復元した任意の同時変更を検出できるとは主張しません。

メタデータは物理レコード最大 1000、パスとトークン合計最大 256 KiB に制限し、レコードトークンは一意です。ホストは生パスを PortablePath に従って厳格に検証し、重複名は別の識別情報を維持します。一度に通常項目を一つ明示選択し、宣言サイズは最大 4 MiB です。ディレクトリはファイル出力にできません。ホストが保存先 Grant、非公開 Workspace、Stage を保持し、実際のバイト数を計量して SHA-256 を計算します。相手にはファイルハンドルも公開権限も渡しません。

既存の安全な I/O が上書きせずに公開できるのは、全出力ブロック、FinishOutput の数とダイジェスト、一意の終端、Close、Shutdown 後の正常終了とクリーンな EOF、ソースの識別情報、最終キャンセル確認がすべて成功した後だけです。異常、終端の重複、クラッシュ、タイムアウトではまず出力を取り消し、破棄時は workspace の整理前に stage を削除します。既存の対象は変更しません。解析と期待ダイジェストは拡張側から得るため、ホストの計算だけでは悪意ある拡張の出力がアーカイブに忠実であると証明できません。

ハンドシェイクは 2 秒、試験の各呼び出しは合計 5 秒に制限し、進捗で期限をリセットしません。プロセスごとの CPU は 3 秒、ファイルディスクリプターは 32 に制限し、ファイルへの直接出力を禁止し、コアダンプを無効にします。IPC 受信キューは最大 4 フレーム、送信キューは 1 フレームで、パイプ書き込みは 2 秒に制限します。stderr は予算ゼロで直接破棄し、信頼できない内容は転送しません。キャンセルは出力先を直ちに取り消し、協力のため 2 秒待ってからプロセスとパイプを終了・回収します。Ack は公開許可ではありません。これらのリソース予算は小さな試験の制約であり、通常の大きなアーカイブの性能を約束するものではありません。

<a name="xdzip-platform-gates"></a>

## プラットフォームの条件

| プラットフォーム | 試験方針と失敗時の動作 |
| --- | --- |
| macOS | 固定された `/usr/bin/sandbox-exec` の deny-default 方針は、該当する非公開実行コピー、システムライブラリー/dyld キャッシュ、ルートのローダー読み取り、`/var` メタデータを許可します。ユーザーファイル、書き込み、ネットワーク、fork は拒否し、環境を消去して stdio のみ継承します。ネイティブ rlimit と 20 ms ごとの RSS 監視により 512 MiB 超過または測定失敗で終了します。RSS はサンプリングのため測定間の短いピークが閾値を超える場合があります。このインターフェースの将来の正式配布は保証しません。 |
| Linux x86_64 / aarch64 | Landlock ABI ≥3 は v3 の全ファイル権限を扱い、書き込み規則なし、実行ファイルとシステムローダーライブラリーのみ読み取り可能です。`no_new_privs` とアーキテクチャを確認した seccomp システムコール許可リストには Rust 標準パイプに必要な readv/writev が含まれ、socket、fork/clone、ptrace、io_uring などを拒否します。CPU 3 秒、アドレス空間 512 MiB とその他の rlimit を適用します。規則、ABI、seccomp、必要なシステムコールがなければ exec 前に失敗します。このプラットフォームの実際の結果を証明できるのはネイティブ CI のみです。 |
| Windows とその他のプラットフォーム | 実行を無条件に拒否します。AppContainer、Job、ハンドル許可リストは未実装・未検証であり、通常の子プロセスをサンドボックスの代わりにしません。 |

Linux API の選択は[カーネルの Landlock 文書](https://docs.kernel.org/userspace-api/landlock.html)と UAPI に従い、将来の Windows 作業は [AppContainer 分離](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation)に従います。ホストのネイティブ unsafe コードはサンドボックスモジュールに限定し、レイアウト、呼び出し前提、失敗時の動作を記載します。pre-exec は割り当てやログ出力をしません。参照相手は明示的な監査モードでのみ制御された fork/_exit/waitpid プローブを追加し、子プロセスは Rust のクリーンアップや形式ロジックを実行しません。プラットフォーム回帰はまず自前の読み書き可能なマーカー、自前の localhost リスナー、self-exec を陽性対照とし、その後、読み取り、書き込み、ネットワーク、直接 fork、self-exec のカーネルによる拒否をマーカー、ファイル、接続の実際の状態に照らして検証します。未検証のプラットフォームを合格とせず、デスクトップでの第三者実行は無効のままです。

<a name="xdzip-owned-xdp1-example-format"></a>

## 自前の XDP1 サンプル形式

XDP1 は相互運用のテストデータにすぎず、実際のアーカイブ形式に代わるものではありません。4 バイトの `XDP1` に `u32 LE` のレコード数が続きます。各レコードは `u16 LE` の UTF-8 名称バイト数、`u32 LE` の内容バイト数、名称、非圧縮内容を含みます。レコードは最大 1000 件で末尾データを許さず、名称と内容はホスト予算内で読み取ります。項目トークンは 4 バイト LE の物理順序番号で、パスによる重複排除はしません。Python の故障モードは適合性テストでのみ明示的に選択し、不正アクセス、順序、リプレイ、重複識別情報、パス、クレジット、ダイジェスト、クラッシュ、キャンセル、重複終端を検証します。
