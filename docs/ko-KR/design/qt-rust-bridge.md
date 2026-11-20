<a name="xdzip-qt-rust-bridge-and-application-service"></a>

# Qt/Rust 브리지와 애플리케이션 서비스

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | [繁體中文](../../zh-TW/design/qt-rust-bridge.md) | [日本語](../../ja-JP/design/qt-rust-bridge.md) | 한국어 | [Français](../../fr-FR/design/qt-rust-bridge.md) | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | [Español](../../es-ES/design/qt-rust-bridge.md) | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

Qt 데스크톱은 연결된 `cxx` 브리지를 통해 Rust 런타임을 호출합니다. 각 `BrowserController`는 별도 서비스 인스턴스를 가진 `RuntimeConnection`을 소유합니다. 브리지는 데스크톱 프로세스 안에서 타입이 지정된 명령과 소유권이 있는 결과를 전달합니다. CLI는 같은 Rust 서비스를 직접 사용합니다. [시스템 아키텍처](system-architecture.md), [데스크톱 세션](../engineering/application-sessions.md)

<a name="xdzip-handles-threads-and-ownership"></a>

## 핸들, 스레드와 소유권

`new_runtime()`은 Rust 명령 스레드와 작업 스레드를 시작합니다. 임시 `Runtime`은 `CommandClient` 하나와 `EventReceiver` 하나를 제공합니다. GUI 스레드는 명령 핸들을 사용합니다. 전용 Qt 수신 스레드는 이벤트 핸들을 소유하고 `next_event()`에서 대기합니다. Rust 명령 스레드는 현재 압축 파일과 인덱스를 소유합니다. 별도의 작업 스레드는 실행 중 준비된 작업을 소유합니다.

CXX는 값 타입과 소유권을 가진 Rust 컨테이너를 전달합니다. 수신 Qt 코드는 값을 소유하는 동안 이를 보유하거나 변환합니다. 압축 파일이나 네이티브 파일 핸들은 Qt로 넘어가지 않습니다. ID나 정확한 바이트 수를 QML로 전달할 때 64비트 전체를 보존해야 하는 값에는 컨트롤러가 문자열을 사용합니다. 생성된 헤더와 정적 라이브러리는 호스트와 함께 빌드됩니다. 이 인터페이스는 제품 빌드에 속합니다. 포함된 작업 프로세스는 별도의 [프로세스 프로토콜](../engineering/native-codecs.md)을 사용합니다.

