<p align="center">
  <img src="design/assets/brand/xdzip-128.png" width="80" height="80" alt="xdzip 앱 아이콘">
</p>

<h1 align="center">xdzip</h1>

<p align="center"><a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja-JP.md">日本語</a> | 한국어 | <a href="README.fr-FR.md">Français</a> | <a href="README.de-DE.md">Deutsch</a> | <a href="README.es-ES.md">Español</a> | <a href="README.pt-BR.md">Português (Brasil)</a> | <a href="README.ru-RU.md">Русский</a></p>

<p align="center"><strong>압축과 압축 풀기를 조금 더 쉽게</strong></p>

<p align="center">xdzip은 오픈 소스 크로스 플랫폼 압축 파일 관리자입니다. Rust로 구현한 형식은 외부 압축 풀기 프로그램 없이 동작합니다. 압축 파일을 탐색하고 검색하거나, 선택한 파일만 풀거나, 암호화 또는 분할 압축 파일을 열거나, 지원하는 형식의 항목을 변경할 수 있습니다.<br>Windows, macOS, Linux에서 ZIP, 7z, RAR, TAR, CAB, ISO/UDF 등을 지원합니다.</p>

<p align="center">
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml/badge.svg?branch=main" alt="지속적 통합"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="Apache License 2.0"></a>
</p>

<p align="center">
  <a href="#xdzip-download-and-install">다운로드</a> / <a href="#xdzip-using-xdzip">사용법</a> / <a href="#xdzip-format-support">지원 형식</a> / <a href="#xdzip-build-from-source">소스에서 빌드</a> / <a href="docs/ko-KR/README.md">문서</a> / <a href="https://github.com/xiaodaozhi/xdzip/issues">의견 보내기</a>
</p>

![xdzip으로 압축 파일 탐색](docs/images/main-window.png)

<a name="xdzip-download-and-install"></a>

## 다운로드 및 설치

