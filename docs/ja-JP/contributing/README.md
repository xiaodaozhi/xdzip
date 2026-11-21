# XDZip のビルドと貢献

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | [繁體中文](../../zh-TW/contributing/README.md) | 日本語 | [한국어](../../ko-KR/contributing/README.md) | [Français](../../fr-FR/contributing/README.md) | [Deutsch](../../de-DE/contributing/README.md) | [Español](../../es-ES/contributing/README.md) | [Português (Brasil)](../../pt-BR/contributing/README.md) | [Русский](../../ru-RU/contributing/README.md)

[貢献の方針](../../../CONTRIBUTING.md)を読み、変更に対応する issue を探すか新しく作成してください。期待する動作、影響するプラットフォーム、検証できる小さな再現例を記載します。プロジェクトが所有するか、再配布可能なテスト用ファイルを使ってください。実装の各部分の場所は[リポジトリの構成](../engineering/repository-layout.md)で確認できます。

<a name="xdzip-build-the-part-you-need"></a>

## 必要な部分をビルドする

コマンドはリポジトリのルートから実行します。Windows では x64 MSVC 開発環境で `python` を使用します。macOS では Apple Silicon ネイティブのターミナルと Xcode Command Line Tools を使います。Ubuntu ではプラットフォーム向けの依存関係一覧に従います。[ビルドガイド](../../engineering/developer-build.md#environment-and-entry-points)には前提条件と、リポジトリで固定した Rust と Qt のツールチェーンが記載されています。

デスクトップを初めてビルドする場合は次のコマンドを使います。

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

準備オプションは既存の rustup を使って固定バージョンをインストールし、必要に応じて固定した Qt SDK をダウンロードします。準備済みなら共有の `build/toolchain/` にある依存関係を再利用します。既定の場所以外に互換性のある Qt SDK がある場合は、該当する各段階で同じ `--qt-prefix` を指定します。デスクトップのテストが必要な変更では、同じ設定で完了したビルドに対して `python3 scripts/desktop.py test` を実行してください。ワーカーの全テストは[テスト入力の準備](../../engineering/developer-build.md)を参照してください。

スタンドアロン CLI を変更する場合は、CLI と同梱ワーカーをビルドして検証します。

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

既定の出力先は `build/cli` です。CLI のビルドには固定した Rust ツールチェーン、CMake、プラットフォームの C++ ツールチェーンが必要です。Ubuntu では `libssl-dev` と `pkg-config` も必要です。プラットフォームごとの詳細は [CLI ビルド仕様](../engineering/command-line.md#xdzip-build-and-command-installation)を参照してください。

<a name="xdzip-verify-the-affected-behavior"></a>

## 影響する動作を検証する

条件に合う既存ビルドがあれば再利用します。ドキュメントのみを変更する場合は下記の検証から始め、関係するジェネレーターの出力を確認します。デスクトップ全体のビルドは、変更内容または必須の検証で求められる場合に実行します。

```sh
python3 scripts/check-docs.py
git diff --check
```

| 変更 | コードと仕様 | 実行または確認する項目 |
| --- | --- | --- |
| ドキュメントと例 | 対応する翻訳、リンク、アンカーを更新します。ルートの README と `docs/` は 10 言語を用意します。`developers/` のガイドは英語（米国）と簡体字中国語です。実装仕様に従い、変更したコマンドは自作の小さなテスト用ファイルで確認します。 | 上記のドキュメント検証を実行します。変更した章へのリンクを開き、ジェネレーターの入力になるページでは生成結果を確認します。 |
| CLI | `crates/xdzip-cli` から共有の Rust ランタイムをたどります。[CLI 仕様](../engineering/command-line.md) | ビルドしたスタンドアロン CLI を検証します。コマンドまたは出力を変更した場合は、stdout、stderr、終了コード、書き込みファイルを確認します。対応する失敗ケースも含めます。 |
| Rust のドメイン、形式、アーカイブタスク | `xdzip-domain`、対象のコーデック、`xdzip-runtime`、または `xdzip-io` を確認します。デスクトップのコマンドは `BrowserController` と `RuntimeConnection` をたどります。[形式一覧](../engineering/format-registry.md)、[ランタイム](../engineering/product-runtime.md)、[ファイル安全性](../design/file-security.md) | `cargo fmt --all -- --check` と対象 crate のテストを実行します。実際のランタイム経路と、該当する失敗、キャンセル、書き込み確定の動作を検証します。ネイティブ形式ワーカーを使う経路は[ワーカーのテスト入口](../../../developers/qa/runtime-worker-tests.md)から検証します。登録表を変えた場合は `python3 scripts/generate-format-registry.py` で再生成し、`--check` で確認して `python3 scripts/test-format-registry.py` を実行します。 |
| QML ウィンドウ、コントロール、ローカライズ | `apps/desktop/ui`、`resources/i18n`、`specs/visual` を確認します。ウィンドウの変更前に[ビジュアル仕様](../../visual/design-standard.md)と[デザイン手順](../visual/design-workflow.md)を読み、[ウィンドウ登録表](../../../specs/visual/window-registry.json)に仕様を記録してください。 | ウィンドウを変更したら `python3 scripts/check-window-contracts.py --self-test`、`python3 scripts/check-operation-components.py`、該当する 2 言語と複数の文字サイズのレイアウトテストを実行します。ローカライズには `python3 scripts/check-ui-i18n.py` を実行します。必要に応じて、対象プラットフォームのネイティブ操作や表示を確認します。 |
| インストール、シェル統合、更新 | [ビルドと配置のガイド](../../engineering/developer-build.md)、[ファイルの関連付けの仕様](../engineering/installers-associations.md)、[更新仕様](../engineering/software-update.md)を参照します。 | 影響するインストールまたは登録経路、実際の成果物の識別情報、エラー、復旧を確認します。該当するプラットフォームのワークフローは [CI 方針](../../../developers/engineering/ci-policy.md)を参照してください。 |

<a name="xdzip-describe-the-evidence"></a>

## 検証結果を記載する

PR には変更した動作、検証に使ったコマンドとプラットフォーム、残っている検証を記載します。画面を使用しない検証、ネイティブデスクトップ、実際のインストールの結果を明記してください。それぞれの対象は[検証ガイド](../../../developers/qa/README.md)に記載されています。各機能の仕様は[技術リファレンス](../reference/README.md)、公開手順は[メンテナーハンドブック](../../../developers/README.md)を参照してください。
