<a name="xdzip-从-cldr-重建-icu-732-数据"></a>

# 從 CLDR 重建 ICU 73.2 資料

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | 繁體中文（台灣）

本流程用於編輯和重建 Linux ICU 73.2 的區域資源。原始碼交付包括固定的 ICU 73.2 原始碼樹、CLDR 43.1 子集，以及構建 CLDR 工具所需的 ICU 73.1 原始碼樹。CLDR 子集有意排除五個 ISO 639-3 檔案、`docs/ldml` 標準文件目錄和兩張文件截圖，並替換 963 張網頁工具圖片，不是完整上游樹。標準文字可檢視 [Unicode 釋出的 43.1 版 UTS #35](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)，其使用條件由權利人說明；其他工具用途或新增資料仍須單獨檢查來源和條款。

以下命令針對 Ubuntu 24.04 x86_64，要求 JDK 17、Ant、Maven、Python 3、C/C++ 編譯工具、`make` 和 UTF-8 locale。Maven 首次構建需要訪問其依賴倉庫；ICU 73.1 的兩個本地構建 JAR 只用於本流程，不能稱為上游釋出 JAR 的逐位元組副本。先將 `XDZIP_DELIVERY` 指向可信原始碼交付目錄，`JAVA_HOME` 指向 JDK 17；使用新目錄，避免覆蓋已有原始碼或構建結果。CLDR 子集的 gzip 位元組可能隨生成環境變化，按該次交付的清單核驗；清單本身也須來自可信渠道。

此子集的其他檔案仍含大量與 ISO 639-3 重合的語言識別符號和別名，並保留一份語言子標籤登錄檔；排除五份原始表不單獨解決這些資料的來源或授權。[來源交付說明](source-delivery.md)和 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)記錄對照及跨版本限制。

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

先生成 ICU4J 引導工具，再構建 CLDR API 和 ICU 轉換器。所有 Maven 快取位於本輪目錄；轉換器原始碼內的本地 Maven 倉庫也只寫入剛解包的副本。工具原始碼可能嘗試訪問 GitHub Packages 的後設資料；不得把認證失敗當成生成成功，須檢查每條命令的退出狀態。

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

如需修改資料，在下一步前編輯隔離副本中的 CLDR `common` 檔案。隨後生成 production 資料和 ICU 資源；此示例跳過上游的 C／Java 原始碼生成，只處理資料檔案。此階段可在依賴準備完成後斷網執行。

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

使用未修改輸入時，本專案對照的 4,047 個生成檔案中有 4,046 個與固定 ICU 原始碼逐位元組相同。唯一差異 `brkitr/root.txt` 需要按上游原始碼內 `icu4c/source/data/cldr-icu-readme.txt` 第 5e 步補回 Thai 和 Mymr 兩項 LSTM 依賴；補回後該檔案也逐位元組一致。修改輸入後，結果應按實際修改複核，不能期待繼續與原檔案完全一致。核對資源後，將其複製到隔離的 ICU 73.2 原始碼樹的 `icu4c/source/data`，補回上述依賴，再按該原始碼樹的 `runConfigureICU` 與 `make` 構建資料。

該流程已對一條自有英語名稱修改驗證至重新生成的資料包和 ICU API 讀取，不代表原供應商 ICU 動態庫已被重建或替換，也不自動完成全部原始碼及許可義務。詳細結果見 [#140](https://github.com/xiaodaozhi/xdzip/issues/140)。
