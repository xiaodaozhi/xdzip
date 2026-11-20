<p align="center">
  <img src="design/assets/brand/xdzip-128.png" width="80" height="80" alt="xdzip のアプリアイコン">
</p>

<h1 align="center">xdzip</h1>

<p align="center"><a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文（台灣）</a> | 日本語 | <a href="README.ko-KR.md">한국어</a> | <a href="README.fr-FR.md">Français</a> | <a href="README.de-DE.md">Deutsch</a> | <a href="README.es-ES.md">Español</a> | <a href="README.pt-BR.md">Português (Brasil)</a> | <a href="README.ru-RU.md">Русский</a></p>

<p align="center"><strong>圧縮と展開を、もう少し手軽に。</strong></p>

<p align="center">xdzip はオープンソースのクロスプラットフォームアーカイブマネージャーです。Rust で実装したフォーマット処理は、外部の展開プログラムを必要としません。アーカイブ内を閲覧、検索して必要なファイルを展開できます。暗号化アーカイブや分割アーカイブを扱い、対応形式では項目の変更もできます。<br>Windows、macOS、Linux で ZIP、7z、RAR、TAR、CAB、ISO/UDF などをサポートします。</p>

<p align="center">
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml/badge.svg?branch=main" alt="継続的インテグレーション"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="Apache License 2.0"></a>
</p>

<p align="center">
  <a href="#xdzip-download-and-install">ダウンロード</a> / <a href="#xdzip-using-xdzip">使い方</a> / <a href="#xdzip-format-support">対応形式</a> / <a href="#xdzip-build-from-source">ソースからビルド</a> / <a href="docs/ja-JP/README.md">ドキュメント</a> / <a href="https://github.com/xiaodaozhi/xdzip/issues">フィードバック</a>
</p>

![xdzip でアーカイブを閲覧](docs/images/main-window.png)

<a name="xdzip-download-and-install"></a>

## ダウンロードとインストール

