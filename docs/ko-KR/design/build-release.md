<a name="xdzip-build-deployment-and-release-architecture"></a>

# 빌드, 배포 및 릴리스 아키텍처

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | [繁體中文](../../zh-TW/design/build-release.md) | [日本語](../../ja-JP/design/build-release.md) | 한국어 | [Français](../../fr-FR/design/build-release.md) | [Deutsch](../../de-DE/design/build-release.md) | [Español](../../es-ES/design/build-release.md) | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-build-and-package-inputs"></a>

## 빌드 및 패키징 입력

CMake는 Qt 데스크톱 앱, C++ 어댑터와 네이티브 보조 프로그램을 빌드합니다. Cargo는 Rust 런타임을 빌드하고 앱에 링크할 CXX 브리지를 생성합니다. 독립 실행형 CLI는 별도의 빌드 진입점이 있으며 Qt가 필요하지 않습니다. `scripts/desktop.py`와 `scripts/cli.py`가 빌드 및 테스트 단계를 조정합니다. 소스는 Rust 1.99.0과 Qt 6.8.2 SDK를 고정합니다. 결과를 비교할 때는 빌드 식별 정보, 대상 아키텍처, 잠금 파일과 도구 모음의 출처를 함께 확인합니다. [도구 모음](../engineering/toolchain.md), [개발 빌드 가이드](../../engineering/developer-build.md)

데스크톱 패키지는 필요한 Qt 라이브러리, QML 모듈, 플랫폼 리소스, 번들 보조 프로그램을 배포합니다. 내부 이미지 처리 프로세스는 이미지 디코딩과 정적 SVG 표시에 Qt를 씁니다. CLI 배포본에는 필요한 압축 파일 처리 프로세스가 포함됩니다. 빌드 프로필, 패키지에 허용된 파일 목록과 배포 검사는 스크립트와 플랫폼 계약에서 관리합니다. 설치되는 종속 구성 요소와 전체 크기는 패키지 전체를 확인해야 알 수 있습니다. [배포 및 종속성 감사](../engineering/deployment-audit.md), [CLI 배포](../../engineering/command-line.md)

<a name="xdzip-current-distribution-channels"></a>

## 현재 배포 채널

| 대상 | 패키지 및 신뢰 범위 |
| --- | --- |
| macOS Apple Silicon | GitHub에서 DMG와 앱 번들이 담긴 ZIP을 제공합니다. 기준 버전은 macOS 14입니다. 패키지는 무결성 확인에 애드혹 서명을 사용합니다. 설치와 업데이트는 해당 플랫폼 안내를 참고하세요. |
| Windows 11 x86_64 | GitHub에서 서명되지 않은 사용자별 설치 프로그램과 포터블 ZIP을 제공합니다. Microsoft Store 버전의 MSIX ID, 서명과 업데이트 채널은 Microsoft Store에서 관리합니다. |
| Ubuntu 24.04 LTS x86_64 | GitHub에서 실행 디렉터리를 압축한 파일과 DEB를 제공합니다. 지원 기준은 X11과 glibc 2.39 이상입니다. |
| 독립 실행형 CLI | 패키지와 업데이트에 고유한 식별 정보를 사용합니다. 데스크톱 앱에 포함된 CLI는 해당 패키지의 통합 및 인계 규칙을 따릅니다. |

[지원 정책](../engineering/support-policy.md)에 지원하는 운영 체제와 아키텍처가 나와 있습니다. 실제 배포 파일은 [프로젝트 README](../../../README.ko-KR.md)와 해당 릴리스 노트를 참고하세요. 연결 및 통합 제한은 [설치 프로그램 가이드](../../engineering/installers-associations.md)에 설명되어 있습니다.

<a name="xdzip-routine-checks-native-builds-and-release"></a>

## 일반 검사, 네이티브 빌드 및 릴리스

일반 [CI 워크플로](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml)는 간단한 검사를 실행합니다. Foundation과 Qt 네이티브 작업은 각각 별도의 수동 워크플로에서 실행합니다. 일반 Apple Silicon CI는 빌드, 테스트 및 시작 검사를 수행하며 다운로드용 앱은 배포하지 않습니다. 전용 릴리스 워크플로에는 승인된 배포, 서명, 패키징 및 게시 단계가 있습니다. [릴리스 절차](../engineering/release-protocol.md)와 [CI 정책](../../../developers/engineering/ci-policy.md)은 통과 조건과 증빙을 정의합니다.

릴리스 매니페스트, 체크섬, 구성 요소 목록, SBOM과 라이선스 자료는 제공하는 결과물을 설명합니다. 릴리스 워크플로는 프로젝트 소스 ZIP을 생성하거나 업로드하지 않습니다. GitHub의 태그 스냅샷은 별도 방식으로 제공됩니다. 실제로 제공하는 구성 요소와 [라이선스 기록](../engineering/licensing.md), [소스 제공 정책](../../engineering/source-delivery.md)에 따라 의무를 확인합니다.

<a name="xdzip-update-and-future-distribution"></a>

## 업데이트 및 향후 배포

업데이트 선택과 검증은 압축 파일 작업과 별개입니다. 업데이트 가능한 설치본은 신뢰 메타데이터, 검증된 배포 파일, 설치 식별 정보와 실행 중인 작업의 인계에 관한 업데이트 계약을 따릅니다. Windows와 Linux는 해당 보조 프로그램을 사용하고 macOS는 Sparkle을 사용합니다. Microsoft Store에서 Store 버전을 업데이트합니다. 포터블 및 개발 구성에는 별도의 제한이 있습니다. [소프트웨어 업데이트](../../engineering/software-update.md)

다른 아키텍처, 운영 체제 기준 버전, 패키지 형식이나 스토어로 확장하려면 별도의 빌드 및 인수 검증 결과가 필요합니다. 이전 배포 결정과 재검토 조건은 [아키텍처 결정 기록](architecture-decisions.md)을 참고하세요.
