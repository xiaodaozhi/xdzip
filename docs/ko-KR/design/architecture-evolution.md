<a name="xdzip-architecture-evolution-and-current-boundaries"></a>

# 아키텍처의 변화와 현재 범위

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | [繁體中文](../../zh-TW/design/architecture-evolution.md) | [日本語](../../ja-JP/design/architecture-evolution.md) | 한국어 | [Français](../../fr-FR/design/architecture-evolution.md) | [Deutsch](../../de-DE/design/architecture-evolution.md) | [Español](../../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-from-framework-experiments-to-the-product"></a>

## 프레임워크 실험에서 제품까지

초기 프레임워크 실험은 가상의 압축 파일 목록과 모의 작업을 사용했습니다. 제품은 Qt Quick과 타입이 지정된 CXX 브리지를 채택한 뒤 실제 압축 파일, 작업, 파일 처리를 Rust로 구현했습니다. 프로토타입에서 사용한 JSON, C ABI 및 시간 측정값은 당시 실험에서 나온 것입니다. [아키텍처 결정 기록](architecture-decisions.md)은 선택 이유와 재검토 조건을 설명합니다.

제품은 행 모델의 범위를 제한하고 쿼리 식별자를 검사해 오래된 결과가 새 화면을 덮어쓰지 않게 합니다. 물리적 항목 식별자, 명시적 형식 기능, 파일 권한, 임시 출력 및 검증 후 최종 저장을 구현합니다. 독립 CLI는 Qt 없이 같은 코어를 호출합니다. [시스템 아키텍처](system-architecture.md)와 [도메인 모델](domain-data.md)을 참조하세요.

<a name="xdzip-current-experimental-and-proposed-work"></a>

## 현재 기능, 실험 및 제안

| 영역 | 현재 상태 |
| --- | --- |
| 실행 환경 및 인덱싱 | 호스트에서 구현합니다. 일반 단일 파일 ZIP/ZIP64는 비공개 디스크 인덱스로 전환할 수 있습니다. 작업 한도와 메타데이터 한도는 별개입니다. [성능 설계](performance.md) |
| 형식 격리 | 내장 형식 라우팅은 프로세스 내부 Rust 또는 함께 제공되는 형식 작업 프로세스를 선택합니다. 레지스트리와 프로필 및 실행 시 허용 조건으로 지원 작업을 결정합니다. [형식 어댑터](codec-extension-sdk.md) |
| 외부 확장 | 독립 읽기 전용 XDP1 시험은 제한된 프로토콜과 플랫폼별 조건을 갖추고 별도의 호스트를 사용합니다. 제품 내 확장의 검색, 설치 및 활성화는 제안 단계입니다. [SDK 시험](../engineering/sdk-trial.md) |
| 영구 상태 및 동시성 | 현재 스케줄러는 무거운 데스크톱 작업을 한 번에 하나씩 허용합니다. 재실행 후에도 유지되는 인덱스, 작업 복구, 일반화된 공용 스케줄링은 설계 옵션입니다. [작업 스케줄링](task-scheduling.md), [파일 저장](file-security.md) |

현재 소스의 기능은 [지원 형식 표](../../../README.ko-KR.md#xdzip-format-support)에서 확인하세요. 출시된 빌드에 대해서는 해당 릴리스 노트와 결과물을 참조하세요. 네이티브 구현, 실험 도구 및 측정 결과에는 각각 적용 범위가 있습니다.