Windows 11 x64 では Microsoft Store で **XDZip** を検索してください。直接ダウンロードする場合やスタンドアロン CLI を使用する場合は、[GitHub の最新安定版](https://github.com/xiaodaozhi/xdzip/releases/latest)から利用するシステム向けのファイルを選択してください。

| システム | 対応範囲 | GitHub での配布形式 |
| --- | --- | --- |
| Windows | サポート期間内の Windows 11、x86_64 | ユーザー単位の `setup.exe`、またはポータブル版 `.zip` |
| macOS | macOS 14 以降、Apple Silicon | `.dmg`、またはアプリバンドル `.app.zip` |
| Linux | Ubuntu 24.04 LTS、x86_64、X11、glibc 2.39 以降 | `.deb` パッケージ、またはランタイムディレクトリの `.tar.gz` |

デスクトップ用の配布ファイル名は `xdzip-<version>-`、スタンドアロン [CLI](#xdzip-command-line) 用は `xdzip-cli-<version>-` で始まります。直接ダウンロードしたファイルは、リリースページの `SHA256SUMS` で確認できます。GitHub の Windows インストーラーは未署名です。macOS の配布ファイルは完全性確認用の ad hoc 署名を使用しており、公証されていません。パッケージごとの起動方法とファイルの関連付けは、[インストールガイド](docs/ja-JP/user-guide/install.md)を参照してください。

Microsoft Store 版は GitHub とは別に公開され、更新は Microsoft Store が管理します。GitHub 版では **xdzip について** から更新を確認できます。条件を満たす Windows のユーザー単位インストール、macOS の「アプリケーション」に配置したアプリ、Ubuntu の DEB インストールは、アプリ内から更新をインストールできます。ポータブル版は手動で更新してください。[設定と更新](docs/ja-JP/user-guide/settings-and-updates.md)と[サポート方針](docs/ja-JP/engineering/support-policy.md)を参照してください。

**バージョンについて。** このページでは現在のソースツリーの機能を説明します。配布パッケージの内容は対応するリリースノートで確認してください。ソースからのビルドは実験的な開発版です。

<a name="xdzip-using-xdzip"></a>

## xdzip の使い方

アーカイブを開くと、フォルダーの閲覧、項目の検索と並べ替え、展開する項目の選択ができます。どちらの並び順でもフォルダーが先に表示されます。リスト表示とグリッド表示、ファイルのプロパティ、設定からの言語選択、ライトとダークのテーマ、文字サイズと表示密度の調整を利用できます。既定では対応するシステムの言語を使い、それ以外では英語（米国）を使います。通常の単一ファイル ZIP/ZIP64 は、閲覧用に最大 100 万件のレコードを索引付けできます。他の形式には、これより低い上限が適用される場合がほとんどです。展開、アーカイブ全体の検証、変更にはそれぞれ別の上限があります。[ユーザーマニュアル](docs/ja-JP/user-guide/README.md)と[大規模アーカイブの索引](docs/ja-JP/engineering/large-index.md)を参照してください。

- **内容のプレビュー。** プロパティカードが表示されている状態でファイルを 1 つ選択すると、対応するプレーンテキスト、ラスター画像、静的 SVG が自動でプレビューされます。macOS では HEIC/HEIF にも対応します。内容をすべて読み取り、形式が提供する検証が終わるとプレビューが表示されます。処理はキャンセルできます。形式、サイズ、デコードの上限については[プレビュー](docs/ja-JP/engineering/content-preview.md)を参照してください。
- **展開とファイルを開く操作。** 展開先と名前の競合時の処理を選び、進行状況を確認し、必要に応じてキャンセルできます。通常ファイルをダブルクリックすると、専用の一時ディレクトリにコピーを展開してからシステムのアプリで開きます。コピーに加えた変更はアーカイブに書き戻されません。キャッシュされたコピーは、7 日を過ぎてから次にファイルを開く際に片付けられます。
- **作成と変更。** ZIP、TAR、TAR.GZ、GZIP、XZ、TAR.XZ、LZ4、TAR.LZ4、Zstandard、TAR.ZST、LZH、7z のアーカイブと ISO9660/Joliet データイメージを作成できます。Windows x64 では ZIP 自己展開ファイルも作成できます。条件を満たす通常の単一ファイルアーカイブでは、項目の追加、置き換え、移動、名前の変更、削除ができます。リストからファイルをドラッグして取り出したり、リストにドラッグして追加を確認したりできます。対応形式と上限は下の表を参照してください。
- **検証。** ファイルを書き込まず、アーカイブ全体をバックグラウンドで検証できます。検証項目は形式によって異なります。TAR、ISO、UDF、Compound File (MSI) などでは構造と読み取り可能性を検証します。ファイル単位の内容チェックサムはありません。[アーカイブの整合性検証](docs/ja-JP/engineering/archive-checking.md)を参照してください。
- **複数のアーカイブ。** 既定では新しいタブで開きます。新しいウィンドウで開くように設定できます。同じログインセッションではアプリのプロセスとトレイを共有します。閲覧画面を閉じても、タスク一覧からバックグラウンドのタスクを参照できます。
- **ファイルマネージャーの操作。** インストール版の Windows 11 では **その他のオプションを表示** からクラシックメニューを開けます。Store 版は新しいメニューにも表示されます。Ubuntu では拡張機能をインストールした「ファイル」(Nautilus) を使用します。macOS ではシステム設定で Finder 拡張機能を有効にするか、登録済みの Finder サービスを使用します。[ファイルマネージャーから展開](docs/ja-JP/user-guide/extract.md#xdzip-from-the-file-manager)を参照してください。

アーカイブの作成やファイルの追加時には、`.DS_Store` ファイルおよび `__MACOSX` と `_MACOSX` ディレクトリを除外します。展開時はパスと展開先の権限を確認し、名前の競合を処理し、出力を一時領域に書き込みます。対応する形式では内容も検証します。特殊オブジェクトやリンクの扱いは形式とプラットフォームによって異なります。[安全な展開の仕様](docs/ja-JP/engineering/zip-safe-extraction.md)を参照してください。

<a name="xdzip-format-support"></a>

## 対応形式

アーカイブを開く、作成する、変更する操作への対応状況は以下の表を参照してください。「変更」は既存アーカイブ内の項目の追加、置き換え、移動、名前の変更、削除を指します。読み取りや検証の可否は実際の内容、圧縮方式、レコードの種類によって決まります。拡張子からファイルを開くと形式の検出を開始します。詳しい上限については[組み込み形式の一覧](docs/ja-JP/engineering/format-registry.md)と[整合性検証ガイド](docs/ja-JP/user-guide/checks-and-tasks.md)を参照してください。

| 形式 | 閲覧、展開、検証 | 作成 | 変更 |
| --- | --- | --- | --- |
| ZIP および ZIPX/XPI/APK/IPA/CBZ の候補 | Store、Deflate、Deflate64、BZip2、LZMA、XZ。対応する AES-256 と分割プロファイル | Store、Deflate、AES-256、連番ボリュームを使用した ZIP | 表の方式を使う条件適合の ZIP/ZIPX |
| TAR、TAR.GZ（`.tgz` を含む） | 通常ファイルとディレクトリ | 対応。TAR は圧縮せずに格納 | 条件を満たす通常の単一ファイルアーカイブ |
| GZIP（`.gz`） | 連結された GZIP メンバーを含む 1 個の論理ファイル。GZIP 内の TAR は TAR ツリーとして表示 | 通常ファイル 1 個、Deflate 0/1/6/9 | 非対応 |
| BZip2、LZMA Alone、Lzip v1 | ストリームとメモリの上限内で通常ファイル 1 個 | 非対応 | 非対応 |
| TAR.BZ2、TAR.LZMA、TAR.LZ | 通常ファイルとディレクトリ | 非対応 | 条件を満たすアーカイブ |
| XZ、TAR.XZ（`.txz` を含む） | 宣言された検証、LZMA2、Delta/BCJ、連結ストリーム | LZMA2 レベル 1/6/9、XZ は通常ファイル 1 個 | TAR.XZ |
| LZ4、TAR.LZ4 | 標準フレーム、独立または連結ブロック、フレームの連結、宣言された検証 | LZ4 レベル 1/6/9、通常の LZ4 は通常ファイル 1 個 | TAR.LZ4 |
| Zstandard、TAR.ZST（`.tzst` を含む） | 標準フレーム、対応するウィンドウ、Huffman/FSE、反復オフセット、宣言された検証 | Zstandard レベル 1/6/9、通常の Zstandard は通常ファイル 1 個 | TAR.ZST |
| 7z | パスワード、暗号化されたファイル名、ソリッドアーカイブ、連番ボリューム | 暗号化、ソリッドアーカイブ、連番ボリューム | 条件を満たす非暗号化、非分割アーカイブ |
| DEB | 一般的な ar メンバー、制御データとデータの TAR ビュー、対応する GZIP/XZ/Zstandard/BZip2/LZMA プロファイル | 非対応 | 非対応 |
| LHA/LZH | ヘッダーレベル 0–3、Store、LH1–LH7、LZS/LZ5、宣言された CRC | LH5 レベル 1/6/9、レベル 2 ヘッダー、通常ファイルとディレクトリ | 項目の CRC を備えた条件適合のレベル 2 アーカイブ |
| [PMA](docs/ja-JP/engineering/pma-streams.md) | レベル 0 の PM0/PM1/PM2、厳密な CP/M フレーミング、宣言された CRC | 非対応 | 非対応 |
| [ARJ](docs/ja-JP/engineering/arj-streams.md) | Store と方式 1–4、ヘッダーの検証、メンバー全体の CRC-32 | 非対応 | 非対応 |
| [ARC](docs/ja-JP/engineering/arc-streams.md) | 旧式と新式の Store、方式 3–9、厳密なフレーミング、メンバー全体の CRC-16 | 非対応 | 非対応 |
| [BH](docs/ja-JP/engineering/bh-containers.md) | `BH 05 07` の通常メンバー、方式 8 の raw Deflate、厳密なサイズ、CRC-32 | 非対応 | 非対応 |
| [ALZ](docs/ja-JP/engineering/ace-alz-egg-format-sources.md) | Store または raw Deflate を使う通常の単一ボリュームメンバー、厳密なサイズ、CRC-32 | 非対応 | 非対応 |
| [EGG](docs/ja-JP/engineering/ace-alz-egg-format-sources.md) | Store または raw Deflate を使う非暗号化、非ソリッドの単一ボリュームメンバー、厳密なサイズ、CRC-32 | 非対応 | 非対応 |
| [PEA](docs/ja-JP/engineering/disk-image-archive-format-sources.md) | PEA 1.x の単一ボリューム、UTF-8 の名前、暗号化されていない PCOMPRESS0 のファイルとディレクトリ。オブジェクト、ストリーム、ボリュームの検証は対象外 | 非対応 | 非対応 |
| [Compound File (MSI)](docs/ja-JP/engineering/wim-msi-installer-format-sources.md#xdzip-compound-file-and-msi) | ストレージの閲覧、raw ストリームの展開。構造と読み取り可能性の検証 | 非対応 | 非対応 |
| [NSIS（`.exe`）](docs/ja-JP/engineering/wim-msi-installer-format-sources.md#xdzip-nsis-and-other-sfx) | 静的な `firstheader` メタデータのみ。展開とアーカイブ全体の検証は利用不可 | 非対応 | 非対応 |
| ASAR | 格納されたファイルとディレクトリ、共有ペイロード範囲、宣言されたファイルとブロックの SHA-256 | 非対応 | 非対応 |
| Z、TAR.Z（`.taz` を含む） | Unix compress の LZW、9–16 ビット辞書、対応する CLEAR とアラインメントのプロファイル | 非対応 | TAR.Z |
| Brotli、TAR.BR | 通常の RFC 7932 ストリーム、10–24 ビットのウィンドウ、すべての静的辞書変換 | 非対応 | TAR.BR |
| AES Crypt（`.aes`） | バージョン 0–3、開くときのパスワード、暗号文全体の HMAC、バージョン固有の長さの規則 | 非対応 | 非対応 |
| UU/UUE、XXE | 従来の文字数付き行と対応する begin-base64 エンベロープ。複数メンバーも扱う | 非対応 | 非対応 |
| RAR | 従来の形式、RAR5、RAR 7 の形式拡張。パスワード、ソリッド、分割アーカイブも扱う | 非対応 | 非対応 |
| CAB | Store、MSZip、Quantum、LZX、同じディレクトリ内のボリューム | 非対応 | 非対応 |
| ISO | ISO9660、Joliet、Rock Ridge | ISO9660 + Joliet データイメージ | 非対応 |
| UDF | UDF | 非対応 | 非対応 |
| ZIP SFX | 検証済みの ZIP 自己展開アーカイブ | Windows x64、Store/Deflate、`.sfx.exe` | 非対応 |

- ZIP の AES-256 は内容だけを暗号化します。ファイル名は表示されます。AES-128/192 と従来の ZipCrypto には対応していません。`.zip.001`、PKZIP の `.z01` など関連する命名規則の分割アーカイブを読み取れます。作成時は連番ボリュームを使用します。[パスワード付きアーカイブの展開](docs/ja-JP/engineering/password-extraction.md)と[分割アーカイブを開く](docs/ja-JP/engineering/archive-opening.md)を参照してください。
- 単独の BZip2、LZMA Alone、Lzip v1 は読み取り専用です。LZMA には内容のチェックサムがないため、構造、サイズ、読み取り可否を検証します。拡張子、項目、メモリー上限、受け入れ条件については[単一ファイルストリームの仕様](docs/ja-JP/engineering/single-file-streams.md)を参照してください。
- XZ は各ストリームが宣言する内容の検証を行います。None を宣言したストリームの検証は構造と読み取り可否が対象です。ZIPX は表に記載した ZIP 方式に対応し、PPMd、JPEG、WavPack の派生形式には対応しません。ZIP/ZIPX の編集では保持する項目の圧縮データと方式を引き継ぎ、新しい項目は Deflate を使います。
- 7z の変更ではアーカイブを再構築します。保持する内容と追加する内容には 512 MiB の作業メモリ上限があります。新規作成では入力をストリームで読み取るため、この再構築上限は適用されません。暗号化された ZIP や 7z、および分割された ZIP や 7z は変更できません。[アーカイブの変更](docs/ja-JP/engineering/archive-transfers.md)を参照してください。
- RAR、CAB、UDF は読み取り専用です。[ISO の作成](docs/ja-JP/engineering/iso-creation.md)は、通常ファイルとディレクトリ、単一ボリューム、4 GiB 未満のファイルに対応し、Joliet 名は各構成要素が UCS-2 で最大 64 文字です。起動可能イメージ、UDF や Rock Ridge の書き込み、イメージの更新には対応していません。イメージを読み取るときにマウントや実行はしません。[RAR の仕様](docs/ja-JP/engineering/rar-format-sources.md)と[形式バックエンドの仕様](docs/ja-JP/engineering/native-codecs.md)を参照してください。
- [ZIP 自己展開ファイル（SFX）の作成](docs/ja-JP/engineering/sfx-creation.md)では、Qt や xdzip のインストールを必要としない Windows x64 向けの展開プログラムを生成します。展開先の選択、競合の処理、キャンセルが可能です。展開されたファイルを実行しません。パスワード、分割ボリューム、自動起動コマンドには対応していません。一般の `.exe` はファイルの関連付けの対象外です。
- DEB は `members/`、`control/`、`data/` をエクスポート用のビューとして表示します。すべて展開すると raw メンバーと展開した内容の双方が対象になります。インストールやスクリプト実行は行いません。制御データは無圧縮、GZIP、XZ、Zstandard に対応し、データはさらに BZip2 と LZMA に対応します。検証ではコンテナの読み取りと宣言された項目を確認します。パッケージの署名確認は行いません。[DEB の仕様](docs/ja-JP/engineering/deb-containers.md)を参照してください。
- Z と TAR.Z は構造と読み取り可否を検証します。チェックサムはありません。単独の Z の作成と、壊れた旧形式ストリームの修復には対応しません。[Unix compress の仕様](docs/ja-JP/engineering/compress-streams.md)を参照してください。
- UU/UUE と XXE はチェックサム無しで構造と読み取り可能性を検証します。ヘッダー内のパスやモードは、出力や実行の権限を与えません。GNU のエンコード済みファイル名と複数部分の修復は、対応する[テキストエンベロープのプロファイル](docs/ja-JP/engineering/text-envelopes.md)の範囲外です。
- 読み取り専用の ASAR を開いたり展開したりしても、アプリは起動しません。外部の `unpacked` データと宣言されたリンクは表示されますが、処理の対象外です。隣接ディレクトリへのアクセスや実行属性の復元は行いません。任意のハッシュがある場合は破損を検出し、ハッシュがないレコードは構造と読み取り可能性だけを検証します。[ASAR の仕様](docs/ja-JP/engineering/asar-containers.md)を参照してください。
- NSIS は、PE オーバーレイに上限内の有効な `firstheader` レコードがある場合のみ検出します。xdzip はその静的メタデータ行を表示します。PE スタブ、インストーラースクリプト、プラグイン、ペイロードを実行しません。インストーラーヘッダーの解凍やペイロードの展開も行いません。`.exe` はドラッグ＆ドロップとシェルの関連付けには表示されません。一般の実行ファイルを扱う機能ではありません。
- LHA/LZH の作成には LH5、レベル 2 のヘッダー、UTF-8 の名前を使います。古い Windows ツールでは旧コードページで名前が表示される場合があります。条件を満たすレベル 2 のアーカイブは編集できます。旧形式の名前は区切り文字を解釈する前にデコードし、内容のデコード時に存在する CRC を検証します。CRC のない古い項目では構造と読み取り可否を検証します。パスワード、分割ボリューム、修復には対応していません。PMA は別の読み取り専用プロファイルです。[LHA の仕様](docs/ja-JP/engineering/lha-containers.md)を参照してください。
- ARJ は読み取り専用です。バイナリと従来のテキストのメンバーは、OS による改行変換を行わず、CRC の対象になっているアーカイブのバイト列をそのまま返します。パスワード、分割ボリューム、復旧またはセキュリティに関するプロファイル、リンク、特殊オブジェクトは対象外です。不明な方式は表示され、アーカイブ全体の検証は失敗します。
- ARC は読み取り専用です。通常ファイルは順に処理し、終端を厳密に検証します。パスワードによるデコード、分割ボリューム、SFX、Spark 拡張は対象外です。不明な方式は ZIP 方式に読み替えずに表示し、アーカイブ全体の検証は失敗します。旧式の XOR 暗号にはヘッダーフラグがありません。デコードには有効なストリーム、厳密なサイズ、CRC が必要です。パスワードや修復で代用することはありません。
- BH は `BH 05 07` の方式 8 の raw Deflate プロファイルを読み取れます。不明な方式、暗号化、ディレクトリ、ボリューム、末尾データ、SFX 接頭部は対象外です。

- ディスク上の索引を使用する大規模な ZIP アーカイブでは、現在、全体の検証や変更に対応していません。選択した通常ファイルはタスクの上限内で展開できます。[大規模アーカイブの索引](docs/ja-JP/engineering/large-index.md)を参照してください。
- スパースファイルを含む TAR と第三者製の拡張機能には現在対応していません。通常の GZIP と ZIP 由来の XPI/APK/IPA/CBZ の候補は[単一ファイルストリームの仕様](docs/ja-JP/engineering/single-file-streams.md)に従います。旧来のファイル名のエンコーディングは可能な範囲で検出します。短い名前や異なるエンコーディングが混在する場合は特定できないことがあります。[ファイル名のエンコーディング](docs/ja-JP/engineering/filename-encoding.md)を参照してください。

- AES Crypt ではメタデータの表示前に、開くためのパスワードが必要です。すべての暗号文の MAC を検証します。バージョン 0–2 では末尾の長さの剰余を認証しません。拡張情報によって権限が付与されることはありません。[AES Crypt の仕様](docs/ja-JP/engineering/aescrypt-envelopes.md)を参照してください。
- Brotli と TAR.BR は形式を候補として仮検出した場合に内容の完全な検証が必要で、チェックサムはありません。TAR.BR の編集では標準の非圧縮メタブロックで書き込むため、アーカイブが大きくなることがあります。共有辞書と大きなウィンドウの拡張は、対応する[Brotli プロファイル](docs/ja-JP/engineering/brotli-streams.md)の対象外です。

<a name="xdzip-command-line"></a>

## コマンドライン

スタンドアロン CLI は Qt やグラフィカルセッションを必要とせず、アーカイブの一覧表示、展開、作成、追加、検証と JSON 出力に対応します。`xdzip-cli-<version>-` の配布ファイルを展開し、macOS または Linux では `./bin/xdzip --help`、Windows PowerShell では `.\bin\xdzip.exe --help` を実行します。同梱のワーカープログラムは実行ファイルの隣に置いてください。最初のコマンドと PATH に追加する方法は [CLI クイックスタート](docs/ja-JP/integration/README.md)を参照してください。ソースからのビルドも可能です。

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/cli.py build
python3 scripts/cli.py test
build/cli/bin/xdzip --help
build/cli/bin/xdzip list archive.zip --json
build/cli/bin/xdzip extract archive.zip -C output
```

Windows では `python` を使い、実行ファイル名に `.exe` を付けてください。Linux で実行するには、システムに OpenSSL 3 と証明書が必要です。ビルドには `libssl-dev` と `pkg-config` も必要です。パスワード、非対話モード、終了コード、コマンドのインストール、更新の制限については [CLI ガイド](docs/ja-JP/engineering/command-line.md)を参照してください。

<a name="xdzip-build-from-source"></a>

## ソースからビルド

デスクトップのインターフェイスは Qt Quick/QML を使用し、薄い C++ 層を介して Rust コアに接続します。形式、クエリ、タスクの処理は Qt から独立しています。7z、RAR、CAB、ISO、UDF の読み取りと 7z の書き込みには、隔離されたワーカープロセスを使います。ISO の作成は Rust ホストで行います。Windows のスタンドアロン SFX は Qt を使わずにシステムダイアログを表示します。

ドキュメントは `docs/`、ジェネレーターや検証が使用する視覚パラメーター、用語、依存関係一覧は `specs/`、詳細なデザインは `design/hifi/` にあります。

Python 3.10 以降、Git、CMake 3.24 以降、Ninja、rustup、お使いのプラットフォーム用のコンパイラと SDK をインストールし、ソースを取得してビルドします。すでに CLI のためにクローンした場合は `prepare` から始めてください。

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

Windows では MSVC x64 開発環境、macOS では Apple Silicon ネイティブのターミナルと Xcode Command Line Tools を使用します。既定の Qt 6.8.2 ツールチェーンは `build/toolchain/` にあります。Qt がインストール済みの場合は `--qt-prefix` を指定できます。依存関係、起動、パッケージ化、検証は[開発者向けビルドガイド](docs/ja-JP/engineering/developer-build.md)を参照してください。CLI とデスクトップのビルド手順は独立しています。デスクトップビルドには専用 CLI も同梱されます。コマンドのエントリーポイントは明示的に有効化してください。

開発用パッケージは実験版です。Windows と Linux の開発成果物は、[Qt コンポーネントのワークフロー](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml)の成功した手動実行からもダウンロードできます。通常の CI では開発用パッケージは自動生成されません。

<a name="xdzip-documentation-and-contributing"></a>

## ドキュメントと貢献

| トピック | 最初に読むページ |
| --- | --- |
| デスクトップの操作とスクリプト | [ユーザーマニュアル](docs/ja-JP/user-guide/README.md)、[コマンドラインの統合](docs/ja-JP/integration/README.md) |
| 設計と実装 | [技術リファレンス](docs/ja-JP/reference/README.md) |
| プライバシーとデータの取り扱い | [プライバシーポリシー](PRIVACY.md)、[中国語版](PRIVACY.zh-CN.md) |
| ビルドと貢献 | [貢献の手引き](docs/ja-JP/contributing/README.md) |
| 変更と今後の作業 | [変更履歴](CHANGELOG.md)、[GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues) |
| 貢献 | [貢献ガイド](CONTRIBUTING.md)、[行動規範](CODE_OF_CONDUCT.md) |
| プロジェクトの方針 | [運営](GOVERNANCE.md)、[セキュリティ](SECURITY.md) |

フィードバックと貢献を歓迎します。不具合の報告には、バージョンまたはコミット、システムとアーキテクチャ、再現手順、実際の結果を含めてください。サンプルは自分で作成し、機密情報を含めないでください。セキュリティ問題は [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new) から非公開で報告してください。

<a name="xdzip-license"></a>

## ライセンス

プロジェクトが独自に作成した内容は [Apache License 2.0](LICENSE) で提供されます。[COPYRIGHT](COPYRIGHT) も参照してください。Qt、コーデック、フォント、その他の依存関係には個別のライセンスが適用されます。出典と通知は[ライセンス文書](docs/ja-JP/engineering/licensing.md)を参照してください。公式リリースでは、対応するソースコードのアーカイブを別途配布しません。詳しくは[ソースの提供](docs/ja-JP/engineering/source-delivery.md)を参照してください。適用されるソース提供、再リンク、通知の義務は引き続き適用されます。リリース候補は個別に確認します。過去の記録は [#4](https://github.com/xiaodaozhi/xdzip/issues/4) と [#53](https://github.com/xiaodaozhi/xdzip/issues/53) を参照してください。
