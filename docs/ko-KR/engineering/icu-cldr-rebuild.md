<a name="xdzip-rebuild-icu-732-data-from-cldr"></a>

# CLDR에서 ICU 73.2 데이터 다시 빌드

[English](../../engineering/icu-cldr-rebuild.md) | [简体中文](../../zh-CN/engineering/icu-cldr-rebuild.md) | [繁體中文（台灣）](../../zh-TW/engineering/icu-cldr-rebuild.md) | [日本語](../../ja-JP/engineering/icu-cldr-rebuild.md) | 한국어 | [Français](../../fr-FR/engineering/icu-cldr-rebuild.md) | [Deutsch](../../de-DE/engineering/icu-cldr-rebuild.md) | [Español](../../es-ES/engineering/icu-cldr-rebuild.md) | [Português (Brasil)](../../pt-BR/engineering/icu-cldr-rebuild.md) | [Русский](../../ru-RU/engineering/icu-cldr-rebuild.md)

이 Linux 절차는 제공된 ICU 소스, CLDR 43.1 부분 집합과 ICU 73.1 부트스트랩 소스를 사용하여 ICU 73.2 로캘 리소스를 다시 빌드합니다. 부분 집합은 ISO 639-3 파일 5개, 표준 문서 디렉터리 `docs/ldml`, 문서 스크린샷 2개를 제외하고 웹 도구 이미지 963개를 대체합니다. [Unicode가 게시한 UTS #35 버전 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)은 권리자의 조건에 따라 별도로 제공됩니다. 다른 용도나 추가 데이터는 출처와 조건을 별도로 검토해야 합니다. JDK 17, Ant, Maven, Python 3, C/C++ 도구 체인, `make`, UTF-8 로캘이 설치된 Ubuntu 24.04 x86_64를 사용하세요. Maven은 처음에 의존성 저장소에 접근해야 합니다. `XDZIP_DELIVERY`는 신뢰할 수 있는 전달물로, `JAVA_HOME`은 JDK 17로 설정합니다. 기존 소스나 빌드 결과를 덮어쓰지 않도록 새 작업 디렉터리를 사용하세요. 로컬에서 빌드한 부트스트랩 JAR이 상위 프로젝트에서 게시한 산출물과 바이트 단위로 동일하다고 주장하지 않습니다. CLDR 부분 집합의 gzip 바이트는 빌드 환경에 따라 달라질 수 있으므로 신뢰할 수 있는 경로로 받은 해당 전달물의 매니페스트와 대조하세요.

부분 집합의 다른 파일에는 ISO 639-3과 일치하는 언어 식별자와 별칭이 다수 남아 있으며 언어 하위 태그 레지스트리도 포함됩니다. 원래 표 5개를 제외하는 것만으로 해당 데이터의 출처나 사용 권한이 해결되지는 않습니다. [소스 전달 가이드](source-delivery.md)와 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)에 비교 내용과 버전 간 한계가 설명되어 있습니다.

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

다음으로 ICU4J 부트스트랩, CLDR API와 ICU 변환기를 빌드합니다. Maven 캐시는 작업 디렉터리에, 변환기의 로컬 Maven 저장소는 압축을 푼 소스 사본에 보관됩니다. 도구가 GitHub Packages 메타데이터에 접근을 시도할 수 있으므로 각 명령의 종료 상태를 확인하고 인증 실패를 생성 성공으로 취급하지 마세요.

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

변경을 테스트할 때는 다음 단계 전에 격리된 CLDR의 `common` 파일을 편집하세요. 아래와 같이 프로덕션 데이터와 ICU 리소스를 생성합니다. 이 예제는 상위 프로젝트의 C/Java 코드 생성을 생략하고 데이터 파일만 처리하며, 의존성이 준비되면 오프라인으로 실행할 수 있습니다.

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

입력을 변경하지 않았을 때 생성된 파일 4,047개 중 4,046개가 고정된 ICU 소스와 바이트 단위로 일치했습니다. 나머지 `brkitr/root.txt`에는 상위 프로젝트의 `icu4c/source/data/cldr-icu-readme.txt` 5e 단계에 설명된 Thai 및 Mymr LSTM 항목이 필요하며, 추가 후 이 파일도 일치했습니다. 변경된 데이터는 의도한 수정에 비추어 검토하고 원본과 바이트 단위로 동일하리라고 기대하지 마세요. 검증된 리소스를 격리된 ICU 73.2 트리의 `icu4c/source/data`에 복사하고 해당 항목을 복원한 다음 그 트리의 `runConfigureICU`와 `make`로 빌드합니다.

영어 이름을 인위적으로 한 번 변경한 결과를 재빌드한 데이터 패키지와 ICU API 읽기로 검증했습니다. 이것은 최초 공급업체의 ICU 공유 라이브러리를 다시 빌드하거나 교체했다는 증거가 아니며, 소스와 라이선스의 모든 의무를 별도로 충족했다는 뜻도 아닙니다. [#140](https://github.com/xiaodaozhi/xdzip/issues/140)을 참고하세요.
