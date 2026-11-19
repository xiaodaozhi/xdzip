# xdzip에서 만드는 Windows용 ZIP 자동 압축 풀기 파일

[English](../../engineering/sfx-creation.md) | [简体中文](../../zh-CN/engineering/sfx-creation.md) | [繁體中文（台灣）](../../zh-TW/engineering/sfx-creation.md) | [日本語](../../ja-JP/engineering/sfx-creation.md) | 한국어 | [Français](../../fr-FR/engineering/sfx-creation.md) | [Deutsch](../../de-DE/engineering/sfx-creation.md) | [Español](../../es-ES/engineering/sfx-creation.md) | [Português (Brasil)](../../pt-BR/engineering/sfx-creation.md) | [Русский](../../ru-RU/engineering/sfx-creation.md)

첫 번째 SFX 프로필은 Windows x64 전용이며, 만들 때 `.sfx.exe` 확장자를 지정합니다. 프로젝트에서 빌드한 스텁 `xdzip-sfx.exe`에 암호화되지 않은 단일 볼륨 Store 또는 Deflate ZIP을 넣습니다. 만드는 동안 스텁을 실행하지 않습니다. 일반 `.exe`는 만들기, 끌어다 놓기, 시스템 파일 연결 등록 범위에 포함되지 않습니다.

앱은 자체 폴더 또는 전용 `cli` 폴더의 상위 경로에서만 스텁을 읽습니다. 링크를 따라가지 않고 크기가 제한된 일반 파일을 열어 PE32+, x64, 섹션 범위, 프로젝트 버전 표식을 검증합니다. ZIP 종료 레코드의 주석에는 스텁과 ZIP의 길이 및 앞선 바이트의 SHA-256을 담은 64바이트 버전 봉투가 있습니다. 실행 중인 SFX는 이 봉투를 검증한 뒤 공통 Rust ZIP 및 압축 풀기 절차를 사용합니다. 손상이나 잘못된 봉투를 찾을 수 있으며, 게시자의 신원을 확인하거나 임의 실행 코드의 출처를 인증하지는 않습니다.

독립 스텁은 Qt나 xdzip 설치 없이 실행됩니다. Windows 대화 상자에서 기존 대상 폴더와 충돌 처리 정책을 선택하고, 진행 상황을 보거나 취소할 수 있습니다. 압축 작업은 UI 스레드 밖에서 Rust로 수행합니다. 기존 권한, 고정된 파일 식별 정보, 자원 제한, 파일별 검증, 임시 파일 쓰기, 충돌 처리 및 정리 절차를 재사용합니다. 취소하면 이미 저장이 확정된 파일이 남고 완료하지 못한 임시 출력은 삭제합니다. 결과와 관계없이 푼 파일을 열거나 실행하거나, 스크립트를 호출하거나, 소프트웨어를 설치하거나, 권한을 높이거나, 파일 연결을 바꾸지 않습니다.

자동화에는 쓰지 않고 검사하는 `--test`와 압축을 푸는 `--extract-to <existing-directory> --conflict skip|rename|overwrite`를 사용할 수 있습니다. 봉투가 잘못되었거나 지원하지 않거나 암호화된 ZIP 프로필은 출력 전에 거부합니다. 페이로드 암호, 분할 볼륨, 사용자 지정 실행 명령, 무인 설치, 실행 파일 자동 시작은 지원하지 않습니다. 생성된 파일은 서명되지 않습니다. 서명 및 게시자 신뢰도는 릴리스에서 별도로 다룹니다. [PE 및 Authenticode 규칙](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)

데스크톱 앱은 Qt Quick을 계속 사용합니다. 독립 SFX는 [ADR-013](../design/architecture-decisions.md#xdzip-adr-013-standalone-windows-sfx-host)에 명시된 제한된 예외에 따라 기본 시스템 대화 상자를 사용합니다.
