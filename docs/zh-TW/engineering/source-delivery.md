<a name="xdzip-对应源码与重建材料"></a>

# 對應原始碼與重建材料

[English](../../engineering/source-delivery.md) | [简体中文](../../zh-CN/engineering/source-delivery.md) | 繁體中文 | [日本語](../../ja-JP/engineering/source-delivery.md) | [Español](../../es-ES/engineering/source-delivery.md)

**發行邊界，2026-11-05：**按所有者決定，此收集器僅供本地審查和重建驗證，不再把對應原始碼壓縮檔作為 GitHub Release 或 Actions 工件提供。公開歷史 Release 和仍可下載的發行 Actions 工件中的對應原始碼包已撤下，後續釋出流程禁止生成或上傳。該決定不改變元件許可，不表示對應原始碼、重連結或其他許可義務已滿足；GitHub 公共倉庫自動生成的標籤原始碼快照不屬於這些 Release 附件。

常規發行及其驗收不得自行執行此收集器，也不得在本地生成對應原始碼包；只有使用者針對原始碼收集另行明確要求時才可執行，上傳限制仍然適用。許可複核針對實際元件和具體義務進行，不將機器報告中的通用待複核標記直接認定為實際缺陷。


scripts/prepare-source-delivery.py 從乾淨提交收集專案原始碼、Cargo.lock 對應的完整 Rust vendor 輸入、固定 Qt 6.8.2 模組原始碼歸檔和重建／替換說明。所有產物位於 build/，不把下載地址稱為已交付原始碼，不自動批准開源許可分支。

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt 執行時模組歸檔為 qtbase、qtdeclarative、qtsvg、qttranslations，另交付構建所需的 qtshadertools、qttools 及固定 Rust 標準庫原始碼元件，來自 [Qt 官方 6.8.2 歸檔](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)。官方 .sha256 值固定於 scripts/corresponding-sources.json；快取與下載均重新校驗，下載先寫 partial，再驗證後提交。只檢查歸檔目錄，不自動解包第三方路徑。Cargo vendor 使用 --locked，保留 registry 校驗與構建依賴，不執行第三方構建指令碼。

原始碼交付另含 Qt Network 內嵌 Public Suffix List 的固定可編輯原始列表和 MPL-2.0 原文。收集時先核對列表、Qt 歸檔和生成器身份，再分別生成 C++ 資料及二進位制測試資料，並與 Qt 歸檔中的檔案逐位元組比較；任一不符均停止交付。此對應關係不代表整個 SDK 構建等價或自動批准發行許可。

原始碼交付還儲存 Qt Core MIME 資料所用的固定 Apache Tika 原始檔案、上游 NOTICE 和 OGL-UK-3.0 原文。收集器用固定 Qt 歸檔內的轉換指令碼重建 Qt 的 MIME 資料，並核對歸檔宣告和生成結果；僅允許 Windows 本地換行差異。完整 Tika NOTICE 覆蓋整個上游發行，不表示其其他依賴包含於 xdzip。

原始碼交付另含 Qt 6.8.2 區域資料生成所用的 CLDR 46 `common` 原始碼和上游許可，不包含 CLDR 原始碼樹的其餘部分。收集時核對 Qt 原始碼歸檔中兩個生成檔案的 CLDR 版本，以及固定上游歸檔摘要和原始檔。Qt 的第二步生成器還需要 ISO 639-3 表，該表不在交付中；試驗用候選的來源、版本和摘要記錄在 `qt-cldr-sources.json`，其[上游下載頁面和條款](https://iso639-3.sil.org/code_tables/download_tables)須單獨核對。候選可重現生成資料不證明它是原上游構建的精確輸入，也不證明原始 SDK 的完整構建身份。

不提供原始 ISO 表不意味著其他 CLDR 資料沒有相關識別符號。CLDR 46 `common/validity/language.xml` 保留壓縮表示的語言程式碼，`common/supplemental/supplementalMetadata.xml` 還含別名。本機對交付子集的[對照](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588994510)從這兩份檔案構造的三字母候選集合覆蓋 CLDR 46 候選 ISO 表 7,920 個 `Id` 中的 7,919 個，並另含 338 個值；CLDR 43.1 子集按相同方法與較新的同一表跨版本對照覆蓋 7,896 個，並另含 338 個值。候選集合不是原表，不含原表的完整欄位；識別符號重合不決定來源、授權或適用條款。不得以排除五個 ISO 命名檔案推斷交付中不存在其他相關資料。

CLDR 43.1 子集的工具資料還保留檔案頭日期為 2023-02-14 的 `language-subtag-registry`。該檔案中 8,049 個三字母語言子標籤有 7,713 個與上述較新候選 ISO 表重合；不同版本的差額不據此歸因。Unicode 的更新說明將此檔案列為從 IANA 取得的語言子標籤登錄檔；IANA 的 CC0 宣告只覆蓋其擁有或有權轉授的權利，並保留第三方可能的權利。檔案內容對照及限制見 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)。

