# 対応するソースと再ビルド資料

[English](../../engineering/source-delivery.md) | [简体中文](../../zh-CN/engineering/source-delivery.md) | [繁體中文](../../zh-TW/engineering/source-delivery.md) | 日本語 | [Español](../../es-ES/engineering/source-delivery.md)

**リリースの境界、2026-11-05：** 所有者の方針により、この収集ツールはローカルレビューと再ビルド検証専用です。対応するソースのアーカイブは GitHub Release や Actions アーティファクトとして提供しなくなりました。過去の公開 Release アセットと、これらのアーカイブを含むダウンロード可能だったリリース Actions アーティファクトは取り下げられ、今後のリリースワークフローはその生成またはアップロードを拒否します。この判断はコンポーネントのライセンスを変更せず、対応するソース、再リンクなどの義務が満たされたことを示すものではありません。公開リポジトリのタグについて GitHub が自動生成するソーススナップショットは、これらの Release 添付物とは別です。

通常のリリースや受け入れでは、この収集ツールを実行したり、対応するソースのアーカイブをローカル生成したりしてはいけません。ユーザーがソース収集を明示的に要求した場合にのみ実行し、その場合もアップロード制限は適用されます。ライセンス審査では実際のコンポーネントと義務を特定し、機械生成の一般的な要確認フラグを確定した欠陥として扱わないでください。

scripts/prepare-source-delivery.py はクリーンなコミットのプロジェクトソース、Cargo.lock に対応する完全な Rust vendor 入力、固定された Qt 6.8.2 モジュールのソースアーカイブ、再ビルドと置換の手順を収集します。すべての出力は build/ に置きます。ダウンロードアドレスは配布済みソースではなく、収集ツールはオープンソースライセンスの選択肢を自動承認しません。

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt ランタイムモジュールのアーカイブは qtbase、qtdeclarative、qtsvg、qttranslations です。ビルドに必要な qtshadertools、qttools と固定された Rust 標準ライブラリのソースコンポーネントも、[公式 Qt 6.8.2 アーカイブ](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)から提供されます。公式 .sha256 値は scripts/corresponding-sources.json に固定しています。キャッシュとダウンロードは再検証し、ダウンロードは最初に部分ファイルへ書き、検証後に確定します。アーカイブのディレクトリだけを検査し、第三者のパスを自動展開しません。Cargo vendor は --locked を使用し、レジストリのチェックサムとビルド依存関係を保持し、第三者のビルドスクリプトを実行しません。

ソース配布には固定された編集可能な Public Suffix List とその MPL-2.0 ライセンス文も含まれます。収集時に入力、Qt アーカイブとジェネレーターの ID を検証し、C++ データとバイナリテストデータの両方を再生成してバイト単位で比較します。不一致では停止し、SDK 全体の等価性やリリースライセンスの承認を示しません。

ソース配布は、Qt Core の MIME データに使う固定された Apache Tika 入力、上流 NOTICE と OGL-UK-3.0 の文書を保持します。Qt アーカイブの帰属表示を検証し、固定アーカイブの変換スクリプトで MIME データを再生成して、ネイティブ Windows の改行のみ差異として許可します。完全な Tika NOTICE は広い上流配布物を対象とし、そのほかの依存関係が xdzip に同梱されていることを意味しません。

