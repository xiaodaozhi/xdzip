<a name="xdzip-从-cldr-重建-icu-732-数据"></a>

# 從 CLDR 重建 ICU 73.2 資料

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | 繁體中文 | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

此 Linux 流程使用交付的 ICU 原始碼、CLDR 43.1 子集與 ICU 73.1 引導程式原始碼，重新建置 ICU 73.2 地區設定資源。CLDR 子集排除五個 ISO 639-3 檔案、標準文件目錄 `docs/ldml` 和兩張文件截圖，並替換 963 張網頁工具圖片。[Unicode 發布的 43.1 版 UTS #35](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)可依權利人的條件另行取得。其他用途或新增資料仍須分別檢查來源及條款。

以下命令適用於 Ubuntu 24.04 x86_64，要求 JDK 17、Ant、Maven、Python 3、C/C++ 編譯工具、`make` 和 UTF-8 地區設定。Maven 初次建置時需要存取相依套件儲存庫。請將 `XDZIP_DELIVERY` 指向可信的交付內容，`JAVA_HOME` 指向 JDK 17；使用新工作目錄，避免覆寫現有原始碼或建置結果。不能宣稱本機建置的引導程式 JAR 與上游發布的成果逐位元組相同。CLDR 子集的 gzip 位元組可能因建置環境而異，請依該次交付的資訊清單核對，並透過可信管道取得清單。

此子集的其他檔案仍含許多與 ISO 639-3 相符的語言識別碼和別名，也保留語言子標籤登錄資料；排除五份原始表格本身無法解決其來源或授權問題。[原始碼交付指南](source-delivery.md)和 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)說明比較方式及跨版本限制。

```sh
export XDZIP_DELIVERY=/path/to/source-delivery
export LC_ALL=C.UTF-8
: "${JAVA_HOME:?Set JAVA_HOME to JDK 17}"
export XDZIP_WORK="$PWD/build/icu-cldr-rebuild"
(set -eu
test ! -e "$XDZIP_WORK"
mkdir -p "$XDZIP_WORK/sources" "$XDZIP_WORK/m2"
python3 - "$XDZIP_DELIVERY" <<'PY'
import hashlib, json, sys
from pathlib import Path
root = Path(sys.argv[1])
manifest = json.loads((root / 'source-delivery.json').read_text(encoding='utf-8'))
expected = {
    'icu-73.2-source-tree.tar.gz': '9032f32c8edb43a7cd842f2c44349eb75756ee29c3a3d6a0bf2a01793a34b84c',
    'cldr-43.1-source-tree-without-iso6393-and-uts35.tar.gz': None,
    'icu-73.1-cldr-bootstrap-source-tree.tar.gz': '11e3d75119767e5a3a9a1ab1637f6e54e978da354cc0a2db230b65f7a9d72aac',
}
for name, pinned in expected.items():
    entries = [item for item in manifest['files'] if item['file'] == name]
    if len(entries) != 1:
        raise SystemExit('Missing or duplicate manifest entry: ' + name)
    path = root / name
    with path.open('rb') as stream:
        actual = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            actual.update(chunk)
    if (actual.hexdigest() != entries[0]['sha256'] or path.stat().st_size != entries[0]['bytes']
            or (pinned and actual.hexdigest() != pinned)):
        raise SystemExit('Source archive verification failed: ' + name)
    print(name + ': OK')
PY
tar -xzf "$XDZIP_DELIVERY/icu-73.2-source-tree.tar.gz" -C "$XDZIP_WORK/sources"
tar -xzf "$XDZIP_DELIVERY/cldr-43.1-source-tree-without-iso6393-and-uts35.tar.gz" -C "$XDZIP_WORK/sources"
tar -xzf "$XDZIP_DELIVERY/icu-73.1-cldr-bootstrap-source-tree.tar.gz" -C "$XDZIP_WORK/sources")
export XDZIP_ICU732="$XDZIP_WORK/sources/icu-680f521746a3bd6a86f25f25ee50a62d88b489cf"
export XDZIP_CLDR="$XDZIP_WORK/sources/cldr-37d524354422f46731db74f3bd6b3f46516c3ced"
export XDZIP_ICU731="$XDZIP_WORK/sources/icu-5861e1fd52f1d7673eee38bc3c965aa18b336062"
```

