# 대응 소스와 재빌드 자료

[English](../../engineering/source-delivery.md) | [简体中文](../../zh-CN/engineering/source-delivery.md) | [繁體中文](../../zh-TW/engineering/source-delivery.md) | [日本語](../../ja-JP/engineering/source-delivery.md) | 한국어 | [Français](../../fr-FR/engineering/source-delivery.md) | [Deutsch](../../de-DE/engineering/source-delivery.md) | [Español](../../es-ES/engineering/source-delivery.md) | [Português (Brasil)](../../pt-BR/engineering/source-delivery.md) | [Русский](../../ru-RU/engineering/source-delivery.md)

**릴리스 경계, 2026-11-05:** 소유자 지시에 따라 이 수집기는 로컬 검토와 재빌드 검증 전용입니다. 대응 소스 아카이브는 더 이상 GitHub Release 또는 Actions 아티팩트로 제공하지 않습니다. 이러한 아카이브를 포함한 과거 공개 Release 에셋과 아직 다운로드할 수 있었던 릴리스 Actions 아티팩트는 철회되었고, 향후 릴리스 워크플로는 생성이나 업로드를 거부합니다. 이 결정은 구성 요소 라이선스를 변경하지 않으며 대응 소스, 재링크 또는 기타 의무가 충족되었음을 입증하지 않습니다. 공개 저장소의 태그에 대해 GitHub가 자동 생성하는 소스 스냅샷은 이러한 Release 첨부 파일과 별개입니다.

일상적인 릴리스와 수용 과정에서는 이 수집기를 실행하거나 대응 소스 아카이브를 로컬에서 생성하면 안 됩니다. 사용자가 소스 수집을 명시적으로 요청한 경우에만 실행하며 업로드 제한은 계속 적용됩니다. 라이선스 검토는 실제 구성 요소와 의무를 확인해야 하며 기계가 생성한 일반적인 검토 표시를 확정된 결함으로 취급하지 않습니다.

scripts/prepare-source-delivery.py는 깨끗한 커밋의 프로젝트 소스, Cargo.lock에 대응하는 완전한 Rust vendor 입력, 고정된 Qt 6.8.2 모듈 소스 아카이브와 재빌드·교체 지침을 수집합니다. 모든 출력은 build/ 아래에 있습니다. 다운로드 주소는 전달된 소스가 아니며 수집기가 오픈 소스 라이선스 선택을 자동 승인하지 않습니다.

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt 런타임 모듈 아카이브는 qtbase, qtdeclarative, qtsvg와 qttranslations입니다. 빌드에 필요한 qtshadertools, qttools와 고정된 Rust 표준 라이브러리 소스 구성 요소도 [공식 Qt 6.8.2 아카이브](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)에서 전달합니다. 공식 .sha256 값은 scripts/corresponding-sources.json에 고정합니다. 캐시와 다운로드를 다시 검증하며 다운로드는 먼저 부분 파일에 저장하고 검증 후 확정합니다. 아카이브 디렉터리만 검사하고 타사 경로를 자동으로 추출하지 않습니다. Cargo vendor는 --locked를 사용하고 레지스트리 체크섬과 빌드 의존성을 유지하며 타사 빌드 스크립트를 실행하지 않습니다.

소스 전달에는 고정된 편집 가능한 Public Suffix List와 MPL-2.0 문서도 포함됩니다. 수집은 입력, Qt 아카이브와 생성기 ID를 검증한 뒤 C++ 데이터와 바이너리 테스트 데이터를 모두 재생성해 바이트 단위로 비교합니다. 불일치 시 중단하며 전체 SDK 동등성이나 릴리스 라이선스 승인을 입증하지 않습니다.

