<a name="xdzip-qt-rust-bridge-and-application-service"></a>

# Qt/Rust ブリッジとアプリケーションサービス

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | [繁體中文](../../zh-TW/design/qt-rust-bridge.md) | 日本語 | [한국어](../../ko-KR/design/qt-rust-bridge.md) | [Français](../../fr-FR/design/qt-rust-bridge.md) | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | [Español](../../es-ES/design/qt-rust-bridge.md) | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

Qt デスクトップはリンクされた `cxx` ブリッジを通じて Rust ランタイムを呼び出します。各 `BrowserController` は別のサービスインスタンスを持つ `RuntimeConnection` を所有します。ブリッジは型付きコマンドと所有権のある結果をデスクトッププロセス内で転送します。CLI は同じ Rust サービスを直接使います。[システムアーキテクチャー](../../design/system-architecture.md)、[デスクトップセッション](../../engineering/application-sessions.md)

<a name="xdzip-handles-threads-and-ownership"></a>

## ハンドル、スレッド、所有権

`new_runtime()` は Rust のコマンドスレッドとタスクスレッドを起動します。一時的な `Runtime` は `CommandClient` と `EventReceiver` を各一つ渡します。GUI スレッドはコマンドハンドルを使います。専用の Qt 受信スレッドはイベントハンドルを所有し、`next_event()` で待機します。Rust コマンドスレッドは現在のアーカイブとそのインデックスを所有します。別のタスクスレッドは実行中の準備済みジョブを所有します。

CXX は値型と所有権を持つ Rust コンテナーを転送します。受信した Qt コードは所有中に値を保持または変換します。アーカイブやネイティブファイルのハンドルは Qt に渡りません。ID や正確なバイト数を QML に渡す際、64 ビットすべてを保持する必要がある値にはコントローラーが文字列を使います。生成ヘッダーと静的ライブラリはホストと一緒にビルドされます。このインターフェースは製品ビルドに属します。同梱ワーカーは別の[プロセスプロトコル](../engineering/native-codecs.md)を使います。

