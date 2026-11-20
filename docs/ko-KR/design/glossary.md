<a name="xdzip-xdzip-terminology-and-implementation-limits"></a>

# XDZip 용어 및 구현 제한

[English](../../design/glossary.md) | [简体中文](../../zh-CN/design/glossary.md) | [繁體中文](../../zh-TW/design/glossary.md) | [日本語](../../ja-JP/design/glossary.md) | 한국어 | [Français](../../fr-FR/design/glossary.md) | [Deutsch](../../de-DE/design/glossary.md) | [Español](../../es-ES/design/glossary.md) | [Português (Brasil)](../../pt-BR/design/glossary.md) | [Русский](../../ru-RU/design/glossary.md)

<a name="xdzip-1-terminology"></a>

## 1. 용어

| 용어 | 정의 |
| --- | --- |
| 압축 파일 | 컨테이너 또는 지원하는 압축 스트림과 그 권한이 부여된 소스. |
| 코덱 | 형식을 감지, 스캔, 읽기 또는 생성하는 구현. |
| 세션 | 열린 소스, 백엔드, 인덱스 및 탐색 상태. 작업은 별도 수명 주기를 갖습니다. |
| 항목 | 실제 압축 파일 레코드. 디렉터리에 표시용 가상 노드가 있을 수도 있습니다. |
| SourceVersion | 허용된 단일 소스 파일이나 볼륨의 특성을 가능한 범위에서 확인한 식별 값. |
| QuerySnapshot | 세션, 쿼리, 세대 및 리비전으로 식별하는 변경 불가능한 현재 결과 순서. |
| 세대 | 이전 결과가 새 결과를 덮어쓰지 않도록 프런트엔드 요청을 교체하는 관계. |
| 준비된 작업 입력 | 실행할 소스, 선택 및 관련 옵션을 담은 한도 내 입력. |
| 작업 | 작업 큐에서 허용된 실행 요청의 수명 주기와 결과. |
| 권한 | 호스트가 보유한 파일 또는 디렉터리 접근 권한. 경로 문자열과 구분합니다. |
| 출력 수신기 | 사용량 측정, 한도 및 트랜잭션 제어를 받는 쓰기 인터페이스. |
| 작업 스레드 또는 프로세스 | 명령과 작업은 호스트 스레드에서 실행합니다. 압축 및 이미지 작업기는 별도 프로세스입니다. |
| 커밋 | 검증된 임시 출력을 권한이 부여된 파일 또는 압축 파일 작업을 통해 최종 저장하는 것. |
| 작업 스냅숏 | 상태, 진행률, 확인 요청 또는 결과의 최신 보고. 중간 보고는 합칠 수 있습니다. |
| 지원 기능 | 구현, 형식 기능, 플랫폼 및 정책을 모두 충족하는 기능. |
| 솔리드 압축 | 여러 파일이 압축 블록을 공유합니다. 일부만 읽어도 앞선 내용을 디코딩해야 할 수 있습니다. |
| 첫 표시 프레임 | 실제로 렌더링하여 표시한 프레임. ui_ready 로그로 대신할 수 없습니다. |

<a name="xdzip-2-units-and-values"></a>

## 2. 단위와 값

메모리 및 파일 크기는 이진 단위 KiB/MiB/GiB를 쓰며 1 MiB = 1024² 바이트입니다. 전송 및 자원 계산은 바이트를 정수로 유지합니다. 시간은 ms 또는 s로 표기하고 실행 경과 시간에는 단조 시계를 사용합니다. 파일의 타임스탬프는 가능한 경우 달력 표현과 소스 정밀도를 보존합니다. 처리량은 MiB/s로 표시하고 압축 입력, 디코딩 데이터 또는 출력 중 어느 데이터를 측정했는지 명시합니다.

CPU 100%는 논리 코어 하나를 완전히 사용하는 것을 뜻합니다. 프로세스 트리는 100%를 넘을 수 있습니다. PSS, RSS, USS 및 물리 사용량은 서로 다른 측정값입니다. 압축 다운로드 크기, 설치 크기 및 실행 파일 크기는 따로 기록합니다.

<a name="xdzip-3-where-to-find-current-limits"></a>

## 3. 현재 제한 찾기

| 주제 | 구현 참조 |
| --- | --- |
| 요청, 페이지 및 메시지 큐 한도 | [Qt/Rust 브리지](qt-rust-bridge.md) |
| 작업 허용, 진행 및 상태 코드 | [작업 스케줄링](task-scheduling.md) |
| 압축 풀기 한도, 파일 권한 및 최종 저장 | [파일 보안](file-security.md) |
| 메타데이터와 디스크 ZIP 인덱스 한도 | [대용량 압축 파일 인덱스](../engineering/large-index.md) |
| 소스 및 볼륨 한도 | [압축 파일 열기](../engineering/archive-opening.md) |
| 텍스트 및 이미지 미리보기 한도 | [내용 미리보기](../engineering/content-preview.md) |
| 작업 프로세스 프레임, 감독 및 자원 제한 | [압축 파일 작업 프로세스](../engineering/native-codecs.md) |
| 실험적 확장 프로토콜 | [SDK 시험](../engineering/sdk-trial.md) |
| 벤치마크 목표 | [성능 설계](performance.md) |

<a name="xdzip-4-versioning"></a>

## 4. 버전 관리

앱 버전, 내부 CXX 빌드 인터페이스, 외부 통신 프로토콜 및 영구 데이터 스키마는 별도의 버전 경계를 가집니다. 제품 버전만으로 SDK 프로토콜 버전을 알 수 없습니다. [코덱 확장 SDK 설계](codec-extension-sdk.md)를 참조하세요.
