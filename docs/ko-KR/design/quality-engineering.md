<a name="xdzip-coding-conventions-and-quality-engineering"></a>

# 코딩 규칙 및 품질 엔지니어링

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | [繁體中文](../../zh-TW/design/quality-engineering.md) | [日本語](../../ja-JP/design/quality-engineering.md) | 한국어 | [Français](../../fr-FR/design/quality-engineering.md) | [Deutsch](../../de-DE/design/quality-engineering.md) | [Español](../../es-ES/design/quality-engineering.md) | [Português (Brasil)](../../pt-BR/design/quality-engineering.md) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-code-and-ownership"></a>

## 코드와 소유권

- **Rust.** 작업 공간 도구 체인, 린트 및 잠금 파일을 따릅니다. 외부 길이, 오프셋 및 정수 변환을 검증합니다. 알 수 없는 크기와 타입이 지정된 오류를 보존합니다. unsafe와 FFI의 전제 조건, 소유권, 수명 및 실패 동작을 기록합니다. 동기화 임계 구역은 짧게 유지하고 압축 파일 입출력은 그 밖에서 수행합니다.
- **C++와 Qt.** C++17, RAII 및 명확한 QObject 소유권을 사용합니다. GUI 스레드에서 모델을 바꾸고, 스레드 사이에 소유권이 있는 값을 넘기며, 큐 알림을 살아 있는 수신기에 연결합니다. 수신기가 GUI로 이벤트를 비워야 하면 종료는 비동기로 유지합니다. 압축 파일 및 파일 정책은 Rust가 담당합니다. [Qt/Rust 브리지](qt-rust-bridge.md)
- **QML.** 상태와 사용자 의도를 컨트롤러를 통해 표현합니다. 공용 컨트롤과 생성된 시각 및 텍스트 리소스를 재사용합니다. 파일 입출력과 압축 파일 해석은 델리게이트에서 수행하지 않습니다. 창을 변경할 때 레지스트리, 네이티브 동작 및 시각 표준을 따릅니다. [Qt Quick 인터페이스](qt-quick-ui.md), [시각 디자인 절차](../visual/design-workflow.md)

불변식과 실패 후 남는 상태를 기록합니다. 파일 코드에서는 커밋 허가가 보호하는 대상, 확인하는 식별 정보 및 실패 후 이미 표시될 수 있는 부분을 설명합니다. [파일 저장](file-security.md)

<a name="xdzip-verification-layers"></a>

## 검증 계층

| 변경 | 관련 증거 |
| --- | --- |
| 도메인, 쿼리, 서비스 | 식별자, 한도, 중복 이름, 순서, 오래된 응답, 제한된 전달 및 취소에 대한 Rust 테스트. |
| 형식 작업 | 프로젝트 자체 테스트 자료, 해당하는 경우 독립 구현 왕복, 거부, 손상, 지원하지 않는 프로필 및 한도 사례. |
| 네이티브 파일 및 작업 | 실제 파일 식별자와 기록한 바이트, 취소와 커밋 간 경합, 기존 출력 보호 및 해당 플랫폼의 정리 결과. |
| 데스크톱, Qt 및 접근성 | 모델과 컨트롤러 테스트, QML 및 공용 컨트롤 검사, 해당하는 네이티브 상호 작용, 실제 시각 또는 보조 기술 검증 증거. |
| 성능 및 배포 | 커밋에 연결된 작업 부하, 렌더러와 자원 측정값, 또는 채널별 빌드 ID, 설치와 업데이트 결과 및 의존성 기록. |

[검증 안내](../../../developers/qa/README.md)에 따라 변경에 필요한 검사를 정합니다. 프로토콜 준수, 테스트 자료나 오프스크린 검사, 네이티브 자동화, 수동 데스크톱 관찰은 각기 다른 사실을 확인합니다. 기존 회귀 진입점과 제공 범위는 [개발 빌드 안내](../engineering/developer-build.md)와 [요구 사항 추적](requirements-traceability.md)에 연결되어 있습니다.

<a name="xdzip-existing-ci-and-additional-techniques"></a>

## CI와 추가 검증 방법

일반 [CI](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml)는 문서, 생성 리소스, 레이아웃 계약, 서식 및 워크플로 구조를 검사합니다. 더 무거운 Rust, 형식, Qt 네이티브 검사는 별도로 실행하는 Foundation 및 Qt 워크플로에서 진행합니다. 릴리스 결과물은 전용 작업에서 처리합니다. 실행 조건은 [CI 정책](../../../developers/engineering/ci-policy.md)에 있습니다. 변경 코드와 대상 플랫폼에 따라 로컬 검사와 호출할 워크플로를 정합니다.

변경에 도움이 되는 경우 퍼징, 새니타이저, 모델 검사 및 성능 벤치마크를 사용합니다. 한도가 명확하고 재배포 가능한 테스트 자료를 고르고 결과는 이 안내와 별도로 보관합니다. 민감한 보고는 [보안 정책](../../../SECURITY.md)을 따릅니다. 검증 계획이나 사례를 예약된 실행 또는 인수 결과로 기록하지 않습니다.

<a name="xdzip-maintaining-the-design"></a>

## 설계 유지보수

구현이 식별자, 명령, 기능, 작업이나 쓰기 범위를 바꾸면 해당 설계, 기술 참조와 인수 링크를 함께 갱신합니다. 모든 언어의 문서 내용을 맞춰 유지합니다. [아키텍처 결정](architecture-decisions.md)은 결정 근거를, [성능 설계](performance.md)는 측정 규칙을 설명합니다. 실제 결과 및 제한은 해당 변경과 함께 별도로 기록합니다.