Windows 11 x64에서는 Microsoft Store에서 **XDZip**을 검색하세요. 직접 다운로드하거나 독립 CLI를 받으려면 [GitHub의 최신 안정 버전](https://github.com/xiaodaozhi/xdzip/releases/latest)에서 사용하는 시스템을 선택하세요.

| 시스템 | 지원 범위 | GitHub 다운로드 |
| --- | --- | --- |
| Windows | 유지보수 중인 Windows 11 x86_64 버전 | 사용자별 `setup.exe` 또는 포터블 `.zip` |
| macOS | macOS 14 이상, Apple Silicon | `.dmg` 또는 `.app.zip` 앱 번들 |
| Linux | Ubuntu 24.04 LTS, x86_64, X11, glibc 2.39 이상 | `.deb` 패키지 또는 `.tar.gz` 런타임 디렉터리 |

데스크톱 배포 파일 이름은 `xdzip-<version>-`로 시작합니다. 독립 [CLI](#xdzip-command-line) 배포 파일은 `xdzip-cli-<version>-`로 시작합니다. 직접 내려받은 파일은 릴리스 페이지의 `SHA256SUMS`로 확인하세요. GitHub의 Windows 설치 프로그램은 서명되지 않았습니다. macOS 다운로드는 무결성 확인용 임시 서명을 사용하며 공증되지 않았습니다. 패키지별 실행 방법과 파일 연결 설정은 [설치 안내](docs/ko-KR/user-guide/install.md)를 참조하세요.

Microsoft Store는 GitHub와 독립적으로 해당 버전을 게시하고 업데이트를 관리합니다. GitHub에서 받은 버전은 **xdzip 정보**에서 업데이트를 확인하세요. 조건을 충족하는 Windows 사용자별 설치, macOS에서 응용 프로그램 폴더에 설치한 앱, Ubuntu DEB 설치는 앱 안에서 업데이트할 수 있습니다. 포터블 버전은 수동으로 업그레이드하세요. [설정 및 업데이트](docs/ko-KR/user-guide/settings-and-updates.md)와 [지원 정책](docs/ko-KR/engineering/support-policy.md)을 참조하세요.

**버전 안내.** 이 페이지의 기능은 현재 소스 트리를 기준으로 합니다. 배포 패키지는 해당 버전의 릴리스 노트를 확인하세요. 소스에서 빌드한 결과는 실험용 개발 버전입니다.

<a name="xdzip-using-xdzip"></a>

## xdzip 사용하기

압축 파일을 열면 내부 폴더를 탐색하고, 항목을 검색하거나 정렬하며, 풀 항목을 선택할 수 있습니다. 어느 순서로 정렬해도 폴더가 먼저 표시됩니다. 목록과 격자 보기, 파일 속성, 설정의 언어 선택, 밝은 테마와 어두운 테마, 글자 크기와 표시 밀도 조절을 지원합니다. 기본값은 지원하는 시스템 언어를 따르며, 지원하지 않는 언어에는 미국 영어를 사용합니다. 일반적인 단일 파일 ZIP/ZIP64는 탐색용으로 최대 100만 개의 레코드를 색인할 수 있습니다. 다른 형식은 대부분 한도가 더 낮습니다. 압축 풀기, 전체 압축 파일 검사, 수정에는 각각 별도 한도가 있습니다. [사용 설명서](docs/ko-KR/user-guide/README.md)와 [대용량 압축 파일 색인](docs/ko-KR/engineering/large-index.md)을 참조하세요.

- **내용 미리보기.** 속성 카드가 표시된 상태에서 파일 하나를 선택하면 지원하는 일반 텍스트, 래스터 이미지 또는 정적 SVG를 자동으로 미리 볼 수 있습니다. macOS는 HEIC와 HEIF도 지원합니다. 내용을 모두 읽고 형식에서 제공하는 검증을 마친 뒤 미리보기를 표시합니다. 도중에 취소할 수 있습니다. 형식, 크기, 디코딩 한도는 [미리보기](docs/ko-KR/engineering/content-preview.md)를 참조하세요.
- **압축 풀기와 열기.** 대상 및 이름 충돌 처리 규칙을 선택하고, 진행 상황을 확인하며, 필요한 경우 취소를 요청할 수 있습니다. 일반 파일을 두 번 클릭하면 전용 임시 폴더에 복사본을 푼 뒤 시스템에서 엽니다. 복사본을 수정해도 원래 압축 파일에 반영되지 않습니다. 캐시에 저장된 복사본은 7일이 지난 뒤 다음 파일 열기 작업 중에 정리합니다.
- **만들기와 수정.** ZIP, TAR, TAR.GZ, GZIP, XZ, TAR.XZ, LZ4, TAR.LZ4, Zstandard, TAR.ZST, LZH, 7z 압축 파일과 ISO9660/Joliet 데이터 이미지를 만들 수 있습니다. Windows x64에서는 ZIP 자동 압축 풀기 파일도 만들 수 있습니다. 조건을 충족하는 일반 단일 파일 압축 파일에서 항목을 추가하거나 교체하고, 이동하거나 이름을 바꾸고 삭제할 수 있습니다. 목록에서 파일을 끌어내거나, 파일을 목록으로 끌어다 놓은 뒤 추가를 확인할 수 있습니다. 형식과 한도는 아래 표를 참조하세요.
- **검사.** 파일을 쓰지 않고 전체 압축 파일을 백그라운드에서 검사합니다. 검사 항목은 형식에 따라 다릅니다. TAR, ISO, UDF, Compound File (MSI) 등은 파일별 내용 체크섬 없이 구조와 읽기 가능 여부를 확인합니다. [압축 파일 무결성 검사](docs/ko-KR/engineering/archive-checking.md)를 참조하세요.
- **여러 압축 파일.** 기본적으로 새 탭에서 열리며 새 창에서 열리도록 설정할 수도 있습니다. 한 로그인 세션은 앱 프로세스와 트레이를 공유합니다. 탐색 화면을 닫은 뒤에도 작업 목록에서 백그라운드 작업을 확인할 수 있습니다.
- **파일 관리자에서 실행.** Windows 11 설치 버전에서는 **더 많은 옵션 표시**에서 클래식 메뉴를 사용하세요. Store 버전은 최신 메뉴에도 표시될 수 있습니다. Ubuntu에서는 확장을 설치한 파일 앱(Nautilus)을 사용하세요. macOS에서는 시스템 설정에서 Finder 확장을 켜거나 등록된 Finder 서비스를 사용하세요. [파일 관리자에서 압축 풀기](docs/ko-KR/user-guide/extract.md#xdzip-from-the-file-manager)를 참조하세요.

압축 파일을 만들거나 파일을 추가할 때 `.DS_Store` 파일과 `__MACOSX`, `_MACOSX` 폴더를 제외합니다. 압축을 풀 때 경로와 대상에 대한 권한을 확인하고, 이름 충돌을 처리하며, 결과를 임시 위치에 쓴 뒤 확정합니다. 지원하는 형식에서는 내용도 검증합니다. 특수 개체와 링크의 처리 방식은 형식과 플랫폼에 따라 다릅니다. [안전한 압축 풀기 규칙](docs/ko-KR/engineering/zip-safe-extraction.md)을 참조하세요.

<a name="xdzip-format-support"></a>

## 지원 형식

아래 표에서 xdzip이 형식을 열거나 만들거나 수정할 수 있는지 확인하세요. ‘수정’은 기존 압축 파일에 항목을 추가, 교체, 이동하거나 이름을 바꾸거나 삭제하는 것을 말합니다. 읽기 및 검사 지원 여부는 실제 내용, 압축 방식, 레코드 유형에 따라 다릅니다. 확장자로 파일을 열면 형식 식별이 시작됩니다. 세부 한도는 [내장 형식 목록](docs/ko-KR/engineering/format-registry.md)과 [무결성 검사 안내](docs/ko-KR/user-guide/checks-and-tasks.md)를 참조하세요.

| 형식 | 탐색, 압축 풀기 및 검사 | 만들기 | 수정 |
| --- | --- | --- | --- |
| ZIP 및 ZIPX/XPI/APK/IPA/CBZ 후보 | Store, Deflate, Deflate64, BZip2, LZMA, XZ 및 지원되는 AES-256과 분할 프로필 | Store, Deflate, AES-256, 번호를 붙인 볼륨을 사용하는 ZIP | 표의 방식을 사용하는 적격 ZIP/ZIPX 압축 파일 |
| TAR, TAR.GZ (`.tgz` 포함) | 일반 파일과 폴더 | 지원. TAR는 압축 없이 파일을 묶음 | 조건에 맞는 일반 단일 파일 압축 파일 |
| GZIP (`.gz`) | 연결된 GZIP 멤버를 포함하는 논리 파일 하나. GZIP에 들어 있는 TAR는 TAR 트리로 표시 | 일반 파일 하나, Deflate 0/1/6/9 | 지원하지 않음 |
| BZip2, LZMA Alone, Lzip v1 | 스트림과 메모리 한도 내 일반 파일 하나 | 지원하지 않음 | 지원하지 않음 |
| TAR.BZ2, TAR.LZMA, TAR.LZ | 일반 파일과 폴더 | 지원하지 않음 | 조건을 충족하는 압축 파일 |
| XZ, TAR.XZ (`.txz` 포함) | 선언된 검사, LZMA2, Delta/BCJ 및 연결된 스트림 | LZMA2 레벨 1/6/9, XZ는 일반 파일 하나 | TAR.XZ |
| LZ4, TAR.LZ4 | 표준 프레임, 독립 또는 연결 블록, 프레임 연결과 선언된 검사 | LZ4 레벨 1/6/9, 일반 LZ4는 일반 파일 하나 | TAR.LZ4 |
| Zstandard, TAR.ZST (`.tzst` 포함) | 표준 프레임, 허용된 윈도, Huffman/FSE, 반복 오프셋과 선언된 검사 | Zstandard 레벨 1/6/9, 일반 Zstandard는 일반 파일 하나 | TAR.ZST |
| 7z | 암호, 암호화된 파일 이름, 솔리드 압축 파일과 번호가 붙은 볼륨 | 암호화, 솔리드 압축 파일과 번호가 붙은 볼륨 | 조건에 맞는 암호화되지 않은 비분할 압축 파일 |
| DEB | 일반 ar 멤버, 제어와 데이터 TAR 보기, 허용된 GZIP/XZ/Zstandard/BZip2/LZMA 프로필 | 지원하지 않음 | 지원하지 않음 |
| LHA/LZH | 레벨 0–3 헤더, Store, LH1–LH7, LZS/LZ5와 선언된 CRC | LH5 레벨 1/6/9, 레벨 2 헤더, 일반 파일과 폴더 | 항목 CRC가 있는 적격 레벨 2 압축 파일 |
| [PMA](docs/ko-KR/engineering/pma-streams.md) | 레벨 0 PM0/PM1/PM2, 정확한 CP/M 구조와 선언된 CRC | 지원하지 않음 | 지원하지 않음 |
| [ARJ](docs/ko-KR/engineering/arj-streams.md) | Store 및 방식 1–4, 검증된 헤더와 멤버 전체의 CRC-32 | 지원하지 않음 | 지원하지 않음 |
| [ARC](docs/ko-KR/engineering/arc-streams.md) | 구형 및 신형 Store와 방식 3–9, 정확한 프레이밍과 멤버 전체의 CRC-16 | 지원하지 않음 | 지원하지 않음 |
| [BH](docs/ko-KR/engineering/bh-containers.md) | `BH 05 07` 일반 멤버, 방식 8 raw Deflate, 정확한 크기와 CRC-32 | 지원하지 않음 | 지원하지 않음 |
| [ALZ](docs/ko-KR/engineering/ace-alz-egg-format-sources.md) | Store 또는 raw Deflate를 사용하는 일반 단일 볼륨 멤버, 정확한 크기와 CRC-32 | 지원하지 않음 | 지원하지 않음 |
| [EGG](docs/ko-KR/engineering/ace-alz-egg-format-sources.md) | Store 또는 raw Deflate를 사용하는 암호화되지 않은 비솔리드 단일 볼륨 멤버, 정확한 크기와 CRC-32 | 지원하지 않음 | 지원하지 않음 |
| [PEA](docs/ko-KR/engineering/disk-image-archive-format-sources.md) | PEA 1.x 단일 볼륨, UTF-8 이름, 암호화되지 않은 PCOMPRESS0 저장 파일과 폴더. 개체, 스트림, 볼륨 검증은 제외 | 지원하지 않음 | 지원하지 않음 |
| [Compound File (MSI)](docs/ko-KR/engineering/wim-msi-installer-format-sources.md#xdzip-compound-file-and-msi) | 스토리지 탐색과 원시 스트림 추출. 구조와 읽기 가능 여부 확인 | 지원하지 않음 | 지원하지 않음 |
| [NSIS (`.exe`)](docs/ko-KR/engineering/wim-msi-installer-format-sources.md#xdzip-nsis-and-other-sfx) | 정적 `firstheader` 메타데이터만 제공. 추출과 전체 검사 불가 | 지원하지 않음 | 지원하지 않음 |
| ASAR | 저장된 파일과 폴더, 공유 페이로드 범위, 파일 및 블록에 선언된 SHA-256 | 지원하지 않음 | 지원하지 않음 |
| Z, TAR.Z (`.taz` 포함) | Unix compress LZW, 9–16비트 사전 및 허용된 CLEAR와 정렬 프로필 | 지원하지 않음 | TAR.Z |
| Brotli, TAR.BR | 일반 RFC 7932 스트림, 10–24비트 윈도, 모든 정적 사전 변환 | 지원하지 않음 | TAR.BR |
| AES Crypt (`.aes`) | 버전 0–3, 열기 암호, 암호문 전체의 HMAC 및 버전별 길이 규칙 | 지원하지 않음 | 지원하지 않음 |
| UU/UUE, XXE | 기존 길이 표기 줄과 허용된 begin-base64 봉투, 여러 멤버 포함 | 지원하지 않음 | 지원하지 않음 |
| RAR | 이전 형식, RAR5 및 RAR 7 형식 확장. 암호와 솔리드 및 분할 압축 파일 포함 | 지원하지 않음 | 지원하지 않음 |
| CAB | Store, MSZip, Quantum, LZX 및 같은 폴더에 있는 볼륨 | 지원하지 않음 | 지원하지 않음 |
| ISO | ISO9660, Joliet 및 Rock Ridge | ISO9660 + Joliet 데이터 이미지 | 지원하지 않음 |
| UDF | UDF | 지원하지 않음 | 지원하지 않음 |
| ZIP SFX | 검증 가능한 ZIP 자동 압축 풀기 파일 | Windows x64, Store/Deflate, `.sfx.exe` | 지원하지 않음 |

- ZIP AES-256은 내용만 암호화합니다. 파일 이름은 그대로 보입니다. AES-128/192와 기존 ZipCrypto는 지원하지 않습니다. `.zip.001`, PKZIP의 `.z01` 및 관련 이름 규칙으로 된 분할 압축 파일을 읽을 수 있습니다. 만들 때는 번호를 붙인 볼륨을 사용합니다. [암호가 있는 파일 풀기](docs/ko-KR/engineering/password-extraction.md) 및 [분할 압축 파일 열기](docs/ko-KR/engineering/archive-opening.md)를 참조하세요.
- 독립 BZip2, LZMA Alone 및 Lzip v1은 읽기 전용입니다. LZMA에는 콘텐츠 체크섬이 없으므로 구조, 크기 및 읽기 가능 여부를 검사합니다. 확장자, 항목, 메모리 제한 및 허용 범위는 [단일 파일 스트림 규칙](docs/ko-KR/engineering/single-file-streams.md)을 참조하세요.
- XZ는 스트림에 선언된 콘텐츠 검사를 수행합니다. None을 선언한 스트림에서는 구조와 읽기 가능 여부를 확인합니다. ZIPX는 표에 나온 ZIP 방식을 읽으며 PPMd, JPEG 및 WavPack 변형은 지원하지 않습니다. ZIP/ZIPX 수정 시 유지하는 항목의 압축 데이터와 방식을 보존하고 새 항목에는 Deflate를 사용합니다.
- 7z를 수정할 때는 압축 파일을 다시 만듭니다. 유지되거나 추가되는 내용의 작업 메모리 한도는 512 MiB입니다. 새로 만들 때는 입력을 스트림으로 읽으므로 이 재구성 한도가 적용되지 않습니다. 암호화되었거나 분할된 ZIP 및 7z는 수정할 수 없습니다. [압축 파일 수정](docs/ko-KR/engineering/archive-transfers.md)을 참조하세요.
- RAR, CAB, UDF는 읽기 전용입니다. [ISO 만들기](docs/ko-KR/engineering/iso-creation.md)는 일반 파일 및 폴더, 단일 볼륨, 4 GiB 미만의 파일, 구성 요소마다 최대 64 UCS-2 문자인 Joliet 이름을 지원합니다. 부팅 이미지, UDF 또는 Rock Ridge 쓰기 및 이미지 업데이트는 지원하지 않습니다. 이미지를 읽을 때 마운트하거나 실행하지 않습니다. [RAR 규칙](docs/engineering/rar-format-sources.md)과 [형식 백엔드 규칙](docs/ko-KR/engineering/native-codecs.md)을 참조하세요.
- [자체 SFX 만들기](docs/ko-KR/engineering/sfx-creation.md)는 Qt나 설치된 xdzip이 없어도 동작하는 Windows x64용 ZIP 압축 해제 프로그램을 만듭니다. 대상 선택, 충돌 처리 및 취소를 지원하며, 푼 파일을 실행하지 않습니다. 암호, 분할 볼륨, 자동 시작 명령은 지원하지 않습니다. 일반 `.exe`는 파일 연결 대상에서 제외됩니다.
- DEB는 `members/`, `control/`, `data/`를 내보내기 보기로 제공하며, 모두 풀면 원시 멤버와 풀린 내용이 함께 나옵니다. 패키지를 설치하거나 스크립트를 실행하지 않습니다. 제어 데이터는 무압축, GZIP, XZ, Zstandard를 지원하고 데이터는 BZip2와 LZMA도 지원합니다. 검사는 컨테이너의 읽기 가능 여부와 선언된 검사 항목을 확인하며 패키지를 인증하지 않습니다. [DEB 규칙](docs/ko-KR/engineering/deb-containers.md)을 참조하세요.
- Z와 TAR.Z는 체크섬 없이 구조 및 읽기 가능 여부를 검사합니다. 독립 Z 생성 및 손상된 이전 스트림 복구는 지원하지 않습니다. [Unix compress 규칙](docs/ko-KR/engineering/compress-streams.md)을 참조하세요.
- UU/UUE 및 XXE는 체크섬 없이 구조와 읽기 가능 여부를 검증합니다. 헤더의 경로와 모드는 파일 출력이나 실행 권한을 주지 않습니다. GNU 방식으로 인코딩한 파일 이름과 여러 부분으로 나뉜 파일의 복구는 [텍스트 봉투 프로필](docs/ko-KR/engineering/text-envelopes.md)의 지원 범위 밖입니다.
- 읽기 전용 ASAR를 열거나 풀어도 해당 앱을 실행하지 않습니다. 외부 `unpacked` 데이터와 선언된 링크는 보이지만 처리하지 않습니다. 인접 폴더에 접근하거나 실행 권한을 복원하지 않습니다. 선택적 해시는 손상을 검출합니다. 해시가 없는 레코드는 구조와 읽기 가능 여부만 확인합니다. [ASAR 규칙](docs/ko-KR/engineering/asar-containers.md)을 참조하세요.
- NSIS는 PE 오버레이에 크기가 제한된 유효한 `firstheader` 레코드가 있을 때만 인식합니다. xdzip은 해당 정적 메타데이터 행을 표시합니다. PE 스텁, 설치 스크립트, 플러그인 또는 페이로드는 실행하지 않습니다. 설치 프로그램 헤더의 압축 해제나 페이로드 추출은 지원하지 않습니다. `.exe` 항목은 끌어다 놓기와 셸 파일 연결에 표시하지 않습니다. 일반 실행 파일은 처리하지 않습니다.
- LHA/LZH를 만들 때 LH5, 레벨 2 헤더, UTF-8 이름을 사용합니다. 이전 Windows 도구는 오래된 코드 페이지로 이름을 표시할 수 있습니다. 조건을 충족하는 레벨 2 압축 파일은 수정할 수 있습니다. 이전 이름은 구분 기호를 해석하기 전에 디코딩하고, 내용 디코딩 시 존재하는 CRC를 검증합니다. CRC가 없는 오래된 항목은 구조와 읽기 가능 여부만 검사합니다. 암호, 분할 볼륨 및 복구는 지원하지 않습니다. PMA는 별도의 읽기 전용 프로필입니다. [LHA 규칙](docs/ko-KR/engineering/lha-containers.md)을 참조하세요.
- ARJ는 읽기 전용입니다. 바이너리와 기존 텍스트 멤버에서는 플랫폼에 따른 줄바꿈 변환 없이 CRC가 적용된 바이트를 그대로 제공합니다. 암호, 분할 볼륨, 복구 및 보안 프로필, 링크와 특수 개체는 허용되지 않습니다. 알 수 없는 방식도 표시되지만 전체 압축 파일 검사는 성공할 수 없습니다.
- ARC는 읽기 전용입니다. 일반 파일을 순차 처리하고 정확한 종료를 확인합니다. 암호 디코딩, 분할 볼륨, SFX 및 Spark 확장은 허용되지 않습니다. 알 수 없는 방식은 ZIP 별칭 없이 표시되며 전체 검사가 성공할 수 없습니다. 이전 XOR 암호화에는 헤더 플래그가 없습니다. 암호나 복구로 대체하지 않고 유효한 스트림, 정확한 크기, CRC를 요구합니다.
- BH는 읽기에서 `BH 05 07` 방식 8 raw Deflate 프로필을 지원합니다. 알 수 없는 방식, 암호화, 폴더, 분할 볼륨, 후행 데이터와 SFX 접두부는 제외합니다.

- 디스크 색인을 사용하는 큰 ZIP은 현재 전체 검사나 수정이 불가능합니다. 작업 한도 내에서 선택한 일반 파일만 풀 수 있습니다. [대용량 압축 파일 색인](docs/ko-KR/engineering/large-index.md)을 참조하세요.
- TAR의 희소 파일과 타사 확장은 현재 지원하지 않습니다. 일반 GZIP과 ZIP 계열 XPI/APK/IPA/CBZ 후보는 [단일 파일 스트림 규칙](docs/ko-KR/engineering/single-file-streams.md)을 따릅니다. 가능한 경우 이전 파일 이름 인코딩을 추정하지만 짧은 이름이나 혼합 인코딩을 항상 정확히 구분할 수는 없습니다. [파일 이름 인코딩](docs/ko-KR/engineering/filename-encoding.md)을 참조하세요.

- AES Crypt는 메타데이터를 표시하기 전에 열기 암호가 필요합니다. 모든 암호문 MAC을 검증해야 합니다. 버전 0–2는 끝에 있는 길이의 나머지를 인증하지 않으며, 확장은 별도 권한을 주지 않습니다. [AES Crypt 규칙](docs/ko-KR/engineering/aescrypt-envelopes.md)을 참조하세요.

- Brotli와 TAR.BR 후보를 약한 식별 방식으로 발견하면 전체 내용을 검증해야 하며, 형식에는 체크섬이 없습니다. TAR.BR을 수정하면 표준 비압축 메타 블록을 사용하여 결과가 더 커질 수 있습니다. 공유 사전과 큰 윈도 확장은 허용된 [Brotli 프로필](docs/ko-KR/engineering/brotli-streams.md)에 포함되지 않습니다.

<a name="xdzip-command-line"></a>

## 명령줄

독립 CLI는 Qt나 그래픽 세션 없이 동작합니다. 압축 파일의 목록 표시, 압축 풀기, 만들기, 추가 및 검사와 JSON 출력을 지원합니다. `xdzip-cli-<version>-` 배포 파일을 풀고 macOS 또는 Linux에서는 `./bin/xdzip --help`, Windows PowerShell에서는 `.\bin\xdzip.exe --help`를 실행하세요. 함께 제공되는 워커를 실행 파일 옆에 두세요. [CLI 빠른 시작](docs/ko-KR/integration/README.md)에는 첫 명령과 PATH 등록 방법이 나와 있습니다. 소스에서 직접 빌드할 수도 있습니다.

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/cli.py build
python3 scripts/cli.py test
build/cli/bin/xdzip --help
build/cli/bin/xdzip list archive.zip --json
build/cli/bin/xdzip extract archive.zip -C output
```

Windows에서는 `python`을 사용하고 실행 파일 이름에 `.exe`를 붙이세요. Linux에서 실행하려면 시스템 OpenSSL 3과 인증서가 필요합니다. 빌드에는 `libssl-dev`와 `pkg-config`도 필요합니다. 암호, 비대화형 사용, 종료 코드, 명령 설치 및 업데이트 제한은 [CLI 안내](docs/ko-KR/engineering/command-line.md)를 참조하세요.

<a name="xdzip-build-from-source"></a>

## 소스에서 빌드

데스크톱 인터페이스는 Qt Quick/QML을 사용하고 얇은 C++ 계층을 통해 Rust 코어에 연결됩니다. 형식, 쿼리 및 작업 로직은 Qt에 의존하지 않습니다. 7z, RAR, CAB, ISO, UDF 읽기와 7z 쓰기는 격리된 워커 프로세스를 사용합니다. ISO 만들기는 Rust 호스트에서 실행됩니다. Windows 독립 SFX는 Qt 없이 시스템 대화 상자를 사용합니다.

프로젝트 문서는 `docs/`에 있습니다. 생성기와 검사에서 사용하는 시각 매개변수, 용어, 종속성 목록은 `specs/`에 있으며, 상세 디자인은 `design/hifi/`에 있습니다.

Python 3.10 이상, Git, CMake 3.24 이상, Ninja, rustup, 플랫폼 컴파일러와 SDK를 설치한 다음 소스를 받아 빌드하세요. CLI 단계에서 이미 복제했다면 `prepare`부터 시작하세요.

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

Windows에서는 MSVC x64 개발 환경을, macOS에서는 네이티브 Apple Silicon 터미널과 Xcode Command Line Tools를 사용하세요. 기본 Qt 6.8.2 도구 체인은 `build/toolchain/`에 있습니다. Qt가 이미 설치되어 있다면 `--qt-prefix`를 지정하세요. 종속성, 실행, 패키징 및 검증은 [개발자 빌드 안내](docs/ko-KR/engineering/developer-build.md)를 참조하세요. CLI와 데스크톱의 빌드 진입점은 별개입니다. 데스크톱 빌드에는 명령 진입점을 명시적으로 활성화해야 하는 전용 CLI도 포함됩니다.

개발 패키지는 실험 빌드입니다. Windows와 Linux 개발 결과물은 수동으로 실행해 성공한 [Qt 구성 요소 워크플로](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml)에서도 받을 수 있습니다. 일상적인 CI에서는 개발 패키지를 자동으로 만들지 않습니다.

<a name="xdzip-documentation-and-contributing"></a>

## 문서와 기여

| 주제 | 시작할 문서 |
| --- | --- |
| 데스크톱 사용과 스크립트 | [사용 설명서](docs/ko-KR/user-guide/README.md), [명령줄 통합](docs/ko-KR/integration/README.md) |
| 제품 설계와 구현 | [기술 문서](docs/ko-KR/reference/README.md) |
| 개인정보와 데이터 처리 | [개인정보 처리방침](PRIVACY.md), [중국어판](PRIVACY.zh-CN.md) |
| 빌드와 기여 | [기여 안내](docs/ko-KR/contributing/README.md) |
| 변경 내용과 예정된 작업 | [변경 이력](CHANGELOG.md), [GitHub 이슈](https://github.com/xiaodaozhi/xdzip/issues) |
| 기여 절차 | [기여 규칙](CONTRIBUTING.md), [행동 강령](CODE_OF_CONDUCT.md) |
| 프로젝트 정책 | [프로젝트 운영](GOVERNANCE.md), [보안](SECURITY.md) |

의견과 기여를 환영합니다. 버그를 신고할 때는 버전 또는 커밋, 시스템과 아키텍처, 재현 단계, 실제 결과를 적어 주세요. 샘플은 직접 만들고 민감한 정보는 포함하지 마세요. 보안 문제는 [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new)에서 비공개로 신고하세요.

<a name="xdzip-license"></a>

## 라이선스

프로젝트 자체 콘텐츠에는 [Apache License 2.0](LICENSE)이 적용됩니다. [COPYRIGHT](COPYRIGHT)도 참조하세요. Qt, 코덱, 글꼴 및 기타 종속성에는 각각의 라이선스가 적용됩니다. 출처와 고지는 [라이선스 문서](docs/ko-KR/engineering/licensing.md)를 참조하세요. 공식 릴리스는 해당 소스 코드의 별도 압축 파일을 제공하지 않습니다. [소스 코드 제공](docs/ko-KR/engineering/source-delivery.md)을 참조하세요. 소스 제공, 다시 링크하기 및 고지에 관한 적용 의무는 그대로 적용됩니다. 각 릴리스 후보는 개별 검토가 필요합니다. 이전 기록은 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)와 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)을 참조하세요.
