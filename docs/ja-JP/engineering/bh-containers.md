<a name="xdzip-bh-containers-and-the-admitted-blakhole-profile"></a>

# BH コンテナーと対応する BlakHole プロファイル

[English](../../engineering/bh-containers.md) | [简体中文](../../zh-CN/engineering/bh-containers.md) | [繁體中文](../../zh-TW/engineering/bh-containers.md) | 日本語 | [한국어](../../ko-KR/engineering/bh-containers.md) | [Français](../../fr-FR/engineering/bh-containers.md) | [Deutsch](../../de-DE/engineering/bh-containers.md) | [Español](../../es-ES/engineering/bh-containers.md) | [Português (Brasil)](../../pt-BR/engineering/bh-containers.md) | [Русский](../../ru-RU/engineering/bh-containers.md)

`xdzip-codec-bh` はホストが管理するソースから BlakHole（`.bh`）の項目を順番に読み取ります。対象は `BH 05 07` 接頭部、観測済みの固定ヘッダー値、通常ファイルのメタデータ、正確な圧縮サイズとデコード後サイズ、項目ごとに宣言された IEEE CRC-32 を備えるものに限ります。物理順序を保つため、同名の項目は個別に扱われます。固定された行の識別子は表示名のデコードに依存しません。

項目の内容に対応するのは方式 8 のみです。これは RFC 1951 の raw Deflate です。上限付きの共有 Deflate リーダーは、ストリームが完全で、圧縮データの範囲を過不足なく消費することを要求します。余分な末尾バイト、途中で切れたストリーム、デコード後サイズや CRC の不一致は失敗になります。不明な方式は警告付きで表示され、対象項目を展開したり検証済みの内容に数えたりすることはできません。

対象のヘッダーは 5 バイトの固定接頭部、リトルエンディアンの 16 ビットのヘッダー長、固定バイト `25 d8 0c 00`、方式、DOS 形式に準じた 4 バイトの時刻、圧縮サイズ、デコード後サイズ、CRC-32、属性 `0x20`、ゼロの予約用 5 バイト、4 バイトのファイル名長、ファイル名のバイト列です。ヘッダー長は `37 + filename_length` と一致する必要があります。項目は隙間なく続き、最後の内容は物理ファイルの終端と一致します。アーカイブのトレーラー、ディレクトリーレコード、SFX 接頭部、再同期、パディング、隣接ファイルの検索は対象外です。

BH には現在も保守されている公開仕様がありません。形式の観測結果は `ashang/libxad` の `support/formats/BLAKHOLE.TXT`（リビジョン `dbe67a865207e49cd917ad9170a11b9eaa83998a`、SHA-256 `fdb2b4d2fffa7760eef4b3ccbabdcd6b2d43df33949a1b793f9efb7e4c2c0f2d`）と、Hawkynt/CompressionWorkbench の `docs/BLAKHOLE-ON-DISK.md`（コミット `ab61c59f789d82c628d3c5b4bac264afbdc0c7a7`、SHA-256 `f79f66d98d6c922ea4b64bde3ff5540a4d894de6632375513f3db89876e27bca`）の指定版を使います。後者のサンプルは出所が確認できる 5 項目入りの一件です。Store、暗号化、ディレクトリー、ソリッドブロック、分割ボリュームの動作を示すものではありません。

IZArc Command Line Add-On v1.1 はビルド時の参照用です。公式ダウンロードの MD5 は `9002c7634768e5336244f9accb2223b0`、ローカルの `IZArcCL.exe` の SHA-256 は `959050AFC513C64FD61F9EE15837054D0FAEE5EFB98C9107B4DC5D9A528A62D4` です。この環境では展開した `IZARCC.exe` と `IZARCE.exe` が、出力前に Windows の `0xC0000005` で終了します。このツールを根拠に Store、圧縮レベル、パスワード、展開への対応は主張しません。参照物は `build/toolchain` にあり、製品の依存関係には含まれません。

BH は読み取り専用です。作成、変更、暗号化、分割ボリューム、Linux 専用の MIME 登録は対象外です。この範囲は形式レジストリーと[アーカイブ検証仕様](archive-checking.md)で定義します。`.bh` 拡張子から検出を開始できます。結果は内容の完全な検証で決まります。