소스 전달은 Qt Core MIME 데이터에 사용된 고정 Apache Tika 입력, 업스트림 NOTICE와 OGL-UK-3.0 문서를 유지합니다. Qt 아카이브의 출처 표시를 검증하고 고정 아카이브의 변환 스크립트로 MIME 데이터를 재생성하며 네이티브 Windows 줄바꿈만 차이로 허용합니다. 완전한 Tika NOTICE는 더 넓은 업스트림 배포를 다루며 그 밖의 의존성을 xdzip이 제공한다는 뜻이 아닙니다.

전달물에는 Qt 6.8.2 로캘 생성에 사용된 CLDR 46 `common` 소스와 업스트림 라이선스도 포함되지만 CLDR 소스 트리의 나머지는 포함되지 않습니다. 수집은 생성된 Qt 소스 두 파일의 CLDR 버전, 고정 업스트림 아카이브 해시와 선택된 입력을 검증합니다. Qt의 두 번째 생성 단계에는 포함되지 않은 ISO 639-3 표가 추가로 필요합니다. `qt-cldr-sources.json`은 테스트한 후보의 소스, 버전과 다이제스트를 기록하고 해당 [업스트림 다운로드 페이지와 조건](https://iso639-3.sil.org/code_tables/download_tables)은 별도 검토가 필요합니다. 이 후보로 생성 데이터를 재현해도 정확한 과거 입력이나 원래 SDK의 전체 빌드 ID를 입증하지 않습니다.

원래 ISO 표를 제외해도 다른 CLDR 데이터에서 관련 ID가 모두 제거되지는 않습니다. CLDR 46의 `common/validity/language.xml`은 압축된 언어 코드를 유지하고 `common/supplemental/supplementalMetadata.xml`에는 별칭이 있습니다. [로컬 비교](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588994510)에서 전달된 이 파일들로 구성한 세 글자 후보 집합은 CLDR 46 후보 ISO 표의 `Id` 값 7,920개 중 7,919개를 포함하고 338개가 추가였습니다. 같은 방법을 CLDR 43.1 부분 집합과 그 이후 표에 적용하면 7,896개를 포함하고 338개가 추가였습니다. 후보 집합은 원래 표가 아니고 모든 필드도 포함하지 않으므로 ID 일치만으로 출처, 허가 또는 적용 조건이 확정되지 않습니다. ISO 이름의 파일 5개를 제외해도 다른 관련 데이터가 없다는 뜻은 아닙니다.

CLDR 43.1 부분 집합에는 헤더 날짜가 2023-02-14인 `language-subtag-registry` 도구 입력도 남아 있습니다. 세 글자 언어 서브태그 8,049개 중 7,713개가 위의 이후 후보 ISO 표 ID와 일치하지만 이 버전 간 차이의 원인은 여기서 단정하지 않습니다. Unicode 업데이트 지침은 이 파일을 IANA에서 얻은 언어 서브태그 레지스트리로 식별합니다. IANA CC0 선언은 IANA가 소유하거나 재허가할 수 있는 권리만 다루며 타사 권리의 가능성을 유지합니다. 비교와 한계는 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)을 참조하세요.

CLDR 43.1 부분 집합은 웹 도구에 사용되는 Noto 글꼴 2개, Glyphicons 파일 4개, Bootstrap과 Autosize 스크립트 및 이모지 이미지도 포함합니다. `cldr43-webapp-notices/`는 각 고정 파일을 검증한 뒤 해당 글꼴과 스크립트, 고정 Noto Emoji 두 버전과 일치하는 이미지 908개의 출처와 라이선스를 제공합니다. 출처와 재배포 조건을 독립적으로 확정하지 못한 나머지 원본 이미지 963개는 수집기가 각 원본 해시를 검증하고 고정 Noto 이미지 698개와 고정 Twemoji 이미지 265개로 교체합니다. 매니페스트는 양쪽 해시, 경로와 소스를 기록하며 침해를 주장하지 않습니다. 이 파일은 xdzip 인터페이스에 포함되지 않습니다. 통지는 IBM 저작권 표시가 있는 웹 도구 소스 파일 3개도 식별하며 해당 릴리스의 루트 README를 근거로 Unicode-DFS-2016에 따라 취급하고 저작권 표시와 라이선스를 유지합니다. 추가 웹 루트 이미지 27개는 고정된 2026년 CLDR 커밋과 바이트 단위로 일치하며 해당 커밋의 Unicode-3.0 라이선스에 따라 완전한 통지와 개별 해시를 유지합니다. 이 평가는 상표, 나열되지 않은 에셋 또는 외부 의존성에는 적용되지 않습니다.