配布物には Qt 6.8.2 のロケール生成に使われた CLDR 46 の `common` ソースと上流ライセンスも含まれますが、CLDR ソースツリーの残りは含まれません。収集時に生成済み Qt ソース 2 ファイルの CLDR バージョン、固定上流アーカイブのハッシュと選択入力を検証します。Qt の第 2 生成段階には含まれていない ISO 639-3 表が別途必要です。`qt-cldr-sources.json` は検証した候補のソース、バージョン、ダイジェストを記録し、その[上流ダウンロードページと利用条件](https://iso639-3.sil.org/code_tables/download_tables)は別途審査が必要です。この候補で生成データを再現しても、正確な過去の入力や元の SDK の完全なビルド ID を証明しません。

元の ISO 表を省いても、他の CLDR データから関連識別子がすべて消えるわけではありません。CLDR 46 の `common/validity/language.xml` は圧縮された言語コードを保持し、`common/supplemental/supplementalMetadata.xml` にはエイリアスがあります。[ローカル比較](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588994510)では、配布したこれらのファイルから構成した 3 文字の候補集合は、CLDR 46 の候補 ISO 表の `Id` 値 7,920 件中 7,919 件を含み、338 件が余分でした。同じ方法を CLDR 43.1 サブセットとその後の表に適用すると、7,896 件を含み、338 件が余分でした。候補集合は元の表ではなく全フィールドも含まないため、識別子の一致だけでは出所、許可や適用条件は確定しません。ISO の名を持つ 5 ファイルを除外しても、ほかの関連データが存在しないことは証明できません。

CLDR 43.1 サブセットには、ヘッダーの日付が 2023-02-14 の `language-subtag-registry` ツール入力も残っています。8,049 件の 3 文字言語サブタグのうち 7,713 件は上記の後年の候補 ISO 表の識別子と一致しますが、このバージョン間差の原因はここでは特定しません。Unicode の更新手順はこのファイルを IANA から取得した言語サブタグレジストリとしています。IANA の CC0 声明は IANA が保有または再許諾できる権利のみを対象とし、第三者の権利が存在する可能性を残します。比較と限界は [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)を参照してください。

CLDR 43.1 サブセットには、その Web ツールで使用する Noto フォント 2 点、Glyphicons ファイル 4 点、Bootstrap と Autosize のスクリプト、および絵文字画像も含まれます。`cldr43-webapp-notices/` は、各固定ファイルを検証したうえで、これらのフォントとスクリプト、および固定された 2 つの Noto Emoji バージョンと一致する 908 枚の画像の出所とライセンスを示します。出所と再配布条件を独立して確定できなかった残りの元画像 963 枚については、収集ツールが各元ハッシュを検証し、固定 Noto 画像 698 枚と固定 Twemoji 画像 265 枚に置き換えます。マニフェストは両方のハッシュ、パスと出所を記録します。これは侵害を主張するものではありません。これらは xdzip のインターフェースには含まれません。通知は IBM の著作権表示を持つ Web ツールのソース 3 ファイルも識別し、そのリリースのルート README に基づいて Unicode-DFS-2016 として扱い、著作権表示とライセンスを保持します。さらに Web ルートの画像 27 枚は固定された 2026 年の CLDR コミットとバイト単位で一致し、そのコミットの Unicode-3.0 ライセンスの下で完全な通知と個別ハッシュを保持します。この評価は商標、未記載アセットや外部依存関係には及びません。

CLDR 43.1 の `docs/rfc` サブセットにある完全なテキスト 2 ファイル、`draft-davis-t-langtag-ext.txt` と `draft-davis-u-langtag-ext-00.txt` は、IETF の[2011 年 T 拡張ドラフト](https://www.ietf.org/archive/id/draft-davis-t-langtag-ext-07.txt)と[2010 年 U 拡張ドラフト](https://www.ietf.org/archive/id/draft-davis-u-langtag-ext-00.txt)にそれぞれバイト単位で一致します。収集時にハッシュを検証し、著作権と通知を保持します。どちらも発行時に有効な条件を参照します。[TLP 4.0](https://trustee.ietf.org/wp-content/uploads/IETF-TLP-4.pdf) の 3.c 節は、Trust が権利を付与できる範囲で、必要な通知を保持し、全文を変更せずに標準化プロセス外で配布することを許可します。この評価はこの 2 つの文書のみを対象とし、他の HTML、XML、アセットや第三者の権利には及びません。

Qt ソースに既に含まれる言語については、元の `qlocale_data_p.h` から限定されたジェネレーター入力を抽出できます。たとえば QtBase を現在のディレクトリに展開後、`python3 scripts/derive-qt-locale-code-input.py --qtbase qtbase-everywhere-src-6.8.2 --output build/qt-locale-codes.tsv` を実行します。これは既存の検索キーだけを対象とし、完全な ISO 639-3 表でも過去の上流入力でもありません。言語追加には別途取得したコードとその条件の審査が必要な場合があります。固定された Qt 6.8.2 ソース、CLDR 46 データとこの派生入力を使った 8 ファイルの再生成は、生成日と改行の差だけを許して確認しました。ISO 表のライセンスや完全な対応ソースはこれによって確定しません。

特定の追加言語については、配布された CLDR 43.1 サブセットを展開し、`tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry` で言語サブタグを確認して 4 つのコード欄を明示指定します。Choctaw の例の ISO 639-2 欄は[米国議会図書館の一覧](https://www.loc.gov/standards/iso639-2/php/code_list.php)、ISO 639-3 は[登録機関のデータ](https://iso639-3.sil.org/code_tables/download_tables)で確認できます。最後の空欄は 2 文字コードがないことを示します。

```sh
python3 scripts/derive-qt-locale-code-input.py --qtbase path/to/qtbase \
  --iana path/to/cldr43/tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry \
  --add-language 'cho,cho,cho,' --output build/qt-locale-codes-with-cho.tsv
```

`--add-language` は繰り返し指定できます。欄は 3 文字コード、ISO 639-2/B、ISO 639-2/T、2 文字コードで、欠ける欄は空にし、Part 2 の 2 欄は両方指定するか両方空にする必要があります。ツールは少なくとも 1 つの 3 文字または 2 文字コードが登録済み IANA 言語サブタグであることを確認し、構文を検証し、Qt の検索キーとの競合を拒否します。各欄が同じ言語を指すことを推測または検証したり、ISO の対応付けを確立したりは**しません**。利用者は各対応と出所を審査する必要があります。Qt 言語の追加には `enumdata.py` の更新、CLDR ロケールデータの提供、Qt の再生成と再ビルドも必要で、この行だけでは言語は追加されません。これは限定された派生表であり、完全な ISO 表や過去の入力 ID、完全な対応ソースの証明ではありません。

分離テストでは、Qt ツールの enum と派生表に私的使用のテストコードを手動追加し、新しい enum とコード項目を生成しました。再ビルドした Linux QtCore は正引き・逆引きに成功し、分離した元の alpha.8 コピーはオフスクリーンでの閲覧、キャンセルと抽出を完了しました。実際の言語や CLDR ロケール形式データは追加しておらず、実入力や条件の審査を代替せず、対応ソースの全義務を確立するものではありません。手順と最初のプローブ失敗は [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6611169790)を参照してください。

別の選択肢として、`iso-codes` の ISO 639-2 と 639-3 JSON データを独立に取得し、Qt 入力に変換できます。たとえば Linux にデータパッケージがインストールされている場合は次のようにします。

```sh
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 /usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 /usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-codes.tsv
```

変換ツールは名前を除くコード対応を抽出しますが、完全な入力からは完全なコード集合が生成されます。無効なコード、重複、Qt 検索キーとの競合を拒否し、ISO 639-2 の予約済みローカル使用範囲を除外し、入力と出力の SHA-256 を報告します。Ubuntu `iso-codes` 4.16.0-1 と Qt 6.8.2 の比較は [#138](https://github.com/xiaodaozhi/xdzip/issues/138)を参照し、ほかのバージョンは別途確認してください。このデータは同梱していません。生成した表の明示ライセンス、出所、再配布条件は独立に審査してください。代替表は Qt の過去の入力や完全なソース準拠を証明しません。

比較を再現するには、固定された [Ubuntu `iso-codes` 4.16.0-1 パッケージ](https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb)を独立に取得します。次の Linux 例には `curl`、`dpkg-deb` と Python 3 が必要で、システムへのインストールを変更せずにローカル展開・変換する前にパッケージハッシュを検証します。

```sh
mkdir -p build && mkdir build/qt-iso-peer &&
curl --fail --location --silent --show-error --connect-timeout 15 --max-time 120 https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb -o build/qt-iso-peer/iso-codes.deb &&
python3 -c "import hashlib,pathlib,sys; p=pathlib.Path('build/qt-iso-peer/iso-codes.deb'); h=hashlib.sha256(p.read_bytes()).hexdigest(); sys.exit(0 if h=='cafca0a3a1f07b44fbba53ab7a1f1d9755e6b80cc041173afe2b52e53bed97a7' else 'Package SHA-256 mismatch')" &&
dpkg-deb -x build/qt-iso-peer/iso-codes.deb build/qt-iso-peer/data &&
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-peer/qt-locale-codes.tsv
```

検証済み入力の SHA-256 は `fa83810fdb59f9d84b4d58486d5e5e48e807d82a98d6a39ef0ba4fc57c2a9327` と `9636ce5266053867627140ce5ada1f9aa897ca07a7501302c1b14b8d1147cdda`、変換出力は `85457a36e013e053b70dea19d500ebec2ae6bedf5dad9af0035b973c8db6a60a` です。Ubuntu の `usr/share/doc/iso-codes/copyright` は LGPL-2.1-or-later を宣言していますが、そのパッケージ単位の声明だけでは、過去の SIL 表の許諾や完全なコード集合の再配布権を確立できません。パッケージも完全な変換表もここには含まれません。出所、条件とその後の再配布を別途審査してください。

あるいは [IANA 言語サブタグレジストリ](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)と[米国議会図書館の ISO 639-2 一覧](https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt)を独立に取得して別の入力を生成できます。ツールは名前なしでコードを抽出し、未一致コードと入出力ハッシュを報告し、無効または曖昧なレコードを拒否します。Linux/macOS シェルの例では新しいディレクトリを使用し、変換前にこの比較の元ファイル両方を検証します。

```sh
(
  set -eu
  mkdir -p build
  mkdir build/qt-iana-loc-inputs
  curl --fail --location --output build/qt-iana-loc-inputs/iana.txt \
    https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
  curl --fail --location --output build/qt-iana-loc-inputs/loc.txt \
    https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt
  python3 -c 'import hashlib,pathlib,sys; p=sys.argv[1:]; sys.exit(0 if all(hashlib.sha256(pathlib.Path(p[i]).read_bytes()).hexdigest()==p[i+1] for i in (0,2)) else "Input SHA-256 mismatch")' \
    build/qt-iana-loc-inputs/iana.txt 755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1 \
    build/qt-iana-loc-inputs/loc.txt 42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371
  python3 scripts/convert-iana-loc-for-qt.py \
    --iana build/qt-iana-loc-inputs/iana.txt \
    --loc build/qt-iana-loc-inputs/loc.txt \
    --output build/qt-iana-loc-inputs/qt-codes.tsv
)
```

比較では 2026-09-17 日付の IANA ファイル、SHA-256 `755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1` を使いました。2026-11-03 に直接取得した UTF-8 LoC ファイルは 16,010 バイトで、SHA-256 は `42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371` です。その 487 レコードの最初の 3 つのコード欄は、以前 Web から再構成したものに個別に一致しました。変換ツールの LF 出力 SHA-256 は `6a75bebfa694258a1a4125fc30512d8ae7994080701377b41306db665ad371c6` でした。これらの入力から既存の Qt 検索キー 346 種すべてに一致する 8268 行が生成されました。生成日と改行を正規化すると再生成した 8 ファイルすべてが一致し、入力の 1 つを変更した負の対照では 7 ファイルが一致しました。IANA の 2 文字サブタグ `bh`、`in`、`iw`、`ji`、`jw`、`mo`、`sh` の 7 件には LoC レコードがなく、ツールが報告します。ほかのバージョンや追加言語は別途検証してください。上流アドレスは変更される可能性があります。ハッシュ不一致を回避せず調査してください。[#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588040592)を参照してください。

[IANA/IETF の声明](https://www.iana.org/help/licensing-terms)は、両者が保有または再許諾できるプロトコルレジストリの権利に CC0 を適用し、第三者の権利が存在する可能性を残します。これによって元の SIL 表の全権利が自動的に確立されるわけではありません。2026-09-17 の IANA スナップショット、LoC 入力、完全な変換比較表はいずれもここに含まれません。CLDR 43.1 サブセットには前述のより古いレジストリが別に含まれます。利用者は出所、適用条件とその後の再配布条件を別途審査する必要があります。この代替手段は Qt の過去の入力も完全な対応ソースの準拠も確立しません。

ここでいう独立比較は、ファイルを別々に取得して変換することを意味し、標準の出所が独立していることを意味しません。[RFC 5646 の 2.2.1 節](https://www.rfc-editor.org/rfc/rfc5646#section-2.2.1)は、ISO 639-2、639-3 と 639-5 を IANA の 3 文字言語サブタグの標準情報源として挙げています。値の一致は独立した著作や完全な再許諾を確立しません。

出力には実際の固定コミットの xdzip-source.tar、rust-vendor.tar.gz、オフライン Cargo 設定、元の Qt アーカイブ、REBUILD.md、ファイルごとの SHA-256・サイズマニフェストが含まれます。マニフェストは収集済みであり準拠未検証と明記され、SDK のビルド機能・パッチ、プラットフォームランタイム、組み込みリソースやライセンス準拠の確認を代替しません。再構成されたプロジェクトソースは ID が不明な場合があり、クリーンな Git ID を捏造せず、外部配布マニフェストを通して元のコミットに関連付ける必要があります。

別の `rar-wire-mappings/` ディレクトリは、制限された有限マッピングの参照出所、説明と正確な元の UnRAR 7.30 beta1 通知を保持し、すべてハッシュ付き配布物に含まれます。固定されたダウンロードファイル名は `unrarsrc-7.3.1.tar.gz` です。その数値ファイル名は別の上流リリースバージョンを確立しません。保守的な通知措置として、配布では関連 Rust ソースコメントに元ライセンスの第 2 段落全体を要求し、欠落または変更時は停止します。RAR クレートは保守的に `Apache-2.0 AND UnRAR` を宣言します。OSI 未承認の `UnRAR` 識別子は SPDX 3.28 以降に存在し、古いツールでは認識されないことがあります。これらの資料には公式エンジンのソースを含まず、参照から得た事実を Apache-2.0 として再ライセンスするものではありません。適用条件と実際の再配布義務には候補ごとの審査が必要です。

Linux ICU のために、固定された完全な ICU 73.2 ライセンス、公式 ICU4C 73.2 リリースソースアーカイブ、固定された編集可能な ICU ソース、CLDR 43.1 ソースサブセット、ICU 73.1 ブートストラップソースを含め、全体マニフェストに入れます。CLDR サブセットは ISO 639-3 の 5 ファイル、UTS #35 の本文と `docs/ldml` 以下の関連ファイル、ドキュメントのスクリーンショット 2 点を除外し、Web ツールの画像 963 枚を置き換えます。配布記録は上流アーカイブのハッシュと各除外・置換を保持します。[Unicode 公開の UTS #35 バージョン 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)は所有者の条件の下で別途利用できます。追加の譲渡可能な許可があるかは未確定です。収集ツールは名前に `iso-639-3` を含むブートストラップアーカイブのファイルも拒否しますが、ファイル名チェックでは別形式のデータの存在を排除できません。収集時には第三者のパスを展開せずにアーカイブハッシュ、バージョン、組み込みライセンスと選択されたソースファイルを確認します。[ICU データ再ビルドガイド](icu-cldr-rebuild.md)にはツールビルド、生成と編集範囲を記録しています。上流バージョンの一致は、供給者がパッチなしでこれらの入力からバイナリをビルドしたことを証明しません。現在の Windows 開発パッケージはソフトウェア OpenGL を省き、専用 Mesa/LLVM 資料を収集しなくなりました。過去の配布物には元の資料と義務が残ります。[ネイティブランタイムのソースと通知](native-runtime-materials.md)を参照してください。

アプリケーションは Qt に動的リンクし、ソース再ビルドと互換ライブラリ置換の手順を提供します。[macOS デプロイ検証](macos-deployment.md)は、Qt ソース全体の再ビルドを主張せず、分離コピーで変更ライブラリの読み込みを対象とします。実際の Qt ソースビルドと製品への読み込みは[自動検証契約](../../../developers/engineering/candidate-verification.md)で別のプロセスとして扱います。SDK のバイト単位のビルド等価性、正確な静的リンク帰属とライセンス選択は、完了済み WP-12 の開発作業を再開するものではなく、[リリースライセンス審査](https://github.com/xiaodaozhi/xdzip/issues/4)に属します。

verify-source-delivery.py はまず配布ファイルのハッシュを再検証し、プロジェクトと vendor の自前入力を新しい build/ ディレクトリに安全に展開し、空の Cargo キャッシュ、CARGO_NET_OFFLINE=true と既存の固定 Qt SDK で実際のアプリをビルドし、起動・終了スモークテストを実行します。これはプロジェクトと Rust 入力のオフライン使用を検証するものであり、Qt モジュールのソースビルドやバイト再現可能なビルドではありません。生成されたソースディレクトリの Git ID は不明であり、固定コミットと配布マニフェストによって外部から関連付けます。

ソース配布は出力ディレクトリから相対的な `vendor` の出力先と明示的なプロジェクトマニフェストを指定して Cargo を呼び出します。現在サポートする crates.io ソースマッピングと相対的な `vendor` 設定だけを受け付けます。絶対ディレクトリ、パスの逸脱、予期しないマッピングは、移動可能な配布物にビルドマシンのパスを埋め込まず失敗します。

通常の名前の単独ライセンスまたは通知ファイルがない固定 vendor パッケージについて、`rust-vendor-notices/` がパッケージごとの固定上流原文を別に索引化します。収集ツールはパッケージの Cargo マニフェストハッシュ、上流 Git コミットとパス、ライセンス本文の SHA-256 と Git blob を検証し、未知の欠落や ID 不一致では停止します。このディレクトリは Cargo vendor ファイルやチェックサムを変更しません。上流のライセンス表現は変更されず、特定のライセンス選択や過去リリースの準拠を承認するものではありません。

[Qt SDK 対応性チェック](qt-sdk-correspondence.md)は固定された対応性マニフェストと再実行可能な AGL コンシューマーパッチを常に含みます。--qt-prefix を指定すると、その SDK のファイルごとのソース宣言チェック、ビルド構成と明示された不足も含めます。配布マニフェストはパッチのサブディレクトリを再帰的にハッシュし、再ビルド検証でそのバイトを再確認します。SDK がない場合に構成検証記録を捏造しません。

配布物は、リリースコミットとアーカイブの `.tag` ツリーオブジェクトの関係をオフライン検証するため、固定された Qt Declarative Git コミットオブジェクトを保持し、配布マニフェストがそのハッシュを記録します。この関係はアーカイブの完全な網羅性や SDK ビルド等価性を確立しません。

実際の Qt ソースビルドと読み込み・互換置換については[自動完了契約](../../../developers/engineering/candidate-verification.md)を参照してください。既存 SDK を使用したオフラインのアプリ再ビルドを、Qt 自体をソースから再ビルドしたと説明してはいけません。
