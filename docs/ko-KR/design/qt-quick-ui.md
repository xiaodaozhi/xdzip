<a name="xdzip-qt-quick-desktop-interface"></a>

# Qt Quick 데스크톱 인터페이스

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | [日本語](../../ja-JP/design/qt-quick-ui.md) | 한국어 | [Français](../../fr-FR/design/qt-quick-ui.md) | [Deutsch](../../de-DE/design/qt-quick-ui.md) | [Español](../../es-ES/design/qt-quick-ui.md) | [Português (Brasil)](../../pt-BR/design/qt-quick-ui.md) | [Русский](../../ru-RU/design/qt-quick-ui.md)

데스크톱은 표시와 상호 작용에 Qt Quick을 사용합니다. C++ 컨트롤러는 사용자 의도를 요청으로 변환하고 보기 상태를 유지합니다. Rust 런타임은 압축 파일 해석, 작업 정책과 파일 권한을 담당합니다. 아래 호출은 [시스템 아키텍처](system-architecture.md)와 [브리지 계약](qt-rust-bridge.md)에 설명되어 있습니다.

<a name="xdzip-windows-sessions-and-models"></a>

## 창, 세션 및 모델

| 구성 요소 | 현재 책임 |
| --- | --- |
| `Browser.qml` 및 `SessionPage.qml` | 홈 화면, 압축 파일 탭, 경로 및 검색 컨트롤, 디렉터리 탐색, 목록 또는 격자, 속성 및 작업 진입점을 제공합니다. 페이지는 현재 컨트롤러 상태를 표시하고 명령을 호출합니다. |
| `ApplicationSessions` | 열기 요청을 독립 탐색 세션으로 보내고 탭 또는 창, 공통 작업 진입점 및 앱 종료를 조정합니다. 열기 방식은 이후 요청에 적용합니다. 페이지를 닫으면 탐색 화면은 해제되며 작업의 컨트롤러는 계속 실행될 수 있습니다. |
| `BrowserController` 및 `ArchiveTableModel` | 선택, 목록 상태 및 결과 식별 정보를 유지합니다. 작은 결과는 완전한 모델로 반영합니다. 큰 결과는 논리 행 수를 표시하고 최대 1,024개 행만 유지하며 없는 페이지를 불러옵니다. 로드하지 않은 행에서는 항목 작업을 요청할 수 없습니다. |
| `TaskListModel` 및 작업 창 | 한도가 있는 작업 스냅샷, 진행, 결과와 현재 필요한 결정을 표시합니다. 합성 데스크톱 식별 정보를 사용하여 작업 명령을 담당 컨트롤러에 전달합니다. |
| 설정 및 플랫폼 어댑터 | 모양, 언어, 글자 크기와 앱 동작을 관리합니다. 네이티브 어댑터는 운영 체제에 따라 알림 영역, 활성화, 파일 대화상자, 파일 연결 및 전송 인계를 담당합니다. |

[데스크톱 진입점](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp), [세션 페이지](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml), [컨트롤러와 모델](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h), [앱 세션](../engineering/application-sessions.md), [설정과 언어](../ux/settings-localization.md)

<a name="xdzip-browsing-and-operation-flow"></a>

## 탐색 및 작업 흐름

열기는 완성된 메타데이터 스냅샷을 게시합니다. 새로운 검색, 정렬 또는 탐색 요청은 자체 결과 식별 정보를 가집니다. 이전 페이지는 현재 모델을 대체할 수 없습니다. 새 쿼리를 기다리는 동안에도 컨트롤러가 유효하다고 판단하면 이미 반영된 결과를 표시합니다. 코어는 한 묶음에 최대 128개 행과 이름 데이터 64 KiB를 전송합니다. [도메인 데이터](domain-data.md), [큰 압축 파일 인덱싱](../engineering/large-index.md)

디렉터리를 열면 그 안으로 이동합니다. 조건을 충족하는 일반 파일 항목을 두 번 클릭하거나 해당 열기 동작을 실행하면 임시 열기 작업이 시작됩니다. 속성 보기에는 별도의 미리 보기 동작이 있습니다. 충돌과 암호 메시지는 현재 작업의 식별 정보를 유지합니다. 탐색 페이지를 닫은 뒤에도 앱의 작업 진입점에서 진행 상태와 결과를 확인합니다. [탐색](../user-guide/browse.md), [임시 열기](../engineering/archive-opening.md), [내용 미리 보기](../engineering/content-preview.md), [작업 창](../ux/task-windows.md)

파일을 끌어 놓으면 대상에 따라 압축 파일을 열거나 만들기 입력을 추가합니다. 지원하는 압축 파일 내부 이동과 외부로 끌기는 작업 준비와 플랫폼별 인계를 사용합니다. 비동기 콘텐츠와 대상 위치 쓰기를 포함한 전송 기능은 플랫폼이 결정합니다. [압축 파일 전송](../engineering/archive-transfers.md)

<a name="xdzip-shared-controls-and-native-boundaries"></a>

## 공통 컨트롤과 네이티브 범위

작업 화면은 `OperationButton`, `OperationFooterButton`, `OperationFrame`, `OperationSelect` 및 `OperationSegmentedGroup`을 재사용합니다. 공통 구성 요소가 치수와 상호 작용을 관리합니다. 공통 선택 컨트롤은 마우스, 키보드 및 접근성 동작을 동일한 상태 모델로 전달합니다. UI 문구와 파일 데이터는 [디자인 표준](../visual/design-standard.md)의 글꼴 및 현지화 규칙을 따릅니다. 압축 파일 안의 이름은 데이터로 유지하며 안전하게 표시합니다.

독립 작업 창은 창 레지스트리와 창별 모달 및 닫기 규칙을 따릅니다. macOS는 네이티브 제목 표시줄 컨트롤을 사용합니다. Windows와 Linux는 시스템 어댑터를 사용합니다. [플랫폼 창 계약](../ux/platform-windows.md)은 실제 창의 인수 조건을 정하고 [데스크톱 통합 계약](../ux/desktop-integration.md)은 알림 영역과 활성화 동작을 설명합니다.

<a name="xdzip-design-and-verification-scope"></a>

## 설계와 검증 범위

소스 코드와 테스트가 제품 동작을 설명합니다. 네이티브 포커스, 메뉴, IME, 접근성 동작, 화면 배율 및 시스템 파일 관리자에 영향이 있으면 해당 플랫폼의 증빙이 필요합니다. 데스크톱 구성을 추가로 지원하거나 새 작업 창을 만들려면 해당 구현과 인수 검증이 필요합니다. [시각 디자인 흐름](../visual/design-workflow.md), [품질 엔지니어링](quality-engineering.md)
