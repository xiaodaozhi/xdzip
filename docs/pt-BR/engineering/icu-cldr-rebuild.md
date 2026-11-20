<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# Recriar os dados do ICU 73.2 a partir do CLDR

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文](../../zh-TW/engineering/icu-cldr-rebuild.md) | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

Este procedimento no Linux recria os recursos de localidade do ICU 73.2 usando o código-fonte ICU entregue, um subconjunto do CLDR 43.1 e o código de inicialização do ICU 73.1. O subconjunto omite cinco arquivos ISO 639-3, o diretório de normas `docs/ldml` e duas capturas da documentação, além de substituir 963 imagens de ferramentas web. A [versão 43.1 da UTS #35 publicada pelo Unicode](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) está disponível separadamente sob os termos de seu titular. Outros usos ou dados adicionados exigem revisão separada de procedência e termos. Use Ubuntu 24.04 x86_64 com JDK 17, Ant, Maven, Python 3, ferramentas C/C++, `make` e localidade UTF-8. Inicialmente, o Maven precisa acessar seus repositórios de dependências. Configure `XDZIP_DELIVERY` com a entrega confiável e `JAVA_HOME` com o JDK 17. Use um novo diretório de trabalho para evitar a sobrescrita de fontes ou resultados existentes. Não se afirma que os JARs de inicialização compilados localmente sejam idênticos byte a byte aos artefatos publicados pelo projeto original. Os bytes gzip do subconjunto CLDR podem variar entre ambientes; confira-os com o manifesto dessa entrega, obtido por um canal confiável.

Outros arquivos do subconjunto ainda contêm muitos identificadores e aliases de idiomas correspondentes à ISO 639-3, além de um registro de subetiquetas de idioma. Omitir as cinco tabelas originais não resolve por si só a procedência ou autorização desses dados. O [guia de entrega do código-fonte](../../engineering/source-delivery.md) e [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) descrevem a comparação e suas limitações entre versões.

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

Em seguida, compile a inicialização ICU4J, a API CLDR e o conversor ICU. O cache do Maven fica no diretório de trabalho, e o repositório Maven local do conversor fica em uma cópia extraída do código. As ferramentas podem tentar acessar metadados do GitHub Packages; verifique o código de saída de cada comando e não trate uma falha de autenticação como geração bem-sucedida.

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

Ao testar uma alteração, edite os arquivos `common` da cópia isolada do CLDR antes da próxima etapa. Gere os dados de produção e os recursos do ICU conforme abaixo. Este exemplo omite a geração de código C/Java do projeto original e processa apenas arquivos de dados; ele pode ser executado offline depois que as dependências estiverem disponíveis.

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

Com entradas sem alterações, 4.046 dos 4.047 arquivos gerados coincidiram byte a byte com o código ICU fixado. O arquivo restante, `brkitr/root.txt`, precisa das entradas LSTM Thai e Mymr descritas na etapa 5e de `icu4c/source/data/cldr-icu-readme.txt` do projeto original; depois de adicionadas, ele também coincidiu. Compare os dados alterados com a edição pretendida; não espere que permaneçam idênticos aos originais. Copie os recursos verificados para `icu4c/source/data` da árvore isolada do ICU 73.2, restaure essas entradas e compile usando `runConfigureICU` e `make` dessa árvore.

Uma alteração sintética em um nome em inglês foi verificada com um pacote de dados recompilado e uma leitura pela API do ICU. Isso não comprova que as bibliotecas compartilhadas ICU do fornecedor original foram recompiladas ou substituídas, nem que todas as obrigações de código-fonte e licença foram cumpridas de forma independente. Consulte [#140](https://github.com/xiaodaozhi/xdzip/issues/140).
