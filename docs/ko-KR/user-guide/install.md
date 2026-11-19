# XDZip 설치 및 압축 파일 열기

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | [繁體中文（台灣）](../../zh-TW/user-guide/install.md) | [日本語](../../ja-JP/user-guide/install.md) | 한국어 | [Français](../../fr-FR/user-guide/install.md) | [Deutsch](../../de-DE/user-guide/install.md) | [Español](../../es-ES/user-guide/install.md) | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

[GitHub의 최신 안정 버전](https://github.com/xiaodaozhi/xdzip/releases/latest)에서 운영체제에 맞는 패키지를 내려받으세요. Windows에서는 Microsoft Store를 이용할 수도 있습니다. GitHub 페이지에는 파일 이름과 버전이 표시되며 직접 다운로드한 파일을 검증할 수 있는 `SHA256SUMS`도 제공됩니다. 시스템 요구 사항은 [지원 정책](../engineering/support-policy.md)을 참조하세요.

| 데스크톱 시스템 | 설치 또는 실행 |
| --- | --- |
| Windows 11 x64 | Microsoft Store에서 **XDZip**을 검색하거나 GitHub의 `-setup.exe`를 실행하여 현재 사용자 계정에 설치하세요. GitHub 설치 프로그램은 서명되어 있지 않습니다. 포터블 ZIP의 경우 패키지 전체의 압축을 푼 뒤 해당 폴더에서 `xdzip.exe`를 실행하세요. |
| macOS 14 이상, Apple Silicon | DMG를 열고 `xdzip.app`을 **응용 프로그램**으로 드래그하세요. `.app.zip` 파일은 먼저 압축을 풀고 앱을 응용 프로그램 폴더로 옮기세요. 응용 프로그램 폴더의 복사본을 실행하세요. GitHub 빌드는 무결성 확인을 위한 임시 서명으로 서명되어 있으며 공증되지 않았습니다. macOS에서 경고가 표시되면 다운로드 출처와 체크섬을 확인하세요. |
| Ubuntu 24.04 LTS x64, X11, glibc 2.39 이상 | 패키지 관리자로 `.deb`를 설치하세요. 포터블 `.tar.gz`는 모든 파일의 압축을 푼 다음 해당 실행 환경 디렉터리에서 `./xdzip`을 실행하세요. 실행기는 실행 환경 파일과 함께 두세요. |

데스크톱 버전은 Windows ARM64, Intel Mac, Linux ARM64 또는 Wayland를 지원하지 않습니다. Linux용 독립 CLI는 X11 없이 실행되며 시스템의 OpenSSL 3 및 인증서가 필요합니다. 파일 이름이 `xdzip-cli-`로 시작하는 항목을 선택하고 [CLI 빠른 시작](../integration/README.md)을 참조하세요.

<a name="xdzip-open-an-archive"></a>

## 압축 파일 열기

XDZip에서 **압축 파일 열기**를 선택하거나 시작 화면으로 압축 파일을 끌어 놓으세요. 설치된 패키지가 등록되어 있으면 파일 관리자에서 **연결 프로그램**을 선택할 수 있습니다. 특정 파일 형식의 기본 앱으로 XDZip을 사용하려면 시스템 설정에서 지정하세요. Microsoft Store 버전은 파일 연결 페이지에서 **Windows에서 기본 앱 설정**을 제공합니다. 기본 앱과 파일 관리자 메뉴의 압축 작업은 별도로 설정합니다. [파일 관리자에서 압축 풀기](extract.md#xdzip-from-the-file-manager) 및 [설치 및 파일 연결](../engineering/installers-associations.md)을 참조하세요.

이어서 [파일을 탐색하고 미리 보는 방법](browse.md)을 알아보세요.
