<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# Повторная сборка данных ICU 73.2 из CLDR

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文](../../zh-TW/engineering/icu-cldr-rebuild.md) | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | Русский

Эта процедура для Linux повторно собирает ресурсы локалей ICU 73.2 из поставленного исходного кода ICU, подмножества CLDR 43.1 и исходного кода начальной сборки ICU 73.1. Подмножество не включает пять файлов ISO 639-3, каталог стандартов `docs/ldml` и два снимка документации, а также заменяет 963 изображения веб-инструментов. [Версия 43.1 UTS #35, опубликованная Unicode](https://www.unicode.org/reports/tr35/tr35-69/tr35.html), доступна отдельно на условиях правообладателя. Другие способы использования и добавленные данные требуют отдельной проверки происхождения и условий. Используйте Ubuntu 24.04 x86_64 с JDK 17, Ant, Maven, Python 3, инструментами C/C++, `make` и локалью UTF-8. Первоначально Maven необходим доступ к репозиториям зависимостей. Укажите в `XDZIP_DELIVERY` доверенную поставку, а в `JAVA_HOME` — JDK 17. Используйте новый рабочий каталог, чтобы не перезаписать существующие исходники или результаты сборки. Не утверждается, что локально собранные JAR для начальной сборки побайтно совпадают с опубликованными исходным проектом артефактами. Байты gzip подмножества CLDR могут различаться в разных средах; проверяйте их по манифесту соответствующей поставки, полученному через доверенный канал.

Другие файлы подмножества по-прежнему содержат множество идентификаторов и псевдонимов языков, совпадающих с ISO 639-3, а также реестр языковых подтегов. Исключение пяти исходных таблиц само по себе не решает вопрос их происхождения или разрешения на использование. [Руководство по поставке исходного кода](../../engineering/source-delivery.md) и [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) описывают сравнение и его ограничения между версиями.

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

Затем соберите начальные компоненты ICU4J, API CLDR и преобразователь ICU. Кеш Maven находится в рабочем каталоге, а локальный репозиторий Maven преобразователя — в распакованной копии исходного кода. Инструменты могут пытаться получить метаданные GitHub Packages; проверяйте код выхода каждой команды и не считайте ошибку аутентификации успешной генерацией.

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

При проверке изменений до следующего шага отредактируйте файлы `common` в изолированной копии CLDR. Создайте производственные данные и ресурсы ICU следующим образом. В примере пропущена генерация кода C/Java исходного проекта и обрабатываются только файлы данных; после подготовки зависимостей его можно выполнять без сети.

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

При неизменённых входных данных 4 046 из 4 047 созданных файлов побайтно совпали с зафиксированным исходным кодом ICU. Оставшемуся `brkitr/root.txt` нужны записи LSTM Thai и Mymr, описанные в шаге 5e исходного файла `icu4c/source/data/cldr-icu-readme.txt`; после их добавления он тоже совпал. Проверяйте изменённые данные относительно предполагаемой правки и не ожидайте их побайтного совпадения с оригиналом. Скопируйте проверенные ресурсы в `icu4c/source/data` изолированного дерева ICU 73.2, восстановите эти записи и выполните сборку с помощью `runConfigureICU` и `make` из этого дерева.

Одно искусственное изменение английского имени проверено с помощью пересобранного пакета данных и чтения через API ICU. Это не доказывает, что общие библиотеки ICU первоначального поставщика были пересобраны или заменены, и не означает независимого выполнения всех обязательств по исходному коду и лицензиям. См. [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
