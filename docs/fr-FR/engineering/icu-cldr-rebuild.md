<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# Reconstruction des données ICU 73.2 à partir de CLDR

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文](../../zh-TW/engineering/icu-cldr-rebuild.md) | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | Français | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

Ce processus Linux reconstruit les ressources de langue ICU 73.2 à partir des sources ICU livrées, d’un sous-ensemble CLDR 43.1 et des sources d’amorçage ICU 73.1. Le sous-ensemble exclut cinq fichiers ISO 639-3, le répertoire de normes `docs/ldml` et deux captures de documentation, et remplace 963 images d’outils web. La [version 43.1 d’UTS #35 publiée par Unicode](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) est disponible séparément selon les conditions de son titulaire. D’autres utilisations ou des données ajoutées nécessitent un examen distinct de leur provenance et de leurs conditions. Utilisez Ubuntu 24.04 x86_64 avec JDK 17, Ant, Maven, Python 3, une chaîne C/C++, `make` et une locale UTF-8. Maven doit initialement accéder à ses dépôts de dépendances. Faites pointer `XDZIP_DELIVERY` vers une livraison de confiance et `JAVA_HOME` vers JDK 17. Utilisez un nouveau répertoire de travail pour ne pas écraser les sources ou résultats existants. Les JAR d’amorçage construits localement ne sont pas présentés comme identiques octet par octet aux artefacts publiés en amont. Les octets gzip du sous-ensemble CLDR peuvent varier selon l’environnement : vérifiez-les avec le manifeste de la livraison, obtenu par un canal de confiance.

Les autres fichiers du sous-ensemble conservent de nombreux identifiants et alias de langues correspondant à ISO 639-3 ainsi qu’un registre de sous-étiquettes de langue. L’exclusion des cinq tables d’origine ne suffit pas à régler leur provenance ou leur autorisation. Le [guide de livraison des sources](source-delivery.md) et [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) décrivent la comparaison et ses limites entre versions.

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

Construisez ensuite l’amorçage ICU4J, l’API CLDR et le convertisseur ICU. Le cache Maven reste dans le répertoire de travail, et le dépôt Maven local du convertisseur dans une copie extraite des sources. Les outils peuvent tenter d’accéder aux métadonnées GitHub Packages : vérifiez le code de sortie de chaque commande et ne considérez pas un échec d’authentification comme une génération réussie.

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

Pour tester une modification, éditez les fichiers `common` de la copie CLDR isolée avant l’étape suivante. Générez les données de production et les ressources ICU comme suit. Cet exemple omet la génération de code C/Java en amont et ne traite que les fichiers de données ; il peut fonctionner hors ligne une fois les dépendances disponibles.

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

Sans modification des entrées, 4 046 fichiers générés sur 4 047 correspondaient octet par octet aux sources ICU fixées. Le fichier restant, `brkitr/root.txt`, nécessite les entrées LSTM Thai et Mymr décrites à l’étape 5e du fichier amont `icu4c/source/data/cldr-icu-readme.txt` ; après leur ajout, il correspondait également. Vérifiez les données modifiées selon l’édition voulue : elles ne doivent pas forcément rester identiques aux originaux. Copiez les ressources vérifiées dans `icu4c/source/data` de l’arbre ICU 73.2 isolé, rétablissez ces entrées et construisez avec `runConfigureICU` et `make` de cet arbre.

Une modification synthétique d’un nom anglais a été vérifiée au moyen d’un paquet de données reconstruit et d’une lecture par l’API ICU. Cela ne prouve pas que les bibliothèques partagées ICU du fournisseur initial ont été reconstruites ou remplacées, ni que toutes les obligations relatives aux sources et aux licences ont été remplies indépendamment. Voir [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
