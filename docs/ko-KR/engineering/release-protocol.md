<a name="xdzip-github-release-protocol-v1"></a>

# GitHub Release 프로토콜 v1

[English](../../engineering/release-protocol.md) | [简体中文](../../zh-CN/engineering/release-protocol.md) | [繁體中文](../../zh-TW/engineering/release-protocol.md) | [日本語](../../ja-JP/engineering/release-protocol.md) | 한국어 | [Français](../../fr-FR/engineering/release-protocol.md) | [Deutsch](../../de-DE/engineering/release-protocol.md) | [Español](../../es-ES/engineering/release-protocol.md) | [Português (Brasil)](../../pt-BR/engineering/release-protocol.md) | [Русский](../../ru-RU/engineering/release-protocol.md)

이 흐름은 GitHub 다운로드를 제공합니다. [Microsoft Store MSIX 흐름](../../../.github/workflows/windows-store-msix.yml)은 같은 버전의 소스를 Microsoft Partner Center용으로 패키징하며, Store 인증과 게시는 Partner Center에서 처리합니다.

전용 [Release 워크플로](../../../.github/workflows/release.yml)는 `workflow_dispatch`로 수동 실행합니다. 정식 실행은 `main`에서 기존 버전 태그로 시작하고 확인된 커밋을 고정합니다. 네 네이티브 작업은 각각 설정, 빌드, 테스트, 패키징을 한 번 수행합니다. macOS 작업은 첫 빌드부터 배포용 파일 연결 우선순위를 사용한 뒤 Qt를 배포하고 ad-hoc 무결성 서명을 적용하여 같은 앱에서 ZIP과 DMG를 만듭니다. 일반 macOS Qt CI는 SDK를 기준으로 검증합니다. Developer ID, Apple 공증 및 Windows 게시자 인증서는 이 릴리스 정책의 범위 밖입니다.

Intel 빌드는 `macos-x86_64` 앱 ZIP, DMG 및 CLI 압축 파일을 별도로 만듭니다. Intel 데스크톱과 CLI는 각각 `release-intel.json`과 `cli-release-intel.json`을 사용하며 서명과 `appcast-intel.xml`을 독립적으로 검증합니다. 이전 클라이언트와의 호환성을 위해 기존 ARM64 매니페스트와 피드 형식은 유지합니다. 릴리스 전에 두 아키텍처의 네이티브 패키지 검증이 모두 통과해야 합니다.

제품 소스는 태그가 가리키는 커밋에서 가져옵니다. 초안 작성, 공개 릴리스 및 공개 다운로드 검증은 모두 워크플로를 실행한 시점의 `main` 커밋에서 동일한 검토 완료 릴리스 노트를 읽습니다.

전체 태그는 VERSION.json에서 도출한 의미 버전 앞에 `v`를 붙입니다. `prerelease_number`는 선택적인 양의 정수이며 없으면 기존 빌드 번호와의 호환성을 유지합니다. alpha.1과 build 14는 서로 다른 식별값입니다. Windows 숫자 리소스와 macOS CFBundleVersion은 계속 단조 증가하는 빌드 번호를 사용합니다. 같은 버전의 공개 자산은 교체할 수 없으며 수정에는 새 버전이 필요합니다. 초안은 다시 실행할 수 있습니다.

릴리스 노트는 지속적으로 보존하고 검토할 수 있는 버전 입력입니다. 기본 경로는 `release-notes/<full-version>.md`이며 CLI에서 `--notes-file`로 검토된 파일을 지정할 수 있습니다. 파일에는 `<!-- xdzip-release {"version":"0.1.0-alpha.6","build":19} -->` 메타데이터 한 줄과 일치하는 버전 제목이 들어갑니다. 버전과 빌드 번호는 VERSION.json과 같아야 합니다. 먼저 `python3 scripts/publish-release.py --check-notes`를 실행합니다. 이 검사는 GitHub에 접속하지 않습니다. 전용 워크플로도 빌드 전에 같은 사전 검사를 수행합니다. 새 버전에는 새 노트가 필요하며 이전 기능 목록이나 승인 상태를 자동으로 상속하지 않습니다.

