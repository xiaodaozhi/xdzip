# 명령줄 런타임 계약

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | [繁體中文](../../zh-TW/engineering/command-line.md) | [日本語](../../ja-JP/engineering/command-line.md) | 한국어 | [Français](../../fr-FR/engineering/command-line.md) | [Deutsch](../../de-DE/engineering/command-line.md) | [Español](../../es-ES/engineering/command-line.md) | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

CLI는 독립 Rust 호스트로, 같은 `xdzip-runtime`을 통해 형식과 안전한 I/O 서비스를 호출합니다. Qt, QML 또는 화면 세션에 의존하지 않습니다. 7z, RAR, CAB, ISO와 UDF는 계속 동봉된 격리 작업 프로세스를 사용하며 PATH에서 다른 압축 풀기 도구를 찾지 않습니다. 플랫폼과 아키텍처는 [지원 정책](support-policy.md)을 따릅니다. 독립 Linux CLI에는 X11이 필요하지 않습니다. 실행에는 시스템 OpenSSL 3과 인증서가, 빌드에는 추가로 libssl-dev와 pkg-config가 필요합니다.

`v0.2.0`부터 공식 릴리스는 세 플랫폼용 독립 CLI 패키지를 제공하며 데스크톱 패키지에는 비공개 CLI가 포함됩니다. 이전 릴리스 파일은 소급하여 바뀌지 않습니다. 다운로드는 [릴리스](https://github.com/xiaodaozhi/xdzip/releases)를 참조하세요. 소스 빌드, 설치와 업데이트 경계는 아래 절과 [릴리스 프로토콜](release-protocol.md)에 설명합니다. 구현과 승인 진행 상황은 [#254](https://github.com/xiaodaozhi/xdzip/issues/254)에서 추적합니다.

## 사용과 출력

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

인수가 없거나 `--help`를 지정하면 도움말을 표시합니다. `version`, `--version`, `--version-json`은 `VERSION.json`과 빌드의 Git 상태에 따른 ID를 보고합니다. `--version-json`은 product, version, build, revision, dirty, component, channel, configuration, distribution, platform과 architecture를 포함하는 독립 객체를 출력합니다. 도움말과 메시지는 `--language zh-CN|en-US`로 선택합니다. 기본적으로 `LC_ALL`, `LC_MESSAGES`, `LANG`을 차례로 확인하며 선택된 값이 `zh`로 시작하면 중국어를 사용합니다. 공개 식별자와 오류 코드는 영어입니다. `l`, `x`, `c`, `a`, `t`는 각각 `list`, `extract`, `create`, `add`, `test`의 별칭입니다. 파일 인수를 그대로 전달하려면 `--`를 사용합니다.

모든 업무 명령은 `--json`을 받습니다. stdout은 고정 필드 schema_version(1), command, status, error_code, data, summary, warnings를 가진 JSON 객체를 정확히 하나 출력합니다. 알 수 없는 크기는 null이며 압축된 TAR 스트림을 공유하는 하위 항목도 포함됩니다. 원시 DEB ar 멤버는 정확한 압축 크기를 유지합니다. 목록 data에는 형식과 항목이 있으며 각 항목은 id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted, warning을 유지합니다. path는 고정된 유효 이름이며 잘못되었거나 해결되지 않은 이름은 null입니다. display_name은 이스케이프 표시 전용입니다. 작업 요약에는 실제 읽기, 디코딩, 검증, 확정, 건너뛰기, 디렉터리와 분할 개수가 포함되고 정리 경고와 불확실한 결과도 유지됩니다.

진행 상황, 오류와 대화형 질문은 stderr로 보냅니다. JSON, 비터미널 또는 `--non-interactive` 호출은 질문하지 않습니다. 암호나 충돌 결정이 없으면 대기 중 작업을 취소하고 정리를 기다린 뒤 interaction_required를 반환합니다. 종료 코드는 0이 성공 또는 명시적 전달, 1이 실패·부분 완료·정리 경고, 2가 잘못된 인수, 3이 상호작용·권한·설치 조건 부족, 130이 확인된 취소입니다. 명시적 건너뛰기는 실행된 정책이며 건너뛴 개수를 별도로 보고합니다. 취소가 수락되지 않았고 확정이 끝났다면 실제 완료 결과를 유지합니다.

<a name="xdzip-archive-behavior"></a>

## 압축 파일 동작

`list`는 물리적 레코드를 반환하고 중복을 유지합니다. `extract`는 슬래시를 사용하는 완전하고 정확하며 대소문자를 구분하는 유효 멤버 경로를 받습니다. 디렉터리는 재귀적으로 확장하고 루트 아래 계층을 유지합니다. 중복 경로는 `ambiguous_member`를 반환합니다. `list`가 반환하는 1부터 시작하는 숫자 ID를 `--entry-id`로 전달하고 여러 ID는 옵션을 반복합니다. 명시된 경로나 ID를 해결할 수 없으면 대상 디렉터리 생성 전에 실패합니다. 와일드카드, stdin에서 압축 파일 읽기, 추출 내용을 stdout으로 쓰기는 지원하지 않습니다.

`--limit` 또는 `--offset`을 지정하면 페이지를 나눕니다. 오프셋은 0부터 시작하며 최대 1,000,000입니다. `--limit`만 지정하면 오프셋은 0이고 `--offset`만 지정하면 한도는 128입니다. 유효한 한도는 1–4,096입니다. JSON `data`에는 `offset`, `total`, null일 수 있는 `next_offset`이 있습니다. `summary.entries`는 실제 반환한 행을 셉니다. 일반 단일 파일 ZIP/ZIP64는 100만 항목을 인덱싱할 수 있지만 페이지 없는 목록은 100,000개 초과를 거부합니다. 두 방식 모두 직렬화된 JSON 행과 렌더링 텍스트에 16 MiB 예산을 사용합니다. 한도 초과 시 부분 행 없이 `resource_limit`을 반환합니다. 페이지 경계는 물리 ID를 바꾸지 않습니다. [대형 압축 파일 인덱싱](large-index.md)을 참조하세요.

extract의 기본 대상은 현재 디렉터리입니다. `-C`가 없는 디렉터리를 지정하면 무조건 재귀 생성하지 않고 가장 가까운 기존 안전 부모부터 구성 요소별로 권한을 얻습니다. 대상, 원본 ID, 링크, Unicode·대소문자 충돌, 준비, 검증과 확정은 계속 [안전한 압축 풀기](zip-safe-extraction.md)와 [파일 이름 인코딩](filename-encoding.md)을 따릅니다. 이름 충돌 기본값은 Ask이며 `--conflict ask|skip|rename|overwrite`로 명시적으로 선택합니다. `--allow-high-ratio`는 높은 압축률만 확인하며 강제 리소스 한도를 제거하지 않습니다.

`create`는 출력 [접미사](format-registry.md)에서 프로필을 선택하며 대상 부모 디렉터리가 존재해야 합니다. `--level store|fast|normal|best`는 선택한 프로필에 맞는지 검증합니다. 일반 TAR과 ISO에는 수준을 생략하거나 `store`를 선택합니다. LHA(`.lzh` 또는 `.lha`)는 `fast`, `normal`, `best`에 LH5 수준 1, 6, 9를 사용합니다. ZIP과 7z는 `--encrypt`와 `--volume-size`를 지원하며 7z는 프로필에 따라 `--encrypt-names`와 `--solid`도 지원합니다. 분할 크기는 양의 정수 바이트 또는 `KiB`, `MiB`, `GiB` 접미사를 받습니다. 선택 프로필이 지원하지 않는 인수는 쓰기 전에 실패합니다.

암호는 숨겨진 터미널 입력 또는 `--password-stdin`의 UTF-8 한 줄로만 전달합니다. 줄 끝은 제거하지만 암호 자체를 자르지는 않습니다. 대화형 암호화 생성에는 두 번 확인이 필요합니다. 평문 암호 인수는 거부하며 환경 변수에서 암호를 읽거나 설정·로그에 쓰지 않습니다. 길이와 영점화는 [암호 계약](password-extraction.md)과 [생성 계약](zip-creation.md)을 따릅니다.

`add`는 호스트 업데이트를 제출하며 기본적으로 압축 파일 루트를 사용합니다. `--into`는 압축 파일 상대 이름으로 기존 안전 디렉터리를 선택합니다. 허용되는 입력은 항목을 추가하거나 대체할 수 있습니다. 내부 이동은 데스크톱 작업입니다. CLI는 허용 형식과 레코드, 입력 보호, 임시 재구축과 최종 대체에 관한 [압축 파일 수정](archive-transfers.md) 검사를 공유합니다. `test`는 멤버 파일을 만들지 않고 전체 압축 파일을 검증합니다. JSON `data.check_kind`는 [압축 파일 검사](archive-checking.md)에 정의된 완료 검사의 강도를 보고합니다.

각 CLI 작업은 독립 런타임을 소유하고 데스크톱 탐색 세션에 참여하지 않습니다. Ctrl+C와 종료 신호는 협력 취소를 요청하고 종결 상태, 정리와 런타임 종료를 기다립니다. 이전에 확정한 파일은 남습니다. 공유 서비스의 `submit_wait`는 짧은 상태 잠금만 순서대로 얻으며 큐 공간을 기다리거나 GUI의 비차단 승인을 바꾸지 않습니다. 카탈로그 페이지에는 물리 항목이 최대 128개입니다. SelectMembers의 개수와 이름에 대한 기존 한도는 유지되며 모든 요청은 불변 세션 ID에 결합됩니다.

<a name="xdzip-build-and-command-installation"></a>

## 빌드와 명령 설치

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

이 진입점은 저장소에서 고정한 Rust, CMake, 플랫폼 C++ 도구 체인과 기존 격리 작업 프로세스를 사용하며 Qt가 필요하지 않습니다. Windows 패키징에는 고정된 앱 로컬 VC++ 런타임과 원본 라이선스를 준비하는 기존 `prepare-windows-runtime.py`도 필요합니다. Qt 통합 빌드 진입점도 GUI 실행 파일, 앱 아이콘 또는 연결 진입점의 이름을 바꾸지 않고 비공개 cli 디렉터리에 CLI를 빌드합니다.

macOS에 포함된 CLI 실행 파일은 `Contents/Helpers/cli/`에 있습니다. 일반 컨텍스트 JSON은 `Contents/Resources/xdzip-cli-context.json`에 있으며 앱 리소스 서명으로 보호됩니다. CLI는 고정된 Contents/Helpers/cli 구조에서만 리소스를 찾고 스키마, 데스크톱 종류와 고정 GUI 상대 경로를 검증합니다. 표식이 없으면 포터블 ID로 취급하고 잘못된 표식은 거부합니다. 반복 빌드는 Helpers 아래 오래된 생성 표식을 제거합니다. 구조 오류를 해결하기 위해 서명 검증을 완화하지 마세요.

새 데스크톱 번들은 컨텍스트에 `worker_relative`도 선언하고 데스크톱 형식 작업 프로세스를 공유합니다. 플랫폼별 고정 상대 경로만 허용합니다. Windows는 인접 데스크톱 작업 프로세스, macOS는 `Contents/MacOS/xdzip-native-codec`, Linux는 빌드에 인접하거나 `bin/`에 배포된 작업 프로세스입니다. 프로세스가 없거나 링크이거나 예상과 다르면 PATH나 다른 복사본을 검색하지 않고 컨텍스트를 거부합니다. 이 필드가 없는 이전 컨텍스트와 독립·관리형 CLI 패키지는 인접 작업 프로세스 검색을 유지합니다. 개발자의 명시적 작업 프로세스 재지정은 계속 우선합니다. 배포된 검색은 `scripts/check-cli.py --default-worker`로 검증하며 일반 테스트 재지정만으로는 증명할 수 없습니다.

독립 패키지를 푼 뒤 bin/xdzip을 실행하고 Windows에서는 .exe를 붙입니다. `bin/xdzip setup`은 사용자 단위의 안정적인 진입점과 버전별 런타임 디렉터리를 명시적으로 설치합니다. Unix의 기본 진입점은 ~/.local/bin이고 Windows는 사용자 LocalAppData의 xdzip/cli/bin입니다. 관리 디렉터리는 플랫폼 사용자 데이터 위치를 사용합니다. `--bin-dir`과 `--install-dir`은 위치를 명시적으로 재지정합니다. 사용자가 직접 디렉터리를 PATH에 추가하며 도구는 셸 시작 파일을 수정하지 않습니다. 기존 진입점이나 등록 파일을 덮어쓰지 않습니다.

관리형 ID에는 유효한 installation.json이 필요합니다. 등록 파일 없이 이름만 versions인 디렉터리는 여전히 포터블입니다.

실행 중인 Windows 실행기를 제거할 때 소유한 복사본의 해시가 여전히 일치하면 같은 명령 디렉터리의 비공개 .xdzip-retired 하위 디렉터리로 옮깁니다. 등록은 즉시 제거하고 결과에 retained_launcher를 기록합니다. 나중에 명시적으로 실행한 setup은 제한된 접수 기록에 있는 복사본 중 해시가 여전히 일치하고 종료된 것만 정리합니다. 수정된 파일은 남기고 덮어쓰지 않습니다. 버전별 디렉터리에 남은 CLI로 setup을 다시 실행해 등록을 복원할 수 있습니다.

`distribution` 필드는 소스와 릴리스 ID를 구분합니다. 일반 소스 빌드는 변경 사항이 없고 Release 구성으로 빌드해도 확인과 다운로드만 지원합니다. 자동 대체에는 전용 Release 파이프라인이 표시한 distribution ID, 유효한 관리형 설치와 인증된 대상 서명이 모두 필요합니다.

Windows 데스크톱 설치 프로그램의 PATH 옵션은 기본적으로 선택되지 않으며 다른 xdzip 명령과의 충돌을 거부합니다. Mac/Linux에서는 데스크톱 패키지의 비공개 CLI로 setup을 실행할 수 있습니다. 이 경우 데스크톱 설치 안의 CLI만 등록하고 별도로 업데이트 가능한 코어를 밖으로 복사하지 않습니다. `setup --remove`는 이 도구가 소유하고 해시가 검증된 진입점과 등록 파일만 제거합니다. 버전별 디렉터리와 복구 자료는 남기며 사용자 입력과 아직 사용 중일 수 있는 런타임 디렉터리는 삭제하지 않습니다.

안정적인 실행기는 셸 이스케이프를 사용하지 않습니다. Unix에서는 실제 CLI를 exec하고 Windows에서는 콘솔 자식 프로세스를 기다리며 취소와 종료 상태를 전달합니다. 원자적인 current.json 포인터가 현재 런타임 디렉터리를 선택합니다. 최초 설치, 새 버전 자체 검사 또는 파일 확정이 실패하면 설치 성공 결과를 만들지 않아야 합니다.

## 업데이트, 데스크톱 전달과 복구

update는 현재 설치를 대상으로 합니다. 독립 CLI는 CLI만 업데이트하고 데스크톱 패키지에 포함된 CLI는 데스크톱 에디션 전체를 업데이트합니다. `--check`는 조회, `--download`는 다운로드와 검증, 기본 작업은 확인 후 설치, `--status`는 접수 기록과 실제 ID 검사를 수행합니다. `--yes`는 이 업데이트만 확인하며 시스템 권한을 대신하지 않습니다. 포터블, 변경된 상태, 소스 또는 ID를 알 수 없는 빌드는 확인과 다운로드만 허용합니다. 독립 자동 대체에는 관리형이며 변경 사항 없는 Release 설치가 필요합니다.

CLI는 고정된 공개 GitHub Releases, 채널과 단조 증가 빌드 번호 규칙을 사용합니다. 세 플랫폼의 CLI 파일은 별도의 cli-release.json과 cli-release.json.sig를 사용합니다. 내장 공개 키가 원시 바이트를 검증한 뒤 크기와 SHA-256을 확인합니다. 서명이 없거나 신뢰할 수 없는 키는 수동 획득 정보만 제공하고 잘못된 서명은 명시적으로 실패합니다. 여섯 패키지용 데스크톱 release.json에는 CLI 항목이 없으며 이전 클라이언트는 원래 프로토콜을 계속 사용합니다.

네트워크는 기본적으로 시스템 인증서와 표준 프록시 환경 변수를 사용합니다. `--proxy`는 자격 증명이 없는 HTTP/SOCKS5 프록시, `--no-proxy`는 직접 연결, `--mirror`는 자격 증명·쿼리·프래그먼트가 없는 HTTPS 접두사를 선택합니다. 원래 GitHub URL, 리디렉션 호스트, 응답 크기, 시간 제한과 다운로드 예산은 계속 제한됩니다. 네트워크, TLS, 오프라인, 속도 제한과 호환되는 새 버전이 없는 결과를 별도로 보고합니다. 자동 폴링이나 재시도는 없습니다.

독립 업데이트는 사용자 소유 비공개 디렉터리에 준비하고 같은 압축 파일 런타임으로 안전하게 풉니다. 고정된 전체 파일 목록과 각 파일 다이제스트, 제품, 구성 요소, 로컬 플랫폼·아키텍처와 서명된 매니페스트 채널을 검증하고 버전 포인터를 바꾸기 전에 새 버전의 `--version-json` 자체 검사를 실행합니다. 트랜잭션은 상호 배타적이며 접수 기록은 대상 ID에 결합됩니다. 실패 시 이전 포인터를 유지하거나 명시적으로 복구에 들어가며 자동 롤백을 꾸며내지 않습니다.

데스크톱 확인과 다운로드는 메인 프로그램의 비 GUI 설치 ID 진입점만 조회하며 QApplication을 만들거나 설치 접수 기록을 소비하지 않습니다. 설치 의도는 같은 사용자의 기존 단일 인스턴스 채널로 전달합니다. 제한된 요청 디렉터리와 후보 버전만 허용하고 임의 명령, 패키지 또는 신뢰 루트는 허용하지 않습니다. 요청·결과 파일은 Rust 네이티브 디렉터리 권한과 안전한 확정을 재사용합니다. 데스크톱 컨트롤러는 후보를 독립적으로 다시 검사하고 일치시키며 정보 창, 설치 프로그램, Sparkle과 모든 작업 종료 장벽을 재사용합니다. 데스크톱 CLI 작업도 설치 사용 중 보호를 유지하므로 설치가 이 작업을 파괴할 수 없습니다.

전달이 수락된 터미널은 handed_off만 보고합니다. 실제 설치와 복구는 데스크톱 접수 기록과 실제 실행 중인 ID에 달려 있으며 다운로드나 전달이 설치 완료를 의미하지 않습니다. 비대화형 모드는 GUI 또는 시스템 권한이 필요한 데스크톱 설치를 거부합니다. 설치 보안과 네이티브 승인 경계는 [업데이트 계약](../../engineering/software-update.md)을 참조하세요. 자동, 네이티브와 수동 승인은 별개입니다.

`.iso` 프로필은 수준 0의 비압축 ISO9660/Joliet 데이터 이미지를 만듭니다. `--level`을 생략하거나 `--level store`를 사용합니다. Windows x64 `.sfx.exe` 프로필은 동봉된 인접 `xdzip-sfx.exe`로 Store/Deflate ZIP을 감쌉니다. 암호화, 솔리드 모드와 분할을 거부합니다. 두 프로필 모두 기존 출력을 편집하지 못합니다. [ISO 생성](iso-creation.md)과 [자체 SFX 생성](sfx-creation.md)을 참조하세요.
