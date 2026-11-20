<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# Regenerar los datos de ICU 73.2 a partir de CLDR

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文（台灣）](../../zh-TW/engineering/icu-cldr-rebuild.md) | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | Español | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

Este procedimiento de Linux regenera los recursos locales de ICU 73.2 a partir del código ICU entregado, un subconjunto de CLDR 43.1 y el código de arranque de ICU 73.1. El subconjunto omite cinco archivos ISO 639-3, el directorio de estándares `docs/ldml` y dos capturas de documentación, y sustituye 963 imágenes de herramientas web. La [versión 43.1 de UTS #35 publicada por Unicode](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) está disponible por separado bajo las condiciones de su titular. Otros usos o datos añadidos requieren una revisión independiente de procedencia y condiciones. Usa Ubuntu 24.04 x86_64 con JDK 17, Ant, Maven, Python 3, herramientas C/C++, `make` y una configuración regional UTF-8. Maven necesita inicialmente sus repositorios de dependencias. Configura `XDZIP_DELIVERY` con la entrega de confianza y `JAVA_HOME` con JDK 17. Usa un directorio de trabajo nuevo para evitar sobrescribir código o resultados existentes. No se afirma que los JAR de arranque compilados localmente sean idénticos byte a byte a los artefactos publicados por el proyecto original. Los bytes gzip del subconjunto CLDR pueden variar entre entornos; verifícalos con el manifiesto de esa entrega, obtenido por un canal de confianza.

Otros archivos del subconjunto conservan numerosos identificadores y alias de idiomas coincidentes con ISO 639-3, además de un registro de subetiquetas de idioma. Omitir las cinco tablas originales no resuelve por sí solo su procedencia o autorización. La [guía de entrega del código fuente](source-delivery.md) y [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) describen la comparación y sus límites entre versiones.

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

A continuación compila el arranque ICU4J, la API de CLDR y el conversor de ICU. La caché de Maven queda en el directorio de trabajo y el repositorio Maven local del conversor en una copia extraída del código. Las herramientas pueden intentar acceder a metadatos de GitHub Packages; comprueba el estado de salida de cada comando y no consideres un fallo de autenticación como una generación satisfactoria.

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

Para probar un cambio, edita los archivos `common` de la copia aislada de CLDR antes del siguiente paso. Genera los datos de producción y los recursos de ICU como se indica a continuación. Este ejemplo omite la generación de código C/Java del proyecto original y solo procesa archivos de datos; puede ejecutarse sin conexión una vez disponibles las dependencias.

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

Con entradas sin modificar, 4.046 de 4.047 archivos generados coincidieron byte a byte con el código ICU fijado. El archivo restante, `brkitr/root.txt`, necesita las entradas LSTM Thai y Mymr descritas en el paso 5e de `icu4c/source/data/cldr-icu-readme.txt` del proyecto original; tras añadirlas también coincidió. Revisa los datos cambiados frente a la modificación prevista; no esperes que sigan siendo idénticos a los originales. Copia los recursos verificados a `icu4c/source/data` del árbol aislado de ICU 73.2, restaura esas entradas y compila con `runConfigureICU` y `make` de ese árbol.

Se verificó un cambio sintético de un nombre en inglés mediante un paquete de datos recompilado y una lectura de la API de ICU. Esto no demuestra que se hayan recompilado o sustituido las bibliotecas compartidas de ICU del proveedor original ni que se hayan cumplido de forma independiente todas las obligaciones relativas al código fuente y las licencias. Véase [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
