<a name="xdzip-ace-alz-and-egg-source-and-licensing-gates"></a>

# ACE、ALZ、EGG の参照資料とライセンス条件

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | [繁體中文](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | 日本語 | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

xdzip の ACE、ALZ、EGG 対応と、その判断に用いる公開技術資料およびライセンス上の制限を説明します。参照ファイルは `build/toolchain/references` に置き、製品の依存関係には含めません。

<a name="xdzip-ace"></a>

## ACE

- Marcel Lemke の技術資料 *Technical information of the archiver ACE v1.2*。入手元は `wdlkmpx/unace`、SHA-256 は `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`。
- 独立実装 `droe/acefile`、コミット `11ed076ae19e8228148731cfbc05f019a395b3b9`。BSD-2-Clause の実装ライセンスの SHA-256 は `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`、実装の SHA-256 は `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`。
- 実装は ACE 1.0 と 2.0 を読み込めるとしています。LZ77、DELTA、EXE、PIC、SOUND、パスワード、ボリュームを含みます。ライセンスは Python 実装だけに適用され、ACE 技術に関する権利を主張していません。

技術資料はブロックヘッダー、CRC、無圧縮のメンバー、旧 LZ77 系を定義するのに十分です。ACE 技術の商用利用を許諾するものではありません。実装から WinAce と unace への参照だけでは、独立した相互運用性のサンプル集にはなりません。製品では ACE を恒久的に対象外とします。xdzip は ACE を実装も登録もせず、拡張子、検出、MIME タイプ、関連付けも追加しません。

<a name="xdzip-alz"></a>

## ALZ

- 形式仕様 `alkegi/docs`、コミット `36230c34d3056604cc63519e9855a79e894496b7`。CC0 ライセンスの SHA-256 は `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`。
- `alz.md` の SHA-256 は `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`。
- 文書は ALZ とローカルファイルのシグネチャー、可変幅のサイズ、CRC-32、UTF-8/CP949 名、Store、RFC 1951 の生 Deflate、ALZ で変更された BZip2 ストリームを定義します。従来の PKWARE 暗号化と分割ボリュームの構造も記載します。

xdzip は限定した読み取り専用の ALZ に対応します。通常の非暗号化単一ボリュームで Store または生 Deflate を扱い、CRC、サイズ、ストリーム終端を厳密に確認します。コーデック、ランタイム、CLI、関連付けは自動テストの対象です。変更版 BZip2、ZipCrypto、分割ボリュームには独立した完全なサンプルと、個別のリソースおよび安全性の検査が必要です。文書が CC0 であっても、すべての実装上の詳細に第三者の特許や商標上の義務がないとは判断できません。

<a name="xdzip-egg"></a>

## EGG

- 公開形式文書 `alkegi/docs`、上記と同じコミット `36230c34d3056604cc63519e9855a79e894496b7` と CC0 ライセンス。
- `egg.md` の SHA-256 は `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`。
- 参照モジュール `dterracino/UnEgg`、コミット `f21e653b02f7ab467bbc16af879e68a48dabcb2b`。EGG 仕様 PDF の SHA-256 は `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`。
- UnEGG ソースライセンスの SHA-256 は、ソースパッケージが `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6`、モジュールパッケージが `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0`。

UnEGG のライセンスでは、商用利用に ESTsoft の許可が必要で、再配布は非商用に限られ、モジュールを EGG 圧縮プログラムの開発に使うことは禁止されています。すべての権利を ESTsoft が留保します。このためモジュールはテスト用の参照に限り、複製、リンク、製品の依存関係への追加は行えません。

形式自体は制限付きのモジュールから独立して実装できます。xdzip は単一ボリュームの Store と生 Deflate を、非暗号化かつ非ソリッドに限って独立実装で扱います。正確なブロックサイズと CRC-32 はコーデック、ランタイム、CLI、関連付けのテスト対象です。標準 BZip2 と資料にある LZMA のフレーム構造には、個別の完全なサンプルと審査が必要です。AZO、AES/LEA、ZipCrypto、分割アーカイブ、ソリッドアーカイブは現時点で対象外です。

ALZ と EGG はそれぞれコンテナのヘッダーを検証し、物理エントリを自身のメタデータスナップショットと照合します。受理後の Store と raw Deflate には同じ上限付きリーダーを使います。このリーダーは宣言された長さと CRC-32 を検証し、入力、デコード済みバイト数、処理量をエントリ全体で累積します。

<a name="xdzip-required-implementation-gates"></a>

## 実装を採用する条件

1. 参照資料は製品ツリーの外に置き、使用前に固定ハッシュを確認します。
2. 採用する各方式に独立した完全なサンプルを用意します。ヘッダーのみ、または Store のみのサンプルでは方式の相互運用性を確認できません。
3. コンテナー、ランタイム、CLI、デスクトップの各入口を同時に追加します。未知の方式は表示し、データを処理する権限は与えません。
4. パスワード、ボリューム、ソリッド状態、特殊項目は、個別の出典、安全性、キャンセルの検証を通過するまで無効にします。
5. 公開機能レジストリを変更する前に、ライセンス上の判断とサンプルの識別情報を issue に記録します。
