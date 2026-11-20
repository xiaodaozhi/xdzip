<a name="xdzip-qt-sdk-and-source-correspondence-verification"></a>

# Qt SDK와 소스 대응 관계 검증

[English](../../engineering/qt-sdk-correspondence.md) | [简体中文](../../zh-CN/engineering/qt-sdk-correspondence.md) | [繁體中文](../../zh-TW/engineering/qt-sdk-correspondence.md) | [日本語](../../ja-JP/engineering/qt-sdk-correspondence.md) | 한국어 | [Français](../../fr-FR/engineering/qt-sdk-correspondence.md) | [Deutsch](../../de-DE/engineering/qt-sdk-correspondence.md) | [Español](../../es-ES/engineering/qt-sdk-correspondence.md) | [Português (Brasil)](../../pt-BR/engineering/qt-sdk-correspondence.md) | [Русский](../../ru-RU/engineering/qt-sdk-correspondence.md)

소스 대응 관계 검증에는 공식 아카이브의 실제 바이트를 사용합니다. 버전 번호가 같다고 해서 SDK 빌드의 동등성이 입증되지는 않습니다. `scripts/qt_sdk_correspondence.py`는 고정된 소스 아카이브, SDK가 제공하는 소스 SPDX, 빌드 선언, 설치된 구성과 알려진 사용 측 패치를 연결하고 결과를 build/에 기록합니다. SDK를 수정하거나 라이선스 선택지를 승인하지 않습니다.

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

기본적으로 build/source-cache를 오프라인으로 읽고 없으면 실패합니다. --fetch를 명시한 경우에만 [고정 매니페스트](../../../scripts/corresponding-sources.json)의 공식 아카이브를 다운로드하고 검증합니다. 필요할 때 실행하는 세 플랫폼 Qt 워크플로는 고정 SDK를 설치한 뒤 --fetch를 실행하고 JSON을 다른 검증 기록과 함께 보관합니다. Mac에서는 여전히 기록만 업로드하며 앱을 배포, 서명 또는 패키징하지 않습니다. 소스 전달 수집기가 --qt-prefix를 받으면 이 기록, 패치와 고정 매니페스트를 전달 해시에 포함합니다. [소스 전달](../../engineering/source-delivery.md)을 참고하세요.

<a name="xdzip-verification-layers"></a>

## 검증 단계

