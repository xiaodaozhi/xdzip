<a name="xdzip-xdzip-design-guide"></a>

# XDZip 설계 안내

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | [繁體中文（台灣）](../../zh-TW/design/index.md) | [日本語](../../ja-JP/design/index.md) | 한국어 | [Français](../../fr-FR/design/index.md) | [Deutsch](../../de-DE/design/index.md) | [Español](../../es-ES/design/index.md) | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

[시스템 아키텍처](system-architecture.md)에서 Qt Quick 데스크톱과 독립 Rust CLI가 압축 실행 환경에 접근하는 방식을 볼 수 있습니다. 설계 문서는 현재 계약, 결정 사항 및 제안의 범위를 설명합니다. 설치와 일상적인 사용은 [사용자 설명서](../user-guide/README.md)를 참고하세요. 프로젝트를 빌드하거나 수정하려면 [빌드 및 기여](../contributing/README.md)를 보세요.

<a name="xdzip-find-a-design-topic"></a>

## 설계 주제 찾기

| 주제 | 읽을 문서 |
| --- | --- |
| 제품 동작 | [제품 요구 사항](product-requirements.md)은 예상 작업과 범위를 정의합니다. [요구 사항 추적](requirements-traceability.md)에서 해당 검증을 찾을 수 있습니다. |
| 압축 파일 데이터 | [도메인 모델](domain-data.md)은 세션, 소스 버전, 물리적 항목, 쿼리 및 작업 입력을 설명합니다. |
| 데스크톱 | [Qt Quick 인터페이스](qt-quick-ui.md)는 화면, 컨트롤러 소유권 및 상호 작용을 설명합니다. [Qt/Rust 브리지](qt-rust-bridge.md)는 실제 명령, ID, 이벤트 및 수명 주기를 다룹니다. |
| 형식 및 확장 범위 | [형식 어댑터와 확장 시험](codec-extension-sdk.md)은 내장 레지스트리, 함께 제공되는 작업 프로세스, 독립 XDP1 시험을 설명합니다. |
| 출력과 백그라운드 작업 | [파일 권한과 안전한 최종 저장](file-security.md)은 임시 출력, 검증 및 커밋을 다룹니다. [작업 스케줄링](task-scheduling.md)은 허용 조건, 안내, 진행 및 취소를 설명합니다. |
| 성능 및 품질 | [성능 설계](performance.md)는 자원 한도, 측정치 및 목표를 설명합니다. [품질 엔지니어링](quality-engineering.md)은 변경에 맞는 검증을 알려 줍니다. |
| 제공 및 결정 | [빌드, 배포 및 릴리스](build-release.md)는 플랫폼, 패키지, 업데이트를 설명합니다. [아키텍처 결정](architecture-decisions.md)은 근거를 기록합니다. [아키텍처의 변화](architecture-evolution.md)는 프로토타입에서 현재 제품에 이른 과정을 설명합니다. |
| 용어 | [용어 및 구현 제한](glossary.md)은 용어를 설명하고 각 하위 시스템의 현재 한도로 연결합니다. |

<a name="xdzip-interpreting-the-references"></a>

## 참조 문서 읽기

현재 동작은 구현 링크와 함께 설명합니다. 설계 옵션과 실험 도구에는 각자의 범위가 표시됩니다. [기술 참조](../reference/README.md)에서 개별 계약을 찾아볼 수 있습니다. [프로젝트 README](../../../README.ko-KR.md)와 해당 릴리스 노트는 지원하는 배포와 제공 기능을 설명합니다. 시간, 리소스 및 네이티브 검증 결과는 기록된 빌드, 플랫폼 및 작업 부하에 적용됩니다.
