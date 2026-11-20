<a name="xdzip-developer-preparation-guide"></a>

# 개발 환경 준비

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | [繁體中文（台灣）](../../zh-TW/engineering/development.md) | [日本語](../../ja-JP/engineering/development.md) | 한국어 | [Français](../../fr-FR/engineering/development.md) | [Deutsch](../../de-DE/engineering/development.md) | [Español](../../es-ES/engineering/development.md) | [Português (Brasil)](../../pt-BR/engineering/development.md) | [Русский](../../ru-RU/engineering/development.md)

[빌드 및 기여](../contributing/README.md)에서 변경에 맞는 빌드와 검증 방법을 선택하세요. [시스템 아키텍처](../design/system-architecture.md)와 [저장소 구조](repository-layout.md)는 Qt, Rust 실행 환경 및 코덱의 경계를 설명합니다.

<a name="xdzip-obtain-and-modify"></a>

## 소스 가져오기 및 변경

최신 `main`에서 시작하고 작업 디렉터리의 다른 변경을 유지하세요. 변경 범위와 검토는 [기여 방침](../../../CONTRIBUTING.md), 승인된 통합은 [저장소 제공 안내](../../../developers/engineering/repository-hygiene.md)를 따릅니다. 계약을 변경하면 모든 해당 언어판과 영향을 받는 검사 사례를 갱신하세요. 색상과 크기는 `specs/visual`을 사용합니다.

<a name="xdzip-documentation-builds"></a>

## 문서 생성과 검사

지원하는 인터페이스 언어마다 루트 README와 docs의 번역 문서를 유지하세요. 기본 영어는 `README.md`와 `docs/`를 쓰고 번역은 `README.<locale>.md`와 `docs/<locale>/`를 사용합니다. 유지보수자 안내서는 미국 영어와 중국어 간체를 사용합니다. 언어 선택 줄은 제목 바로 아래에 둡니다. 현재 언어는 일반 텍스트로 표시하고 다른 언어는 같은 주제의 문서로 연결합니다. 중국어 간체 문서에서는 간체 유지보수자 페이지에, 그 외에는 기본 영어 유지보수자 페이지에 연결합니다. 문서를 추가하거나 옮길 때는 인덱스, 해당 문서를 가리키는 링크 및 절 앵커를 갱신하세요.

문서 도구에는 Python 3.10+, lxml, Pandoc 및 Graphviz가 필요합니다. 시스템 CJK 글꼴을 권장합니다. `python3 scripts/build-design-html.py`는 `docs/zh-CN/design/`의 중국어 소스를 사용해 세부 설계 문서 16편과 한 파일 판을 build/docs-html 아래에 생성합니다. 다운로드 번들은 dist/ 아래에 기록합니다. 다른 기술 및 협업 문서는 docs/README.md에서 찾아보세요. HTML은 문서 출력 형식입니다. 제품 인터페이스는 Qt Quick을 사용합니다.

`python3 scripts/check-docs.py`는 번역 범위, 양방향 언어 링크, 본문 링크의 대상 언어, Markdown 앵커, JSON 및 디자인 리소스를 오프라인으로 검사합니다. HTML 생성기는 출력 페이지 링크와 구조를 검증합니다. 검사기 변경 후 `python3 scripts/test-check-docs.py`를 실행하세요.

<a name="xdzip-product-toolchain-preparation"></a>

## 제품 도구 체인 준비

macOS에는 Xcode Command Line Tools와 Qt가 필요합니다. Windows에는 MSVC Build Tools, Windows SDK 및 호환 Qt가 필요합니다. Linux에는 컴파일러, 대상 glibc 환경 및 Qt의 Wayland/X11 의존성이 필요합니다. 루트 도구 체인 파일과 CI가 버전을 고정합니다. 소스와 라이선스는 [도구 체인 표](toolchain.md)에 기록하세요. 지원 운영체제와 유지보수 범위는 [지원 정책](support-policy.md)을 참조하세요. 개별 출처와 라이선스는 해당 릴리스 검토 이슈에서 다룹니다.

[Qt 개발 앱](../../../apps/desktop/README.md)은 깨끗한 체크아웃에서 빌드하고 실행하는 명령, CMake 옵션, macOS 아키텍처 및 배포 대상, 오프스크린 테스트 범위를 설명합니다. 빌드 또는 설치가 바뀌면 공백과 중국어 문자가 있는 경로를 다시 확인하세요. 로컬 개발은 유료 서명 ID를 요구하지 않습니다. 로컬 DMG의 ad-hoc 무결성 서명과 Mac 앱을 배포하지 않는 CI의 범위는 [빌드 안내](developer-build.md)를 참조하세요.

<a name="xdzip-independent-engineering-probes"></a>

## 독립 기술 검사 도구

[도구 체인 프로브](../../../tools/toolchain-probe/README.md)는 독립 Qt Quick 창의 최소 빌드와 CTest 명령을 제공합니다. README에서 버전과 검사 범위를 확인하세요. 결과는 [초기 검증 이슈](https://github.com/xiaodaozhi/xdzip/issues/1)에 있습니다.

[통합 CMake/Cargo 브리지 프로브](../../../tools/bridge-probe/README.md)는 루트 빌드 진입점, 고정된 의존성 및 수명 주기 검사를 제공합니다. 의존성 기록은 [cxx 선택 이유](cxx-admission.md)를 참조하세요.
