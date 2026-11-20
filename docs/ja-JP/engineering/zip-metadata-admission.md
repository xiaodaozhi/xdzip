<a name="xdzip-zip-metadata-dependency-admission"></a>

# ZIP メタデータ依存の採用

[English](../../engineering/zip-metadata-admission.md) | [简体中文](../../zh-CN/engineering/zip-metadata-admission.md) | [繁體中文](../../zh-TW/engineering/zip-metadata-admission.md) | 日本語 | [한국어](../../ko-KR/engineering/zip-metadata-admission.md) | [Français](../../fr-FR/engineering/zip-metadata-admission.md) | [Deutsch](../../de-DE/engineering/zip-metadata-admission.md) | [Español](../../es-ES/engineering/zip-metadata-admission.md) | [Português (Brasil)](../../pt-BR/engineering/zip-metadata-admission.md) | [Русский](../../ru-RU/engineering/zip-metadata-admission.md)

このページは、最初の読み取り専用ディレクトリースキャンで行った 2026-10-11 の採用判断を記録します。展開、作成、サードパーティープラグインの実行は対象外です。現在の ZIP メタデータは[独立した Rust レコード層](../../engineering/independent-rust-codecs.md)が処理し、rawzip は開発時のテスト参照にのみ使います。

当時は [rawzip 0.5.2](https://github.com/nickbabcock/rawzip) を選びました。正確なバージョンとチェックサムは[固定マニフェスト](../../../specs/engineering/zip-metadata-dependencies.json)とルートの Cargo.lock にあります。既定の機能を無効にし、std を明示的に有効にしました。ランタイム依存や build.rs はありませんでした。MIT ライセンスの全文は [third_party/rawzip/LICENSE](../../../third_party/rawzip/LICENSE) に保存し、依存を使うリリースに同梱します。

当時は README、Cargo 設定、ディレクトリーを読む実装を確認しました。借用した Read + Seek 入力から中央ディレクトリーのレコードを一件ずつ読み、物理順序と同名項目を保持します。ホストは読み取り量と名前の合計サイズを制限しました。読み取り専用の候補として zip 9.0.1 も確認しましたが、名前で引く構造は物理レコードの重複保持に適しませんでした。AES の書き込みには後から zip-rs 9.0.3 を別に導入し、読み取り専用の物理索引は置き換えませんでした。独立実装はこの索引の仕様を保持します。

RustSec のスナップショット `7eebec69c352c7191b1f13eb95dd510eeca5d1de` には rawzip の勧告がありませんでした。RUSTSEC-2026-0202 は 1.0.195 未満の cxx の let_cxx_string! に影響し、固定した 1.0.205 には修正が含まれます。ここに記録するのは対象の勧告を確認した結果です。

メタデータアダプターはデコーダーを有効にせず、内容、CRC、ローカルヘッダーと中央ディレクトリーの整合を検証しません。不明な方式は番号のみ表示します。一覧にあっても展開できるとは限りません。読み取り専用の M2 では単一ボリューム ZIP64 を追加し、100,000 レコードと 2^53 を超える宣言サイズを正確に確認しました。[分割ボリュームホスト](archive-opening.md)は対応する組を単一ボリュームの論理ビューに変換します。未変換のマルチディスクレコードは拒否します。Unicode 名の拡張と旧文字コードは[文字コード仕様](filename-encoding.md)に従います。実際の内容は[展開用の依存採用](zip-extraction-admission.md)と[安全な展開](../../engineering/zip-safe-extraction.md)で別に扱います。制限と失敗は[実装仕様](../../engineering/readonly-browser.md)を参照してください。

回帰試験では実際の ZIP64、不正なロケーターと終端、ZIP64 拡張の開始ディスク、名前・表示・読み取りの上限、キャンセル、ソース変更を検証します。独立実装では自己展開のオフセット、曖昧な終端、シードを固定した上限付き変異も確認します。製品の依存関係検査でライブラリー除去を確認します。テスト依存はロックファイルに残る場合があります。

展開の段階 M3 で追加した内容のデコードとホストによる出力は[展開用の依存採用](zip-extraction-admission.md)にあります。このページは読み取り専用の M2 の記録です。
