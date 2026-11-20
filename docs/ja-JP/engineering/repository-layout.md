<a name="xdzip-repository-layout-and-dependencies"></a>

# リポジトリの構成と依存関係

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | [繁體中文](../../zh-TW/engineering/repository-layout.md) | 日本語 | [한국어](../../ko-KR/engineering/repository-layout.md) | [Français](../../fr-FR/engineering/repository-layout.md) | [Deutsch](../../de-DE/engineering/repository-layout.md) | [Español](../../es-ES/engineering/repository-layout.md) | [Português (Brasil)](../../pt-BR/engineering/repository-layout.md) | [Русский](../../ru-RU/engineering/repository-layout.md)

現在の製品モジュールはルートの Cargo.toml と CMakeLists.txt で定義します。表は実際に存在するディレクトリを示しています。アーキテクチャで予定している分割は、まだ独立したクレートになっていません。

| ディレクトリ | 責務と依存関係 |
| --- | --- |
| apps/desktop | Qt/QML、薄い C++ コントローラー、モデル、設定、システム連携。ブリッジ経由でコアにアクセスします。 |
| crates/xdzip-cli | Qt を使わない引数処理、端末、JSON、管理対象インストール、更新ホスト。ランタイム、安全な I/O、更新コアに直接依存します。形式の解釈は他の層に委ねます。 |
| crates/xdzip-domain | 純粋な型と不変条件。Qt や具象コーデックへの依存は禁止です。 |
| crates/xdzip-codec-api | 機能、借用入力、エンコード出力、キャンセルの契約。ドメインに依存します。 |
| crates/xdzip-compression | Deflate/Deflate64/GZIP、LZMA/LZMA2、BZip2、PPMd7、MSZIP、LZX、Quantum の共通アルゴリズムとラッパー。サードパーティーの本番依存関係や、パスおよび出力の確定に関する権限を持ちません。[契約](../../engineering/independent-rust-codecs.md)を参照してください。 |
| crates/xdzip-crypto | 形式に依存しない固定メモリの SHA/HMAC および BLAKE ハッシュ。機密状態をゼロで消去でき、採用済みのプリミティブを再利用します。パスワードの解釈や出力の確定判断は担当しません。 |
| crates/xdzip-codec-zip, xdzip-codec-tar | ZIP と TAR/GZIP の実装。任意の出力パスを扱いません。 |
| crates/xdzip-codec-lha, xdzip-codec-arj, xdzip-codec-arc, xdzip-codec-bh | [LHA](lha-containers.md)/[PMA](pma-streams.md)、[ARJ](arj-streams.md)、[ARC](arc-streams.md)、[BH](bh-containers.md) のフレームとメンバーの独立プロファイル。ファイルシステムの権限は持ちません。製品での利用可否は形式レジストリで決まります。 |
| crates/xdzip-codec-7z | 自前の 7z コンテナー、方式グラフ、フィルター、AES の組み合わせ、ストリーミング作成、上限付きの書き換え。ファイルパスは管理しません。 |
| crates/xdzip-codec-cab | 読み取り専用の自前 CAB コンテナー。ホストが許可したボリュームと共通アルゴリズムを使います。元の名前、UTF-8 の宣言、通常の同名レコードを保持します。ボリュームの検出と出力の確定はホストの担当です。[契約](../../engineering/independent-rust-codecs.md#cab只读规格)と[資料](cab-format-sources.md)を参照してください。 |
| crates/xdzip-codec-optical | ISO9660/Joliet/Rock Ridge/UDF リーダーと、新しいデータイメージ用の ISO9660/Joliet ライター。パスを管理せず、ストリームを使用します。 |
| crates/xdzip-codec-bridge | 隔離プロセスと Rust の 7z、RAR、CAB、光学メディアのコーデックをつなぐ薄い CXX 境界。許可済みのストリーム、リソース制限、キャンセルを扱います。 |
| crates/xdzip-codec-native, tools/native-codec | Rust の 7z/RAR/光学メディア/CAB バックエンドの隔離プロセス。薄い C++ 層はプロトコル、システムの暦とエンコーディングの機能だけを提供します。ネイティブ受信側は事前計上済みの出力と補助処理を独立に検査します。[I/O 契約](native-codecs.md)を参照してください。 |
| crates/xdzip-extract-plan | メタデータの事前検査と上限付きの報告。出力先の許可や書き込み権限は持ちません。 |
| crates/xdzip-io | 入力ツリー、ネイティブの権限とファイル識別、安全な一時準備、ファイルの確定。 |
| crates/xdzip-runtime | セッション、上限付きのメモリまたはディスク索引、照会、選択、上限付きのプレビュー、作成・展開タスクとスケジュール。 |
| crates/xdzip-qt-bridge | cxx の型ブリッジ、二つのハンドル、ライフサイクル。ランタイムに依存します。 |
| crates/xdzip-plugin-protocol | 標準ライブラリーだけを使う上限付き CBOR とフレーム処理。パス、プロセス、Qt の権限は持ちません。 |
| crates/xdzip-plugin-host, examples/sdk | 独立した読み取り専用 SDK の試験。制御された I/O、ネイティブサンドボックスの条件、独立した Python の通信相手。製品の形式検出と実行時の依存関係には含まれません。 |
| tools | ブリッジ、形式、I/O、プロセス数を調べる独立ツール。GUI 製品の入口は別にあります。 |
| scripts, .github | 生成、検査、配置ツール、CI、共同作業用テンプレート。 |
| release-notes | レビュー可能な各バージョンのリリースノートとバージョン・ビルドのメタデータ。実行報告と生成パッケージは別に置きます。 |
| design/hifi, design/html, design/assets | 高忠実度の原稿、インタラクティブな設計、ランタイム用素材の入力。 |
| docs | 長期保守する Markdown の設計、技術契約、再利用可能な検証方法、必要な画像。実行報告は別に置きます。 |
| specs | 機械可読の視覚仕様、用語、依存関係の一覧。製品リソース、ジェネレーター、検査で使用します。 |
| third_party | コードとともに保管するサードパーティーのソースとライセンス資料。 |

独立した xdzip-index と xdzip-platform、および外部形式に対する完全な適合確認ツールは、今後のアーキテクチャに含まれます。現在の索引はランタイムにあり、プラットフォーム固有のファイル操作は io とデスクトップアダプターで実装します。既存の plugin-host は[読み取り専用 SDK の試験](sdk-trial.md)用です。テストは主に Rust の各モジュール、apps/desktop/tests、スクリプトの検査で管理します。未実装の空のモジュールは作りません。

製品の CAB 処理は Rust に固定しています。公式の Cab ハンドラーと CAB 固有の LZX/Quantum はコンパイルしません。同じ隔離プロセスを使い、製品実行ファイル、テスト用スイッチ、失敗時の代替経路は追加しません。完全なフレームの処理量を事前計上する方法は codec-api で定義します。コンテナーはフレームごとの許容量を保持し、ネイティブ受信側とランタイムはそれぞれの許容量と出力制限を独立に検査します。[自前コーデックの契約](../../engineering/independent-rust-codecs.md#cab只读规格)を参照してください。

依存関係の設計については[システムアーキテクチャ](../design/system-architecture.md)を参照してください。ドメインからランタイムへの逆方向の依存、コーデックから QML への直接通知、プラットフォーム層での UI 文言決定は避けます。生成されたバインディング、サンプル、報告、ビルド出力はソースディレクトリの外にある build/、target/、dist/ に置きます。過去のバージョンとの互換性確認に必要な入力については[インストールと関連付け](../../engineering/installers-associations.md#兼容基线的长期输入)を参照してください。

`tools/image-preview/` は QtGui/QtSvg を使用する内部の静止画像処理プロセスです。検証済みの入力と RGBA 出力にはそれぞれ上限があり、アーカイブや出力先のパスは保持しません。対象プラットフォームと失敗時の動作は[プレビューの契約](content-preview.md)を参照してください。