CLDR 43.1 `docs/rfc` 부분 집합의 완전한 텍스트 파일 두 개 `draft-davis-t-langtag-ext.txt`와 `draft-davis-u-langtag-ext-00.txt`는 IETF의 [2011년 T 확장 초안](https://www.ietf.org/archive/id/draft-davis-t-langtag-ext-07.txt)과 [2010년 U 확장 초안](https://www.ietf.org/archive/id/draft-davis-u-langtag-ext-00.txt)에 각각 바이트 단위로 일치합니다. 수집은 해시를 검증하고 저작권과 통지를 유지합니다. 두 문서는 발행 당시 유효한 조건을 참조합니다. [TLP 4.0](https://trustee.ietf.org/wp-content/uploads/IETF-TLP-4.pdf) 3.c절은 Trust가 권리를 부여할 수 있는 범위에서 필수 통지를 유지하고 전체를 수정 없이 표준화 과정 밖에서 배포할 수 있도록 허용합니다. 이 평가는 두 문서만 다루며 다른 HTML, XML, 에셋 또는 타사 권리에는 적용되지 않습니다.

이미 Qt 소스에 있는 언어는 원본 `qlocale_data_p.h`에서 제한된 생성기 입력을 추출할 수 있습니다. 예를 들어 QtBase를 현재 디렉터리에 추출한 후 `python3 scripts/derive-qt-locale-code-input.py --qtbase qtbase-everywhere-src-6.8.2 --output build/qt-locale-codes.tsv`를 실행합니다. 이는 기존 검색 키만 다루며 완전한 ISO 639-3 표나 과거 업스트림 입력이 아닙니다. 언어를 추가하려면 별도로 얻은 코드와 조건 검토가 여전히 필요할 수 있습니다. 고정 Qt 6.8.2 소스, CLDR 46 데이터와 이 파생 입력으로 8개 파일을 재생성한 결과는 생성 날짜와 줄바꿈 차이만 허용하여 확인했습니다. 이는 ISO 표 라이선스나 완전한 대응 소스를 확정하지 않습니다.

특정 추가 언어는 전달된 CLDR 43.1 부분 집합을 추출하고 `tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry`에서 언어 서브태그를 확인하여 코드 필드 4개를 명시적으로 제공합니다. Choctaw 예시의 ISO 639-2 필드는 [미국 의회도서관 목록](https://www.loc.gov/standards/iso639-2/php/code_list.php), ISO 639-3은 [등록 기관 데이터](https://iso639-3.sil.org/code_tables/download_tables)에서 확인할 수 있습니다. 마지막 빈 필드는 두 글자 코드가 없음을 나타냅니다.

```sh
python3 scripts/derive-qt-locale-code-input.py --qtbase path/to/qtbase \
  --iana path/to/cldr43/tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry \
  --add-language 'cho,cho,cho,' --output build/qt-locale-codes-with-cho.tsv
```

`--add-language`는 반복할 수 있습니다. 필드는 세 글자 코드, ISO 639-2/B, ISO 639-2/T와 두 글자 코드입니다. 없는 필드는 비워 두며 Part 2 필드는 둘 다 제공하거나 둘 다 비워야 합니다. 도구는 세 글자 또는 두 글자 코드 중 적어도 하나가 등록된 IANA 언어 서브태그인지 확인하고 구문을 검증하며 Qt 검색 키 충돌을 거부합니다. 필드가 같은 언어를 가리키는지 추론하거나 검증하지 않으며 ISO 대응도 확립하지 **않습니다**. 수신자는 각 대응과 출처를 검토해야 합니다. Qt 언어 추가에는 `enumdata.py` 업데이트, CLDR 로캘 데이터 제공 및 Qt 재생성과 재빌드도 필요하며 이 행만으로 언어가 추가되지 않습니다. 이는 제한된 파생 표일 뿐 완전한 ISO 표나 과거 입력 ID 또는 완전한 대응 소스의 증거가 아닙니다.

격리 테스트에서 Qt 도구 enum과 파생 표에 개인 사용 테스트 코드를 수동 추가하자 새 enum과 코드 항목이 생성되었습니다. 재빌드한 Linux QtCore는 정방향·역방향 검색을 통과했고 격리된 원본 alpha.8 복사본은 오프스크린 탐색, 취소와 압축 해제를 완료했습니다. 이는 실제 언어나 CLDR 로캘 형식 데이터를 추가하지 않았고 실제 입력과 조건 검토를 대체하지 않으며 대응 소스의 모든 의무를 입증하지 않습니다. 단계와 초기 프로브 실패는 [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6611169790)을 참조하세요.

다른 방법으로 `iso-codes` ISO 639-2와 639-3 JSON 데이터를 독립적으로 얻어 Qt 입력으로 변환할 수 있습니다. 예를 들어 데이터 패키지가 설치된 Linux에서 다음과 같이 합니다.

```sh
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 /usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 /usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-codes.tsv
```

변환기는 이름 없이 코드 대응을 추출하지만 완전한 입력은 완전한 코드 집합을 생성합니다. 잘못된 코드, 중복과 Qt 검색 키 충돌을 거부하고 ISO 639-2 예약 로컬 사용 범위를 건너뛰며 입출력 SHA-256을 보고합니다. Qt 6.8.2와 Ubuntu `iso-codes` 4.16.0-1 비교는 [#138](https://github.com/xiaodaozhi/xdzip/issues/138)을 참조하고 다른 버전은 별도 확인하세요. 데이터는 포함되지 않습니다. 생성된 표의 명시된 라이선스, 출처와 재배포 조건을 독립적으로 검토하세요. 대체 표는 Qt의 과거 입력이나 완전한 소스 준수를 입증하지 않습니다.

비교를 재현하려면 고정된 [Ubuntu `iso-codes` 4.16.0-1 패키지](https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb)를 독립적으로 얻습니다. 다음 Linux 예시는 `curl`, `dpkg-deb`와 Python 3이 필요하며 시스템 설치를 변경하지 않고 로컬 추출과 변환 전에 패키지 해시를 검증합니다.

```sh
mkdir -p build && mkdir build/qt-iso-peer &&
curl --fail --location --silent --show-error --connect-timeout 15 --max-time 120 https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb -o build/qt-iso-peer/iso-codes.deb &&
python3 -c "import hashlib,pathlib,sys; p=pathlib.Path('build/qt-iso-peer/iso-codes.deb'); h=hashlib.sha256(p.read_bytes()).hexdigest(); sys.exit(0 if h=='cafca0a3a1f07b44fbba53ab7a1f1d9755e6b80cc041173afe2b52e53bed97a7' else 'Package SHA-256 mismatch')" &&
dpkg-deb -x build/qt-iso-peer/iso-codes.deb build/qt-iso-peer/data &&
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-peer/qt-locale-codes.tsv
```

검증된 입력 SHA-256은 `fa83810fdb59f9d84b4d58486d5e5e48e807d82a98d6a39ef0ba4fc57c2a9327`과 `9636ce5266053867627140ce5ada1f9aa897ca07a7501302c1b14b8d1147cdda`이고 변환 출력은 `85457a36e013e053b70dea19d500ebec2ae6bedf5dad9af0035b973c8db6a60a`입니다. Ubuntu의 `usr/share/doc/iso-codes/copyright`는 LGPL-2.1-or-later를 선언하지만 그 패키지 수준 선언만으로 과거 SIL 표 허가나 완전한 코드 집합의 재배포 권리를 확립하지 않습니다. 패키지와 완전한 변환 표는 여기에 포함되지 않습니다. 출처, 조건과 후속 재배포를 별도 검토하세요.

또는 [IANA 언어 서브태그 레지스트리](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)와 [미국 의회도서관 ISO 639-2 목록](https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt)을 독립적으로 얻어 다른 입력을 생성합니다. 도구는 이름 없이 코드를 추출하고 일치하지 않는 코드와 입출력 해시를 보고하며 잘못되거나 모호한 레코드를 거부합니다. Linux/macOS 셸 예시는 새 디렉터리를 사용하고 변환 전에 이 비교의 원시 파일 두 개를 검증합니다.

```sh
(
  set -eu
  mkdir -p build
  mkdir build/qt-iana-loc-inputs
  curl --fail --location --output build/qt-iana-loc-inputs/iana.txt \
    https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
  curl --fail --location --output build/qt-iana-loc-inputs/loc.txt \
    https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt
  python3 -c 'import hashlib,pathlib,sys; p=sys.argv[1:]; sys.exit(0 if all(hashlib.sha256(pathlib.Path(p[i]).read_bytes()).hexdigest()==p[i+1] for i in (0,2)) else "Input SHA-256 mismatch")' \
    build/qt-iana-loc-inputs/iana.txt 755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1 \
    build/qt-iana-loc-inputs/loc.txt 42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371
  python3 scripts/convert-iana-loc-for-qt.py \
    --iana build/qt-iana-loc-inputs/iana.txt \
    --loc build/qt-iana-loc-inputs/loc.txt \
    --output build/qt-iana-loc-inputs/qt-codes.tsv
)
```

비교에는 날짜가 2026-09-17이고 SHA-256이 `755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1`인 IANA 파일을 사용했습니다. 2026-11-03에 직접 얻은 UTF-8 LoC 파일은 16,010바이트이며 SHA-256은 `42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371`입니다. 487개 레코드의 첫 세 코드 필드는 이전 웹 재구성과 각각 일치했습니다. 변환기의 LF 출력 SHA-256은 `6a75bebfa694258a1a4125fc30512d8ae7994080701377b41306db665ad371c6`이었습니다. 두 입력은 기존의 서로 다른 Qt 검색 키 346개 모두에 일치하는 8268개 행을 생성했습니다. 생성 날짜와 줄바꿈을 정규화한 뒤 재생성 파일 8개가 모두 일치했고 입력 하나를 변경한 음성 대조는 7개가 일치했습니다. 두 글자 IANA 서브태그 `bh`, `in`, `iw`, `ji`, `jw`, `mo`, `sh` 7개에는 LoC 레코드가 없어 도구가 보고합니다. 다른 버전과 추가 언어는 별도로 검증하세요. 업스트림 주소가 바뀔 수 있으므로 해시 불일치 시 검사를 우회하지 말고 조사하세요. [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588040592)을 참조하세요.

[IANA/IETF 선언](https://www.iana.org/help/licensing-terms)은 보유하거나 재허가할 수 있는 프로토콜 레지스트리 권리에 CC0를 적용하며 타사 권리 가능성을 유지합니다. 원본 SIL 표의 모든 권리를 자동 확립하지 않습니다. 2026-09-17 IANA 스냅샷, LoC 입력과 완전한 변환 비교 표는 여기에 포함되지 않습니다. CLDR 43.1 부분 집합은 위에 설명한 이전 레지스트리를 별도로 포함합니다. 수신자는 출처, 적용 조건과 후속 재배포 조건을 별도 검토해야 합니다. 이 대안은 Qt의 과거 입력이나 완전한 대응 소스 준수를 확립하지 않습니다.

여기서 독립 비교는 파일을 별도로 확보하고 변환한다는 뜻이며 표준 출처가 독립적이라는 뜻은 아닙니다. [RFC 5646 2.2.1절](https://www.rfc-editor.org/rfc/rfc5646#section-2.2.1)은 ISO 639-2, 639-3과 639-5를 IANA 세 글자 언어 서브태그의 표준 소스로 명시합니다. 값의 일치는 독립 저작이나 완전한 재허가를 입증하지 않습니다.

출력에는 실제 고정 커밋의 xdzip-source.tar, rust-vendor.tar.gz, 오프라인 Cargo 구성, 원래 Qt 아카이브, REBUILD.md와 파일별 SHA-256·크기 매니페스트가 포함됩니다. 매니페스트는 수집되었으나 준수 검증은 되지 않았다고 명시하며 SDK 빌드 기능·패치, 플랫폼 런타임, 내장 리소스 또는 라이선스 준수 검사를 대체하지 않습니다. 재구성된 프로젝트 소스는 ID가 알려지지 않을 수 있으므로 깨끗한 Git ID를 조작하지 않고 외부 전달 매니페스트를 통해 원래 커밋에 연결해야 합니다.

별도 `rar-wire-mappings/` 디렉터리는 제한된 유한 매핑 참조 출처, 설명과 정확한 원본 UnRAR 7.30 beta1 통지를 유지하며 모두 해시된 전달물에 포함됩니다. 고정 다운로드 파일 이름은 `unrarsrc-7.3.1.tar.gz`입니다. 숫자 파일 이름은 별도 업스트림 릴리스 버전을 입증하지 않습니다. 보수적인 통지 조치로 전달은 관련 Rust 소스 주석에 원래 라이선스의 두 번째 문단 전체를 요구하고 누락되거나 변경되면 중단합니다. RAR 크레이트는 보수적으로 `Apache-2.0 AND UnRAR`를 선언합니다. OSI 승인을 받지 않은 `UnRAR` ID는 SPDX 3.28 이상에 나타나며 오래된 도구는 인식하지 못할 수 있습니다. 이 자료에는 공식 엔진 소스가 없고 참조에서 파생된 사실을 Apache-2.0으로 재라이선스하지 않습니다. 적용 조건과 실제 재배포 의무는 후보별 검토가 필요합니다.

Linux ICU를 위해 고정된 완전한 ICU 73.2 라이선스, 공식 ICU4C 73.2 릴리스 소스 아카이브, 고정 편집 가능한 ICU 소스, CLDR 43.1 소스 부분 집합과 ICU 73.1 부트스트랩 소스를 포함하고 전체 매니페스트에 넣습니다. CLDR 부분 집합은 ISO 639-3 파일 5개, UTS #35 텍스트와 `docs/ldml` 아래 관련 파일, 문서 스크린샷 2개를 제외하고 웹 도구 이미지 963개를 교체합니다. 전달 기록은 업스트림 아카이브 해시와 각 제외·교체를 유지합니다. [Unicode가 게시한 UTS #35 버전 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)은 소유자의 조건에 따라 별도 제공되며 양도 가능한 추가 허가가 있는지는 아직 확립되어야 합니다. 수집기는 이름에 `iso-639-3`이 있는 부트스트랩 아카이브 파일도 거부하지만 파일 이름 검사만으로 다른 형태의 데이터를 배제할 수 없습니다. 수집은 타사 경로를 추출하지 않고 아카이브 해시, 버전, 내장 라이선스와 선택된 소스 파일을 확인합니다. [ICU 데이터 재빌드 가이드](icu-cldr-rebuild.md)는 도구 빌드와 생성·편집 범위를 기록합니다. 업스트림 버전이 일치해도 공급자가 패치 없이 이 입력으로 바이너리를 빌드했다는 증거는 아닙니다. 현재 Windows 개발 패키지는 소프트웨어 OpenGL을 제외하고 전용 Mesa/LLVM 자료를 더 이상 수집하지 않으며 과거 전달물은 원래 자료와 의무를 유지합니다. [네이티브 런타임 소스와 통지](native-runtime-materials.md)를 참조하세요.

앱은 Qt를 동적 링크하고 소스 재빌드와 호환 라이브러리 교체 지침을 제공합니다. [macOS 배포 검증](macos-deployment.md)은 Qt 소스 전체 재빌드를 주장하지 않고 격리 복사본에서 수정된 라이브러리 로딩을 다룹니다. 실제 Qt 소스 빌드와 제품 로딩은 [자동 검증 계약](../../../developers/engineering/candidate-verification.md)의 별도 과정입니다. SDK 바이트 단위 빌드 동등성, 정확한 정적 링크 출처와 라이선스 선택은 완료된 WP-12 개발 작업을 새로 재개하는 것이 아니라 [릴리스 라이선스 검토](https://github.com/xiaodaozhi/xdzip/issues/4)에 속합니다.

verify-source-delivery.py는 먼저 전달 파일 해시를 다시 검증하고 프로젝트 및 vendor 자체 입력을 새 build/ 디렉터리에 안전하게 추출하며 빈 Cargo 캐시, CARGO_NET_OFFLINE=true와 기존 고정 Qt SDK로 실제 앱을 빌드한 뒤 시작·종료 스모크 테스트를 실행합니다. 이는 프로젝트와 Rust 입력의 오프라인 사용을 검증하며 Qt 모듈 소스 빌드나 바이트 단위 재현 가능한 빌드는 검증하지 않습니다. 생성된 소스 디렉터리의 Git ID는 알 수 없으며 고정 커밋과 전달 매니페스트를 통해 외부에서 연결합니다.

소스 전달은 출력 디렉터리에서 상대 `vendor` 대상과 명시적 프로젝트 매니페스트로 Cargo를 호출합니다. 현재 지원하는 crates.io 소스 매핑과 상대 `vendor` 구성만 수락합니다. 절대 디렉터리, 경로 이탈 또는 예상하지 못한 매핑은 이동 가능한 전달물에 빌드 머신 경로를 포함하지 않고 실패합니다.

기존의 일반적인 이름을 가진 독립 라이선스 또는 통지 파일이 없는 고정 vendor 패키지는 `rust-vendor-notices/`가 패키지별 고정 업스트림 원문을 별도로 색인합니다. 수집기는 패키지 Cargo 매니페스트 해시, 업스트림 Git 커밋과 경로, 라이선스 텍스트 SHA-256 및 Git blob을 검증하며 알려지지 않은 누락이나 ID 불일치 시 전달을 중단합니다. 이 디렉터리는 Cargo vendor 파일이나 체크섬을 수정하지 않습니다. 업스트림 라이선스 표현은 변경되지 않으며 특정 라이선스 선택이나 이전 릴리스 준수를 승인하지 않습니다.

[Qt SDK 대응성 검사](qt-sdk-correspondence.md)는 고정 대응성 매니페스트와 재실행 가능한 AGL 소비자 패치를 항상 포함합니다. --qt-prefix를 지정하면 해당 SDK의 파일별 소스 선언 검사, 빌드 구성과 명시된 누락도 포함합니다. 전달 매니페스트는 패치 하위 디렉터리를 재귀적으로 해시하고 재빌드 검증에서 해당 바이트를 다시 확인합니다. SDK가 없으면 구성 검증 기록을 조작하지 않습니다.

전달물은 릴리스 커밋과 아카이브의 `.tag` 트리 객체의 관계를 오프라인으로 검증하기 위해 고정된 Qt Declarative Git 커밋 객체를 유지하며 전달 매니페스트가 이를 해시합니다. 이 관계는 아카이브 전체 범위나 SDK 빌드 동등성을 입증하지 않습니다.

실제 Qt 소스 빌드와 로딩·호환 교체는 [자동 완료 계약](../../../developers/engineering/candidate-verification.md)을 참조하세요. 기존 SDK를 사용한 오프라인 앱 재빌드를 Qt 자체를 소스에서 재빌드한 것으로 설명해서는 안 됩니다.
