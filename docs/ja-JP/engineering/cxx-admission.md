<a name="xdzip-cxx-bridge-dependency-rationale"></a>

# cxx ブリッジを採用した理由

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | [繁體中文（台灣）](../../zh-TW/engineering/cxx-admission.md) | 日本語 | [한국어](../../ko-KR/engineering/cxx-admission.md) | [Français](../../fr-FR/engineering/cxx-admission.md) | [Deutsch](../../de-DE/engineering/cxx-admission.md) | [Español](../../es-ES/engineering/cxx-admission.md) | [Português (Brasil)](../../pt-BR/engineering/cxx-admission.md) | [Русский](../../ru-RU/engineering/cxx-admission.md)

Qt と Rust は `cxx` が生成する型付きブリッジで所有権を扱います。Qt オブジェクトは C++ が、ドメインとランタイムのロジックは Rust が保持します。[Cargo.lock](../../../Cargo.lock) と[ツールチェーン方針](toolchain.md)に従い、`cxx` と `cxx-build` のバージョンは完全に一致させます。cxx はサードパーティーの形式拡張向けの安定 ABI ではありません。手書きの C ABI で依存は減らせますが、文字列、自由関数、所有権の不変条件の保守が増えます。

<a name="xdzip-source-and-licensing"></a>

## ソースとライセンス

ソースの配布元は crates.io、上流は [dtolnay/cxx](https://github.com/dtolnay/cxx) です。ハッシュ、バージョン、機能、ビルドスクリプト、ライセンスの宣言はロックファイルと[ブリッジの依存関係一覧](../../../specs/engineering/bridge-dependencies.json)に記録します。公開時は実際に該当する通知を再確認してまとめます。ロックファイルには対象プラットフォームで条件が当てはまらない依存関係も含まれます。実際にリンクするバイナリー、コンパイラー、Qt、システムライブラリーは別に確認します。

<a name="xdzip-build-and-ownership-boundaries"></a>

## ビルドと所有権の境界

cxx-build はソースとヘッダーを生成し、C++ ビルドツールを呼び出します。cxx は付属のブリッジをコンパイルし、link-cplusplus は C++ 標準ライブラリーをリンクします。Cargo は取得した依存関係をロックファイルに照らして検証します。ビルドスクリプトや任意機能の追加時は、ネットワークと実行の動作を再確認します。以前の検査結果は新しいバージョンの安全性を示すものではありません。

std/alloc と、製品で実際に必要な機能を保持します。削減時には UTF-8、エラー伝播、一意なハンドル、バックプレッシャー、終了処理、保留作業の排出を再検証します。実際のインターフェースは[ランタイム仕様](product-runtime.md)と[ブリッジ設計](../design/qt-rust-bridge.md)にあります。ライセンス検証は[リリース審査手順](../../../developers/qa/release-review.md)に従います。

個別のプローブ、CI 実行、受け入れ結果は [WP-01 の Issue](https://github.com/xiaodaozhi/xdzip/issues/1)と Actions 成果物に記録します。
