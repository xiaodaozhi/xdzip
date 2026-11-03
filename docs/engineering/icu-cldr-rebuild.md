# 从 CLDR 重建 ICU 73.2 数据

本流程用于编辑和重建 Linux ICU 73.2 的区域资源。源码交付包括固定的 ICU 73.2 源码树、CLDR 43.1 子集，以及构建 CLDR 工具所需的 ICU 73.1 源码树。CLDR 子集有意排除五个 ISO 639-3 文件并替换一张网页工具图片，不是完整上游树；本页验证的既有资源转换未要求补回这些文件，其他工具用途或新增数据仍须单独检查来源和条款。

以下命令针对 Ubuntu 24.04 x86_64，要求 JDK 17、Ant、Maven、Python 3、C/C++ 编译工具、`make` 和 UTF-8 locale。Maven 首次构建需要访问其依赖仓库；ICU 73.1 的两个本地构建 JAR 只用于本流程，不能称为上游发布 JAR 的逐字节副本。先将 `XDZIP_DELIVERY` 指向可信源码交付目录，`JAVA_HOME` 指向 JDK 17；使用新目录，避免覆盖已有源码或构建结果。CLDR 子集的 gzip 字节可能随生成环境变化，按该次交付的清单核验；清单本身也须来自可信渠道。

English: This Linux workflow rebuilds ICU 73.2 locale resources using the delivered ICU source, a CLDR 43.1 subset and ICU 73.1 bootstrap source. The subset omits five ISO 639-3 files and replaces one web tool image; the verified existing-resource conversion did not require the omitted files, but other uses or added data need separate provenance and terms review. Use Ubuntu 24.04 x86_64 with JDK 17, Ant, Maven, Python 3, a C/C++ toolchain, `make` and a UTF-8 locale. Maven initially needs its dependency repositories. Set `XDZIP_DELIVERY` to the trusted delivery and `JAVA_HOME` to JDK 17. The locally built bootstrap JARs are not claimed to be byte-identical to upstream published artifacts. Gzip bytes for the CLDR subset may vary across build environments, so verify it against that delivery's manifest; obtain the manifest through a trusted channel.

此子集的其他文件仍含大量与 ISO 639-3 重合的语言标识符和别名，并保留一份语言子标签注册表；排除五份原始表不单独解决这些数据的来源或授权。[来源交付说明](source-delivery.md)和 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)记录对照及跨版本限制。

English: Other files in the subset retain many language identifiers and aliases matching ISO 639-3, as well as a language-subtag registry; omitting the five original tables does not by itself resolve their provenance or authorization. The [source delivery guide](source-delivery.md) and [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) describe the comparison and its cross-version limits.

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
    'cldr-43.1-source-tree-without-iso6393.tar.gz': None,
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
tar -xzf "$XDZIP_DELIVERY/cldr-43.1-source-tree-without-iso6393.tar.gz" -C "$XDZIP_WORK/sources"
tar -xzf "$XDZIP_DELIVERY/icu-73.1-cldr-bootstrap-source-tree.tar.gz" -C "$XDZIP_WORK/sources")
export XDZIP_ICU732="$XDZIP_WORK/sources/icu-680f521746a3bd6a86f25f25ee50a62d88b489cf"
export XDZIP_CLDR="$XDZIP_WORK/sources/cldr-37d524354422f46731db74f3bd6b3f46516c3ced"
export XDZIP_ICU731="$XDZIP_WORK/sources/icu-5861e1fd52f1d7673eee38bc3c965aa18b336062"
```

先生成 ICU4J 引导工具，再构建 CLDR API 和 ICU 转换器。所有 Maven 缓存位于本轮目录；转换器源码内的本地 Maven 仓库也只写入刚解包的副本。工具源码可能尝试访问 GitHub Packages 的元数据；不得把认证失败当成生成成功，须检查每条命令的退出状态。

English: Build the ICU4J bootstrap, CLDR API and ICU converter next. Maven caches and the converter's local Maven repository are confined to the extracted copies. The tools may attempt GitHub Packages metadata access; check each command's exit status and do not treat an authentication failure as a successful generation.

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

如需修改数据，在下一步前编辑隔离副本中的 CLDR `common` 文件。随后生成 production 数据和 ICU 资源；此示例跳过上游的 C／Java 源码生成，只处理数据文件。此阶段可在依赖准备完成后断网运行。

English: Edit the isolated CLDR `common` files before the next step when testing a change. Generate production data and ICU resources as follows. This example omits upstream C/Java code generation and handles data files only; it can run offline once the dependencies are ready.

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

使用未修改输入时，本项目对照的 4,047 个生成文件中有 4,046 个与固定 ICU 源码逐字节相同。唯一差异 `brkitr/root.txt` 需要按上游源码内 `icu4c/source/data/cldr-icu-readme.txt` 第 5e 步补回 Thai 和 Mymr 两项 LSTM 依赖；补回后该文件也逐字节一致。修改输入后，结果应按实际修改复核，不能期待继续与原文件完全一致。核对资源后，将其复制到隔离的 ICU 73.2 源码树的 `icu4c/source/data`，补回上述依赖，再按该源码树的 `runConfigureICU` 与 `make` 构建数据。

English: With unmodified inputs, 4,046 of 4,047 generated files matched the pinned ICU source byte for byte. The remaining `brkitr/root.txt` needs the Thai and Mymr LSTM entries described by step 5e of upstream `icu4c/source/data/cldr-icu-readme.txt`; after that addition it also matched. Review changed data against the intended edit, copy the verified resources into the isolated ICU 73.2 tree's `icu4c/source/data`, restore those entries, and build using that tree's `runConfigureICU` and `make`.

该流程已对一条自有英语名称修改验证至重新生成的数据包和 ICU API 读取，不代表原供应商 ICU 动态库已被重建或替换，也不自动完成全部源码及许可义务。详细结果见 [#140](https://github.com/xiaodaozhi/xdzip/issues/140)。

English: One synthetic English-name edit was verified through a rebuilt data package and an ICU API read. This does not establish that the original supplier's ICU shared libraries were rebuilt or replaced, or independently complete all source and licensing obligations. See [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
