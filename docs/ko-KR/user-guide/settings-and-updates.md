# 설정과 업데이트

[English](../../user-guide/settings-and-updates.md) | [简体中文](../../zh-CN/user-guide/settings-and-updates.md) | [繁體中文](../../zh-TW/user-guide/settings-and-updates.md) | [日本語](../../ja-JP/user-guide/settings-and-updates.md) | 한국어 | [Français](../../fr-FR/user-guide/settings-and-updates.md) | [Deutsch](../../de-DE/user-guide/settings-and-updates.md) | [Español](../../es-ES/user-guide/settings-and-updates.md) | [Português (Brasil)](../../pt-BR/user-guide/settings-and-updates.md) | [Русский](../../ru-RU/user-guide/settings-and-updates.md)

<a name="xdzip-set-up-your-workspace"></a>

## 작업 환경 설정

앱 메뉴 또는 도구 모음에서 **설정**을 엽니다. 언어, 테마, 글자 크기, 화면 밀도를 각 설정에서 선택하세요. 자동 감지는 지원되는 시스템 언어를 사용합니다. 압축 파일을 탭 또는 창 중 어디에 열지 선택하세요. 플랫폼에서 지원하면 최근 파일과 알림 영역 아이콘도 설정할 수 있습니다. 변경 내용은 바로 적용됩니다. 저장할 수 없다면 해당 페이지의 오류 메시지를 확인하세요. [설정 동작](../ux/settings-localization.md)

파일 연결은 운영체제에서 변경합니다. 파일 관리자 메뉴 동작은 별도로 설정합니다. macOS에서는 시스템이 Finder 확장을 관리합니다. Finder 서비스가 등록된 경우 해당 서비스를 사용할 수 있습니다. [파일 관리자 작업](../engineering/shell-actions.md)

<a name="xdzip-update-the-app"></a>

## 앱 업데이트

- **Microsoft Store 버전.** Microsoft Store에서 업데이트하세요. 앱이 MSIX 패키지 ID로 실행되면 정보 페이지에 GitHub 업데이트 항목이 표시되지 않습니다.
- **GitHub에서 설치한 경우.** **xdzip 정보**를 열고 업데이트를 확인하세요. 앱 안에서 설치하려면 지원되는 Windows 사용자별 설치, macOS 응용 프로그램 폴더의 지원되는 앱 또는 Ubuntu DEB 설치가 필요합니다. 검증된 릴리스 패키지도 필요합니다. Windows에서는 설치 프로그램 확인을 요청할 수 있습니다. Ubuntu는 시스템 인증을 사용합니다. 실행 중인 작업으로 설치가 지연되면 작업이 끝날 때까지 기다리거나 업데이트 대기를 취소하세요.
- **포터블 또는 개발 빌드.** 알맞은 패키지를 내려받으세요. Windows와 Linux에서는 새 ZIP 또는 TAR.GZ 패키지 전체를 별도 폴더에 풉니다. macOS에서는 기존 앱을 종료한 뒤 응용 프로그램 폴더에서 바꾸세요. [설치 절차](install.md)를 참조하세요.

GitHub 업데이트 확인은 사용자가 시작하며 지원되는 프록시 설정을 사용합니다. 수동 다운로드만 제공하는 경우 릴리스 링크를 사용하고 패키지를 검증하세요. 다운로드 후 설치가 필요합니다. 업데이트한 뒤 **xdzip 정보**를 다시 열어 실행 중인 버전을 확인하세요. Microsoft Store와 GitHub는 별도로 릴리스합니다. [업데이트 동작 및 복구](../../engineering/software-update.md)