1. 아카이브 SHA-256은 고정값과 일치해야 하며 아카이브 루트, 멤버 경로와 고정된 `.tag` 마커도 일치해야 합니다. 마커의 Git 객체 유형은 별도로 기록합니다. 트리 마커라면 먼저 고정된 원본 커밋 객체의 해시를 다시 계산하고 tree 필드를 검증합니다. 상위 프로젝트 소스를 압축 해제하지 않고 파일을 개별적으로 읽습니다.
2. SDK가 모듈의 .source.spdx를 제공하면 선언된 각 SHA-1을 아카이브 내용과 비교합니다. 일치 수, 차이가 있는 파일의 양쪽 해시, SDK가 선언했지만 아카이브에 없는 파일과 아카이브에서 다루지 않은 파일 수를 기록합니다. SHA-1은 상위 프로젝트의 원래 선언만 확인하며 전체 아카이브는 별도의 고정 SHA-256으로 연결합니다. 누락된 선언, 차이와 미포함 파일을 통과로 채워 넣지 않습니다.
3. SDK 모듈 빌드 SPDX 해시, 컴파일러 선언과 configure 인자를 보관합니다. 설치된 modules/*.json, qconfig*.pri와 QT_FEATURE 구성 헤더도 읽어 내용과 해시를 보관합니다. 출력에서는 상위 빌더의 절대 디렉터리를 자리 표시자로 바꾸며 파일 해시는 원본 증거를 연결합니다. 이 구성은 전체 CMake 캐시, 빌드 환경 또는 도구 체인 자료가 아닙니다. SDK 기본 아키텍처 필드는 실제 라이브러리의 모든 아키텍처를 나타내지 않으므로 범용 라이브러리를 x86_64로 잘못 분류하는 데 사용할 수 없습니다.
4. [AGL 패치](../../../scripts/qt-patches/macos-no-agl.patch)는 이 프로젝트가 Qt를 사용할 때 오래된 AGL에 연결하지 않도록 SDK의 lib/cmake/Qt6/FindWrapOpenGL.cmake만 변경합니다. 도구는 고정된 QtBase 아카이브에서 원본을 읽고 통합 diff를 재적용하며 패치, 원본과 결과의 SHA-256을 검증한 뒤 실제 SDK 파일을 확인합니다. 원본 또는 알려진 패치 버전만 허용합니다. Windows SDK의 CRLF는 줄 끝을 정규화한 후에만 비교하고 실제 바이트 SHA-256, LF SHA-256과 변환 표시를 보관합니다. 그 밖의 알 수 없는 변경은 실패합니다. 이는 Qt 런타임 기계어를 변경하지 않으며 Qt 재빌드를 입증하지 않습니다. 전체 SDK 수정 이력은 여전히 검증해야 합니다.

소스 대응 기록은 스키마 2를 사용하며 고정 archive_commit, 실제 아카이브 archive_marker, archive_marker_type과 commit_tree_verified를 각각 보관합니다. Qt Declarative 6.8.2의 `.tag`는 Git 트리 `0a8139c574fe150aba92d5f2484a8f3fdb774977`이며 릴리스 태그는 커밋 `75534f3e7fff24ed7ccb364e2ed9950a73da879f`에 대응합니다. 저장소에 고정된 base64 원본 커밋 객체로 커밋 해시를 오프라인에서 다시 계산하고 tree 필드를 검증할 수 있습니다. 현재 고정된 다른 모듈 마커는 커밋입니다. 트리 관계는 이 두 Git 객체 사이의 관계만 입증하며 아카이브가 전체 트리를 포함하거나 SDK 빌드가 동등함을 입증하지 않습니다.

qt-license-sources.json의 source_commits는 라이선스 텍스트를 가져오기 위해 고정한 커밋만 식별하며 아카이브 마커와 릴리스 커밋과 별도로 저장합니다. SDK 빌드 커밋으로 표현할 수 없습니다. 과거 스키마 1 자료는 Qt Declarative 트리 마커를 archive_commit에 넣었으며 게시된 자산은 변경하지 않습니다. 이후 설명은 객체 유형을 바로잡고 원래 SHA-256으로 라이선스 텍스트를 계속 검증해야 합니다.

<a name="xdzip-interpreting-results-and-release-boundaries"></a>

## 결과 해석과 릴리스 경계

QtNetwork 보안 파생물은 `network_security`를 별도로 기록하며, 증빙 기록은 패치, 빌더, 원본 QtCore와 실제 QtNetwork 다이제스트, 기능 및 실제 전송 검증을 연결합니다. 도구는 증빙 기록을 읽을 때 실제 SDK 바이트를 다시 검사합니다. 이 증거는 수정된 라이브러리에만 적용되며 상위 SPDX를 파생물의 증거로 다시 쓰거나 다른 모듈의 미포함 사실을 바꾸지 않습니다. 입력과 유지 관리는 [Qt 패치 노트](../../../patches/qt/README.md)를 참고하세요.

모듈의 소스 SPDX 제공 여부, 일치, 누락 및 미포함 수, 구성과 패치 상태는 현재 실행에서 생성한 JSON을 따릅니다. 한 SDK의 관찰 결과를 다른 SDK에 일반화하지 마세요. 파일별 선언이 없으면 대응 관계가 통과했다고 추론하지 말고 구체적인 빈틈을 보존하세요.

보고서 상태는 항상 evidence-collected-not-build-equivalence입니다. [실제 Qt 소스 빌드와 로드 또는 교체](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)는 별도 절차이지만 원래 SDK의 바이트 단위 재현을 입증하지 않습니다. 전체 도구 체인 및 시스템 SDK 자료, 미포함 또는 생성 파일, 런타임 출처와 라이선스 이행이 모두 명확해질 때까지 대응 소스 의무를 전부 충족했다거나 바이트 단위로 빌드를 재현할 수 있다고 주장하지 마세요. 기존 [런타임 귀속](runtime-attribution.md)의 미완료 상태, 라이선스 선택과 플랫폼별 런타임 검토는 유지됩니다.
