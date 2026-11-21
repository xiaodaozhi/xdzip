<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# ICU-73.2-Daten aus CLDR neu erstellen

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文](../../zh-TW/engineering/icu-cldr-rebuild.md) | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | Deutsch | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

Dieser Linux-Ablauf erstellt ICU-73.2-Sprachressourcen aus dem gelieferten ICU-Quellcode, einer CLDR-43.1-Teilmenge und ICU-73.1-Bootstrap-Quellen neu. Die Teilmenge lässt fünf ISO-639-3-Dateien, das Normenverzeichnis `docs/ldml` und zwei Dokumentationsscreenshots aus und ersetzt 963 Bilder von Webwerkzeugen. Die [von Unicode veröffentlichte UTS-#35-Version 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) ist separat unter den Bedingungen ihres Rechteinhabers verfügbar. Andere Verwendungen oder zusätzliche Daten benötigen eine eigene Prüfung von Herkunft und Bedingungen. Verwenden Sie Ubuntu 24.04 x86_64 mit JDK 17, Ant, Maven, Python 3, C/C++-Werkzeugkette, `make` und UTF-8-Locale. Maven benötigt anfangs Zugriff auf seine Abhängigkeits-Repositories. Setzen Sie `XDZIP_DELIVERY` auf die vertrauenswürdige Lieferung und `JAVA_HOME` auf JDK 17. Verwenden Sie ein neues Arbeitsverzeichnis, um vorhandene Quellen und Build-Ergebnisse nicht zu überschreiben. Die lokal gebauten Bootstrap-JARs werden nicht als bytegleich mit den upstream veröffentlichten Artefakten bezeichnet. Die gzip-Bytes der CLDR-Teilmenge können je nach Build-Umgebung variieren; prüfen Sie sie anhand des Manifests dieser Lieferung, das Sie über einen vertrauenswürdigen Kanal beziehen.

Andere Dateien der Teilmenge enthalten weiterhin viele zu ISO 639-3 passende Sprachkennungen und Aliasse sowie ein Sprachsubtag-Register. Das Weglassen der fünf ursprünglichen Tabellen klärt deren Herkunft oder Berechtigung nicht von selbst. Der [Leitfaden zur Quelllieferung](source-delivery.md) und [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) beschreiben den Vergleich und seine versionsübergreifenden Grenzen.

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

Erstellen Sie als Nächstes den ICU4J-Bootstrap, die CLDR-API und den ICU-Konverter. Der Maven-Cache liegt im Arbeitsverzeichnis, das lokale Maven-Repository des Konverters in einer entpackten Quellkopie. Werkzeuge können versuchen, auf Metadaten von GitHub Packages zuzugreifen. Prüfen Sie den Exitstatus jedes Befehls und werten Sie einen Authentifizierungsfehler nicht als erfolgreiche Generierung.

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

Bearbeiten Sie beim Testen einer Änderung vor dem nächsten Schritt die `common`-Dateien der isolierten CLDR-Kopie. Generieren Sie Produktionsdaten und ICU-Ressourcen wie folgt. Das Beispiel lässt die upstream C/Java-Codegenerierung aus und verarbeitet nur Datendateien; nach Bereitstellung der Abhängigkeiten kann es offline laufen.

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

Bei unveränderten Eingaben stimmten 4.046 von 4.047 erzeugten Dateien byteweise mit den festgelegten ICU-Quellen überein. Die verbleibende Datei `brkitr/root.txt` benötigt die Thai- und Mymr-LSTM-Einträge aus Schritt 5e der upstream-Datei `icu4c/source/data/cldr-icu-readme.txt`; nach deren Ergänzung stimmte auch sie überein. Prüfen Sie geänderte Daten anhand der beabsichtigten Änderung; sie müssen nicht bytegleich zu den Originalen bleiben. Kopieren Sie die geprüften Ressourcen in `icu4c/source/data` des isolierten ICU-73.2-Quellbaums, stellen Sie diese Einträge wieder her und bauen Sie mit dessen `runConfigureICU` und `make`.

Eine synthetische Änderung eines englischen Namens wurde mit einem neu gebauten Datenpaket und einem ICU-API-Lesezugriff geprüft. Das belegt weder, dass die gemeinsam genutzten ICU-Bibliotheken des ursprünglichen Lieferanten neu gebaut oder ersetzt wurden, noch dass sämtliche Quellcode- und Lizenzpflichten unabhängig erfüllt sind. Siehe [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
