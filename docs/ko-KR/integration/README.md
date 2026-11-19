# 스크립트에서 XDZip 사용하기

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | [繁體中文（台灣）](../../zh-TW/integration/README.md) | [日本語](../../ja-JP/integration/README.md) | 한국어 | [Français](../../fr-FR/integration/README.md) | [Deutsch](../../de-DE/integration/README.md) | [Español](../../es-ES/integration/README.md) | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

독립 CLI는 Qt나 데스크톱 세션 없이 동작합니다. [GitHub 릴리스](https://github.com/xiaodaozhi/xdzip/releases)에서 시스템에 맞는 `xdzip-cli-` 패키지를 선택하세요. 패키지 전체를 풀고 워커를 실행 파일 옆에 두세요. 압축을 푼 폴더에서 macOS 또는 Linux는 `./bin/xdzip --help`, PowerShell은 `.\bin\xdzip.exe --help`를 실행합니다.

예제는 PATH에 있는 `xdzip`을 사용합니다. 압축을 푼 CLI로 `setup`을 실행해 사용자용 실행기를 등록하고, 필요하면 명령 디렉터리를 PATH에 추가하세요. `xdzip --version-json`으로 찾은 프로그램이 `"component":"cli"`를 반환하는지 확인합니다. 데스크톱 실행기와 CLI의 명령 이름이 같을 수 있습니다. 충돌하면 압축을 푼 CLI의 경로를 명시하세요. [명령 설치 및 제거](../engineering/command-line.md#xdzip-build-and-command-installation)

<a name="xdzip-create-and-check"></a>

## 만들기 및 검사

새로 만든 쓰기 가능한 비공개 작업 디렉터리에 `hello.txt`가 담긴 `docs` 폴더를 만드세요. 두 번째 파일 `notes.txt`는 그 폴더 옆에 둡니다. 아래 명령은 ZIP을 만들고 기존 압축 파일 안의 `docs/` 폴더에 두 번째 파일을 추가한 뒤 결과를 검사합니다.

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --conflict skip --json
xdzip test sample.zip --json
```

[형식 표](../../../README.ko-KR.md#xdzip-format-support)에서 만들기 확장자와 옵션을 선택하세요. 출력은 입력 트리 밖에 두고 이미 존재하는 상위 출력 폴더를 사용하세요. `add`는 조건에 맞는 기존 압축 파일이 필요합니다. `--into`로 압축 파일 안에 이미 있는 폴더를 지정합니다. 생략하면 루트에 추가합니다. `test`는 푼 파일을 만들지 않습니다. 성공하면 `data.check_kind`가 검사 수준을 나타냅니다. [형식 목록](../engineering/format-registry.md)과 [무결성 검사](../engineering/archive-checking.md)도 참조하세요.

<a name="xdzip-list-select-and-extract"></a>

## 목록 표시, 선택 및 압축 풀기

같은 작업 디렉터리에서 위의 압축 파일을 계속 사용합니다. 새 대상을 사용하거나 기존 파일 처리 방법을 명시하세요.

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

멤버 경로는 정확히 일치해야 하고 대소문자를 구분하며 슬래시를 사용합니다. 같은 경로의 레코드가 여러 개면 `list`가 반환한 `id`로 선택하세요. ID는 1부터 시작합니다. 목록에서 대상 파일의 ID가 2이면 두 번째 명령의 `docs/hello.txt` 대신 `--entry-id 2`를 사용할 수 있습니다. 여러 멤버는 `--entry-id`를 반복 지정하세요. 멤버를 찾을 때는 `path`, 표시할 때는 `display_name`을 사용합니다. 압축 파일을 변경했다면 목록을 다시 조회하세요.

첫 페이지는 오프셋 0부터 시작합니다. 다음 페이지를 읽으려면 같은 `--limit`을 사용하고 `data.next_offset`을 `--offset`으로 전달하세요. `next_offset`이 null이면 끝입니다. 오프셋은 0부터 세는 행 위치입니다. ID는 물리 레코드를 식별합니다. 각 명령이 압축 파일을 다시 열므로 페이지를 읽는 동안 원본을 변경하지 마세요. [CLI 선택 및 목록 한도](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-run-without-prompts"></a>

## 묻지 않고 실행

`--json`은 대화형 질문을 끕니다. 무인 작업이 끝까지 실행되어야 한다면 충돌 정책을 명시하세요. `extract`, `test`, 암호화하는 `create` 등 암호를 지원하는 명령에서는 `--password-stdin`으로 전용 표준 입력 파이프에 UTF-8 암호 한 줄을 보내세요. 암호화해서 만들 때는 `--encrypt`도 필요합니다. 필요한 결정이나 암호가 없으면 CLI는 `interaction_required`를 반환합니다. [암호 및 형식 옵션](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-interpret-results"></a>

## 결과 해석

`--json`으로 압축 파일을 작업한 경우 stdout의 UTF-8 개체를 파싱하세요. `schema_version`, `status`, `error_code`, `data`, `summary`, `warnings`를 읽습니다. 진단을 위해 stderr를 사용할 수 있게 두세요. `--version-json`은 별도의 빌드 식별 개체를 반환합니다.

| 종료 코드 | 처리 방법 |
| --- | --- |
| 0 | 결과와 건너뜀 수를 읽습니다. 해당 명령에서 `handed_off` 상태는 데스크톱으로 전달했음을 나타냅니다. |
| 1 | 오류, 경고, 요약을 읽어 실패, 부분 완료 또는 정리 경고를 확인합니다. |
| 2 | 명령이나 인수를 수정합니다. |
| 3 | 보고된 대화형 작업, 권한 승인, 플랫폼 또는 설치 요건을 확인합니다. |
| 130 | 취소가 확인됐습니다. 저장이 확정된 파일이 있는지 확인합니다. |

알 수 없는 값은 null일 수 있습니다. 실패하거나 취소해도 앞서 저장이 확정된 파일은 남을 수 있습니다. 다시 시도하기 전에 `summary.committed_files` 필드가 있다면 확인하세요. 데스크톱 전달, 업데이트 처리 기록 및 전체 출력 규칙은 [CLI 참고 문서](../engineering/command-line.md)를 참조하세요.
