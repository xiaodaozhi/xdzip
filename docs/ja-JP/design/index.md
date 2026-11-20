<a name="xdzip-xdzip-design-guide"></a>

# XDZip の設計ガイド

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | [繁體中文](../../zh-TW/design/index.md) | 日本語 | [한국어](../../ko-KR/design/index.md) | [Français](../../fr-FR/design/index.md) | [Deutsch](../../de-DE/design/index.md) | [Español](../../es-ES/design/index.md) | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

まず[システム構成](../../design/system-architecture.md)を読むと、Qt Quick デスクトップと独立した Rust CLI がアーカイブのランタイムを呼び出す仕組みが分かります。設計文書は現在の仕様、決定事項、提案の範囲を説明します。インストールと日常操作は[ユーザーマニュアル](../user-guide/README.md)を参照してください。ビルドや変更は[ビルドとコントリビュート](../contributing/README.md)を参照してください。

<a name="xdzip-find-a-design-topic"></a>

## 設計項目を探す

| 項目 | 参照先 |
| --- | --- |
| 製品の動作 | [製品要件](product-requirements.md)は想定する操作と範囲を定めます。[要件の追跡](requirements-traceability.md)から対応する検証を確認できます。 |
| アーカイブデータ | [ドメインモデル](domain-data.md)はセッション、ソースのバージョン、物理項目、クエリー、操作の入力を説明します。 |
| デスクトップ | [Qt Quick インターフェース](qt-quick-ui.md)はビュー、コントローラーの所有権、操作を扱います。[Qt/Rust ブリッジ](qt-rust-bridge.md)は実際のコマンド、ID、イベント、ライフサイクルを説明します。 |
| 形式と拡張の範囲 | [形式アダプターと拡張の試験](codec-extension-sdk.md)は組み込みレジストリー、付属ワーカー、独立した XDP1 試験を説明します。 |
| 出力とバックグラウンド処理 | [ファイルの許可と安全な確定](file-security.md)は一時出力、検証、コミットを扱います。[タスクスケジューリング](task-scheduling.md)は受理、確認、進行状況、キャンセルを説明します。 |
| 性能と品質 | [性能設計](performance.md)は予算、測定、目標を説明します。[品質管理](quality-engineering.md)は変更に対する検証を示します。 |
| 配布と決定事項 | [ビルド、配備、リリース](build-release.md)はプラットフォーム、パッケージ、更新を扱います。[アーキテクチャーの決定](architecture-decisions.md)は理由を記録し、[アーキテクチャーの変遷](architecture-evolution.md)は試作から製品に至る経緯を説明します。 |
| 用語 | [用語と実装上の制限](glossary.md)は用語を説明し、各サブシステムの現在の上限へのリンクを掲載しています。 |

<a name="xdzip-interpreting-the-references"></a>

## 参照資料の読み方

現在の動作は実装へのリンクとともに記載しています。設計案と実験用ツールにはそれぞれ対象範囲があります。[技術リファレンス](../reference/README.md)から個別の仕様を参照できます。[プロジェクト README](../../../README.ja-JP.md)と該当するリリースノートは配布方法と提供機能を説明します。時間、リソース、実機検証の結果は記録されたビルド、プラットフォーム、作業負荷に適用されます。
