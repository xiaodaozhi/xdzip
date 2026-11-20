<a name="xdzip-zip-extraction-and-output-dependency-admission"></a>

# ZIP 展開と出力に関する依存関係の採用

[English](../../engineering/zip-extraction-admission.md) | [简体中文](../../zh-CN/engineering/zip-extraction-admission.md) | [繁體中文](../../zh-TW/engineering/zip-extraction-admission.md) | 日本語 | [한국어](../../ko-KR/engineering/zip-extraction-admission.md) | [Français](../../fr-FR/engineering/zip-extraction-admission.md) | [Deutsch](../../de-DE/engineering/zip-extraction-admission.md) | [Español](../../es-ES/engineering/zip-extraction-admission.md) | [Português (Brasil)](../../pt-BR/engineering/zip-extraction-admission.md) | [Русский](../../ru-RU/engineering/zip-extraction-admission.md)

このページは、安全な展開とパスワード付き展開で共通する依存関係の境界を定めます。初回採用時の依存関係の出所、ライセンス、機能の抜粋は[固定された基礎資料](../../../specs/engineering/zip-extraction-dependencies.json)にあります。AES を含む現在の正確なバージョンと crates.io のチェックサムはルートの Cargo.lock に従います。有効な依存関係全体は `cargo tree --locked -e features` で確認してください。基礎資料の抜粋を現在の完全な依存関係一覧として扱わないでください。

| 依存関係 | 用途と判断 |
| --- | --- |
| xdzip-compression | プロジェクト独自の共通 Deflate と CRC-32。固定バッファーを用い、最終ブロックと圧縮バイトの正確な消費を厳密に検証します。独立した形式層が ZIP コンテナー、記述子、AES ラッピングを処理し、ホストの入力と出力先を借用しつつ、出力の承認と完全な検証の境界を維持します。flate2 は開発テストの参照用に限ります。 |
| cap-std / cap-fs-ext 4.0.3 | 保持されたディレクトリー能力、nofollow によるサブディレクトリーのオープン、排他的な一時準備、相対的な公開を提供します。文字列を連結し canonicalize 後に作成する方式と異なり、ホストは各構成要素を承認して ID を再確認し、汎用 unpack は呼び出しません。cap-primitives などの推移的パッケージも固定します。 |
| getrandom 0.4.2 | プライベートな作業領域の nonce に OS の乱数を使用し、タイムスタンプ、疑似乱数値、アーカイブ名は使用しません。既定の機能は無効です。 |
| rustix 1.1.5 | POSIX の実効ユーザー ID とファイルシステムの基本操作に固定済みバージョンを再利用します。既定の機能は無効で、std/process と推移的に必要な fs のみを有効にします。 |
| windows-sys 0.61.2 | Windows オブジェクトの所有者と DACL を照会し SID を検査します。手書きの unsafe は分離し、有効なハンドル、DWORD に整列したバッファー、RAII による LocalFree を借用します。読み取り専用属性だけの検査と異なり、DACL 検査はプライベート出力の前提を確認します。新しい FFI ではハンドル、バッファー、解放の前提と照会失敗時の動作を検証する必要があります。 |
| unicode-normalization 0.1.25 | タスク名の別名に対する NFD と小文字化の採用キーを生成し、出力名の元の表記を保持します。このキーをファイルシステムの完全な大文字小文字の正規化とは扱いません。ASCII 小文字を超えて、macOS でよくある Unicode 等価な名前を扱います。 |
| AES, HMAC, SHA-1, PBKDF2, constant_time_eq | 採用された汎用暗号プリミティブです。正確なバージョンと機能は[依存関係契約](../../engineering/dependencies.md)に記載します。ZIP の CTR カウント、ソルトと検証値の配置、暗号文の認証、エラー分類はプロジェクトが独立して実装します。zip 9.0.3 は開発テストの参照用に限ります。 |
| zeroize 1.9.1 | パスワード、導出鍵、テスト用プローブの制限された stdin バッファーをゼロ化するライフサイクルを提供します。AES/HMAC/SHA-1 もそれぞれの消去機能を有効にします。Qt、IME、過去のメモリーコピーすべての消去を保証するものではありません。 |

汎用依存関係の Cargo 設定、ライセンス、公開 API を確認してください。独立した形式実装の出所に関する規則は[独立実装契約](../../engineering/independent-rust-codecs.md)にあります。cap ファミリーは Bytecode Alliance が保守し、getrandom と rustix はシステムインターフェースを提供し、Unicode normalization は汎用 Unicode ライブラリーです。更新時もバージョンの固定とネイティブ境界の回帰テストが必要であり、保守者の存在は回帰テストや脆弱性情報の確認に代わりません。新しいパッケージは実行時にネットワーク要求を行いません。ビルドスクリプトはコンパイラーやターゲットの設定を調べるか Rust のシステムコールバックエンドを選ぶだけで、外部コードをダウンロードせず、C デコーダーもコンパイルしません。ライセンス通知には上流の全文を収集し、実際のライセンス式は固定資料に従います。

[検証方法](../../../developers/qa/README.md)に従い、正確なバージョンを対象とした OSV 照会の実際の応答を保存してください。照会の失敗を脆弱性情報なしと解釈してはいけません。これは完全な cargo-audit ではありません。既存の cxx の脆弱性情報とバージョン選択は引き続き [M2 依存関係記録](../../engineering/dependencies.md)にあります。

メタデータの採用はデコードを自動的に許可しません。独立したアダプターはローカルヘッダー、ストリーム終端、CRC、32/64 ビットのデータ記述子を検証し、中央およびローカルの ZIP64 センチネル値に従って幅を選択します。独立した Python force_zip64 テストデータは、有効な幅の混在と改ざんされたサイズを対象とします。rawzip、zip、flate2 は ZIP とホストの通常およびビルド依存関係の閉包に含まれません。過去の固定資料の抜粋は当時の出所のみを保持します。

固定資料には、ターゲット横断の Cargo 解決依存関係をすべて記載します。getrandom の WASI/WIT/WebAssembly ツールチェーンはサポート対象外のターゲットにのみ適用され、macOS、Windows、Linux のデスクトップ向けコンパイルや実行には入りません。全ターゲットの Cargo 一覧をデスクトップ実行時の範囲と解釈してはいけません。

開発パッケージの通知は `cargo metadata --filter-platform <rustc host>` による実際の解決ノードを使い、現在のプラットフォームのワークスペースとビルドツールのライセンスを収集して target.json を記録します。コンパイルされていない WASI/UEFI パッケージはデスクトップ配布の依存関係として扱いません。ライセンス本文が欠ければパッケージングは失敗し、Cargo のライセンス名は本文の代わりになりません。r-efi 6.0.0 の完全な MIT 許諾と著作権表示は AUTHORS にあり、収集ツールはこのバージョンと固定 SHA-256 に限ってこの本文の例外を認めます。
