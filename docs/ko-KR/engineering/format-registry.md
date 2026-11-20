<a name="xdzip-built-in-format-declarations"></a>

# 내장 형식 선언

[English](../../engineering/format-registry.md) | [简体中文](../../zh-CN/engineering/format-registry.md) | [繁體中文](../../zh-TW/engineering/format-registry.md) | [日本語](../../ja-JP/engineering/format-registry.md) | 한국어 | [Français](../../fr-FR/engineering/format-registry.md) | [Deutsch](../../de-DE/engineering/format-registry.md) | [Español](../../es-ES/engineering/format-registry.md) | [Português (Brasil)](../../pt-BR/engineering/format-registry.md) | [Русский](../../ru-RU/engineering/format-registry.md)

내장 형식의 선언 원본은 [archive-formats.json](../../../specs/engineering/archive-formats.json)입니다. 안정적인 형식 식별자와 표시 이름, 컨테이너 계열, 외부 압축, 기존 전용 작업 프로세스 식별자, 지원하는 압축 방식 계열, 생성용 확장자, 검사 범위 및 진입점 후보를 기록합니다. 방식 계열은 각 형식 계약을 요약합니다. 실제 레코드, 호스트 정책, 플랫폼 기능을 확인해 작업을 시작하기 전에 사용할 수 있는 매개변수, 이전 변형 및 조합을 결정합니다.

검사 범위는 성공한 [무결성 검사](archive-checking.md)가 확인한 범위를 나타냅니다. `supported_content`는 지원하는 항목의 내용을 검사합니다. `structure_and_readability`는 구조와 읽기 가능 여부를 검사하며 내용 체크섬을 보장하지 않습니다. `format_declared_checks`는 스트림에 선언된 검사값을 확인합니다. 검사값이 없는 프로필도 이 범주에 포함됩니다. 검사를 실행할 수 있는지 여부와 검사 범위는 별개입니다. 예를 들어 NSIS 프로필은 제한 범위의 설치 프로그램 메타데이터를 인식하며 검사 작업은 지원하지 않습니다. 생성 방식 식별자는 각 형식이 허용하는 쓰기 방식 범위 안에서 해석합니다.

`creation_methods`는 호스트 전송 식별자를 방식 이름에 연결하고, `creation_levels`는 형식별 허용 수준을 정합니다. 생성된 GUI 선택 항목과 Rust 설명자에 두 정보가 모두 포함됩니다. CLI, GUI, 호스트는 동일한 선언으로 Deflate와 LZMA2 외의 방식도 확인합니다. Store와 Copy는 식별자 0, Deflate는 8, LZMA2는 32를 그대로 사용합니다. Zstandard의 호스트 식별자는 93입니다. LZ4의 256은 해당 컨테이너로 한정되며 ZIP 방식으로 해석하지 않습니다. 생성을 지원하지 않는 형식에는 생성 수준이 없습니다. 다시 쓰기 지원은 생성과 별개입니다. 생성기는 형식별 식별자가 없거나 모호할 때, 또는 수준이 유효하지 않을 때 거부합니다.

| 작업 | 내장 지원 범위 |
| --- | --- |
| 새 압축 파일 만들기 | ZIP, 7z, TAR, TAR.GZ (`.tar.gz`, `.tgz`), 독립 GZIP, XZ/TAR.XZ, LZ4/TAR.LZ4, Zstandard/TAR.ZST, LHA (`.lha`, `.lzh`), ISO9660/Joliet 및 Windows x64용 ZIP SFX 래퍼입니다. 독립 스트림은 일반 파일 하나를 받습니다. [생성](zip-creation.md)과 [단일 파일 스트림](single-file-streams.md)을 참조하세요. |
| 기존 압축 파일 다시 쓰기 | 조건을 충족하는 ZIP/ZIPX, 7z, 레벨 2 LHA/LZH, TAR, TAR.GZ, TAR.XZ, TAR.ZST, TAR.LZ4, TAR.BZ2, TAR.LZMA, TAR.LZ, TAR.Z, TAR.BR를 지원합니다. 암호화되거나 분할된 압축 파일, 지원하지 않는 레코드나 저장 방식을 쓰는 압축 파일은 제외합니다. 자세한 내용은 [수정 계약](archive-transfers.md)을 참조하세요. |
| 탐색, 압축 풀기, 항목 열기, 내보내기 또는 검사 | 작업마다 기능과 레코드를 별도로 검사합니다. 예를 들어 일부 설치 프로그램은 메타데이터만 확인할 수 있습니다. [읽기 전용 탐색](readonly-browser.md)과 [검사](archive-checking.md)를 참조하세요. |

