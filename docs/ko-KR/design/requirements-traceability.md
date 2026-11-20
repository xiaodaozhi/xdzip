<a name="xdzip-requirements-and-verification-traceability"></a>

# 요구 사항과 검증 추적성

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | [繁體中文](../../zh-TW/design/requirements-traceability.md) | [日本語](../../ja-JP/design/requirements-traceability.md) | 한국어 | [Français](../../fr-FR/design/requirements-traceability.md) | [Deutsch](../../de-DE/design/requirements-traceability.md) | [Español](../../es-ES/design/requirements-traceability.md) | [Português (Brasil)](../../pt-BR/design/requirements-traceability.md) | [Русский](../../ru-RU/design/requirements-traceability.md)

[제품 요구 사항](product-requirements.md)은 요구 사항 ID와 현재 범위를 정의합니다. 각 행은 해당 동작이 바뀔 때 검증할 내용을 설명합니다. 실행 결과, 플랫폼 범위와 제한은 해당 변경 기록에 남깁니다. [인수 사례](../../../developers/qa/acceptance.md)에 검증 진입점이 나와 있습니다.

<a name="xdzip-requirement-matrix"></a>

## 요구 사항 표

| 요구 사항 | 설계 참조 | 검증 중점 |
| --- | --- | --- |
| P01 풀지 않고 탐색 | [도메인](domain-data.md), [데스크톱](qt-quick-ui.md) | T01 실제 압축 파일을 열고 탐색 중 파일이 추출되지 않는지 확인합니다. |
| P02 탐색, 검색 및 선택 | [도메인](domain-data.md), [성능](performance.md) | T02 같은 이름 항목의 식별 정보, 안정적인 쿼리, 오래된 페이지, 대용량 인덱스 저장과 페이지 간 선택을 확인합니다. |
| P03 지정한 범위 압축 풀기 | [파일 승인](file-security.md), [작업](task-scheduling.md) | T03 디렉터리 확장, 중복, 이름 변경, 건너뛰기, 덮어쓰기와 실제 디스크 결과를 확인합니다. |
| P04 압축 파일 만들기 | [형식](codec-extension-sdk.md), [파일 승인](file-security.md) | T04 변경된 프로필을 독립적으로 만들고 읽는 시험, 임시 출력 검증과 기존 파일 보호를 확인합니다. |
| P05 형식별 기능 준수 | [브리지](qt-rust-bridge.md), [형식](codec-extension-sdk.md) | T05 레지스트리, 세션별 동작과 지원하지 않는 프로필의 실행 시 거부를 확인합니다. |
| P06 작업 제한과 취소 | [브리지](qt-rust-bridge.md), [작업](task-scheduling.md) | T06 가득 찬 대기열, 사용자 결정 요청 전달, 취소와 확정의 경쟁 및 비동기 종료를 확인합니다. |
| P07 파일과 리소스 경계 보호 | [파일 승인](file-security.md), [형식](codec-extension-sdk.md) | T07 해당하는 네이티브 링크 및 식별 검사, 경로 검증, 할당량과 보호 대상을 확인합니다. |
| P08 접근 가능한 데스크톱 제공 | [데스크톱](qt-quick-ui.md) | T08 영향받는 플랫폼에서 모델과 컨트롤, 네이티브 키보드, IME, 접근성, 배율과 포커스를 확인합니다. |
| P09 리소스 제한 및 측정 | [성능](performance.md) | T09 변경에 따라 비교 가능한 빌드, 프로세스 트리 지표, 시작, 정렬, 출력 속도 또는 패키지 크기를 확인합니다. |
| P10 형식 로직에서 Qt 분리 | [아키텍처](system-architecture.md), [형식](codec-extension-sdk.md) | T10 Qt 없는 코어, 명시적 어댑터 선택과 형식별 회귀를 확인합니다. |
| P11 비밀 정보와 데이터 보호 | [파일 승인](file-security.md), [개인정보](../engineering/privacy.md) | T11 용도가 제한된 암호, 민감 정보가 제거된 진단과 의도된 네트워크 접근을 확인합니다. |
| P12 배포 감사 | [품질](quality-engineering.md), [릴리스](build-release.md) | T12 고정 입력과 결과물 식별, 구성 목록, 라이선스 자료, 패키지 신뢰 및 필요한 설치 검사를 확인합니다. |
| P13 외부 확장 통제 | [형식](codec-extension-sdk.md), [SDK 실험](../engineering/sdk-trial.md) | T13 실험 범위의 상호 운용성과 네이티브 권한을 확인합니다. 제품의 확장 설치는 설계 옵션입니다. |
| P14 미리 보기와 열기 제한 | [데스크톱](qt-quick-ui.md), [파일 승인](file-security.md) | T14 자동 미리 보기, 지원 콘텐츠, 프로세스 한도와 비공개 임시 출력을 확인합니다. |
| P15 실제 결과 보고 | [브리지](qt-rust-bridge.md), [파일 승인](file-security.md), [작업](task-scheduling.md) | T15 원본과 대상 변경, 이미 확정한 출력 이후의 실패와 취소, 정리 및 불확실성을 확인합니다. |

<a name="xdzip-cross-cutting-verification"></a>

## 공통 검증

| 검증 묶음 | 적용되는 변경 | 검증 중점 |
| --- | --- | --- |
| T16 브리지 소유권과 수명 주기 | 브리지, 연결 또는 컨트롤러 수명 변경 | 현재 요청 식별 정보, 수신자 깨우기, 유지된 작업과 안전한 소멸을 확인합니다. |
| T17 형식 상호 운용성과 오류 경로 | 형식 또는 코덱 변경 | 필요한 독립 유효 샘플, 잘림이나 지원하지 않는 프로필과 출력 한도를 확인합니다. |
| T18 설치, 업데이트 및 제거 | 플랫폼 또는 배포 채널 변경 | 실제 패키지, 깨끗한 환경의 런타임 종속성, 설치 식별, 업데이트 인계와 지원되는 제거 또는 복구를 확인합니다. |

<a name="xdzip-representative-workflows"></a>

## 대표 작업 흐름

1. ZIP을 열어 검색하고 선택 결과를 고정한 뒤 압축을 풀어 출력 바이트를 비교합니다. “모두 풀기”는 전체 압축 파일을 뜻합니다.
2. 같은 이름 또는 충돌하는 항목을 정해진 정책으로 풀고 결과 파일 이름과 상태를 확인합니다.
3. 확정 전과 확정 중에 취소하여 완료된 파일이 보고되는지 확인하고, 만들기가 확정 전에 실패하면 이전에 검증된 압축 파일이 보존되는지 확인합니다.
4. 관리 대상 테스트 중 원본 또는 대상을 바꾸고 실제 결과를 확인합니다. 비정상 종료 후 자동 재실행을 가정하지 말고 남은 항목을 확인합니다.
5. 포함된 형식 프로세스 또는 별도 SDK 실험을 계약에 따라 실패시키고 미완료 데이터가 성공 결과로 게시되지 않는지 확인합니다.

문서의 작성, 형식과 링크는 정적 검사로 확인합니다. 테스트 데이터, 오프스크린, 네이티브 자동화 및 수동 인수 증거는 각각의 범위에 적용됩니다. [품질 엔지니어링](quality-engineering.md), [검증 안내](../../../developers/qa/README.md)