CLDR 43.1 子集還包含網頁工具使用的兩份 Noto 字型、四份 Glyphicons 檔案、Bootstrap 與 Autosize 兩份指令碼，以及表情圖片。原始碼交付的 `cldr43-webapp-notices/` 單獨提供上述字型、指令碼和 908 份與兩個固定 Noto Emoji 版本逐位元組一致的圖片的來源與許可，收集時逐份核對固定摘要。另外 963 張原圖的來源和分發許可尚未獨立確認，收集器核對每張原圖摘要後，以 698 張固定版本 Noto 圖片和 265 張固定版本 Twemoji 圖片替換；交付清單逐項記錄原圖與替代圖的摘要、路徑和來源。這不意味著原圖侵權。這些檔案不屬於 xdzip 介面，三份保留 IBM 版權標記的網頁工具原始碼也在該通知中逐項記錄，依據該版根目錄說明按 Unicode-DFS-2016 處理並保留原版權及許可。網頁根目錄另有 27 個圖片與固定 CLDR 2026 提交逐位元組一致，按該提交的 Unicode-3.0 許可保留完整許可和逐項摘要；此判斷不擴充套件到商標或未列出的素材及外部依賴。

CLDR 43.1 子集 `docs/rfc` 中兩份完整文字 `draft-davis-t-langtag-ext.txt` 和 `draft-davis-u-langtag-ext-00.txt` 分別與 IETF 儲存的 [2011 年 T 擴充套件草案](https://www.ietf.org/archive/id/draft-davis-t-langtag-ext-07.txt)及 [2010 年 U 擴充套件草案](https://www.ietf.org/archive/id/draft-davis-u-langtag-ext-00.txt)逐位元組一致，原始碼收集時核對其摘要並保留原版權及宣告。兩份文字指向發表時生效的 IETF 條款；[TLP 4.0](https://trustee.ietf.org/wp-content/uploads/IETF-TLP-4.pdf)第 3.c 節允許在其可授予權利的範圍內於標準流程外完整、未經修改地分發檔案，並須保留要求的宣告。該判斷僅覆蓋這兩份原文，不自動覆蓋其他 HTML、XML、素材或第三方權利。

對 Qt 原始碼中已有的語言，還可從原 `qlocale_data_p.h` 提取一個供 Qt 生成器讀取的有限輸入。例如，將 QtBase 歸檔解包到目前目錄後執行 `python3 scripts/derive-qt-locale-code-input.py --qtbase qtbase-everywhere-src-6.8.2 --output build/qt-locale-codes.tsv`。此輸入僅覆蓋原檔案中已有的查詢項，不是完整 ISO 639-3 表，也不是上游歷史輸入；新增語言時可能仍需另行取得程式碼並核對條款。用固定 Qt 6.8.2 原始碼、CLDR 46 資料及此派生輸入重生成八個原始碼檔案的驗證，僅允許生成日期和換行差異。該方法不自動解決 ISO 表的授權或完整對應原始碼判斷。

要在有限輸入中增加具體語言，可解出交付的 CLDR 43.1 子集，使用其中 `tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry` 校驗語言子標籤，並明確填寫四個程式碼欄位。以下 Choctaw 示例的 ISO 639-2 欄位可對照[美國國會圖書館列表](https://www.loc.gov/standards/iso639-2/php/code_list.php)，ISO 639-3 欄位可對照[登記機構的資料](https://iso639-3.sil.org/code_tables/download_tables)；末尾空欄位表示沒有雙字母程式碼：

```sh
python3 scripts/derive-qt-locale-code-input.py --qtbase path/to/qtbase \
  --iana path/to/cldr43/tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry \
  --add-language 'cho,cho,cho,' --output build/qt-locale-codes-with-cho.tsv
```

`--add-language` 可重複，四個欄位依次為三字母程式碼、ISO 639-2/B、ISO 639-2/T、雙字母程式碼；沒有的欄位留空，兩個 ISO 639-2 欄位須同時填寫或同時留空。工具校驗三字母或雙字母程式碼至少有一項存在於 IANA 語言子標籤、欄位格式及 Qt 查詢鍵不衝突，**不**從 IANA 自動推斷或證明各欄位屬於同一語言或 ISO 欄位的對應關係；填寫者須自行核對具體對映和來源。新增 Qt 語言還須按照 Qt 上游流程修改 `enumdata.py`、提供該語言的 CLDR 區域資料並重新生成、編譯，單獨增加這一行不會自動新增語言。派生表不包含完整 ISO 程式碼表，也不證明歷史輸入或完整對應原始碼義務。

在隔離測試中，向 Qt 工具列舉和該派生表手工增加一個私用測試程式碼，可生成新列舉與程式碼表；Linux QtCore 重建後的正反向程式碼查詢透過，原始 alpha.8 的隔離副本也完成離屏瀏覽、取消和解壓。該實驗沒有新增真實語言或 CLDR 區域格式資料，不能替代真實語言輸入及其條款核對，也不能證明全部對應原始碼義務。步驟和首次探針失敗的記錄見 [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6611169790)。

另一種方式是自行取得 `iso-codes` 的 ISO 639-2 與 639-3 JSON 資料，再用轉換器生成 Qt 可讀取的輸入。例如，在安裝了該資料包的 Linux 環境中執行：

```sh
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 /usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 /usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-codes.tsv
```

轉換器只提取語言程式碼及其對映，不保留名稱；若輸入是完整程式碼集，輸出仍包含完整程式碼集。它拒絕無效程式碼、重複程式碼和 Qt 查詢鍵衝突，並跳過 ISO 639-2 的本地使用保留範圍。工具輸出輸入及結果的 SHA-256。Qt 6.8.2 與 Ubuntu `iso-codes` 4.16.0-1 的重建對照見 [#138](https://github.com/xiaodaozhi/xdzip/issues/138)；其他版本須自行檢查生成結果。`iso-codes` 資料不隨本交付提供，其宣告許可、來源及生成表的分發條件需由獲取者核對；可用替代表不證明它是 Qt 歷史輸入，也不自動完成對應原始碼或許可判斷。

為復現該對照，可從 [Ubuntu 官方歸檔的 iso-codes 4.16.0-1 包](https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb)自行取得資料。以下示例需要 Linux 的 `curl`、`dpkg-deb` 和 Python 3；它先校驗包摘要，再在本地解包和轉換，不修改系統安裝：

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

已核驗兩份輸入的 SHA-256 分別為 `fa83810fdb59f9d84b4d58486d5e5e48e807d82a98d6a39ef0ba4fc57c2a9327` 和 `9636ce5266053867627140ce5ada1f9aa897ca07a7501302c1b14b8d1147cdda`，轉換輸出為 `85457a36e013e053b70dea19d500ebec2ae6bedf5dad9af0035b973c8db6a60a`。Ubuntu 包內 `usr/share/doc/iso-codes/copyright` 宣告 LGPL-2.1-or-later；這項包級宣告不單獨證明 SIL 程式碼錶的歷史授權或完整程式碼集再分發權。專案不將該包或完整轉換結果放入原始碼交付，使用者應分別核對來源、適用條款及後續分發範圍。

也可自行取得 [IANA 語言子標籤登錄檔](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)和[美國國會圖書館 ISO 639-2 列表](https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt)，使用以下命令生成另一份輸入。工具只提取程式碼，不保留名稱；它報告未匹配的程式碼、輸入與輸出摘要，並拒絕無效或有歧義的記錄。下面的 Linux／macOS shell 示例使用新目錄，校驗本次核驗的兩個原始檔案後才轉換：

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

對照所用 IANA 檔案日期為 2026-09-17，原始檔案 SHA-256 為 `755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1`。2026-11-03 從 LoC 直接取得的 UTF-8 原始檔案為 16,010 位元組、SHA-256 `42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371`；其 487 條記錄的前三個程式碼欄位與此前網頁重建記錄逐項一致。轉換器的 LF 輸出摘要為 `6a75bebfa694258a1a4125fc30512d8ae7994080701377b41306db665ad371c6`。該組合生成 8268 行，現有 Qt 語言的 346 個不同查詢鍵全部一致；使用固定 Qt 與 CLDR 重生八個檔案，僅規範化生成日期和換行後全部匹配，修改一項輸入的負對照匹配七個。對照中 `bh`、`in`、`iw`、`ji`、`jw`、`mo`、`sh` 七個 IANA 雙字母子標籤沒有 LoC 對應記錄，工具會報告它們；其他版本和新增語言須單獨核對。上游地址可能更新，摘要不符時須重新核驗，不應跳過檢查。詳情見 [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588040592)。

[IANA 與 IETF 的宣告](https://www.iana.org/help/licensing-terms)將其擁有或有權轉授的協議登錄檔相關權利按 CC0 處理，並保留第三方可能擁有的權利；該宣告不自動證明 SIL 原始表的全部權利。專案不在此交付本次對照使用的 2026-09-17 IANA 快照、LoC 原始輸入或完整轉換結果；前述 CLDR 43.1 子集另含較早的登錄檔。接收者須分別核對來源、適用條款與後續分發條件。該替代輸入不證明 Qt 上游歷史輸入，也不單獨證明對應原始碼義務已經完成。

這裡的獨立對照指分別取得檔案並執行轉換，不表示標準來源獨立。[RFC 5646 第 2.2.1 節](https://www.rfc-editor.org/rfc/rfc5646#section-2.2.1)將 ISO 639-2、639-3 和 639-5 列為 IANA 三字母語言子標籤的標準來源，因此數值重合不能證明資料獨立創作或完整轉授權。


輸出包含實際固定提交的 xdzip-source.tar、rust-vendor.tar.gz、離線 Cargo 配置、Qt 原始歸檔、REBUILD.md 和逐檔案 SHA-256／大小清單。清單明確為 collected-not-compliance-verified，不能替代 SDK 構建 features／補丁、平臺執行庫、內嵌資源和許可履約的核對。專案原始碼重新配置的身份可能為 unknown，應由外部交付清單繫結原提交，不能偽造乾淨 Git 身份。

`rar-wire-mappings/` 單獨保留有限 RAR 線對映的固定參考來源、說明和 UnRAR 7.30 beta1 精確原始許可，全部進入交付檔案雜湊清單。下載歸檔名為 `unrarsrc-7.3.1.tar.gz`，不據檔名另推上游發行版本。交付前還核對相關 Rust 原始碼註釋中完整保留的上游許可第 2 段，作為保守通知措施；缺失或變更時停止。RAR crate 的機器可讀宣告採用保守的 `Apache-2.0 AND UnRAR`；`UnRAR` 是 SPDX 3.28 起可識別的非 OSI 批准標識，舊版工具可能不認識。它不攜帶官方引擎原始碼，也不將事實來源重新許可為 Apache-2.0，條款適用範圍和實際再分發義務繼續按候選複核。

另附固定的 ICU 73.2 完整許可、ICU4C 官方釋出原始碼歸檔、包含可編輯資料的固定 ICU 原始碼樹、CLDR 43.1 原始碼子集，以及生成工具使用的 ICU 73.1 原始碼樹，服務於 Linux ICU，並納入交付總清單。CLDR 子集排除五個 ISO 639-3 檔案、`docs/ldml` 下的 UTS #35 標準正文及配套檔案、兩張文件截圖，並替換 963 張網頁工具圖片；原歸檔摘要、逐檔案排除和替換清單保留在交付記錄中。[Unicode 釋出的 43.1 版 UTS #35](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)可單獨查閱，其使用條件由權利人說明。是否存在可轉授的額外授權仍待核實。收集器也拒絕引導原始碼歸檔中名稱包含 `iso-639-3` 的檔案，名稱檢查不能排除其他形式的資料。收集時校驗歸檔摘要、版本、歸檔內許可及指定原始檔，不解包第三方路徑。[ICU 資料重建說明](icu-cldr-rebuild.md)記錄工具構建、資料生成和修改範圍；上游版本相符不證明供應商二進位制由這些材料無補丁構建。目前 Windows 開發包不包含軟體 OpenGL 庫，不再為該庫收集 Mesa／LLVM 材料；歷史交付保留原有材料和義務，見[原生執行庫來源與通知](native-runtime-materials.md)。

應用動態連結 Qt，提供原始碼重建入口和相容庫替換說明。[macOS 部署核驗](macos-deployment.md)覆蓋隔離副本的修改庫載入，不聲稱已從原始碼重建整套 Qt。實際 Qt 原始碼構建與產品載入另有獨立流程，見[自動驗證契約](../../../developers/engineering/candidate-verification.md)。SDK 位元組級構建等價性、精確靜態連結歸屬和許可分支仍屬於[發行許可複核](https://github.com/xiaodaozhi/xdzip/issues/4)，不重新列為已完成 WP-12 的開發待辦。

verify-source-delivery.py 先重新核對交付檔案雜湊，將自有專案和 vendor 安全解到新的 build/ 目錄，使用空 Cargo 快取、CARGO_NET_OFFLINE=true 和現有固定 Qt SDK 構建真實應用，再執行啟動／關閉冒煙。它驗證專案與 Rust 交付輸入可離線使用，不聲稱 Qt 已從模組原始碼重建，不聲稱位元組級可重複構建。生成原始碼目錄的 Git 身份為 unknown，由外部固定提交及交付清單繫結。

原始碼交付在輸出目錄內以相對 `vendor` 引數執行 Cargo，並顯式指定專案清單；只接受目前受支援的 crates.io 源對映與相對 `vendor` 配置。輸出包含絕對目錄、路徑逃逸或意外源對映時直接失敗，避免將構建機路徑寫入可遷移交付。

對於鎖定 vendor 原始碼中沒有常見檔名的獨立許可或通知檔案的包，交付在 `rust-vendor-notices/` 另存按包索引的固定上游原文。收集器核對包的 Cargo 清單雜湊、上游 Git 提交與路徑、許可原文 SHA-256 和 Git blob；未知缺口或身份不符時停止。補充目錄不改變 Cargo vendor 原檔案及校驗，許可表示式保留上游原樣，不由此批准具體許可分支或既有發行的合規狀態。

新增[Qt SDK 對應核驗](qt-sdk-correspondence.md)：交付包總是包含固定對應清單和可重放的 AGL 消費端補丁，傳入 --qt-prefix 後額外包含該 SDK 的逐檔案原始碼宣告核對、構建配置和明確缺口。交付清單遞迴雜湊補丁子目錄，重建驗證同樣複核這些位元組。未傳 SDK 時不偽造配置核驗記錄。

交付還保留固定的 Qt Declarative Git 提交物件輸入，供離線核對釋出提交與歸檔 `.tag` 樹物件的關係；交付清單會雜湊該輸入。樹物件對應不代表原始碼歸檔覆蓋整個樹，或 SDK 與該樹構建等價。

實際 Qt 原始碼構建和載入／相容替換流程見[自動化收尾契約](../../../developers/engineering/candidate-verification.md)，不再把應用使用現有 SDK 的離線重建誤稱為 Qt 本身的原始碼重建。