接著建置 ICU4J 引導程式、CLDR API 和 ICU 轉換器。Maven 快取位於工作目錄，轉換器的本機 Maven 儲存庫位於解壓後的原始碼副本。工具可能嘗試存取 GitHub Packages 的中繼資料；請檢查每個命令的結束狀態，不要將驗證失敗視為產生成功。

```sh
(set -eu
export MAVEN_OPTS='-Xmx3g -Djava.awt.headless=true'
(cd "$XDZIP_ICU731/icu4j" && ant -noinput releaseCLDR)
for XDZIP_ARTIFACT in icu4j utilities; do
  mvn -B -ntp -Dmaven.repo.local="$XDZIP_WORK/m2" install:install-file \
    -DgroupId=com.ibm.icu -DartifactId="$XDZIP_ARTIFACT-for-cldr" \
    -Dversion=73.1-release-73-1 -Dpackaging=jar -DgeneratePom=true \
    -Dfile="$XDZIP_ICU731/icu4j/release_cldr/$XDZIP_ARTIFACT.jar"
done
(cd "$XDZIP_CLDR/tools" && mvn -B -ntp -Dmaven.repo.local="$XDZIP_WORK/m2" -pl cldr-code -am -DskipTests package)
(cd "$XDZIP_ICU732/tools/cldr/lib" && mvn -B -ntp -Dmaven.repo.local="$XDZIP_WORK/m2" install:install-file \
  -Dproject.parent.relativePath= -DgroupId=org.unicode.cldr -DartifactId=cldr-api \
  -Dversion=0.1-SNAPSHOT -Dpackaging=jar -DgeneratePom=true -DlocalRepositoryPath=. \
  -Dfile="$XDZIP_CLDR/tools/cldr-code/target/cldr-code.jar")
(cd "$XDZIP_ICU732/tools/cldr/cldr-to-icu" && mvn -B -ntp -Dmaven.repo.local="$XDZIP_WORK/m2" -DskipTests package))
```

測試變更時，請在下一步前編輯隔離的 CLDR 副本中的 `common` 檔案。接著依下列方式產生正式資料和 ICU 資源。此範例省略上游的 C/Java 程式碼產生程序，只處理資料檔；相依套件準備完成後可離線執行。

```sh
(set -eu
export CLDR_DIR="$XDZIP_CLDR"
java -Xmx4g -jar "$XDZIP_CLDR/tools/cldr-code/target/cldr-code.jar" \
  org.unicode.cldr.tool.GenerateProductionData -s "$XDZIP_CLDR/common" -d "$XDZIP_WORK/production/common"
export CLDR_DIR="$XDZIP_WORK/production"
export CLDR_DATA_DIR="$CLDR_DIR"
export ANT_OPTS='-Xmx4g -Djava.awt.headless=true'
mkdir "$XDZIP_WORK/resources"
(cd "$XDZIP_ICU732/tools/cldr/cldr-to-icu" && ant -noinput -f build-icu-data.xml init-args convert-impl \
  -DcldrDir="$CLDR_DATA_DIR" -DoutDir="$XDZIP_WORK/resources" -DdontGenCode=true))
```

使用未修改的輸入時，產生的 4,047 個檔案中有 4,046 個與固定版本的 ICU 原始碼逐位元組相同。剩餘的 `brkitr/root.txt` 需要上游 `icu4c/source/data/cldr-icu-readme.txt` 第 5e 步所述的 Thai 和 Mymr LSTM 項目；加入後該檔案也相同。請依預期修改內容檢查變更後的資料，不要期待它繼續與原始檔案逐位元組相同。將驗證過的資源複製到隔離的 ICU 73.2 原始碼樹的 `icu4c/source/data`，補回上述項目，再使用該原始碼樹的 `runConfigureICU` 與 `make` 建置。

一項人工修改的英文名稱已透過重建的資料套件和 ICU API 讀取驗證。這不能證明原供應商的 ICU 共用函式庫已重建或替換，也不代表已另外完成所有原始碼及授權義務。詳見 [#140](https://github.com/xiaodaozhi/xdzip/issues/140)。