[ブリッジの宣言と変換](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs)、[Qt 接続](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp)、[Rust サービス](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

<a name="xdzip-request-query-and-task-identities"></a>

## 要求、クエリー、タスクの識別

| 識別情報 | 用途 |
| --- | --- |
| 要求 ID | Rust サービスが受け入れ時に接続内の ID を割り当てます。`Submission` は `Accepted`、`Busy`、`Closed`、`Invalid`、`Exhausted` を返します。拒否時の ID はゼロです。 |
| セッション ID | 現在開いているアーカイブのスナップショットを識別します。そのアーカイブを操作するコマンドは一致する必要があります。新たに開くと新しいセッション ID を取得します。 |
| クエリー識別情報とオフセット | ページはセッション、クエリー ID、UI 世代、改訂、オフセットに結び付きます。コントローラーは行の受け入れ前にこれらを検証します。 |
| 物理項目またはナビゲーション ID | 物理項目はソートとページ分割の間も識別情報を維持します。合成ディレクトリは独自の識別情報を持ちます。行番号、表示名、クエリー範囲からファイルシステムパスへのアクセス権が生じることはありません。 |
| タスクとプロンプトの ID | タスク ID はその接続に属します。プロンプトはタスク内で保留中の競合またはパスワードへの一つの応答を識別します。アプリのタスク項目は所有コントローラーも把握します。 |
| イベントシーケンス | 受信側がイベントを取り出す際、接続ごとに単調増加する番号を割り当てます。Qt は最後に消費した番号より新しくないイベントを破棄します。タスク更新は同じシーケンスを共有し、元の要求 ID で関連付けます。 |

<a name="xdzip-commands-in-the-current-bridge"></a>

## 現在のブリッジのコマンド

| コマンドまたはメソッド | 用途と通常結果 |
| --- | --- |
| `Open`, `OpenWithPassword` | 選択したソースを開いて走査します。成功時は最初の `Page` と上限付きナビゲーションデータを含みます。暗号化ディレクトリではセッションを開く前にパスワードが必要なことがあります。 |
| `Browse` | クエリーを開始するか、一致する識別情報とオフセットで継続します。各コマンドの結果は最大一つの `Page` で、128 行、名前 64 KiB 以下です。 |
| `FreezeSelection`, `Preflight` | 現在のセレクターを `Selected` 内の所有権付き物理 ID またはナビゲーション ID に解決するか、メタデータの検出結果を `Preflight` にまとめます。実際の操作ではスナップショットと選択範囲を再検証します。 |
| `Preview` | 一つの物理項目を解決します。成功時は所有権のある上限付き `Preview` バイト列を項目とセッションの識別情報とともに返します。 |
| `CheckTarget`, `InspectInputs` | ランタイムが検査用作業領域を開いて清掃した後、残存数を `TargetChecked` で返します。作成の検査はセッションゼロを使います。`Inputs` はディレクトリを再帰走査せず入力順に浅いメタデータを報告します。どちらの応答も確定の権限を与えません。 |
| `Extract`, `Create`, `Check`, `OpenEntry` | 意図を検証してタスクをキューに入れます。`TaskAccepted` は割り当てられたタスク ID を含みます。出力先の認可と確定はランタイムと I/O 層が行います。 |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | アーカイブ編集とデータ公開は重いタスクのキューに入ります。`Update` または `Move` を受け入れるとサービスは現在のスナップショットを破棄し、後続要求が古い項目 ID を使えないようにします。`ExportManifest` は相対転送メタデータを通常応答として返します。 |
| `Snapshot`, `CloseSession` | 現在のアーカイブセッション ID またはゼロを返します。指定セッションを閉じると閲覧スナップショットを破棄しますが、独立して保持されたタスク入力は存続できます。 |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | 通常コマンドキュー外の制御メソッドです。要求キャンセルは独自の要求 ID と `Finished` を持ち、タスク制御は既存のタスクに作用します。 |

Rust サービスは `Catalog` や `SelectMembers` など CLI 専用の操作も公開します。ブリッジは送信前に入力サイズと列挙値を検証します。たとえば作成パスは一つ 4 KiB、合計 4 MiB 以下です。ランタイムはセッションと操作の意味を検証し、I/O 層はファイルを開いて出力の権限を取得します。[ランタイムサービス契約](../engineering/product-runtime.md)、[CLI 契約](../engineering/command-line.md)、[ファイル認可](file-security.md)

<a name="xdzip-event-delivery-and-backpressure"></a>

## イベント配信とバックプレッシャー

受理された通常要求は通常の配信では最大一つのデータ結果と、それに続く一つの `Finished` を生成します。エラーはコードを持つ `Finished` で表されます。コントローラーは成功を待って対応するページまたはプレビューを公開します。後続に取って代わられた結果は破棄されますが、要求自体は終了します。大規模インデックス経路は Qt に最大 1,024 行をキャッシュし、不足ページを要求します。展開確認を表示する前にページをまたぐ選択を固定します。[大規模アーカイブのインデックス](../engineering/large-index.md)、[内容プレビュー](../engineering/content-preview.md)

`TaskAccepted` と `Finished` は通常要求の配信に属します。`TaskUpdate` はタスクごとの独立した保留枠を使い、元の要求 ID、タスク ID、ソースセッション ID を運びます。更新が `TaskAccepted` より先、または `Finished` より後に届くことがあります。コントローラーは早着したタスクを元の要求に関連付け、後続更新をそのタスクに振り分けます。後続のタスク更新が通常要求を再び完了することはありません。保留枠はタスクの最新スナップショットを保持します。進捗をまとめることができ、終端スナップショットは受信側が取り出すまで残ります。応答待ちの間、プロンプトはスナップショットに現れます。[タスク状態とキャンセル](task-scheduling.md)

- **Rust.** 接続ごとに通常受け入れは未完了要求 128 件と、キュー内コマンドのバイト予算 8 MiB を許可します。通常イベントのペイロード予算は 32 MiB です。受理された通常要求は結果と完了のための枠を予約します。要求キャンセルの確認応答には独自の上限付きキューがあり、シャットダウンはそこに空きを必要としません。
- **Qt.** メールボックスは最大 256 イベント、うちペイロードイベント最大 128 件、計上されたペイロード 32 MiB を保持します。容量が尽きた場合、受信側は GUI が取り出すのを待ちます。一つの通知チェーンが一回につき最大 32 イベントまたは 4 ms を処理します。通知は GUI スレッドに届き、キュー内のイベントは所有データを保持します。

<a name="xdzip-prompts-failures-and-shutdown"></a>

## プロンプト、障害、シャットダウン

- 競合の判断には `decide_task` を使います。タスクのパスワードには `respond_password` と別のパスワードプロンプト ID を使います。どちらも現在の未回答プロンプトを検証します。パスワード応答は解放時にゼロ化される Rust ストレージに入り、結果や進捗イベントには現れません。`OpenWithPassword`、`Preview`、暗号化作成は操作固有の入力秘密を持ち、ブリッジが対応する Rust パスワード型に移します。[パスワード処理](../engineering/password-extraction.md)
- 安定したエラーコードが受け入れ、要求、タスクの結果を表します。例は `stale_session`、`stale_query`、`source_changed`、`unsafe_target`、`target_changed`、`disk_full`、`password_required` です。コントローラーは表示用にコードを翻訳します。ソースの変更は該当する閲覧スナップショットを無効にします。`RuntimeFault` と予期しない切断は保留中の UI 作業を `runtime_unavailable` で無効にします。接続喪失により元の終端イベントが届かない場合があります。
- `shutdown` は受け入れを閉じ、入力待ちのタスクを含む受理済み作業をキャンセルします。Qt 受信側は Rust スレッドの終了中もイベントを取り出し続けます。Rust 受信側はスレッドを待ってから `ShutdownReady` を返し、その後ブリッジが `End` を返します。Qt はメールボックスを空にした後、正常な切断を報告します。接続が早期に破棄された場合、GUI スレッドで待機せずメールボックスを閉じて受信側を起こします。[タスクの終了](task-scheduling.md)、[ランタイムのライフサイクル](../engineering/product-runtime.md)

この境界を変更するときは、[ブリッジ検査ツール](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md)、Rust サービスのテスト、[デスクトップコントローラーのテスト](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp)が異なる層をカバーします。対象の確認には[開発ビルドガイド](../../engineering/developer-build.md)を使います。