노트는 플랫폼별 승인 범위, 실제 커밋과 증거를 중국어와 영어로 기록합니다. 기존 수동 승인과 실행하지 않은 검사는 구분하여 보존하고 단계 진행은 계속 issue로 추적합니다. 초안 생성과 게시는 같은 노트 스냅샷을 사용합니다. 초안을 재실행할 때 본문이 검토된 파일과 다르면 자산을 업로드하거나 삭제하기 전에 중단하고 입력을 명시적으로 일치시켜야 합니다. 이미 공개된 버전은 자산을 교체하는 재실행을 계속 거부합니다. 공개 전 변경 사항은 [변경 기록의 미출시 섹션](../../../CHANGELOG.ko-KR.md#unreleased)에 기록하고 공개 후 해당 버전으로 옮깁니다. 모든 언어의 변경 기록을 함께 갱신합니다.

초안의 자산을 바꾸기 전 게시 도구가 GitHub 태그, 초안 상태와 채널 표시를 다시 읽습니다. stable은 사전 릴리스가 아니어야 하고 GitHub Latest가 되어야 합니다. alpha, beta, rc는 사전 릴리스로 유지하고 Latest를 교체해서는 안 됩니다. 게시 후와 공개 첨부 검증 후 각각 상태를 다시 확인하고 모두 일치할 때만 성공을 보고합니다. 게시 후 검사가 실패해도 릴리스는 이미 공개되었을 수 있습니다. 스크립트는 소유자의 처리를 위해 중단하며 공개 릴리스나 자산을 자동으로 철회, 수정 또는 교체하지 않습니다.

정식 릴리스 전 로컬 사전 검사와 해당 노트 검토를 마칩니다. 패키지 검증은 같은 네이티브 플랫폼에서 독립적으로 실행합니다. 같은 실행의 전달물이라도 저장소, 커밋, 플랫폼, 아키텍처와 압축 파일 SHA-256을 확인한 후에만 허용합니다. Windows와 Linux는 배포된 런타임을 한 번 검증하고 실제 설치 프로그램 수명 주기를 실행합니다. macOS는 실제 DMG와 앱 ZIP을 모두 검증합니다. 필요 시 후보 실행은 같은 작업을 수행한 뒤 조립된 Actions 아티팩트를 다운로드하여 체크섬을 확인합니다. 후보 경로에는 저장소 읽기 전용 권한만 사용하며 태그와 릴리스 서명 비밀이 필요하지 않습니다.

릴리스 계약 작업은 같은 커밋의 격리된 형식 작업 프로세스를 먼저 빌드하고 `XDZIP_NATIVE_CODEC`와 `XDZIP_NATIVE_CODEC_WORKER`를 사용해 Rust 워크스페이스 테스트를 실행합니다. 세 플랫폼의 네이티브 작업은 앱, 배포와 설치 수명 주기를 각각 검증합니다. macOS 개발용 연결 우선순위 테스트는 개발 메타데이터를 지정한 컴파일된 번들의 임시 복사본을 사용합니다. 배포 번들은 네이티브 실행과 패키징에 실제 배포 우선순위를 유지합니다.

Windows 배포 검증마다 별도의 임시 테스트 복사본을 사용하고 이전 로그를 보존합니다. 반복 검증에서 기존 디렉터리를 재사용하거나 삭제하지 않으며 제품 런타임 디렉터리에는 테스트 DLL이 없습니다.

릴리스에는 항상 `release.json`과 `SHA256SUMS`가 포함됩니다. 매니페스트 `schema_version=1`의 루트 필드는 product, version, channel, build, tag, commit(40자의 전체 SHA), repository, assets입니다. 각 자산에는 filename, platform, architecture, package_type, minimum_system, bytes, sha256 및 태그 다운로드 경로에 고정된 url이 포함됩니다.

| 플랫폼, 아키텍처 | 패키지 형식 | 최소 시스템 |
| --- | --- | --- |
| windows, x86_64 | zip, exe | 유지 관리 중인 Windows 11 |
| macos, arm64 | app.zip, dmg | macOS 14.0 |
| linux, x86_64 | tar.gz, deb | Ubuntu 24.04, glibc 2.39, X11 |

여섯 패키지 형식이 모두 필요합니다. 매니페스트 검증은 누락된 패키지, 중복 대상, 일치하지 않는 채널, 변경 가능한 다운로드 URL, 잘못된 크기나 해시를 거부합니다. 릴리스 첨부에는 플랫폼 고지, 라이선스, SBOM, 배포와 검증 자료가 포함되며 대응 소스 압축 파일은 포함되지 않습니다. Windows는 앱 로컬 VC++ 런타임을 사용하고 Windows SDK 그래픽 컴파일러 라이브러리를 배포하지 않습니다.

2026-11-05부터 소유자는 GitHub Releases와 Actions 아티팩트에서 이전 대응 소스 압축 파일을 제거하고 이후 릴리스 흐름에서 이를 생성하거나 업로드하지 않도록 지시했습니다. `scripts/prepare-source-delivery.py`는 로컬 검토용으로 유지합니다. 릴리스 조립과 게시는 파일 이름에 `corresponding-source`가 포함된 압축 파일을 거부합니다. 이 정책은 구성 요소 라이선스를 변경하지 않으며 대응 소스, 재링크 또는 다른 의무의 충족을 증명하지도 않습니다. 릴리스 검토는 실제 배포의 의무를 계속 해결해야 하며 필수 조건을 다른 방법으로 충족할 수 없으면 해당 바이너리 배포를 차단해야 합니다. GitHub가 공개 저장소의 태그에 대해 자동 생성하는 소스 스냅샷은 이러한 Release 첨부와 다릅니다.

일반 릴리스에서 소스 전달 수집기를 자체적으로 로컬 실행해서는 안 됩니다. 사용자가 소스 수집을 별도로 명시적으로 요청할 때만 실행하고 업로드 제한도 계속 준수합니다. 일반적인 자동 검토 표시는 실제 구성 요소와 구체적인 의무를 기준으로 평가하며 표시만으로 새로운 릴리스 결함을 확정하지 않습니다.

macOS 앱 ZIP, DMG와 고지 자료는 배포하고 서명한 동일한 임시 앱이 남아 있는 동안 내보내야 하며, 그 후에 준비된 `.app`과 임시 등록을 정리합니다. 정리 후 다른 앱을 다시 배포하여 ZIP을 채워서는 안 됩니다.

순서: 로컬 사전 검사, 버전 커밋 고정, 태그 생성, Release 워크플로 한 번 수동 실행, 세 플랫폼 빌드와 독립적인 시작 및 설치 수명 주기 검증, 초안 매니페스트 조립, 모든 조건을 충족한 뒤 채널별 게시, 모든 첨부 재다운로드 및 검증. stable은 명시적으로 Latest로 표시한 일반 Release이고 alpha, beta, rc는 Pre-release입니다. 어느 플랫폼이든 실패하면 초안을 공개하지 않습니다. 게시 전후 SHA256SUMS를 확인합니다. 출처·라이선스 검토나 수동 승인이 미완료이면 노트에 제한을 남기며 자동화로 그 결론을 대신하지 않습니다.

업데이트 클라이언트는 GitHub Releases를 페이지별로 조회하고 초안을 건너뛰며 매니페스트 버전을 검증한 뒤 의미 버전, 사용자가 허용한 채널, 플랫폼, 아키텍처와 패키지 형식으로 선택합니다. 게시 날짜로 업그레이드를 추론하거나 파일 이름을 추측하지 않습니다. 사전 릴리스는 이를 제외하는 `/releases/latest`를 사용하지 않습니다. stable은 stable만 받고, alpha는 alpha·beta·rc·stable을 허용하며, beta는 alpha로 돌아가지 않고, rc는 rc와 stable만 받습니다. 개발 버전 비교에는 dev를 지원합니다. 의미 버전과 단조 증가하는 빌드 번호가 모두 높아야 합니다. 다운로드 후 크기와 SHA-256을 확인하며 값이 없거나 다르면 거부합니다.

<a name="xdzip-trusted-update-attachments"></a>

## 신뢰할 수 있는 업데이트 첨부

새 릴리스에는 schema_version=1, algorithm=ed25519, key_id와 128자의 소문자 16진수 서명을 담은 `release.json.sig`가 포함됩니다. 서명은 `release.json`의 정확한 원시 바이트를 대상으로 하므로 검증 전에 파싱하고 다시 직렬화하지 않습니다. 클라이언트와 Qt 없는 설치 도우미는 각각 내장된 `resources/update-trust.json` 공개키 집합을 사용하며 다운로드한 매니페스트나 사용자 요청의 신뢰 루트는 허용하지 않습니다. 설치 시 알 수 없는 공개키, 중복 key_id, 취약한 키와 빈 신뢰 집합을 거부합니다.

macOS 네이티브 작업은 앱 ZIP을 패키징하고 검증합니다. 이어 Linux `sign` 작업은 검증된 바이트를 받아 `scripts/sign-macos-update.py`를 실행하고 `xdzip-update-tool sign-archive`를 호출하여 `macos-update.json`을 만듭니다. 이 파일에는 schema_version, version, build, filename, bytes, sha256 및 Sparkle base64 `ed_signature`가 기록됩니다. 조립 단계가 압축 파일과 서명을 다시 확인하고 같은 불변 후보만 포함한 `appcast.xml`을 생성합니다. 선택적 매니페스트 필드 `macos_appcast_url`은 같은 태그 아래의 해당 파일에 고정됩니다. 피드는 build를 Sparkle 버전으로, 전체 의미 버전을 shortVersionString으로, macOS 14.0을 최소 버전으로 사용하며 간단한 노트만 포함합니다. 전체 노트는 GitHub Release 본문에 남고 클라이언트는 외부 노트를 가져오지 않습니다.

압축 파일과 피드는 별도의 Sparkle 키를 사용하며 피드 자체도 서명해야 합니다. 크로스 플랫폼 릴리스 도구는 [Sparkle 원본 서명 형식](https://github.com/sparkle-project/Sparkle/blob/2.x/common_cli/Signing.swift)에 따라 전체 원시 XML에 서명하고 base64 edSignature와 원시 바이트 길이가 든 sparkle-signatures 주석 블록을 추가합니다. 첨부 파일만 서명하는 방식이 아닙니다. 생성 후 독립적으로 검증하며 바이트가 바뀌면 다시 서명해야 합니다. Mac 패키지의 SUPublicEDKey는 `resources/sparkle-update-key.json`의 공개키와 일치해야 합니다.

릴리스 환경은 기존 키와 공개키 ID를 `XDZIP_UPDATE_SIGNING_KEY_HEX`, `XDZIP_UPDATE_KEY_ID`, `XDZIP_SPARKLE_SIGNING_KEY_HEX`로 제공합니다. 소유자는 클라이언트와 설치 도우미에 버전 관리되는 두 생산 공개키 리소스를 제공하며 테스트 공개키를 내장하거나 빌드 시 개인키를 생성하지 않습니다. 소유자는 먼저 공개 신뢰 루트를 클라이언트 버전에 배포한 후 신뢰할 수 있는 릴리스 환경에 일치하는 개인키를 설정합니다. 키 교체 시 서명 키를 바꾸기 전에 새 공개키를 배포해 기존 클라이언트가 검증할 수 있게 합니다. 공개키가 내장되지 않은 이전 클라이언트는 먼저 해당 키가 포함된 버전을 수동 설치해야 하며 다운로드한 매니페스트로 신뢰 루트를 추가할 수 없습니다. 기존 공개 릴리스는 사후 서명하거나 교체하지 않습니다.

릴리스 스크립트는 두 개인키 환경 변수를 제거한 자식 프로세스에서 서명 도구를 빌드하며 컴파일된 도구 프로세스만 개인 서명 키를 받습니다. Cargo 빌드 스크립트, 컴파일러와 의존성 해석에는 서명 권한을 주지 않습니다.

마지막 서명 작업은 검증된 압축 파일과 업데이트 매니페스트를 `release-signing`에서 서명합니다. 별도의 게시 작업도 이 환경을 필요로 하며 릴리스를 공개하기 전에 서명과 초안 다운로드를 검증합니다. 마지막 읽기 전용 작업은 공개 릴리스를 다운로드해 서명된 자산과 비교합니다. 모든 작업은 고정된 커밋과 같은 실행의 아티팩트를 사용합니다.

[생산 업데이트 승인](../../../.github/workflows/production-update-acceptance.yml)은 `main`과 `contents: read`에 고정된 별도의 수동 진입점이며 동일한 `release-signing` 환경과 소유자 승인을 사용합니다. Release를 실행하거나 태그, 초안 또는 릴리스를 만들지 않습니다. 먼저 비밀 없이 도구를 컴파일하고, 기존 개인키에서 공개키를 독립적으로 도출해 현재 커밋의 두 공개키 및 공개키 ID와 엄격히 비교합니다. 항목이 존재한다는 사실만으로 일치를 증명하지 않습니다. 세 플랫폼의 비공개 이전 빌드 200과 새 빌드 201은 생산 공개키를 유지하고 비밀 없이 빌드합니다. 서명은 이 소스 커밋, 비공개 제품 식별값 및 해시에 완전히 묶인 고정 후보만 허용합니다. 이후 네이티브 업그레이드, 실패 복구와 일반 재시작은 비밀 없는 프로세스에서 실행합니다. 임의의 입력 매니페스트, 자산, 원격 아티팩트 또는 릴리스 매개변수는 없습니다.

비밀 없는 macOS 도구 체인 준비 중 `rust-toolchain.toml`에 선언된 `rustfmt`와 `clippy`를 한 번 설치한 뒤 압축 파일 허용 테스트를 실행합니다. 테스트 키 업데이트 회귀도 생산 개인키를 사용하거나 검증하지 않고 준비와 허용 범위를 검사합니다.

생산 승인 패키지, 원본 매니페스트, 서명, appcast와 앱은 일회용 러너에 남으며 업로드하지 않습니다. 제한된 수집기는 명시적으로 허용된 로그, 상태, 식별값과 해시 기록만 업로드하고 결과 위치에 서명된 첨부가 있으면 거부합니다. 생산 키 일치, 서명 검증, 실제 설치와 수동 데스크톱 승인을 각각 기록합니다. macOS는 네이티브 Apple Silicon과 Intel과 실제 `SUPublicEDKey` 검사를 사용하며 Windows Server 테스트는 Windows 11 수동 승인을 대신하지 않습니다. 이 흐름은 GitHub Secret 값을 로컬에서 읽지 않으며 릴리스를 승인하지도 않습니다.

`prepare-release.py --require-update-signature`는 키, 서명 또는 네이티브 압축 파일 메타데이터가 하나라도 없으면 중단합니다. `publish-release.py`는 업로드 전에 매니페스트, 같은 후보의 피드와 압축 파일을 독립적으로 다시 검증합니다. 서명, appcast와 상태 첨부는 모두 SHA256SUMS에 포함됩니다. 이미 공개된 릴리스에 이러한 첨부를 추가하거나 교체하지 않습니다.

<a name="xdzip-independent-cli-attachments"></a>

## 독립 CLI 첨부

v0.2.0부터 완전한 릴리스에는 `xdzip-cli-<version>-windows-x86_64.zip`, `xdzip-cli-<version>-macos-arm64.tar.gz`, `xdzip-cli-<version>-linux-x86_64.tar.gz`도 제공됩니다. 세 파일은 모두 같은 커밋의 네이티브 CLI 빌드와 검증에서 나오며 작업 프로세스, 실제 런타임 의존성과 고지를 보존합니다. 식별값은 변경 없는 Release 빌드여야 합니다. 일반 CI에 Mac 앱 패키지를 배포하거나 업로드할 권한이 추가되는 것은 아닙니다.

CLI는 `cli-release.json`과 `cli-release.json.sig`를 사용합니다. 매니페스트는 schema_version=1, product=xdzip, component=cli입니다. version, channel, build, tag, commit, repository는 데스크톱 매니페스트와 정확히 일치합니다. 자산에는 이 세 대상만 들어가며 데스크톱 자산과 같은 필드를 가집니다. Mac 패키지 형식은 tar.gz이고 Linux 최소 시스템은 display_server를 생략합니다. 그 밖의 시스템, 아키텍처, 크기와 해시 요구사항은 유지합니다. 고정 게시자는 계속 xiaodaozhi/xdzip입니다.

데스크톱 release.json에는 component나 CLI 항목 없이 여섯 패키지 형식만 계속 포함됩니다. 이전 클라이언트는 새 독립 첨부를 무시합니다. CLI 매니페스트 서명은 내장 Ed25519 신뢰 루트를 재사용하며 sign-cli와 verify-cli가 정확한 원시 바이트를 각각 서명하고 독립적으로 검증합니다. 서명, 식별값, 세 CLI 패키지와 모든 체크섬이 완료된 후에만 전체 조립물을 공개할 수 있습니다. 기존 공개 릴리스에는 사후 추가, 변경 또는 교체를 하지 않습니다.
