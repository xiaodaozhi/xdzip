<a name="xdzip-product-design-and-technical-reference"></a>

# 製品の設計と技術リファレンス

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | 日本語 | [한국어](../../ko-KR/reference/README.md) | [Français](../../fr-FR/reference/README.md) | [Deutsch](../../de-DE/reference/README.md) | [Español](../../es-ES/reference/README.md) | [Português (Brasil)](../../pt-BR/reference/README.md) | [Русский](../../ru-RU/reference/README.md)

デスクトップと CLI の実行経路は[システム構成](../../design/system-architecture.md)を参照してください。各動作を定める仕様は下の章から探せます。対応操作の一覧は[対応形式の表](../../../README.ja-JP.md#xdzip-format-support)にあります。使い方や開発準備は[ユーザーマニュアル](../user-guide/README.md)、[CLI の使用例](../integration/README.md)、[コントリビュートガイド](../contributing/README.md)を参照してください。

<a name="xdzip-architecture-and-interfaces"></a>

## アーキテクチャーとインターフェース

- [設計の概要](../design/index.md)
- [製品要件と操作の範囲](../design/product-requirements.md)
- [ドメインモデル、項目の識別、ソースのバージョン](../design/domain-data.md)
- [Qt/Rust のコマンドとイベントのブリッジ](../design/qt-rust-bridge.md)
- [タスクのスケジューリング、進行状況、キャンセル、終了](../design/task-scheduling.md)
- [性能予算と検証](../design/performance.md)
- [用語と実装上の制限](../design/glossary.md)
- [要件と検証の追跡](../design/requirements-traceability.md)
- [アーキテクチャーの決定](../design/architecture-decisions.md)
- [試作からデスクトップと Rust ランタイムへの変遷](../design/architecture-evolution.md)
- [リポジトリー構成と依存関係](../engineering/repository-layout.md)
- [インターフェース仕様と検証](../engineering/contracts.md)
- [製品のランタイムと Qt のライフサイクル](../engineering/product-runtime.md)
- [アプリのセッションと終了](../../engineering/application-sessions.md)
- [cxx ブリッジの採用理由](../engineering/cxx-admission.md)
- [形式ワーカーで使うホストの暦と文字コード](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## アーカイブの読み書き

- [ファイルの許可、一時出力、安全な確定](../design/file-security.md)
- [読み取り専用の閲覧仕様](../../engineering/readonly-browser.md)
- [読み取り専用の ZIP 閲覧](../engineering/zip-readonly-browsing.md)
- [大規模アーカイブの索引とオンデマンド閲覧](../engineering/large-index.md)
- [分割ボリュームの読み取りと一時的な展開](../engineering/archive-opening.md)
- [ファイル名の文字コード判定](../engineering/filename-encoding.md)
- [内容のプレビュー](../engineering/content-preview.md)
- [展開前のメタデータ検証](../engineering/extraction-preflight.md)
- [安全な ZIP 展開の実装](../../engineering/zip-safe-extraction.md)
- [アーカイブの整合性検証](../engineering/archive-checking.md)
- [項目を開く、ドラッグ＆ドロップ、アーカイブの変更](../../engineering/archive-transfers.md)
- [アーカイブのパスワードと暗号化ディレクトリー](../engineering/password-extraction.md)
- [ZIP 作成仕様](../../engineering/zip-creation.md)
- [ISO9660 と Joliet の作成](../engineering/iso-creation.md)
- [Windows 用の組み込み ZIP 自己展開アーカイブ](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## 対応形式と実装

- [組み込みの機能、作成プロファイル、検証範囲](../engineering/format-registry.md)
- [形式アダプターと外部 SDK 試験の設計](../design/codec-extension-sdk.md)
- [読み取り専用の外部 SDK 試験と実行範囲](../engineering/sdk-trial.md)
- [隔離された形式ワーカーの仕様](../engineering/native-codecs.md)
- [自作の Rust 形式とアルゴリズム仕様](../../engineering/independent-rust-codecs.md)
- [補助アルゴリズムの作業量計上](../engineering/auxiliary-decoding-work.md)
- [ZIP メタデータ依存の採用](../engineering/zip-metadata-admission.md)
- [ZIP 展開と出力の依存採用](../engineering/zip-extraction-admission.md)
- [7z の形式資料と独立実装](../engineering/sevenzip-format-sources.md)
- [RAR の独立実装の根拠と互換性の範囲](../../engineering/rar-format-sources.md)
- [CAB の形式資料と実装の根拠](../../engineering/cab-format-sources.md)
- [TAR/GZIP の実装仕様](../engineering/tar-gzip.md)
- [単一ファイルストリームと ZIP 派生パッケージ](../engineering/single-file-streams.md)
- [LZ4 ストリーム仕様](../engineering/lz4-streams.md)
- [Zstandard ストリーム仕様](../engineering/zstandard-streams.md)
- [Unix compress Z の仕様](../engineering/compress-streams.md)
- [通常の Brotli ストリーム仕様](../engineering/brotli-streams.md)
- [LHA のデータ構造とアルゴリズム](../../engineering/lha-containers.md)
- [PMA コンテナーと PMarc 方式](../engineering/pma-streams.md)
- [ARJ の圧縮ストリームとコンテナーの対応範囲](../engineering/arj-streams.md)
- [ARC コンテナーと圧縮ストリームのプロファイル](../engineering/arc-streams.md)
- [BH コンテナーと対応する BlakHole プロファイル](../engineering/bh-containers.md)
- [ACE、ALZ、EGG の資料とライセンス条件](../engineering/ace-alz-egg-format-sources.md)
- [読み取り専用の AES Crypt エンベロープ](../engineering/aescrypt-envelopes.md)
- [DEB コンテナー仕様](../engineering/deb-containers.md)
- [ASAR コンテナー仕様](../engineering/asar-containers.md)
- [UU、UUE、XXE のテキストエンベロープ](../engineering/text-envelopes.md)
- [WIM、MSI、インストーラー形式の資料とライセンス](../engineering/wim-msi-installer-format-sources.md)
- [ディスクイメージとその他のアーカイブ形式の資料とライセンス](../engineering/disk-image-archive-format-sources.md)
- [Bandizip の形式一覧と xdzip での実現可能性](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## デスクトップの操作とビジュアル設計

- [Qt Quick のビューとコントローラーの責務](../design/qt-quick-ui.md)
- [デスクトップのビジュアル仕様](../../visual/README.md)
- [UI 設計標準](../../visual/design-standard.md)
- [UI 設計と修正の手順](../visual/design-workflow.md)
- [ファイルの関連付けアイコン](../visual/file-association-icons.md)
- [HTML のビジュアル再現性検証](../visual/html-fidelity.md)
- [操作と文言の補足仕様](../ux/interaction-spec.md)
- [メインウィンドウの機能、コンポーネント、操作](../../ux/main-window-components.md)
- [ディレクトリーツリーの設計と統合](../../ux/directory-tree.md)
- [メインウィンドウのようこそ画面](../ux/welcome.md)
- [ようこそ画面の読み込み表示と作成完了](../ux/welcome-opening.md)
- [展開時のパスワード操作](../ux/password-extraction.md)
- [タスクウィンドウと整合性検証](../ux/task-windows.md)
- [設定ウィンドウと表示言語](../ux/settings-localization.md)
- [簡体字中国語 UI の用語](../ux/chinese-terminology.md)
- [システムメニューと通知領域](../ux/desktop-integration.md)
- [各プラットフォームのウィンドウとタイトルバー](../ux/platform-windows.md)
- [高忠実度の状態設計と実装仕様](../ux/hifi-state-spec.md)
- [v0.1 の操作プロトタイプと機能対応](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## プラットフォーム統合、リリース、信頼性

- [インストーラーとファイルの関連付け](../../engineering/installers-associations.md)
- [対応システムと保守方針](../engineering/support-policy.md)
- [ファイルマネージャーの操作と独立タスクホスト](../engineering/shell-actions.md)
- [ユーザーが開始する更新の仕様](../../engineering/software-update.md)
- [データのライフサイクルと診断](../engineering/privacy.md)
- [製品バージョンとビルドの識別](../engineering/versioning.md)
- [GitHub Release プロトコル v1](../engineering/release-protocol.md)
- [macOS のローカル開発用 DMG](../engineering/macos-deployment.md)
- [Linux の独立ランタイムディレクトリーと検証](../engineering/linux-deployment.md)
- [配布依存関係の検証と SBOM](../engineering/deployment-audit.md)
- [実際のコンポーネントの帰属表示とライセンス資料](../engineering/runtime-attribution.md)
- [ネイティブランタイムの出所と通知](../engineering/native-runtime-materials.md)
- [Qt SDK とソースの対応検証](../engineering/qt-sdk-correspondence.md)
- [Windows 開発ツールとランタイムのライセンス](../engineering/windows-licensing.md)
- [ライセンス決定と第三者の資料](../engineering/licensing.md)
- [依存関係と形式バックエンドの採用](../../engineering/dependencies.md)
- [対応するソースと再ビルド用の資料](../../engineering/source-delivery.md)
- [CLDR から ICU 73.2 のデータを再構築](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## ビルドと CLI のリファレンス

- [ビルド、配備、リリースの設計](../design/build-release.md)
- [コーディング規約と検証の層](../design/quality-engineering.md)
- [開発環境の準備](../engineering/development.md)
- [開発ビルドとプラットフォーム検証](../../engineering/developer-build.md)
- [ツールチェーンとプラットフォームの固定](../engineering/toolchain.md)
- [CLI コマンド、JSON、インストール、更新の記録](../engineering/command-line.md)
