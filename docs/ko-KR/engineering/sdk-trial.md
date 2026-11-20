<a name="xdzip-external-read-only-sdk-trial"></a>

# 외부 읽기 전용 SDK 시험

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | [繁體中文（台灣）](../../zh-TW/engineering/sdk-trial.md) | [日本語](../../ja-JP/engineering/sdk-trial.md) | 한국어 | [Français](../../fr-FR/engineering/sdk-trial.md) | [Deutsch](../../de-DE/engineering/sdk-trial.md) | [Español](../../es-ES/engineering/sdk-trial.md) | [Português (Brasil)](../../pt-BR/engineering/sdk-trial.md) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol`과 `xdzip-plugin-host`는 [형식 확장 설계](../design/codec-extension-sdk.md)에 설명된 독립 읽기 전용 시험을 구현합니다. 데스크톱과 CLI의 형식 탐지, 설치, 패키징, 자동 시작에서는 제외됩니다. 사용자가 시험 명령을 명시적으로 실행하고 패키지와 원본을 선택하며 제품의 타사 확장은 계속 비활성화됩니다. `xdp1`만 협상하며 생성, 암호, 분할 볼륨, 링크, GUI 또는 임의의 호스트 경로 권한은 없습니다. 완전한 플러그인 생태계나 SDK 릴리스가 아닙니다.

<a name="xdzip-reproducible-entry-points"></a>

## 재현 가능한 진입점

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

Windows에서는 `python`과 `.exe`를 사용합니다. `test`는 네이티브 참조 프로그램을 빌드하고 Rust 프레이밍 회귀, 독립 Python 상호 운용성, 악의적 프로토콜, 취소, 원본 변경, 충돌, 커널 권한 회귀를 실행합니다. Python은 테스트용으로 컴파일된 신뢰할 수 있는 적합성 진입점에서만 실행합니다. 배포되는 호스트는 인터프리터, 인수, 셸 또는 샌드박스 없는 실행 스위치를 허용하지 않습니다. 예제는 `build/` 아래에 자체 패키지, 매니페스트, XDP1만 생성하며 릴리스나 타사 다운로드를 발생시키지 않습니다. 로그에는 작업 영역과 사용자 홈 경로를 남기지 않고 JSON은 커밋, 수정 상태와 바이너리 해시를 연결합니다. 둘 다 로컬 또는 CI 산출물입니다. 동일한 네이티브 Foundation 매트릭스가 이 진입점을 호출합니다. 별도의 [SDK 시험 워크플로](../../../.github/workflows/sdk-trial.yml)는 관련 없는 형식 및 Qt 파이프라인을 반복하지 않고 먼저 플랫폼별 샌드박스 호환성을 확인합니다. Foundation 워크플로도 `sdk_only=true`로 재사용 가능한 시험을 호출할 수 있지만 전체 Foundation 검사를 건너뛰므로 전체 Foundation 결과를 입증하지 않습니다. Windows에서는 실행 거부를 검증합니다.

<a name="xdzip-package-admission"></a>

## 패키지 허용

엄격한 JSON 매니페스트는 `deny_unknown_fields`를 사용합니다. 필드는 `manifest_version=1`, `plugin_id`, `version`, `author`, `protocol_majors=[1]`, `targets`, 패키지 내부 `entrypoint`, 소문자 16진수 `sha256`, `formats=["xdp1"]`, `permissions`, `resources`, `license_files`, HTTPS `source_url`입니다. 위의 `example`이 생성하는 참조 매니페스트는 현재 스키마를 실행할 수 있는 진입점입니다.

`permissions`는 정확히 `broker_source_read=true`, `broker_output_write=true`, `network=false`이며 `resources`는 정확히 `memory_bytes=536870912`, `threads=1`입니다. 매니페스트는 최대 32 KiB, 네이티브 실행 파일은 최대 64 MiB이며 라이선스 파일은 최대 네 개, 각각 최대 128 KiB입니다. 진입점과 라이선스 파일은 절대 경로, 상위 디렉터리 이동, 링크나 재분석 지점을 통한 이탈을 허용하지 않습니다. 각 패키지 멤버는 보유한 cap-std 디렉터리에서 링크를 따르지 않고 구성 요소별로 열며 검증 후 원래 문자열로 다시 열지 않습니다. 호스트는 보유한 일반 파일에서 복사하고 실제 바이트 수, SHA-256, 파일 버전을 복사 전후에 검증한 뒤 비공개 임시 복사본만 실행하고 프로세스와 IPC가 종료될 때까지 보존합니다. 선언된 해시는 선택한 패키지 내용의 일관성을 나타내며 게시자를 인증하거나 코드의 안전성을 증명하지 않습니다.

<a name="xdzip-wire-protocol-and-state"></a>

## 통신 프로토콜과 상태

자체 제한 CBOR 부분 집합은 [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)를 따르며 타사 CBOR 구현을 사용하지 않습니다. Rust 참조 상대는 프레이밍 crate를 재사용하고 [Python 상대](../../../examples/sdk/reference.py)는 Rust나 생성된 프로토콜 코드를 가져오지 않고 인코딩, 디코딩, XDP1 파싱을 독립적으로 구현합니다. 상호 운용성은 실제 콘텐츠의 여러 블록과 호스트 다이제스트를 다룹니다.

프레임은 `u32 LE length + CBOR`이며 할당 전에 길이를 검사합니다. 프레임 최대 1 MiB, 개별 bytes/text 최대 256 KiB, 깊이 최대 16, 배열 최대 128, 맵 최대 32, 디코딩 노드 최대 8192입니다. UInt, bytes, 엄격한 UTF-8 텍스트, 배열, UInt 키 맵, bool만 허용합니다. 가장 짧은 형태가 아닌 정수, 중복 키, 무한 길이, 음수, 태그, 부동 소수점, 후행 데이터는 거부합니다. 엔벨로프 맵 키 `1..5`는 kind, request, operation, sequence, payload입니다. sequence는 각 방향에서 1부터 연속 증가하고 request는 호출마다 증가하며 operation은 호스트의 임의의 0이 아닌 식별자입니다. 모든 응답은 현재 식별 정보와 정확히 일치해야 합니다. 알 수 없는 payload 키 `<128`은 거부하며 `>=128`은 무시할 수 있는 선택 필드입니다.

| kind | 메시지 | payload 숫자 키 |
| --- | --- | --- |
| 1 | Hello | 1 최소 major, 2 최대 major, 3 패키지 ID, 4 `xdp1` |
| 2 | Welcome | 1 major, 2 블록 제한, 3 단일 블록 크레딧, 4 총 읽기 예산, 5 출력 예산 |
| 10 | Probe | 1 원본 토큰, 2 원본 길이 |
| 11 | Open | 1 원본 토큰 |
| 12 | Scan | 1 세션 토큰 |
| 13 | ReadEntry | 1 세션, 2 항목, 3 출력 대상, 4 크레딧 수 1, 5 출력 예산, 6 첫 블록 크레딧 토큰 |
| 14 | Close | 1 세션 |
| 20, 21 | ReadAt, ReadData | 1 원본 토큰, 2 오프셋, 3 길이 또는 바이트 |
| 30 | EntryBatch | 1 세션, 2 항목. 각 레코드: 1 토큰, 2 원시 경로, 3 유형 0 파일 / 1 디렉터리, 4 알려진 크기 |
| 31 | ScanFinished | 1 실제 레코드 수 |
| 40 | WriteData | 1 출력 대상, 2 0부터 시작하는 블록 순서, 3 바이트, 4 현재 크레딧 토큰 |
| 41 | FinishOutput | 1 출력 대상, 2 선언된 출력 수, 3 SHA-256 바이트 |
| 42 | WriteAck | 1 출력 대상, 2 다음 블록 순서, 3 새 임의 크레딧 토큰 |
| 50, 51, 52 | Cancel, CancelAck, Shutdown | 빈 맵 |
| 61 | OperationFinished | 1 결과, 0 성공 / 1 취소. Probe는 2 형식, Open은 2 세션도 포함 |

프레이밍이 인식하더라도 이 시험의 상태 머신은 다른 kind를 거부합니다. Hello가 먼저 도착해야 하고 양쪽이 major 1을 선택합니다. 핸드셰이크 식별 정보는 0이며 이후 호출은 0이 아닙니다. ScanFinished 뒤에도 유일한 성공 종료 상태가 필요합니다. 한 번에 하나의 작업만 실행하며 확장을 백그라운드에서 재시도하지 않습니다. 현재 단일 블록 크레딧은 설계상 최대 네 개보다 엄격합니다. 예측할 수 없는 크레딧 토큰은 각 출력 블록을 실제로 수락한 뒤 갱신하며 미리 사용하거나 재생할 수 없습니다.

<a name="xdzip-broker-budgets-and-publication"></a>

## 브로커, 예산, 게시

원본 토큰, 출력 대상, 블록별 크레딧은 임의의 128비트 값이며 작업 간 재사용할 수 없습니다. 상대에게 압축 파일이나 대상 경로를 제공하지 않습니다. 호스트는 승인된 원본 범위 내에서 읽기당 최대 256 KiB를 반환하며 오프셋 오버플로, EOF 초과, 길이 0, 누적 읽기 64 MiB 초과는 실패합니다. 각 읽기 전후와 최종 게시 전에 보유 객체와 대응 경로의 `FileVersion`을 확인합니다. 이는 기존 식별 정보, 시간, 길이 지문을 재사용하며 같은 계정의 공격자가 모든 메타데이터를 복원한 임의의 동시 변경을 감지한다고 주장하지 않습니다.

메타데이터는 물리 레코드 최대 1000개, 경로와 토큰 합계 최대 256 KiB로 제한하며 레코드 토큰은 유일해야 합니다. 호스트는 원시 경로를 PortablePath에 따라 엄격히 검증하고 중복 이름은 별도 식별 정보를 유지합니다. 한 번에 일반 항목 하나를 명시적으로 선택하며 선언 크기는 최대 4 MiB입니다. 디렉터리를 파일 출력으로 사용할 수 없습니다. 호스트가 대상 Grant, 비공개 Workspace와 Stage를 보유하고 실제 바이트를 계량하며 SHA-256을 계산합니다. 상대에게 파일 핸들이나 게시 권한은 제공하지 않습니다.

기존 안전 I/O는 모든 출력 블록, FinishOutput의 수와 다이제스트, 유일한 종료 상태, Close, Shutdown 후 정상 종료와 깨끗한 EOF, 원본 식별 정보, 최종 취소 검사가 모두 성공한 뒤에만 덮어쓰기 없이 게시할 수 있습니다. 이상, 중복 종료 상태, 충돌 또는 시간 초과 시 먼저 출력을 취소하고 객체 파괴는 workspace를 정리하기 전에 stage를 제거합니다. 기존 대상은 변경하지 않습니다. 파싱과 예상 다이제스트는 여전히 확장에서 제공하므로 호스트 계산만으로 악의적 확장의 출력이 압축 파일에 충실한지 증명할 수 없습니다.

핸드셰이크는 2초, 각 시험 호출은 총 5초로 제한하며 진행률로 기한을 초기화하지 않습니다. 프로세스당 CPU는 3초, 파일 디스크립터는 32로 제한하고 직접 파일 출력은 금지하며 코어 덤프를 비활성화합니다. IPC 수신 큐는 최대 네 프레임, 송신 큐는 한 프레임이며 파이프 쓰기 제한은 2초입니다. stderr는 예산 0으로 직접 버리고 신뢰할 수 없는 콘텐츠는 전달하지 않습니다. 취소하면 출력 대상을 즉시 취소하고 2초 동안 협력을 기다린 뒤 프로세스와 파이프를 종료하고 회수합니다. Ack는 게시 허가가 아닙니다. 이 리소스 예산은 작은 시험의 제한이며 일반적인 대형 압축 파일의 성능을 약속하지 않습니다.

<a name="xdzip-platform-gates"></a>

## 플랫폼 조건

| 플랫폼 | 시험 정책과 실패 동작 |
| --- | --- |
| macOS | 고정된 `/usr/bin/sandbox-exec` deny-default 정책은 정확한 비공개 실행 복사본, 시스템 라이브러리/dyld 캐시, 루트 로더 읽기와 `/var` 메타데이터를 허용합니다. 사용자 파일, 쓰기, 네트워크, fork는 거부하며 환경을 비우고 stdio만 상속합니다. 네이티브 rlimit과 20 ms RSS 감시는 512 MiB 초과 또는 측정 실패 시 종료합니다. RSS는 표본 측정이므로 표본 사이의 짧은 피크가 한계를 넘을 수 있으며 이 인터페이스의 향후 정식 배포를 약속하지 않습니다. |
| Linux x86_64 / aarch64 | Landlock ABI ≥3은 모든 v3 파일 권한을 처리하며 쓰기 규칙 없이 실행 파일과 시스템 로더 라이브러리만 읽도록 허용합니다. `no_new_privs`와 아키텍처를 확인한 seccomp 시스템 호출 허용 목록은 Rust 표준 파이프에 필요한 readv/writev를 포함하고 socket, fork/clone, ptrace, io_uring 등을 거부합니다. CPU 제한 3초, 주소 공간 제한 512 MiB와 기타 rlimit이 적용됩니다. 규칙, ABI, seccomp 또는 필수 호출이 없으면 exec 전에 실패합니다. 이 플랫폼의 실제 결과는 네이티브 CI만 입증합니다. |
| Windows 및 기타 플랫폼 | 실행을 무조건 거부합니다. AppContainer, Job과 핸들 허용 목록은 구현 및 검증되지 않았으며 일반 하위 프로세스로 샌드박스를 대체하지 않습니다. |

Linux API 선택은 [커널 Landlock 문서](https://docs.kernel.org/userspace-api/landlock.html)와 UAPI를 따르며 향후 Windows 작업은 [AppContainer 격리](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation)를 따릅니다. 호스트의 네이티브 unsafe 코드는 샌드박스 모듈로 제한하고 구조, 호출 전제, 실패 동작을 문서화합니다. pre-exec에서는 할당이나 로그 기록을 하지 않습니다. 참조 상대는 명시적 감사 모드에서만 제어된 fork/_exit/waitpid 프로브를 추가하며 자식은 Rust 정리나 형식 로직을 실행하지 않습니다. 플랫폼 회귀는 먼저 자체 읽기/쓰기 가능 마커, 자체 localhost 리스너, self-exec을 양성 대조군으로 사용하고, 실제 마커, 파일, 연결 상태를 기준으로 커널의 읽기, 쓰기, 네트워크, 직접 fork, self-exec 거부를 검증합니다. 테스트하지 않은 플랫폼을 통과로 표시하지 않으며 데스크톱의 타사 실행은 계속 비활성화됩니다.

<a name="xdzip-owned-xdp1-example-format"></a>

## 자체 XDP1 예제 형식

XDP1은 상호 운용성 테스트 자료일 뿐 실제 압축 형식을 대체하지 않습니다. 4바이트 `XDP1` 다음에 `u32 LE` 레코드 수가 옵니다. 각 레코드에는 `u16 LE` UTF-8 이름 바이트 수, `u32 LE` 콘텐츠 바이트 수, 이름과 압축되지 않은 콘텐츠가 포함됩니다. 최대 1000개 레코드를 허용하고 후행 데이터는 허용하지 않으며 이름과 콘텐츠는 호스트 예산 내에서 읽습니다. 항목 토큰은 4바이트 LE 물리 순번이며 경로 기반 중복 제거를 하지 않습니다. Python 오류 모드는 적합성 테스트에서만 명시적으로 선택하여 무단 접근, 순서, 재생, 중복 식별 정보, 경로, 크레딧, 다이제스트, 충돌, 취소와 중복 종료 상태를 검증합니다.
