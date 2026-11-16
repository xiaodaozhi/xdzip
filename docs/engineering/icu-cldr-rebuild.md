# Rebuild ICU 73.2 data from CLDR

This Linux workflow rebuilds ICU 73.2 locale resources using the delivered ICU source, a CLDR 43.1 subset and ICU 73.1 bootstrap source. The subset omits five ISO 639-3 files, the `docs/ldml` standards directory and two documentation screenshots, and replaces 963 web tool images. The [Unicode-published UTS #35 version 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) is available separately under its owner's terms. Other uses or added data need separate provenance and terms review. Use Ubuntu 24.04 x86_64 with JDK 17, Ant, Maven, Python 3, a C/C++ toolchain, `make` and a UTF-8 locale. Maven initially needs its dependency repositories. Set `XDZIP_DELIVERY` to the trusted delivery and `JAVA_HOME` to JDK 17. Use a new working directory to avoid overwriting existing source or build results. The locally built bootstrap JARs are not claimed to be byte-identical to upstream published artifacts. Gzip bytes for the CLDR subset may vary across build environments, so verify it against that delivery's manifest; obtain the manifest through a trusted channel.

Other files in the subset retain many language identifiers and aliases matching ISO 639-3, as well as a language-subtag registry; omitting the five original tables does not by itself resolve their provenance or authorization. The [source delivery guide](source-delivery.md) and [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) describe the comparison and its cross-version limits.

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

Build the ICU4J bootstrap, CLDR API and ICU converter next. Maven caches and the converter's local Maven repository are confined to the extracted copies. The tools may attempt GitHub Packages metadata access; check each command's exit status and do not treat an authentication failure as a successful generation.

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

Edit the isolated CLDR `common` files before the next step when testing a change. Generate production data and ICU resources as follows. This example omits upstream C/Java code generation and handles data files only; it can run offline once the dependencies are ready.

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

With unmodified inputs, 4,046 of 4,047 generated files matched the pinned ICU source byte for byte. The remaining `brkitr/root.txt` needs the Thai and Mymr LSTM entries described by step 5e of upstream `icu4c/source/data/cldr-icu-readme.txt`; after that addition it also matched. Review changed data against the intended edit; do not expect it to remain byte-identical to the originals. Copy the verified resources into the isolated ICU 73.2 tree's `icu4c/source/data`, restore those entries, and build using that tree's `runConfigureICU` and `make`.

One synthetic English-name edit was verified through a rebuilt data package and an ICU API read. This does not establish that the original supplier's ICU shared libraries were rebuilt or replaced, or independently complete all source and licensing obligations. See [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
