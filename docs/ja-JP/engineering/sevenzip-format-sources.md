<a name="xdzip-7z-format-sources-and-independent-implementation"></a>

# 7z 形式の資料と独立実装

[English](../../engineering/sevenzip-format-sources.md) | [简体中文](../../zh-CN/engineering/sevenzip-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/sevenzip-format-sources.md) | 日本語 | [한국어](../../ko-KR/engineering/sevenzip-format-sources.md) | [Français](../../fr-FR/engineering/sevenzip-format-sources.md) | [Deutsch](../../de-DE/engineering/sevenzip-format-sources.md) | [Español](../../es-ES/engineering/sevenzip-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/sevenzip-format-sources.md) | [Русский](../../ru-RU/engineering/sevenzip-format-sources.md)

この文書は再利用可能な資料と実装の境界を記録します。移行の受け入れと残る機能は [issue #50](https://github.com/xiaodaozhi/xdzip/issues/50) で追跡します。Rust への固定ルーティングと製品の公式依存関係の除去は、複数プラットフォーム、手動、リリースの受け入れを自動的に証明するものではありません。一般的な置換規則は[独立実装契約](independent-rust-codecs.md)にあります。

<a name="xdzip-specifications-and-reference-identity"></a>

## 仕様と参照資料の識別

コンテナーフィールド、可変長整数、メソッドグラフ、プロパティ、ファイルの対応付けは公式の `DOC/7zFormat.txt` と `DOC/Methods.txt` に従います。固定された参照資料は [7-Zip 26.05 の公式ソースアーカイブ](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz)、SHA-256 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e` です。二つの文書の SHA-256 はそれぞれ `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b` と `b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7` です。

LZMA ビットストリームは Igor Pavlov による 2015-06-14 の[公式仕様パッケージ](https://www.7-zip.org/a/lzma-specification.7z)も参照します。アーカイブの SHA-256 は `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`、仕様本文は `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb` です。読むのは形式の説明、仕様の数学、疑似コードだけで、パッケージ内の参照デコーダーは Rust 移植の入力に使いません。文書やサンプルのダウンロードによって新たな再配布ライセンスは付与されません。

汎用 Deflate は RFC 1951 に従い、ZIP、GZIP、MSZIP と一つの実装を共有します。Deflate64 はその実装で 64 KiB の履歴、距離コード 30/31、長さコード 285 の拡張規則を明示的に選択します。通常の Deflate/MSZIP は変わりません。[PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) の拡張 Deflate 節は概要にすぎません。具体的なビットストリームの違いは固定参照の事実分析と独立した読み取り検証で補い、概要を完全な仕様として扱ってはいけません。AES-CBC の数学と独立ベクトルは [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf) に従います。7z のプロパティ、UTF-16LE パスワード導出、ゼロ埋めは追加の形式規則であり、ZIP AES や PKCS#7 から推論できません。BZip2 マニュアルと PPMd の形式名は歴史的なビットストリーム、モデル再起動、終了を完全には規定せず、マニュアルを読むだけでこれらを実装できると説明してはいけません。

<a name="xdzip-reference-analysis-and-original-coding"></a>

## 参照分析と独自コード

分析者は固定した元のアーカイブから次のグループを選んで読み、欠けているフィールド、数学的状態、容量による再起動、終了、エラーの関係を補います。引き継ぎには事実と識別要件だけを含め、参照コード、行ごとの書き直し、クラスや配列の配置、関数分割、コピーした制御フロー、翻訳したコメントを含めません。Rust のアルゴリズムは対応する参照実装を読んでいない実装者が独立して構成します。分析者は自分が分析したアルゴリズムを実装しません。コンテナーの実装者も公式コンテナー実装を読まず、文書と事実の引き継ぎから独立してコードを書きます。

| 分析対象 | 固定アーカイブ内の参照範囲 | ライセンスの区別 |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`、`C/Lzma2Dec.c/.h`、エンコーダーの range、chunk/reset、プロパティの部分、C++ Lzma/Lzma2 ラッパー | これらの C ファイルはパブリックドメインを宣言。C++ ラッパーはアーカイブの既定条件を使用 |
| コンテナーの詳細 | `CPP/7zip/Archive/7z/7zIn.cpp` の外部属性、サブストリームのチェックサム継承、ファイルプロパティ | アーカイブ既定の LGPL-2.1-or-later |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`、MyAes、FilterCoder、7z コーデックラッパーのブロック長、パスワード、埋め、完了の部分 | アーカイブ既定の LGPL-2.1-or-later。基盤 AES 実装は読まず移植しない |
| BZip2 | BZip2Decoder、Const、Crc、通常の 7z と独立した bz2 ラッパーにおけるエラー処理の違い | アーカイブ既定の LGPL-2.1-or-later。形式名を理由に他の実装のライセンスを割り当てない |
| PPMd7 | `C/Ppmd7.c/.h`、`Ppmd7Dec.c`、`Ppmd7Enc.c` の数学的状態、仮想容量、再起動の事実、C++ Ppmd ラッパーのプロパティと終了の制約 | これらの C ファイルはパブリックドメインを宣言。C++ ラッパーは既定条件を使用。コンテキスト配置、アロケーター実装、制御フローはコピーしない |
| Deflate64 | `CPP/7zip/Compress/DeflateConst.h` と DeflateDecoder の 64 ビットモード部分、Deflate64Register、7z プロパティの受理 | アーカイブ既定の LGPL-2.1-or-later。引き継ぎは通常 Deflate との形式上の違いだけ |
| ISA、Delta、Swap | `C/Bra.c`、`Bra86.c`、`Bra.h`、`Delta.c`、`SwapBytes.c` のスカラーの意味と C++ のメソッド登録、プロパティ、末尾ラッパー | これらの C ファイルはパブリックドメインを宣言。C++ ラッパーは別に既定条件を使用。SIMD 戦略は形式規則ではない |
| BCJ2 | `C/Bcj2.c/.h` の四つのストリームと数学的状態、C++ Bcj2Coder のデコーダー完了とサイズのラッパー | C ファイルはパブリックドメインを宣言。C++ ラッパーは既定条件を使用。エンコーダー本体は分析もコピーもしない |

固定アーカイブとファイルの識別情報、実際に読んだ範囲、事実の引き継ぎ要約を開発証拠とともに保持します。参照資料を更新したら再確認し、古い行番号で新バージョンの動作を証明しません。仕様の定数、識別子、数学的関係は形式実装に使えますが、内部最適化や歴史的に未定義の動作は製品への許可を与えません。パブリックドメイン宣言はアーカイブ全体、文書、C++ ラッパー、その他のサンプルには及びません。

これは厳密なクリーンルームや法的リスクゼロを主張するものではありません。第三者資料、公式バックエンドを含む過去の配布、独立したテストツールはそれぞれの条件に従います。現在の製品は公式バックエンドをビルドしません。[ライセンス](licensing.md)と[対応するソース](source-delivery.md)を参照してください。

<a name="xdzip-interoperability-boundaries"></a>

## 相互運用の境界

自作の書き込み側と読み取り側の間で成功しても、独立した参照を代替できません。テストは独立した圧縮器の既知の出力、自作圧縮ストリームの公式実装による読み取り、入力全体の消費、終了、CRC、破損、クォータ、キャンセル、ソースエラーを別々に検証します。コンポーネント検証とホスト、プロセス、プラットフォームの受け入れは別々に記録します。

歴史的な互換性の許容は個別に確認しなければなりません。例えば BZip2 の 7z ノードは、最後のバイトの未使用ビットが非ゼロであることと、ブロック末尾のちょうど四つの反復バイトに追加カウントバイトがないことを許容する場合がありますが、正しい CRC、宣言サイズ、ストリームの完全な終了は引き続き必要です。これによって末尾の余分なバイトや連結ストリームが許可されるわけではありません。最終バイト内のビットと追加バイトは同じ種類の埋めではありません。

PPMd7 は外側の長さが既知か未知かで終了プロファイルを区別します。正確な出力後、長さ既知の 7z ノードは追加の END 記号を消費せず range の終端状態と圧縮範囲を確認します。長さ未知の共通リーダーは END を要求します。形式が定めるモデル容量の再起動とホストのメモリー受理は別であり、メモリー不足で架空の形式再起動を起こしてはいけません。独立した比較入力には固定された公式 C エンコーダーが生成した容量再起動ストリームを含みます。生成器と参照の元ソースはローカルテスト専用で、製品の依存関係ではありません。