[브리지 선언과 변환](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs), [Qt 연결](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp), [Rust 서비스](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

<a name="xdzip-request-query-and-task-identities"></a>

## 요청, 쿼리와 작업 식별 정보

| 식별 정보 | 용도 |
| --- | --- |
| 요청 ID | Rust 서비스가 승인 시 연결 범위의 ID를 할당합니다. `Submission`은 `Accepted`, `Busy`, `Closed`, `Invalid`, `Exhausted`를 반환합니다. 거부 시 ID는 영입니다. |
| 세션 ID | 현재 열린 압축 파일 스냅샷을 식별합니다. 해당 파일에 작용하는 명령은 이 ID와 일치해야 합니다. 새로 열면 새 세션 ID를 받습니다. |
| 쿼리 식별 정보와 오프셋 | 페이지는 세션, 쿼리 ID, UI 세대, 리비전과 오프셋에 연결됩니다. 컨트롤러는 행을 받기 전에 이를 검증합니다. |
| 물리 항목 또는 탐색 ID | 물리 항목은 정렬과 페이지 이동 중 식별 정보를 유지합니다. 합성 디렉터리는 자체 식별 정보를 가집니다. 행 인덱스, 표시 이름 또는 쿼리 범위는 파일 시스템 경로에 대한 접근 권한을 부여하지 않습니다. |
| 작업 및 프롬프트 ID | 작업 ID는 해당 연결에 속합니다. 프롬프트는 작업 내에서 대기 중인 충돌 또는 암호 응답 하나를 식별합니다. 앱의 작업 항목은 소유 컨트롤러도 압니다. |
| 이벤트 시퀀스 | 수신자는 이벤트를 꺼낼 때 연결별로 단조 증가하는 시퀀스를 할당합니다. Qt는 마지막으로 소비한 번호보다 새롭지 않은 이벤트를 버립니다. 작업 업데이트도 이 시퀀스를 공유하고 원래 요청 ID로 상관관계를 맺습니다. |

<a name="xdzip-commands-in-the-current-bridge"></a>

## 현재 브리지의 명령

| 명령 또는 메서드 | 용도와 일반 결과 |
| --- | --- |
| `Open`, `OpenWithPassword` | 선택한 원본을 열고 스캔합니다. 성공 응답에는 첫 `Page`와 제한된 탐색 데이터가 포함됩니다. 암호화된 디렉터리는 세션을 열기 전에 암호가 필요할 수 있습니다. |
| `Browse` | 쿼리를 시작하거나 일치하는 식별 정보와 오프셋으로 계속합니다. 명령마다 최대 하나의 `Page`를 반환하며 128행과 이름 64 KiB로 제한됩니다. |
| `FreezeSelection`, `Preflight` | 현재 선택자를 `Selected`의 소유된 물리 또는 탐색 ID로 해석하거나 메타데이터 검사 결과를 `Preflight`에 요약합니다. 실제 작업을 위해 스냅샷과 선택 범위를 다시 검증합니다. |
| `Preview` | 물리 항목 하나를 해석합니다. 성공하면 소유권이 있고 제한된 `Preview` 바이트를 항목 및 세션 식별 정보와 함께 반환합니다. |
| `CheckTarget`, `InspectInputs` | 런타임이 검사 작업 공간을 열고 정리한 뒤 `TargetChecked`에 남은 개수를 반환합니다. 생성 검사는 세션 영을 사용합니다. `Inputs`는 디렉터리를 재귀 스캔하지 않고 입력 순서의 얕은 메타데이터를 보고합니다. 어느 응답도 확정 권한을 부여하지 않습니다. |
| `Extract`, `Create`, `Check`, `OpenEntry` | 의도를 검증하고 작업을 큐에 넣습니다. `TaskAccepted`에는 할당된 작업 ID가 있습니다. 대상 승인과 확정은 런타임과 I/O 계층이 수행합니다. |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | 압축 파일 편집과 데이터 게시는 무거운 작업 큐에 들어갑니다. `Update` 또는 `Move`를 승인한 뒤 서비스는 현재 스냅샷을 버려 후속 요청이 이전 항목 ID를 사용하지 못하게 합니다. `ExportManifest`는 상대 전송 메타데이터를 일반 응답으로 반환합니다. |
| `Snapshot`, `CloseSession` | 현재 압축 파일 세션 ID 또는 영을 반환합니다. 지정 세션을 닫으면 탐색 스냅샷은 버리지만 독립적으로 유지된 작업 입력은 살아 있을 수 있습니다. |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | 일반 명령 큐 밖의 제어 메서드입니다. 요청 취소에는 자체 요청 ID와 `Finished`가 있으며 작업 제어는 기존 작업에 적용됩니다. |

Rust 서비스는 `Catalog`, `SelectMembers` 같은 CLI 전용 작업도 제공합니다. 브리지는 제출 전에 입력 크기와 열거형 값을 검증합니다. 예를 들어 생성 경로는 각각 4 KiB, 합계 4 MiB로 제한됩니다. 런타임은 세션과 작업 의미를 검증하고 I/O 계층은 파일을 열고 출력 권한을 얻습니다. [런타임 서비스 계약](../engineering/product-runtime.md), [CLI 계약](../engineering/command-line.md), [파일 권한](file-security.md)

<a name="xdzip-event-delivery-and-backpressure"></a>

## 이벤트 전달과 역압

승인된 일반 요청은 정상 전달 시 최대 하나의 데이터 결과와 그 뒤의 `Finished` 하나를 생성합니다. 오류는 오류 코드를 담은 `Finished`로 나타냅니다. 컨트롤러는 성공 완료를 기다린 후 해당 페이지나 미리 보기를 게시합니다. 대체된 결과는 버리지만 해당 요청은 계속 완료됩니다. 대형 인덱스 경로는 Qt에 최대 1,024행을 캐시하고 빠진 페이지를 요청합니다. 추출 확인을 표시하기 전에 여러 페이지에 걸친 선택을 고정합니다. [대용량 압축 파일 인덱싱](../engineering/large-index.md), [콘텐츠 미리 보기](../engineering/content-preview.md)

`TaskAccepted`와 `Finished`는 일반 요청 전달에 속합니다. `TaskUpdate`는 작업별 별도 대기 슬롯을 사용하며 원래 요청 ID, 작업 ID, 원본 세션 ID를 담습니다. 업데이트는 `TaskAccepted`보다 먼저 또는 `Finished`보다 나중에 올 수 있습니다. 컨트롤러는 일찍 온 작업을 원래 요청에 연결하고 후속 업데이트를 그 작업으로 보냅니다. 이후 작업 업데이트가 일반 요청을 다시 완료하지는 않습니다. 대기 슬롯은 작업의 최신 스냅샷을 유지합니다. 작업 진행은 합쳐질 수 있고 종료 스냅샷은 수신자가 가져갈 때까지 남습니다. 작업이 응답을 기다리는 동안 프롬프트가 스냅샷에 나타납니다. [작업 상태와 취소](task-scheduling.md)

- **Rust.** 연결마다 일반 승인은 미완료 요청 128개와 큐 명령 바이트 예산 8 MiB를 허용합니다. 일반 이벤트 페이로드 예산은 32 MiB입니다. 승인된 일반 요청은 결과와 완료를 위한 공간을 예약합니다. 요청 취소 확인 응답에는 별도 제한된 큐가 있고 종료에는 그 큐의 공간이 필요하지 않습니다.
- **Qt.** 메일박스는 최대 256개 이벤트, 그중 최대 128개의 페이로드 이벤트와 계산된 페이로드 32 MiB를 보유합니다. 용량이 소진되면 수신자는 GUI가 비울 때까지 기다립니다. 큐에 들어간 하나의 알림 체인은 차례마다 최대 32개 이벤트 또는 4 ms 동안 처리합니다. 알림은 GUI 스레드에 전달되고 큐의 이벤트는 소유 데이터를 유지합니다.

<a name="xdzip-prompts-failures-and-shutdown"></a>

## 프롬프트, 실패와 종료

- 충돌 결정에는 `decide_task`를 사용합니다. 작업 암호에는 `respond_password`와 별도의 암호 프롬프트 ID를 사용합니다. 둘 다 현재 응답되지 않은 프롬프트를 검증합니다. 암호 응답은 해제 시 0으로 지워지는 Rust 저장소에 들어가며 결과나 진행 이벤트에 나타나지 않습니다. `OpenWithPassword`, `Preview`, 암호화 생성은 작업별 입력 비밀을 담고 브리지는 이를 해당 Rust 암호 타입으로 이동합니다. [암호 처리](../engineering/password-extraction.md)
- 안정적인 오류 코드는 승인, 요청 및 작업 결과를 설명합니다. 예를 들면 `stale_session`, `stale_query`, `source_changed`, `unsafe_target`, `target_changed`, `disk_full`, `password_required`가 있습니다. 컨트롤러는 표시할 코드를 번역합니다. 원본이 바뀌면 해당 탐색 스냅샷이 무효화됩니다. `RuntimeFault`와 예상치 못한 연결 해제는 대기 중인 UI 작업을 `runtime_unavailable`로 무효화하며 연결 손실로 원래 종료 이벤트가 도착하지 않을 수 있습니다.
- `shutdown`은 승인을 닫고 입력 대기 작업을 포함한 승인된 작업을 취소합니다. Qt 수신자는 Rust 스레드가 끝나는 동안 계속 이벤트를 비웁니다. Rust 수신자는 스레드를 기다린 뒤 `ShutdownReady`를 내보내고 이어 브리지가 `End`를 내보냅니다. Qt는 메일박스를 비운 후 정상 연결 해제를 보고합니다. 연결이 일찍 소멸하면 GUI 스레드에서 기다리지 않고 메일박스를 닫고 수신자를 깨웁니다. [작업 종료](task-scheduling.md), [런타임 수명 주기](../engineering/product-runtime.md)

이 경계를 변경할 때 [브리지 검사 도구](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md), Rust 서비스 테스트와 [데스크톱 컨트롤러 테스트](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp)는 서로 다른 계층을 검증합니다. 관련 대상을 찾으려면 [개발 빌드 안내](../engineering/developer-build.md)를 사용하세요.
