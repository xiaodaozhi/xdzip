<a name="xdzip-interface-contracts-and-checks"></a>

# 인터페이스 계약과 검사

[English](../../engineering/contracts.md) | [简体中文](../../zh-CN/engineering/contracts.md) | [繁體中文](../../zh-TW/engineering/contracts.md) | [日本語](../../ja-JP/engineering/contracts.md) | 한국어 | [Français](../../fr-FR/engineering/contracts.md) | [Deutsch](../../de-DE/engineering/contracts.md) | [Español](../../es-ES/engineering/contracts.md) | [Português (Brasil)](../../pt-BR/engineering/contracts.md) | [Русский](../../ru-RU/engineering/contracts.md)

공식 인터페이스는 [도메인과 데이터](../design/domain-data.md), [Qt/Rust 브리지](../design/qt-rust-bridge.md), [형식 확장 설계](../design/codec-extension-sdk.md), [작업 스케줄링](../design/task-scheduling.md)에 정의합니다. 이 페이지는 인터페이스를 확정하기 전에 실행할 검사를 정합니다. 내부 브리지, 두 종류의 핸들 및 수명 주기는 구현되었습니다. 범위는 [실행 환경 계약](product-runtime.md)을 참조하세요. 쿼리 세대, 소스 버전 검사, 생성 및 압축 풀기 작업도 구현되었습니다. [탐색](readonly-browser.md), [안전한 압축 풀기](zip-safe-extraction.md) 및 [생성 계약](zip-creation.md)을 참조하세요. 타사 프로세스 프로토콜은 별도의 [읽기 전용 SDK 시험](sdk-trial.md)을 제공합니다. 안정적인 SDK로 배포되거나 제품에 통합된 상태는 아닙니다. 표는 통합 전 확인할 사항을 설명합니다.

| 경계 | 구현 전 확인 | 검사할 실패 동작 |
| --- | --- | --- |
| UI → 브리지 | 핸들 소유권, RequestId, 비차단 제출, 종료 중 거부 | 가득 찬 큐, 반복 종료, 삭제된 객체 |
| 브리지 → UI | session/query/generation/revision, 페이지 배치, 이벤트 순서 | 오래된 응답 폐기, 작은 정수형으로 변환 실패, 최종 상태 보존 |
| 실행 환경 → 코덱 | 확장자가 아닌 기능, 입력 읽기, 출력 수신기, 취소 토큰 | 미지원 기능, CRC 오류, 짧은 읽기 또는 쓰기 |
| 실행 환경 → 안전한 입출력 | 승인된 루트, 매핑, 임시 출력, 커밋 허가 및 영수증 | 링크 경합, 디스크 공간 부족, 변경된 덮어쓰기 대상 |
| 확장 → 호스트 | 핸드셰이크 버전, 프레임 한도, 기능, 세션 수명 주기 | 잘못되거나 지나치게 큰 프레임, 시간 초과, 순서 변경, 취소 거부 |

각 인터페이스는 입력 유효성, 소유권, 스레드, 리소스 한도, 오류 범주, 멱등성 및 취소 경계를 정의합니다. 알 수 없는 값과 0은 구분합니다. TaskId와 EntryId는 각각의 용도가 있습니다. 선택 식별자는 항목에 연결되며 현재 행 번호에 의존하지 않습니다.

CLI와 GUI는 Rust 서비스를 공유합니다. Catalog는 페이지마다 최대 128개의 물리적 메타데이터 항목을 반환합니다. SelectMembers는 같은 불변 세션에서 전체 유효 경로와 EntryId를 확인한 뒤 기존 작업을 제출합니다. 목록 ID는 해당 스캔에만 유효하며 압축 파일이 바뀌면 유지되지 않습니다. submit_wait는 잠깐의 작업 허용 잠금만 기다립니다. 큐 용량과 종료 중 거부 규칙은 그대로입니다. GUI는 이 내부 명령의 QML 인코딩을 외부에 노출하지 않습니다. CLI는 이스케이프된 표시 문자열로 파일을 쓰지 않습니다.

고정 호스트 메타데이터는 open_regular_nofollow를 디렉터리 권한, 크기 및 식별 정보 검사와 함께 사용합니다. 경로 끝의 심볼릭 링크와 재분석 지점은 거부합니다. 사용자가 선택한 압축 파일을 읽을 때는 기존 경로 별칭 동작을 유지합니다. 데스크톱 CLI 요청은 같은 사용자 단일 인스턴스의 선택적 확장 `cli_protocol=1`로 제한된 요청 디렉터리를 보냅니다. 기존 열기 프로토콜은 버전 1을 유지합니다. 확장이 없는 인스턴스는 CLI 전달에 응답할 수 없습니다. 결과, 종료 코드 및 설치 범위는 [CLI 계약](command-line.md)을 참조하세요.

내부 계약은 구현과 함께 바뀔 수 있으며 같은 커밋에서 갱신합니다. 타사 통신 프로토콜은 독립 구현 두 개가 서로 동작하고 호환성 테스트를 통과해야 v1 안정 버전으로 부릅니다. 알 수 없는 버전은 명시적으로 거부하거나 협상 규칙에 따라 호환 버전의 사용을 요청합니다.
