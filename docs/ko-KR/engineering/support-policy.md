<a name="xdzip-system-support-and-maintenance-policy"></a>
# 시스템 지원 및 유지보수 정책

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | [繁體中文](../../zh-TW/engineering/support-policy.md) | [日本語](../../ja-JP/engineering/support-policy.md) | 한국어 | [Français](../../fr-FR/engineering/support-policy.md) | [Deutsch](../../de-DE/engineering/support-policy.md) | [Español](../../es-ES/engineering/support-policy.md) | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

프로젝트 소유자는 2026-10-19에 초기 지원 정책을 승인했으며, `v0.1.0`은 2026-11-06에 공개되었습니다. 최신 버전과 다운로드는 [릴리스 페이지](https://github.com/xiaodaozhi/xdzip/releases)를 참조하세요. 이 정책은 지원 범위를 정합니다. 최소 버전 확인은 그대로 적용됩니다. CI와 사용자 인수 검증 결과는 실제로 테스트한 환경에 적용됩니다. 수동 검토는 사용자의 결정에 따라 완료되었습니다.

<a name="xdzip-initial-support-scope"></a>
## 초기 지원 범위

| 플랫폼 | 초기 지원 범위 | 검증과 제한 |
| --- | --- | --- |
| macOS | macOS 14 이상, Apple Silicon arm64 | 배포 대상 14.0, 네이티브 CI, 로컬 개발용 DMG. Intel 및 Universal 패키지는 제공하지 않으며 이후 모든 시스템 버전을 보장하지 않습니다. |
| Windows | Microsoft에서 계속 유지보수하는 Windows 11 x86_64 버전 | MSVC 빌드, 사용자별 설치 프로그램, CI에서 실제 설치, 업그레이드, 롤백, 제거 검증. Windows Server CI가 모든 Windows 11 버전을 검증하지는 않습니다. ARM64 패키지는 제공하지 않습니다. |
| Linux | Ubuntu 24.04 LTS x86_64, X11, glibc 2.39 이상 | 현재 .deb 패키지와 Qt SDK 없는 격리된 실행 환경을 검증합니다. 다른 배포판, 이전 glibc, Wayland, ARM64는 대상이 아닙니다. |

최소 macOS 버전은 현재 배포 대상을 따릅니다. 예전 설계에 적힌 버전 13은 검증되지 않았습니다. Windows 10의 일반 지원은 2025-10-14에 종료되었으며 Windows 10은 초기 지원 대상에 포함되지 않습니다. LTSC와 ESU도 자동으로 지원 대상에 포함되지 않습니다. [Microsoft의 제품 수명 주기 발표](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support)를 참조하세요. [Qt 6.8 지원 플랫폼](https://doc.qt.io/qt-6.8/supported-platforms.html) 목록은 종속 라이브러리의 지원 범위입니다. xdzip의 동작은 별도로 검증합니다.

<a name="xdzip-maintenance"></a>
## 유지보수

이 개인 프로젝트는 가용한 자원 내에서 유지보수됩니다. 유료 서비스, 정해진 응답 기한, 실제로 수행한 범위를 넘어서는 보안 약속은 제공하지 않습니다. 최신 공개 안정 버전이 우선이며 개발 미리보기에서는 호환성이 달라질 수 있습니다. 릴리스 노트에는 알려진 문제와 이전 버전으로 돌아가는 방법을 기록합니다. 보안 문제가 발견되면 관련 다운로드가 철회될 수 있습니다. 사용자는 ‘xdzip 정보’에서 업데이트를 확인하고 설치를 시작할 수 있습니다. 백그라운드 확인이나 무인 설치는 수행하지 않습니다. 현재 소스에는 CLI 업데이트 명령도 포함됩니다. [명령줄 계약](command-line.md)에는 지원되는 설치 유형과 이미 공개된 배포 파일과의 차이가 설명되어 있습니다.

검증을 재현할 수 있도록 후보 빌드는 Qt 6.8.2와 기존의 고정된 Rust 및 Cargo 버전을 사용합니다. 이전 패치에 취약점이 없다거나 무료 LTS 업데이트가 영구 제공된다는 보장은 없습니다. 공개하기 전에 유지보수 담당자는 Qt 공식 보안 권고와 사용 가능한 오픈 소스 업데이트를 확인합니다. 유지보수 중인 공개 버전을 우선 검토합니다. 종속 항목에 패치가 없는 보안 문제가 있고 xdzip에 영향을 주는 경우 해당 릴리스를 보류합니다. Qt를 업그레이드할 때는 소스, 모듈 라이선스, Windows 비공개 인터페이스, 동적 교체, 세 플랫폼 빌드, 실제 메뉴와 창 동작을 다시 검증합니다. 상용 유지보수 구매는 선택 사항입니다.

<a name="xdzip-approved-scope-and-changes"></a>
## 승인 범위와 변경

승인 범위는 초기 플랫폼, 가용한 자원 내에서 최신 공개 버전만 유지보수하는 것, 응답 기한을 정하지 않는 것, 개발 빌드를 실험용으로 표시하는 것입니다. `v0.1.0` 결과는 릴리스와 당시 검증 이슈에 기록되어 있습니다. 이 결과가 모든 시스템 버전에서의 성능을 보장하지는 않습니다. 아키텍처, 시스템, 배포 경로를 추가하려면 구현과 검증을 거친 뒤 이 정책을 갱신합니다. [ADR-011](../design/architecture-decisions.md#xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy)을 참조하세요.

공식 서명, 공증, 유료 계정은 더 이상 요구 사항이 아닙니다. 취약점 신고 방법은 [보안 정책](../../../SECURITY.md)을 참조하세요.
