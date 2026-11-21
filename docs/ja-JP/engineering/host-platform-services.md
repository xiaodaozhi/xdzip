<a name="xdzip-host-calendar-and-encoding-services-for-format-workers"></a>
# フォーマットワーカーが使用するホストの暦と文字エンコーディング

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | [繁體中文](../../zh-TW/engineering/host-platform-services.md) | 日本語 | [한국어](../../ko-KR/engineering/host-platform-services.md) | [Français](../../fr-FR/engineering/host-platform-services.md) | [Deutsch](../../de-DE/engineering/host-platform-services.md) | [Español](../../es-ES/engineering/host-platform-services.md) | [Português (Brasil)](../../pt-BR/engineering/host-platform-services.md) | [Русский](../../ru-RU/engineering/host-platform-services.md)

Rust はフォーマットのフィールド、パスワードの互換性規則、検証を扱います。OS の過去のタイムゾーン規則、OEM と ANSI のコードページ、ローカルの C エンコーディングはホスト環境に属し、`ReadSource` のオプションサービスを通じて参照できます。フォーマット層はファイル名、HostOS、現在の UTC オフセットからこれらを推測してはいけません。サービスは Qt に依存せず、パスへのアクセス権やファイルの確定権限を与えません。

<a name="xdzip-calendar-projection"></a>

## 暦への変換

`Timestamp` は、1601 年から 100 ns 単位で数える FILETIME と、1970 年から符号なし整数秒で数える Unix 時刻を区別します。`ReadSource::local_calendar` はオプションの `Calendar` を返します。通常ファイルとメモリソースでは、既定で決定的な UTC 変換を使います。製品のワーカーは、同期 CXX コールバックを通じて OS のローカルタイムゾーン規則を使います。Windows はシステムのタイムゾーン API、Unix は `localtime_r` を使い、どちらも変換結果を確認します。変換できなければ暦を返しません。未初期化のフィールドは読み取ってはいけません。

Rust は DOS の表示値を生成する前に、年月日と時分秒を検証します。7z は従来の UTC と表示範囲 1980..2107 を維持します。RAR は内部の剰余演算、時刻の上書き、プラットフォーム差異に関する[固定済みの互換性規則](rar-format-sources.md)を適用してから、ホストの暦と従来の年の折り返しを使います。元の精度、未設定マーカー、完全な物理ヘッダーは非公開フォーマットインデックスに保持され、表示精度で上書きされません。安全性を優先する RAR の処理は、変換できない時刻を 0 として表示します。参照実装では、確認していない OS エラーの後に未定義データをコピーする場合があります。

<a name="xdzip-controlled-encoding-mappings"></a>

## 制御された文字コード変換

| サービス | 入力と出力 | ホストの境界 |
| --- | --- | --- |
| `oem_to_ansi_wide` | NUL を含む UTF-16 512 単位から UTF-16 512 単位 | Windows の OEM と ANSI を経由する一連の変換で、実際の文字対応と代替動作を保持 |
| `wide_to_ansi` | NUL を含む UTF-16 最大 512 単位から 512 バイト | 現在の Windows ANSI コードページ。共通の置換文字を仮定しない |
| `wide_to_local_prefix` | NUL を含むワイドスカラー最大 128 個から 512 バイト | 現在の Unix の C エンコーディング。失敗時は固定した互換性規則に従って消去と再試行を行い、変換済みの接頭部分を保持 |

返される長さは NUL を含みません。ホストサービスがない既定実装は `None` を返します。フォーマット層では、定義済みの移植可能な代替処理を使うか、明示的に変換を拒否します。コードページを推測してはいけません。製品のコールバックは、サイズ、NUL、結果範囲を検証します。利用できない変換には一貫して Unsupported を返します。

Windows の [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) の長さ引数は入力のワイド文字を数え、NUL の後ろにあるパディングも変換を続けます。出力容量のバイト数は表しません。ホストは `GetCPInfo(CP_OEMCP)` で 1 単位当たり最大 4 バイトのコードページを確認し、2,048 バイトの一時バッファーで全 512 単位を変換します。その後でバイト 511 を NUL に設定して ANSI に変換します。必要な容量が確認できない場合は変換を拒否します。512 個のワイド文字を 512 バイトに受け取ったり、容量要件を避けるために入力を先に削ったりしてはいけません。実際の OS 検証では DBCS の 511 単位境界全体と、WinAPI を直接呼び出した結果との独立した比較を含めます。

512 単位の受付、最初の NUL、OEM 補正の条件、127 単位およびバイト単位の切り詰め、旧来のマーカープロトコル、macOS の明示的な UTF8 は Rust のフォーマット互換性規則です。OS の変換が本来先に行われる場合は、その前にパスワードを切り詰めてはいけません。DBCS シーケンス内の切断を任意に文字境界へ移してはいけません。

<a name="xdzip-ownership-and-failure"></a>

## 所有権と失敗時の扱い

スライスを借用するのは同期呼び出しの間だけです。C++ はポインターを保持しません。機密の入出力を消去する時期は Rust の呼び出し側が管理します。上限付きの C++ の一時ワイドバッファーとナローバッファーは、すべての戻り経路で消去します。OS の内部や過去のすべてのメモリコピーの消去は保証されません。パスワードをログに残したり、プロセス引数や環境変数に入れたりする機能はありません。

読取側はサービスを呼び出す前にキャンセルを確認します。サービスの結果は展開出力に含まれず、読み取り、デコード、書き込みのバイト数を変えません。失敗時はソース、キャンセル、上限の既存分類を保ち、パスワードエラーと混同してはいけません。パスワードと完全性の原因を区別できないかどうかは、実際の検証情報に基づいてフォーマット層だけが判定します。
