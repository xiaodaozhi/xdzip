<a name="xdzip-product-version-and-build-identity"></a>

# 製品バージョンとビルドの識別

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | [繁體中文（台灣）](../../zh-TW/engineering/versioning.md) | 日本語 | [한국어](../../ko-KR/engineering/versioning.md) | [Français](../../fr-FR/engineering/versioning.md) | [Deutsch](../../de-DE/engineering/versioning.md) | [Español](../../es-ES/engineering/versioning.md) | [Português (Brasil)](../../pt-BR/engineering/versioning.md) | [Русский](../../ru-RU/engineering/versioning.md)

任意の `prerelease_number` はプレリリースの連番をビルド番号から分離します。たとえば alpha.1 でビルド 14 を使い、次の alpha.2 でもビルド番号を増やせます。この項目がなければ、従来の `channel.build` の解釈を使います。マニフェストと成果物の名前は[リリースプロトコル](release-protocol.md)で定めます。

[VERSION.json](../../../VERSION.json) は製品バージョン、チャンネル、ビルド番号の唯一の管理元です。ビルドは現在の値を読み取ります。バージョン変更は[変更履歴](../../../CHANGELOG.md)、配布や検証は[プロジェクトの Issue](https://github.com/xiaodaozhi/xdzip/issues)を参照してください。バージョン番号を増やすだけでリリースは承認されません。

過去版との互換性検証は固定したソースと現在のビルドを使い、アップグレード、旧版への復帰、設定の互換性を確認します。[旧版製品の準備スクリプト](../../../scripts/prepare-previous-product.py)は製品全体の基準を、[設定互換性スクリプト](../../../scripts/check-settings-compat.py)は設定の基準を管理します。変わり得るコミットや現在のバージョンはここに転記しません。設定のスキーマと製品バージョンは独立して管理します。

- `version` は先頭ゼロを付けない `major.minor.patch` です。非互換の変更、互換性を保つ機能追加、修正は各部分に対応します。0.x の間も互換性の境界を説明します。
- `channel` は development、alpha、beta、rc、stable を扱います。非安定版は `version-channel-abbreviation.build` と表示し、development は dev に短縮します。stable は基本バージョンを表示します。
- `build` は 1〜65535 の整数です。外部配布用の新規ビルドで増やします。ローカルコンパイルでは自動で増えず、タグやリリースを自動で作りません。Windows のリソース用には、数値バージョンの各部分も 65535 以下です。
- CMake は JSON を直接読み取ります。`python3 scripts/sync-version.py` は Cargo ワークスペースと HTML の派生バージョンを更新します。CI では `--check` を使います。内部 Rust パッケージはワークスペースのバージョンを継承します。同期後に Cargo.lock を更新し、`cargo check --workspace --locked` を実行します。
- Windows の四つの数値から成るリソースは基本バージョンとビルド番号を組み合わせます。文字列のバージョンはチャンネルを保持します。macOS の `CFBundleShortVersionString` は基本バージョン、`CFBundleVersion` はビルド番号で、長い表示用バージョンはチャンネルを保持します。[Apple の資料](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion)を参照してください。各プラットフォームの数値形式に従い、システムの数値項目に Git コミットは入れません。
- Windows のタスクマネージャーなどで使う `FileDescription` は `xdzip Archive Manager` です。`ProductName` は `xdzip` のままです。実行ファイル名と関連付けの識別情報は表示名に依存しません。ビルドで実際の EXE リソースを検証します。
- 各デスクトップビルドは短い Git コミットと未コミット変更の有無を生成します。Git がないときは不明とします。マシン固有のパス、ユーザー名、一定しない時刻は含みません。内容が変わったときだけヘッダーを更新します。コミット後にビルドし直すと、正しいクリーンなコミット識別情報になります。

QML モジュール、ブリッジプロトコル、設定スキーマのバージョンは製品バージョンと別に管理します。

`xdzip --version`、設定 → xdzip について、プレビュー用アプリは同じビルド情報を読み取ります。CLI は GUI を作る前にバージョン引数を処理します。「xdzip について」はバージョン、ビルド、コミット、実際の Qt、システム、アーキテクチャーを表示し、開発チャンネルと未コミット変更を表示します。コピーする JSON には個人ファイル、パス、マシン名、資格情報を含めません。プロジェクトのホームはユーザーが操作した場合だけブラウザーで開きます。

情報ページの大きなアイコンはプラットフォーム用の 512 px PNG を使います。HTML は同じ原稿のベクターを使い、72 px の UI アイコンを拡大しません。

情報ページは設定ウィンドウの論理寸法を使い、フッターはありません。[同梱する表示言語](../ux/settings-localization.md)、ライトとダークのテーマ、100%、125%、150% の文字サイズに対応します。[情報ページの設定デザイン](../../../design/hifi/sources/settings.html?tab=about)を参照してください。HTML は同じ JSON から生成します。ネイティブ Qt やコミットの値を取得できない場合は不明のままです。

実際の CLI とネイティブのバージョン検証に `scripts/check-build-version.py`、`scripts/test-version.py`、`scripts/check-html-about.py`、`BrowserTest::versionAboutWindow` を使います。スクリーンショットと測定結果は `build/review/about/` に保存します。ソース、システムメタデータ、実行中のバージョンを一致させます。バージョン番号だけでは署名、実機での受け入れ、互換性を確定できません。
