<a name="xdzip-build-deployment-and-release-architecture"></a>

# ビルド、配置、リリースの設計

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | [繁體中文](../../zh-TW/design/build-release.md) | 日本語 | [한국어](../../ko-KR/design/build-release.md) | [Français](../../fr-FR/design/build-release.md) | [Deutsch](../../de-DE/design/build-release.md) | [Español](../../es-ES/design/build-release.md) | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-build-and-package-inputs"></a>

## ビルドとパッケージの入力

CMake は Qt デスクトップアプリ、C++ アダプター、ネイティブの補助プログラムをビルドします。Cargo は Rust ランタイムをビルドし、リンクする CXX ブリッジを生成します。独立 CLI には Qt を必要としない専用のビルド入口があります。`scripts/desktop.py` と `scripts/cli.py` がビルドとテストを調整します。ソースは Rust 1.99.0 と Qt 6.8.2 SDK を固定しています。結果を比較するときは、ビルド識別情報、対象アーキテクチャ、ロックファイル、ツールチェーンの入手元を一緒に確認します。[ツールチェーン](../engineering/toolchain.md)、[開発ビルドガイド](../../engineering/developer-build.md)

デスクトップアプリのパッケージは、必要な Qt ライブラリー、QML モジュール、プラットフォームリソース、同梱の補助プログラムを配置します。内部の画像処理プロセスは、画像のデコードと静的 SVG の表示に Qt を使います。CLI の配布物には必要なアーカイブ処理プロセスが含まれます。ビルドプロファイル、パッケージに含められるファイルの一覧、配置時の検査は、スクリプトとプラットフォームの契約で管理します。インストールされる依存関係とパッケージ全体の容量は、パッケージ全体を調べて確認します。[配置と依存関係の監査](../engineering/deployment-audit.md)、[CLI の配布](../engineering/command-line.md)

<a name="xdzip-current-distribution-channels"></a>

## 現在の配布経路

| 対象 | パッケージと信頼の範囲 |
| --- | --- |
| macOS Apple Silicon | GitHub で DMG とアプリバンドルの ZIP を提供します。基準は macOS 14 です。パッケージには整合性確認のためのアドホック署名を使用します。インストールと更新についてはプラットフォーム別の手順を参照してください。 |
| Windows 11 x86_64 | GitHub で署名のないユーザー単位のインストーラーとポータブル ZIP を提供します。Microsoft Store 版の MSIX ID、署名、更新経路は Microsoft Store が管理します。 |
| Ubuntu 24.04 LTS x86_64 | GitHub で実行用ディレクトリのアーカイブと DEB を提供します。サポート基準は X11 と glibc 2.39 以降です。 |
| 独立 CLI | パッケージと更新に独自の識別情報を持ちます。デスクトップアプリに同梱の CLI は、そのパッケージの統合と引き継ぎの規則に従います。 |

[サポート方針](../engineering/support-policy.md)では対象の OS とアーキテクチャを定めます。実際の配布ファイルは[プロジェクト README](../../../README.ja-JP.md)と該当するリリースノートで確認できます。ファイルの関連付けと統合の制限は[インストーラーガイド](../../engineering/installers-associations.md)を参照してください。

<a name="xdzip-routine-checks-native-builds-and-release"></a>

## 通常検査、ネイティブビルド、リリース

通常の [CI ワークフロー](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml)は軽量な検査を実行します。ネイティブの Foundation と Qt のジョブは、別々の手動ワークフローです。通常の Apple Silicon CI はビルド、テスト、起動確認を行い、ダウンロード用アプリの配置は行いません。専用のリリースワークフローには、許可された配置、署名、パッケージ化、公開の手順があります。[リリース手順](../engineering/release-protocol.md)と [CI 方針](../../../developers/engineering/ci-policy.md)に必要条件と検証記録を定めます。

リリースマニフェスト、チェックサム、構成一覧、SBOM、ライセンス資料は配布する成果物を記述します。リリースワークフローでプロジェクトのソース ZIP を生成したりアップロードしたりすることはありません。GitHub のタグスナップショットは別の仕組みで提供されます。実際に同梱するコンポーネントと[ライセンス記録](../engineering/licensing.md)、[ソース提供方針](../../engineering/source-delivery.md)に基づいて義務を確認します。

<a name="xdzip-update-and-future-distribution"></a>

## 更新と今後の配布

更新の選択と検証は、アーカイブのタスクとは独立しています。対象となるインストール済みビルドは、信頼メタデータ、検証済み配布ファイル、インストール識別情報、実行中のタスクがある場合の引き継ぎに関する更新の契約に従います。Windows と Linux は該当する補助プログラム、macOS は Sparkle を使用します。Store 版は Microsoft Store が更新します。ポータブル版と開発構成には固有の制限があります。[ソフトウェア更新](../../engineering/software-update.md)

対象アーキテクチャ、OS の基準、パッケージ形式、ストアを追加する場合は、個別のビルドと受け入れの記録が必要です。従来の配布に関する決定と再検討の条件は[アーキテクチャ決定記録](architecture-decisions.md)にあります。