사양을 변경한 후 `python3 scripts/generate-format-registry.py`를 실행하세요. 이 명령은 Rust 코덱 설명자, 끌어서 놓기 및 셸에 사용할 C++ 후보, JavaScript UI 선택 항목, 설치 프로그램 아이콘의 Python 매핑을 생성합니다. `--check`는 파일을 변경하지 않고 누락되었거나 오래된 출력을 검사합니다. `python3 scripts/test-format-registry.py`는 모순된 기능, 중복 식별자와 확장자, 생성 대상 언어에서 유효하지 않은 값, 오래된 사용 코드를 확인합니다. 일반 CI에서 두 검사 모두 실행합니다.

열기 작업은 먼저 [압축 파일 열기](archive-opening.md)의 진입점과 소스 규칙을 적용한 뒤 허용된 바이트를 탐지하고 검사합니다. 확장자나 MIME 유형으로 탐지를 시작할 수 있습니다. 호스트는 소스 식별자, 작업 매개변수, 출력 권한 및 최종 저장을 추가로 확인합니다. `worker_id`는 내장 읽기 경로를 지정합니다. ISO는 작업 프로세스에서 읽고, 새 ISO 이미지는 Rust 호스트에서 씁니다. 레지스트리는 새 압축 파일 생성과 기존 압축 파일 수정을 구분합니다.

필요에 따라 끌어서 놓기와 셸 메뉴 진입점을 구분합니다. 일반 `.gz`는 두 진입점과 단일 파일 생성을 지원합니다. `.tar.gz`와 `.tgz`는 TAR 컨테이너 동작을 유지합니다. XPI/APK/IPA/CBZ는 ZIP 검증과 허용된 프로필을 따릅니다. [단일 파일 스트림](single-file-streams.md)을 참조하세요. 분할 볼륨의 이름 표기와 소스 접근 권한은 [압축 파일 열기](archive-opening.md)를 따릅니다. 번호가 붙은 파일은 레지스트리에서 별도로 재해석하지 않습니다. Windows는 마지막 확장자로 아이콘을 연결하므로 `.tar.gz`는 `gz` 연결을 사용합니다.

구현되었고 자동 검증이 마련된 선언만 추가하세요. 소스를 변경하면 관련 [자체 형식 계약](../../engineering/independent-rust-codecs.md), [형식 작업 프로세스 계약](native-codecs.md), 사용자 기능 문서와 [인수 기준](../../../developers/qa/acceptance.md)을 함께 업데이트하세요. 계획과 진행 상황, 실행 결과는 이슈에 기록하고 생성된 테스트 증거는 `build/` 또는 CI 결과물에 보관하세요.

`creation_wrappers`는 Windows x64용 ZIP SFX 래퍼와 명시적 확장자 `.sfx.exe`를 선언합니다. ZIP 방식 식별자를 재사용하며 암호화와 분할 볼륨은 비활성화합니다. 일반 EXE 처리 기능은 포함되지 않습니다. ISO9660 생성은 호스트 방식 식별자 260과 고정된 수준 0을 사용합니다. [ISO 생성](iso-creation.md)과 [내장 SFX 생성](sfx-creation.md)을 참조하세요.
