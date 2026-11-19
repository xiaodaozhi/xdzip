# 압축 파일 검사 및 작업 관리

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | [繁體中文（台灣）](../../zh-TW/user-guide/checks-and-tasks.md) | [日本語](../../ja-JP/user-guide/checks-and-tasks.md) | 한국어 | [Français](../../fr-FR/user-guide/checks-and-tasks.md) | [Deutsch](../../de-DE/user-guide/checks-and-tasks.md) | [Español](../../es-ES/user-guide/checks-and-tasks.md) | [Português (Brasil)](../../pt-BR/user-guide/checks-and-tasks.md) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-test-an-archive"></a>

## 압축 파일 검사

1. 압축 파일을 열고 도구 모음에서 **검사**를 선택하세요.
2. 암호 입력 창이 표시되면 암호를 입력하고 결과를 기다리세요. 압축을 푼 파일을 쓰지 않고 지원되는 내용을 검사합니다.
3. 실패하면 사유를 확인하세요. 대상 항목이 표시된 경우 해당 항목도 확인하세요. 일부 형식은 체크섬이나 인증을 제공하고 다른 형식은 구조와 읽기 가능 여부를 확인합니다. 성공한 검사는 해당 형식에서 지원하는 검사 범위에만 해당합니다. [무결성 검사 동작](../engineering/archive-checking.md)

검사할 수 없는 압축 파일도 있습니다. 디스크 인덱스를 사용하는 ZIP에서는 작업 제한 내에서 선택 항목을 풀 수 있습니다. NSIS는 설치 프로그램의 메타데이터만 표시합니다. [큰 압축 파일의 제한](../engineering/large-index.md)과 [지원 형식](../../../README.ko-KR.md#xdzip-format-support)을 참조하세요.

<a name="xdzip-progress-cancellation-and-results"></a>

## 진행 상황, 취소 및 결과

작업 메뉴에서 진행 중인 작업으로 돌아가거나 결과를 확인할 수 있습니다. 압축 파일 탭을 닫아도 백그라운드 작업은 계속됩니다. 작업을 계속하려면 암호를 입력하거나 파일명 충돌을 처리해야 할 수 있습니다.

작업 창에서 **취소**를 선택하면 중지를 요청합니다. 작업에 최종 상태가 표시될 때까지 기다리세요. 이미 저장이 확정된 파일은 남을 수 있습니다. 결과 상세 내용을 펼쳐서 완료, 건너뜀, 미완료 항목을 구분하세요. 표시된 개수와 사유를 보고 다시 시도할 대상을 정하세요. 완료된 작업의 기록을 지우면 작업 항목이 삭제되지만 디스크의 파일은 유지됩니다. [작업 창 동작](../ux/task-windows.md)
