<a name="xdzip-developer-preparation-guide"></a>

# 開発環境の準備

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | [繁體中文](../../zh-TW/engineering/development.md) | 日本語 | [한국어](../../ko-KR/engineering/development.md) | [Français](../../fr-FR/engineering/development.md) | [Deutsch](../../de-DE/engineering/development.md) | [Español](../../es-ES/engineering/development.md) | [Português (Brasil)](../../pt-BR/engineering/development.md) | [Русский](../../ru-RU/engineering/development.md)

変更に合うビルドと検証方法は[ビルドとコントリビュート](../contributing/README.md)を参照してください。[システム構成](../design/system-architecture.md)と[リポジトリー構成](repository-layout.md)は、Qt、Rust ランタイム、コーデックの境界を説明します。

<a name="xdzip-obtain-and-modify"></a>

## ソースを取得して変更する

最新の `main` を起点にし、作業ディレクトリーにあるほかの変更を保持します。変更範囲とレビューは[コントリビュート方針](../../../CONTRIBUTING.md)、許可された統合手順は[リポジトリーの提供手順](../../../developers/engineering/repository-hygiene.md)に従います。仕様を変更した場合は、対応する言語版と影響する検証を更新します。色と寸法は `specs/visual` を使います。

<a name="xdzip-documentation-builds"></a>

## ドキュメントの生成と検証

対応する表示言語ごとにルートの README と docs の訳文をそろえます。既定の英語は `README.md` と `docs/`、翻訳は `README.<locale>.md` と `docs/<locale>/` を使います。メンテナーハンドブックは米国英語と簡体字中国語です。タイトルの下に言語選択を置き、現在の言語を通常の文字で表示し、ほかの言語は同じ主題のページへリンクします。簡体字中国語の記事からは簡体字のメンテナーページへ、ほかの言語からは既定の英語ページへリンクします。ページの追加や移動時には索引、リンク元、章のアンカーを更新します。

ドキュメントのツールには Python 3.10+、lxml、Pandoc、Graphviz が必要です。システムの CJK フォントを推奨します。`python3 scripts/build-design-html.py` は `docs/zh-CN/design/` の中国語を元に、16 の詳細な設計文書と単一ファイル版を build/docs-html に生成します。ダウンロード用の一式は dist/ に書き出します。ほかの技術や共同作業の説明は docs/README.md を参照してください。HTML はドキュメントの出力形式です。製品 UI は Qt Quick を使います。

`python3 scripts/check-docs.py` は翻訳範囲、相互の言語リンク、本文リンクの対象言語、Markdown のアンカー、JSON、デザイン素材をオフラインで確認します。HTML ジェネレーターは出力のリンクと構造を検証します。チェッカーの変更時は `python3 scripts/test-check-docs.py` を実行します。

<a name="xdzip-product-toolchain-preparation"></a>

## 製品のツールチェーンを準備する

macOS では Xcode Command Line Tools と Qt、Windows では MSVC Build Tools、Windows SDK、対応する Qt を用意します。Linux ではコンパイラー、対象の glibc 環境、Qt の Wayland/X11 依存関係が必要です。開発用バージョンはルートのツールチェーンファイルと CI で固定します。ソースとライセンスは[ツールチェーン表](toolchain.md)に記録します。対応 OS と保守は[サポート方針](support-policy.md)を参照してください。個々の出所とライセンスは該当するリリース審査の Issue で確認します。

[Qt 開発アプリ](../../../apps/desktop/README.md)には、クリーンなチェックアウトからビルドと起動を行うコマンド、CMake のオプション、macOS のアーキテクチャーと配備対象、オフスクリーン試験の境界があります。ビルドやインストールを変更したら、空白や中国語文字を含むパスを再検証します。ローカル開発に有料の署名 ID は不要です。ローカル DMG の ad-hoc 完全性署名と、CI で Mac アプリの配備を行わない範囲は[ビルドガイド](developer-build.md)を参照してください。

<a name="xdzip-independent-engineering-probes"></a>

## 独立した技術プローブ

[ツールチェーンのプローブ](../../../tools/toolchain-probe/README.md)は、独立した Qt Quick ウィンドウの最小ビルドと CTest コマンドを提供します。バージョンと試験範囲はその README に、実際の結果は[初期検証の Issue](https://github.com/xiaodaozhi/xdzip/issues/1)にあります。

[統合 CMake/Cargo ブリッジプローブ](../../../tools/bridge-probe/README.md)は、ルートからのビルド、固定の依存関係、ライフサイクル検証を提供します。依存関係は[cxx の採用理由](cxx-admission.md)を参照してください。
