<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# CLDR から ICU 73.2 のデータを再構築

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文（台灣）](../../zh-TW/engineering/icu-cldr-rebuild.md) | 日本語 | [한국어](../../ko-KR/engineering/icu-cldr-rebuild.md) | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

この Linux 手順では、提供された ICU ソース、CLDR 43.1 のサブセット、ICU 73.1 のブートストラップソースを使用して ICU 73.2 のロケールリソースを再構築します。サブセットには 5 個の ISO 639-3 ファイル、標準文書の `docs/ldml` ディレクトリー、ドキュメントのスクリーンショット 2 枚が含まれず、Web ツールの画像 963 点が置き換えられています。[Unicode が公開する UTS #35 バージョン 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) は権利者の条件に従って別途利用できます。その他の用途や追加データには、出所と条件の個別確認が必要です。JDK 17、Ant、Maven、Python 3、C/C++ ツールチェーン、`make`、UTF-8 ロケールを備えた Ubuntu 24.04 x86_64 を使用してください。Maven は最初に依存関係のリポジトリーへのアクセスが必要です。`XDZIP_DELIVERY` に信頼できる配布物、`JAVA_HOME` に JDK 17 を設定します。既存のソースやビルド結果を上書きしないよう、新しい作業ディレクトリーを使用してください。ローカルでビルドしたブートストラップ JAR が上流公開物とバイト単位で一致するとは主張しません。CLDR サブセットの gzip バイト列はビルド環境によって変わることがあるため、信頼できる経路で取得したその配布物のマニフェストと照合してください。

サブセットの他のファイルには、ISO 639-3 と一致する多数の言語識別子と別名、および言語サブタグのレジストリーが残っています。元の 5 表を除外するだけでは、それらの出所や使用許諾は解決しません。[ソース配布ガイド](source-delivery.md)と [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) に比較とバージョン間の制約を記載しています。

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

次に ICU4J ブートストラップ、CLDR API、ICU コンバーターをビルドします。Maven キャッシュは作業ディレクトリー内に、コンバーターのローカル Maven リポジトリーは展開されたソースのコピー内に置かれます。ツールが GitHub Packages のメタデータにアクセスしようとする場合があります。各コマンドの終了ステータスを確認し、認証失敗を生成成功として扱わないでください。

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

変更をテストする場合は、次のステップの前に分離された CLDR の `common` ファイルを編集します。次のように製品用データと ICU リソースを生成します。この例は上流の C/Java コード生成を省き、データファイルのみを扱います。依存関係が準備できればオフラインで実行できます。

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

入力を変更しない場合、生成された 4,047 ファイルのうち 4,046 ファイルが固定された ICU ソースとバイト単位で一致しました。残る `brkitr/root.txt` には、上流の `icu4c/source/data/cldr-icu-readme.txt` の手順 5e に記載された Thai と Mymr の LSTM エントリーが必要です。追加後はこれも一致しました。変更後のデータは意図した編集に照らして確認し、元とバイト単位で一致することを期待しないでください。検証済みリソースを分離した ICU 73.2 ツリーの `icu4c/source/data` にコピーし、それらのエントリーを復元して、そのツリーの `runConfigureICU` と `make` でビルドします。

英語名の人工的な変更 1 件について、再構築したデータパッケージと ICU API の読み取りで検証しました。これは元の提供元の ICU 共有ライブラリーが再構築または置換されたことや、ソースとライセンスに関するすべての義務が別途満たされたことを示すものではありません。[#140](https://github.com/xiaodaozhi/xdzip/issues/140) を参照してください。
