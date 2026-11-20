<a name="xdzip-format-adapters-and-the-extension-trial"></a>

# 形式アダプターと拡張機能の試行

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | [繁體中文](../../zh-TW/design/codec-extension-sdk.md) | 日本語 | [한국어](../../ko-KR/design/codec-extension-sdk.md) | [Français](../../fr-FR/design/codec-extension-sdk.md) | [Deutsch](../../de-DE/design/codec-extension-sdk.md) | [Español](../../es-ES/design/codec-extension-sdk.md) | [Português (Brasil)](../../pt-BR/design/codec-extension-sdk.md) | [Русский](../../ru-RU/design/codec-extension-sdk.md)

<a name="xdzip-three-execution-paths"></a>

## 3 つの実行経路

| 経路 | 実装と利用可能性 |
| --- | --- |
| ホスト内の組み込みアダプター | デスクトップと CLI は同じ固定形式レジストリーとランタイムディスパッチを使用します。ZIP、TAR、ストリームフィルター、DEB や ASAR などの対応コンテナーの Rust 実装はホストプロセス内で呼び出されます。 |
| 同梱アーカイブワーカー | ホスト内の `xdzip-codec-native` が `xdzip-native-codec` を起動します。C++ プロトコル層は `xdzip-codec-bridge` を通じて Rust の 7z、RAR、CAB、光学メディアのバックエンドを呼び出します。該当する 7z の作成と書き換えも同じ経路を使用します。ISO 作成はプロセス内の光学メディア書き込みを使用します。 |
| 外部 SDK 試行 | `xdzip-plugin-host` と `xdzip-plugin-protocol` は独立した明示的な XDP1 実験です。デスクトップと CLI の製品ビルドは第三者拡張機能を検出、インストール、実行しません。 |

最初の 2 経路は組み込みの製品コンポーネントです。同梱ワーカーは専用プロトコルとリソース監督を持つ別プロセスです。外部試行は独自のパッケージ検証、バージョン交渉、プラットフォーム権限の条件を持ちます。[システムアーキテクチャー](../../design/system-architecture.md)、[アーカイブワーカー契約](../engineering/native-codecs.md)、[SDK 試行契約](../engineering/sdk-trial.md)

<a name="xdzip-product-routing-and-operation-capabilities"></a>

## 製品のルーティングと操作能力

[形式レジストリーのソース](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json)は拡張子、コンテナーとフィルターの型、対応操作、方式、作成オプションを宣言します。ジェネレーターは Rust レジストリー、デスクトップ一覧、設計成果物を同期します。読み取り時、ランタイムはソースバイトを調べ、形式を検証してからバックエンドを選択します。作成時は要求された出力名が作成プロファイルを選び、ランタイムがオプションを照合します。拡張子一覧は発見と UI のヒントに使用します。検出と走査の成功によって開いた形式が確定します。

読み取り、プレビュー、展開、アーカイブ全体の検査、作成、書き換えにはそれぞれ異なる制約があります。表示された項目でも対応するデータ方式がない場合があります。パスワード、ボリューム、特殊レコードも受け入れに影響し、ランタイムは ISO/UDF の構造検査や未対応の書き換えプロファイルなど形式ごとの違いを扱います。[ランタイムルーティング](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs)、[形式レジストリー契約](../engineering/format-registry.md)、[アーカイブ検査](../engineering/archive-checking.md)

<a name="xdzip-internal-rust-adapter-interfaces"></a>

## 内部 Rust アダプターインターフェース

| インターフェース | 責務 |
| --- | --- |
| `ReadSource` と `MetadataCodec` | 該当するボリュームメタデータを含め、ホストが承認したソースを読み取りまたはシークし、制限された形式レコードを生成します。 |
| `PayloadCodec`、`PayloadReader`、`DecodedSink` | 内容を解決し、適格な物理レコードをホストの出力先へデコードします。バックエンドが報告する場合、通常のバイト、スキップ時のデコード作業、補助作業を区別します。 |
| `CreationCodec`、`EncodeInput`、`EncodeOutput` | 固定された入力メタデータとインデックス付きストリームからエンコードします。ホスト所有の出力先に書き込み、書き込み側が必要とする場合は制限されたランダム書き込みを行います。 |
| `RewriteCodec` | 保持した元の項目、変更、承認済み追加から、対応するアーカイブを新しい一時アーカイブへ書き換えます。 |

パス、ソースの再検証、キャンセル、パスワード、割り当て、最終公開はホストが管理します。コーデックは操作に必要なインターフェースを通じて能力を受け取ります。アーカイブワーカーは許可されたソース範囲を要求し、制限されたレコードまたは出力ブロックを渡します。ホストは動作前にプロトコル状態とバイト割り当てを検証します。ワーカーの障害や無効な応答は公開を承認できません。ワーカー境界は障害とリソースを分離します。任意の外部コードに対するカーネル権限制限は試行の別の要件です。[メタデータ契約](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs)、[データ契約](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs)、[エンコードと書き換えの契約](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs)、[ファイル承認](file-security.md)

これらの Rust trait は内部のソースインターフェースです。製品アダプターはワークスペースと明示的なランタイムルーティングに統合します。許可された操作については、有効なファイル、未対応機能、リソース制限、破損、出力動作をそれぞれテストする必要があります。[独立した Rust コーデック契約](../../engineering/independent-rust-codecs.md)と[開発ガイド](../contributing/README.md)を使用してください。

<a name="xdzip-implemented-external-sdk-trial"></a>

## 実装済みの外部 SDK 試行

この試行は明示的に選択されたネイティブパッケージマニフェストとホストが選んだソースを受け入れます。プロジェクト所有の XDP1 テストデータを走査し、通常項目を 1 つ読み取れます。パッケージ受け入れではマニフェストバージョン 1、実行ターゲットとハッシュ、パッケージ内パス、固定の権限プロファイルと制限を検証します。ホストは検証済みのプライベートコピーを実行します。宣言されたハッシュはパッケージの整合性を示し、発行者の認証には別の信頼方針が必要です。

バージョン 1 の試行プロトコルは長さプレフィックス付きの制限された CBOR を使い、操作の ID と順序を確認します。ホストは不透明トークンを通じてソース読み取りと 1 つの出力先を許可します。相手側はバイト範囲と 1 ブロックの出力クレジットを受け取ります。ホストは内容を計測、検証し、プロトコルの完了とソース状態を確認し、相手側が正常終了した後にのみ上書きせず公開します。サンプルは相手側にアーカイブのパスワード、分割ボリューム、作成、更新、リンク、ホストパスを公開しません。

この試行には macOS と Linux の明示的なネイティブ権限条件があり、プラットフォーム要件と失敗時に拒否する動作は試行契約に定義されています。Windows は実行を拒否します。ネイティブの相互運用テストと拒否テストはデスクトップとは独立してこれらの境界を検証します。実際のスキーマ、予算、テスト入口は[試行の実行と通信契約](../engineering/sdk-trial.md)を参照してください。[試行ホスト](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs)、[フレーミング実装](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## 現在の実装外の設計案

製品向け拡張機能マネージャー、発行者の信頼と更新方針、試行プロトコルを超える互換性、外部書き込みやパスワードの能力は設計段階です。製品への採用には、対応する各プラットフォームと機能について、権限制限、相互運用性、障害時の動作の検証が必要です。同梱ワーカーと XDP1 実験はこの機能を有効にしません。[アーキテクチャー決定](architecture-decisions.md)
