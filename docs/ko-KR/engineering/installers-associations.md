# 설치 프로그램과 파일 연결

[English](../../engineering/installers-associations.md) | [简体中文](../../zh-CN/engineering/installers-associations.md) | [繁體中文](../../zh-TW/engineering/installers-associations.md) | [日本語](../../ja-JP/engineering/installers-associations.md) | 한국어 | [Français](../../fr-FR/engineering/installers-associations.md) | [Deutsch](../../de-DE/engineering/installers-associations.md) | [Español](../../es-ES/engineering/installers-associations.md) | [Português (Brasil)](../../pt-BR/engineering/installers-associations.md)

Windows 11 x64에서는 Microsoft Store에서 **XDZip**을 검색해 Store 버전을 설치하거나 업데이트하세요. Windows EXE 또는 포터블 ZIP, macOS 패키지와 Linux 패키지는 [최신 GitHub 릴리스](https://github.com/xiaodaozhi/xdzip/releases/latest)에서 다운로드합니다. 두 채널은 독립적으로 게시됩니다. 설치할 버전의 목록이나 릴리스 노트를 확인하세요.

Windows MSIX는 패키지 매니페스트에서 같은 확장자를 ‘연결 프로그램’ 후보로 선언하며 사용자의 기본 선택을 빼앗지 않습니다. 연결 페이지에는 모든 적격 형식이 비활성화된 스위치와 함께 표시됩니다. 현재 기본값은 Windows 설정에서 확인합니다. ‘기본 앱 설정 열기’는 `registeredAUMID`를 사용해 현재 패키지로 이동합니다. ID 조회가 실패하면 일반 기본 앱 선택기를 엽니다. 패키지는 일반 설치 버전의 등록을 패키지 상태로 읽거나 UserChoice 값을 쓰지 않습니다. 실제 파일 활성화, 아이콘, 공존과 제거는 대상 시스템의 수용 검증이 필요합니다. [MSIX 빌드](../../engineering/developer-build.md#windows-msix-实验构建)를 참조하세요.

형식별 이미지와 플랫폼 매핑은 [파일 연결 아이콘 명세](../visual/file-association-icons.md)를 참조하세요. Linux는 시스템 MIME 아이콘을 유지합니다. 설치는 기본값이나 기존 등록 범위를 변경하지 않습니다.

MSIX 연결은 일반 Windows 설치 프로그램과 같은 정규 ICO 프레임에서 추출한 형식별 PNG 리소스를 참조합니다. ZIP, 7z, RAR과 그 밖의 매핑된 형식은 문서 이미지, 크기 프로필과 투명 여백을 유지합니다. 앱 아이콘에는 판이 없는 별도의 대상 크기 리소스가 있습니다. 두 집합은 패키지의 `resources.pri`에 색인되며 Store 목록 이미지를 바꾸어도 설치된 아이콘은 교체되지 않습니다. Windows가 아이콘 캐시와 기본 처리기 선택을 관리하며 설치는 그 선택을 자동으로 바꾸지 않습니다.

`v0.2.0`부터의 데스크톱 패키지와 현재 소스 빌드는 CLI를 별도 `cli/` 디렉터리, macOS에서는 `Contents/Helpers/cli/`에 배치하고 기존 GUI 및 파일 연결 진입점을 유지합니다. 이전 릴리스 에셋에 이 기능이 소급 추가되지는 않습니다. Windows 설치 프로그램은 선택적인 사용자별 PATH 옵션을 제공하고 같은 이름의 명령 충돌을 감지합니다. 제거할 때는 자신의 설치 디렉터리만 등록 해제합니다. Mac/Linux는 비공개 CLI의 `setup`을 명시적으로 실행합니다. 독립 CLI 패키지도 `setup`으로 사용자 수준 버전 디렉터리와 안정된 실행기를 설치하며 셸 구성을 변경하지 않습니다. `setup --remove`는 등록을 해제하고 검사용 버전 디렉터리는 유지합니다. [CLI 설치 계약](command-line.md#xdzip-build-and-command-installation)을 참조하세요.

아래 패키징 명령은 실험적인 개발 아티팩트를 생성합니다. 공식 GitHub 패키지는 [릴리스 프로토콜](release-protocol.md), Store 패키지는 [MSIX 워크플로](../../engineering/developer-build.md#microsoft-store-msix)를 따릅니다. macOS 직접 다운로드는 Developer ID나 공증 없이 ad-hoc 무결성 서명을 사용합니다. 지원 시스템은 [지원 정책](support-policy.md)을 참조하세요.

현재 소스 빌드는 Windows 설치 프로그램 게시자, Linux 패키지 관리자와 업데이트 정책 공급자로 `xiaodaozhi`를 사용합니다. 이 메타데이터는 게시자 인증서 서명을 대체하지 않으며 기존 릴리스는 원래 메타데이터를 유지합니다.

macOS 수동 업데이트에는 해당 Release DMG를 권장합니다. app.zip을 사용하는 경우 먼저 xdzip.app을 압축 해제해 다운로드 폴더 밖으로 옮기고 실행 중인 이전 버전을 종료한 뒤 응용 프로그램의 복사본을 교체합니다. 다운로드는 설치가 아니며 공증되지 않은 패키지는 시스템 경고를 표시할 수 있습니다. 폴백 조건은 [소프트웨어 업데이트 계약](software-update.md)을 참조하세요.

컨텍스트 메뉴, 독립 작업 호스트와 자체 등록의 수명 주기는 [파일 관리자 작업 계약](shell-actions.md)을 참조하세요. 컨텍스트 작업은 탐색용 메인 창을 열지 않으며 완료 시 대상 폴더도 기본적으로 열지 않습니다.

## 공유 빌드 진입점

환경 준비와 순서는 [개발자 빌드 가이드](../../engineering/developer-build.md)를 참조하세요. 같은 커밋의 깨끗한 빌드와 테스트 후 `python3 scripts/desktop.py prepare --packaging`으로 고정 자료를 준비하고 package와 verify를 실행합니다. 일반 개발에서는 전체 CI 스크립트를 직접 사용하지 않습니다.

Windows/Linux의 `package --kind runtime|installer|all`은 배포와 감사 체인을 공유하며 기본값은 `all`입니다. runtime이 먼저 배포 디렉터리를 생성하고 installer는 같은 output-dir의 검증된 runtime에서 빌드하며 Linux는 tar.gz도 만듭니다. macOS는 로컬 `all` 개발 DMG만 지원합니다. 아티팩트와 영수증은 지정 output-dir에 있으며 기본값은 build/package입니다. 준비 과정에서 고정 패키징 도구와 자료를 다운로드할 수 있지만 xdzip이나 시스템 컴파일러·SDK를 자동 설치하지 않습니다. Windows는 CRT/Inno Setup을 준비하고 Linux는 미리 설치된 patchelf와 Debian 도구가 필요하며 Mac은 시스템 배포 및 ad-hoc 무결성 도구를 사용합니다.

일반 `verify`는 기존 런타임과 패키지를 검증합니다. Windows는 격리 복사본에서 SDK 검색 경로를 지우고 Linux는 격리 런타임 컨테이너를 사용하며 Mac은 DMG 재배치와 Cocoa/Metal을 검사합니다. 기본적으로 설치, 업그레이드·롤백 또는 연결 수명 주기를 실행하거나 사용자 기본값을 쓰지 않습니다. 아래 설치 메커니즘과 일회용 CI 검사는 별도 게이트이며 런타임 성공으로 대체할 수 없습니다.

## Windows

설치 프로그램은 패키지에 포함된 원본 Microsoft VC++ 런타임 계약을 표시합니다. 라이선스 페이지는 해당 구성 요소를 식별하고 동의를 요구한 뒤 중국어·영어로 된 Windows-runtime-NOTICE.txt를 표시합니다. 포터블 패키지에도 같은 원문과 설명이 포함됩니다. 프로젝트 자체 코드는 Apache-2.0을 유지하며 Qt와 다른 구성 요소의 원래 라이선스와 교체 권리는 변경되지 않습니다. 컴파일 전에 해시와 완전한 설명을 확인하고 첨부 자료는 같은 텍스트를 유지합니다. [Windows 라이선스](windows-licensing.md)를 참조하세요.

포터블 패키지와 설치 프로그램은 같은 배포 디렉터리를 사용합니다. [deploy_windows.py](../../../scripts/deploy_windows.py)는 windeployqt를 호출한 뒤 고정 Qt Quick Basic 정책에 따라 FluentWinUI3, Fusion, Imagine, Material, Universal, Windows, 스타일에서만 참조되는 NativeStyle/Effects, qmltooling과 .qmltypes를 제거합니다. Basic, Controls impl, Templates, 런타임 QML과 qmldir은 남으며 Qt Widgets qmodernwindowsstyle은 계속 네이티브 컨트롤을 제공합니다. 새 배포 디렉터리에는 DXC, DXIL 또는 D3DCompiler가 없으며 어디서든 이 이름이 나타나면 배포가 실패합니다. 새 배포에는 `opengl32sw.dll`을 포함하지 않습니다. 기존 루트 복사본은 관리된 준비 영역에서 제거하며 다른 곳의 같은 이름 잔여물은 배포를 실패시킵니다. 새 스타일이나 Effects 기능 추가에는 배포 정책 업데이트와 재검증이 필요합니다.

Qt Quick은 Windows에서 기본적으로 Direct3D 11을 사용합니다. 하드웨어나 드라이버 기능이 부족하면 Qt가 Direct3D WARP로 폴백할 수 있으며 이는 소프트웨어 OpenGL과 다릅니다. 새 패키지는 `QT_OPENGL=software` 또는 OpenGL 백엔드를 통한 Mesa 소프트웨어 폴백을 보장하지 않습니다. 과거 패키지와 라이선스 자료는 변경하지 않습니다. [Qt의 Windows 그래픽 문서](https://doc.qt.io/qt-6.8/windows-graphics.html)는 기본 장치와 명시적으로 요청한 소프트웨어 장치를 설명합니다.

VC++는 [고정 소스와 해시](../../../scripts/windows-runtime.json)의 공식 Microsoft x64 14.44.35211 런타임을 사용해 앱 로컬로 배포합니다. [준비 스크립트](../../../scripts/prepare-windows-runtime.py)는 고정 WiX 도구로 추출하고 EXE/MSI 설치 프로그램을 실행하거나 System32에서 복사하지 않고 모든 DLL과 라이선스 문서를 검증합니다. 필요한 MSVCP/VCRUNTIME DLL, 라이선스와 출처를 전달하며 도구, CAB, MSI와 실행하지 않은 vc_redist 설치 프로그램은 제품에 포함하지 않습니다. Windows 10/11은 UCRT를 제공합니다. 앱 로컬 런타임은 제품 업데이트와 함께 유지해야 하며 시스템 전체 런타임 업데이트가 이를 교체한다고 가정할 수 없습니다. [Microsoft 배포 방법](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method)을 참조하세요.

[배포 회귀](../../../scripts/test-windows-deployment.py)는 제거와 런타임 누락의 경계를 확인합니다. [실제 런타임 검증](../../../scripts/verify_windows_runtime.py)은 소프트웨어 OpenGL DLL을 거부하고 격리 복사본에 테스트 DLL을 추가하며 SDK 검색 경로를 지우고 시스템 번역, 설정 재시작, 네이티브 제목 표시줄과 중국어·영어 시작을 확인합니다. 렌더링 재정의를 지운 뒤 기본 D3D11 장면 그래프 초기화를 확인하고 WARP를 명시적으로 요청하여 실제 선택된 Microsoft 소프트웨어 장치를 검증합니다. 명시적 요청은 실패 시 자동 폴백을 입증하거나 수동 시각 수용을 대체하지 않습니다. 테스트 파일은 제품 디렉터리에 들어가지 않으며 깨끗한 Windows 설치·업그레이드·제거 수용을 대체하지 않습니다.

build-windows-installer.py는 깨끗한 빌드의 deployed 감사만 수락하고 각 파일의 SHA-256을 다시 확인합니다. 고정 Inno Setup 6.7.3, 공식 설치 프로그램 해시, 소스 커밋, 중국어 간체·번체 리소스와 라이선스 문서는 [도구 체인 구성](../../../scripts/installer-toolchain.json)을 참조하세요. 컴파일러는 제품에 들어가지 않으며 통지에는 라이선스 문서가 포함됩니다. 마법사는 공유 언어 레지스트리의 10개 언어를 지원합니다.

설치 프로그램은 고정 AppId와 PrivilegesRequired=lowest를 사용하며 관리자 권한 없이 기본적으로 현재 사용자의 LocalAppData/Programs/xdzip에 설치합니다. 주 실행 파일 xdzip.exe는 설치 루트에 있고 Qt 라이브러리는 그 옆에 있습니다. 플러그인, QML, 번역과 통지는 필요한 하위 디렉터리를 유지합니다. 바로 가기, 연결과 제거 표시 아이콘은 루트를 가리키며 versions/커밋 해시 제품 계층은 없습니다. 커밋과 바이너리 해시는 설치 경로가 아닌 빌드와 설치 영수증에 남습니다. Inno Setup의 [누적 제거 로그](https://jrsoftware.org/ishelp/topic_appendnotes.htm)는 각 설치가 소유한 파일을 기록하고 추가 파일을 재귀 삭제하지 않습니다. 설정과 최근 항목은 기존 사용자 구성 디렉터리에 남으며 설치 프로그램이 제거하지 않습니다.

새 구조는 감사된 상대 경로와 SHA-256을 가진 제한된 install-manifest.txt를 기록합니다. 업그레이드 전에 이전 매니페스트 경로, 중복과 크기를 검증하고 정리 경로의 경로 이탈, 장치 별칭과 재분석 지점을 거부합니다. 제거할 파일이 수정되었거나 검증할 수 없으면 쓰기 전에 설치를 중단합니다. 설치 성공 후 이전 매니페스트에 나열되고 더 이상 제공되지 않으며 해시가 여전히 일치하는 파일만 정리합니다. 빈 디렉터리만 제거하고 추가 파일은 유지합니다. 한 설치에서 완료 이벤트가 반복되어도 정리는 한 번만 실행됩니다. 정리 중 변경되거나 사용 중인 파일은 재시도 기록과 0이 아닌 종료 코드 20을 남깁니다. 재시도 기록을 쓰지 못하면 수동 복구가 필요하며 트랜잭션 복구나 정전 안전성을 주장하지 않습니다. 이 사용자별 설치는 외부 프로세스가 디렉터리를 동시에 수정하지 않는다고 가정하며 경로·해시 재검사는 핸들 기반 악의적 경합 격리가 아닙니다.

기존 버전 디렉터리 구조는 직접 업그레이드됩니다. 실행 명령과 바로 가기는 루트로 이동하고 설정과 최근 항목 위치는 유지됩니다. 이전 설치 프로그램은 별도의 소유권 매니페스트를 제공하지 않았으므로 과거 versions/ 디렉터리는 일반 제거까지 남습니다. Inno의 누적 로그는 원래 기록된 파일을 정리하지만 디렉터리를 재귀 삭제하지 않습니다. 사용자가 추가한 파일은 남습니다. 롤백으로 선택한 이전 버전을 다시 설치할 수 있으며 오래된 QA 픽스처는 과거 구조를 유지하고 나중에 루트 구조로 다시 업그레이드할 수 있습니다. 새 설치는 versions/를 만들지 않습니다. 이전 설치 디렉터리를 자동으로 완전히 삭제하는 기능은 포함되지 않습니다.

제품은 전체 수명 주기 동안 이름 있는 뮤텍스 `Local\xdzip.desktop.running`을 유지합니다. [AppMutex](https://jrsoftware.org/ishelp/topic_setup_appmutex.htm)는 CloseApplications=no로 설치·제거를 차단하며 앱을 강제로 종료하지 않습니다. FFI 핸들 생성이 실패하면 시작을 막고 프로세스 종료 시 해제됩니다. 작업 종료는 기존 결정과 Rust 종료 대기 장벽을 계속 사용합니다.

일반 Windows 설치 프로그램은 HKCU에 아카이브 후보를 등록합니다. 자체 ProgID, 자체 Applications 항목 2개, Capabilities, RegisteredApplications와 자체 OpenWithProgids 값을 기록합니다. HKLM, 확장자 기본값, 보호된 UserChoice/UserChoiceLatest와 다른 앱 후보는 건드리지 않습니다. 등록된 각 확장자에는 `xdzip.Archive.ZIP` 같은 ProgID와 할당된 ICO가 있습니다. 이전 `xdzip.Archive`는 직접 열기 명령을 유지하고 `Applications/xdzip.exe`와 함께 ‘연결 프로그램’에서 숨겨집니다. 업그레이드는 기존 기본값과의 호환성을 유지하면서 이전 추천 항목을 제거합니다. Windows는 `.tar.gz` 같은 복합 파일 이름의 아이콘을 최종 확장자로 선택합니다.

일반 설치 앱은 시작 시와 앱 언어 변경 성공 후 ZIP, TAR, TGZ, GZ, 7Z, RAR, CAB, ISO와 UDF의 핵심 ProgID 9개에 대한 설명을 동기화합니다. 설치 프로그램은 저장된 앱 언어 또는 자동 모드의 시스템 표시 언어로 설명을 선택합니다. 마법사 언어는 독립적입니다. 실행 중 동기화 전에 설치 ID, 실행 파일 위치, 매니페스트, 연결 실행기와 기존 열기 명령 9개를 확인합니다. 등록이 누락되거나 일치하지 않으면 그대로 둡니다. 일부 쓰기는 다음 정상 시작 시 다시 시도할 수 있습니다. 포터블, 개발 및 격리 설정 테스트 프로세스는 이 동기화를 실행하지 않습니다. 다른 등록 확장자는 설치 프로그램이 제공한 설명을 유지합니다. Explorer 캐시와 선택된 기본값이 실제 표시에 영향을 줄 수 있습니다.

일회용 Windows CI는 설치된 제품의 시작 시 설명 동기화와 실행 중 중국어 간체·영어 전환을 확인합니다. 포터블 복사본과 격리 설정 프로세스로 쓰기가 없음을 검증하고 아이콘 9개, 명령, 다른 후보와 기본값이 바뀌지 않는지도 확인합니다. 이 자동 검사는 Explorer 또는 설정의 수동 표시 수용을 대체하지 않습니다.

모든 형식 ProgID의 따옴표로 묶인 명령은 `xdzip-association.exe -- "%1"`을 실행합니다. 해당 Applications 등록은 완전한 SupportedTypes를 포함하므로 ‘연결 프로그램’에 xdzip 앱 항목 하나가 표시됩니다. 실행기는 같은 브랜드 앱 ICO와 별도의 OriginalFilename 및 제품 버전 리소스를 내장합니다. 앱 아이콘은 형식별 문서 아이콘과 별도로 검증하세요. 문서 DefaultIcon은 실행 파일의 앱 리소스를 대체할 수 없습니다. 작은 GUI 실행기는 파일 인수 하나를 받고 자체 스레드에서 COM 단일 스레드 아파트를 초기화한 뒤 명령 인터프리터 없이 명시적 `xdzip.Archive` 클래스를 사용한 ShellExecuteEx를 통해 기존 열기 명령에 위임합니다. 호출 후 COM을 해제합니다. 이전 버전 롤백은 공유 항목을 갱신하므로 형식별 후보도 롤백된 프로그램을 계속 가리킵니다. COM 초기화 실패, 진입점 누락, 잘못된 인수 또는 시스템 거부는 다른 앱을 시도하거나 기본값을 바꾸지 않고 0이 아닌 값을 반환합니다. 이는 롤백 설치가 현재 설치 디렉터리에 실행기를 유지하는 데 의존하며 설치·롤백과 누적 제거는 실제 테스트가 필요합니다.

설치 프로그램은 공유 ProgID와 등록된 형식별 ProgID에 `AllowSilentDefaultTakeOver`를 REG_NONE으로 설정합니다. Windows는 이를 명시적 ‘연결 프로그램’ 후보로 계속 제공할 수 있습니다. 설치 프로그램은 제한된 네이티브 레지스트리 호출로 이미 생성한 자체 키에 값을 설정합니다. 실패하면 설치를 중단하고 핸들을 해제합니다. [Windows ProgID 규칙](https://learn.microsoft.com/en-us/windows/win32/shell/fa-progids)을 참조하세요.

일반 설치 프로그램과 MSIX는 [형식 레지스트리](format-registry.md)에서 Windows ‘연결 프로그램’ 후보를 생성합니다. 이 매핑은 핵심 ZIP, TAR, TGZ, GZ, 7z, RAR, CAB, ISO, UDF와 추가 등록 확장자를 포함합니다. `.gz`는 내용에 따라 독립 GZIP 또는 TAR.GZ로 연결될 수 있습니다. 일반 `.img`, `.001`, `.rNN`은 명시적 열기 또는 지원 진입점용입니다. 연결은 시스템에서 xdzip으로 가는 경로를 제공하며 리더는 계속 파일을 조사하고 작업의 형식과 소스 요구 사항을 적용합니다.

## Ubuntu 24.04

build-linux-installer.py는 같은 검증된 디렉터리에서 .deb를 만듭니다. 비공개 준비 디렉터리는 출력 경로로 구분하므로 같은 패키지 이름을 다른 후보 디렉터리에서 재빌드해도 기존 준비 영역을 덮어쓰지 않습니다. 패키지 이름은 xdzip, 버전은 제품 채널과 커밋에 연결되며 아키텍처는 amd64입니다. 시스템 의존성은 실제 배포 매니페스트와 desktop-database 및 잠금 도구에서 나오며 glibc는 최소 2.39입니다. Qt/QML/플러그인과 통지는 /opt/xdzip/versions/에 있고 /usr/bin/xdzip은 따옴표로 묶인 고정 대상 전달 진입점입니다. dpkg가 이전 패키지 파일을 교체하고 제거하며 롤백에는 원래 .deb를 보관해야 합니다.

설치된 [xdzip.desktop](../../../apps/desktop/packaging/linux/xdzip.desktop)은 패키지 MIME 후보를 선언합니다. ZIP과 선택된 별칭, TAR과 필터링된 TAR, 독립 압축 스트림, 7z, RAR, CAB 같은 아카이브 형식과 ISO/UDF 이미지가 포함됩니다. `%f`는 단일 파일 실행을 요청하고 `--`는 인수 경계를 표시합니다. 패키지는 `*.udf` MIME 선언을 제공하고 일반 이미지와 바이너리 형식은 변경하지 않습니다. 설치와 제거는 `mimeapps.list`를 다시 쓰거나 사용자 기본값을 바꾸거나 시스템 MIME 아이콘을 교체하지 않고 MIME 및 데스크톱 데이터베이스를 갱신합니다. 제거는 자체 실행기, 아이콘과 런타임 파일을 삭제하고 사용자 설정을 유지합니다.

DEB는 [AppStream 카탈로그 항목](../../../apps/desktop/packaging/linux/xdzip-local.yml)을 Ubuntu App Center가 읽을 수 있는 `/var/lib/swcatalog/yaml/xdzip-local.yml`에 설치합니다. 이 항목은 패키지 이름 `xdzip`과 `xdzip.desktop`을 연결하여 설치된 패키지가 앱 목록에 나타날 수 있게 합니다. dpkg는 XDZip 제거 시 카탈로그 파일을 삭제합니다.

설치된 제품은 /run/xdzip/usage.lock에 공유 flock을 유지합니다. 유지보수 스크립트는 먼저 비차단 배타 잠금을 얻고 활성 프로세스가 있으면 실패합니다. 스크립트는 root 소유 /run/xdzip 아래 설치 중 게이트를 설정하며 제품은 잠금 획득 후 다시 확인해 dpkg 훅 사이의 시작을 방지합니다. 심볼릭 링크나 잘못된 소유자·권한은 거부하며 QFile을 닫으면 잠금이 해제됩니다. tmpfiles.d는 시스템 시작 시 런타임 잠금을 준비합니다. 이는 SDK 개발 앱에 영향을 주지 않습니다.

성공 및 dpkg 중단 경로는 게이트를 정리합니다. 중단이나 정전으로 미완료 트랜잭션이 남을 수 있으므로 sudo dpkg --configure -a를 실행하고 실제 오류를 수정하세요. 시작할 수 없는 상태는 설치 성공이 아닙니다. 스크립트는 사용자 디렉터리를 재귀적으로 정리하지 않으며 아티팩트는 build/ 또는 CI에서만 생성됩니다.

## macOS와 파일 열기

메인 앱과 전달 패키지는 Info.plist에서 문서와 ICNS를 Viewer/Alternate로 선언하며 [기본 소유권을 주장하지 않습니다](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html). 개발·테스트 빌드는 기본 Viewer/None이며 시스템 ‘연결 프로그램’ 후보에 들어가지 않습니다. 주요 개발 패키지나 전달 빌드는 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`를 명시적으로 설정해야 합니다. 기본값은 사용자와 Launch Services가 결정하며 앱은 적극적으로 변경하지 않습니다. 개발 검증 복사본의 임시 등록은 [Mac 유지보수 방법](macos-deployment.md)에 따라 정리하고 기존 앱과 사용자 기본값을 유지합니다.

CAB, ISO와 UDF는 같은 Viewer 역할을 사용합니다. ISO는 `public.iso-image`를 사용합니다. 앱은 `.img`나 일반 `public.disk-image`를 주장하지 않습니다. Viewer는 Launch Services 연결을 설명하며 사용 가능한 작업은 [형식 백엔드](native-codecs.md)가 결정합니다. 별도의 [ISO 생성 프로필](iso-creation.md)은 새 이미지를 쓸 수 있습니다.

문서 선언은 [형식 레지스트리](format-registry.md)의 모든 셸 후보 최종 확장자를 포함하고 해당 번들 ICNS를 사용합니다. 복합 확장자는 최종 확장자의 형식을 공유합니다. 예를 들어 `.tar.gz`와 `.gz`는 GZIP 기본값을 공유하고 `.tgz`는 시스템의 압축 TAR 형식을 사용합니다. Finder 캐시, 선택된 기본값과 시스템 형식 인식이 실제 표시에 영향을 줍니다.

macOS RAR, CAB와 UDF 가져온 형식은 실제 파일을 식별할 수 있도록 `public.data`를 준수해야 합니다. CAB는 시스템 `com.microsoft.cab`과 호환성 `com.microsoft.cab-archive`를 모두 선언합니다. 설정은 실제 우선 형식이 이 중 하나면 그 형식을 조회·변경하고 그렇지 않으면 호환성 ID를 사용합니다. UDF Viewer는 `LSItemContentTypes` 없이 `.udf` 확장자를 선언하여 이전 설치의 잘못된 UTI 선언이 여전히 사용되어도 실제 파일이 후보를 찾을 수 있게 합니다. Apple은 이 [확장자 선언 키](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)를 사용 중단했으므로 지원 macOS 버전에서 계속 검증해야 합니다. 설정은 실제 인식된 `.udf` 형식을 조회·변경하고 사용할 수 없으면 기존 ID로 폴백합니다. 설치나 등록은 기존 기본값을 적극적으로 덮어쓰지 않습니다. 처리기가 없던 경우 Launch Services가 [암묵적 규칙](https://developer.apple.com/library/archive/documentation/Carbon/Conceptual/LaunchServicesConcepts/LSCGlossary/LSCGlossary.html)에 따라 새 후보를 현재 처리기로 결정할 수 있습니다. 임시 등록을 제거하면 원래 상태가 복원되어야 하며 이 결정은 앱이 사용자 기본 설정을 기록한 것이 아닙니다.

FileOpenRouter는 QML 전에 [QFileOpenEvent](https://doc.qt.io/qt-6.8/qfileopenevent.html)를 받고 로컬 파일만 수락하며 기존 열기·활성화 라우팅을 재사용합니다. GUI 명령줄에서 `--` 뒤의 인수는 파일 경로이므로 진단 플래그와 혼동하지 않습니다. 대기 중 의도는 최대 128개를 큐에 넣고 활성 작업을 선점하지 않습니다. 한 로그인 세션의 일반 제품 프로세스는 인스턴스를 공유하며 여러 파일 요청은 순서대로 열고 마지막 항목을 활성화합니다. 탭과 창 동작은 [앱 세션 계약](application-sessions.md)을 참조하세요.

실행 인수는 QML 로드 전에 큐에 들어갑니다. 메인 창은 숨긴 상태로 시작하고 컨트롤러 연결 시 창 가시성을 기다리지 않고 첫 사용 가능한 요청을 즉시 제출합니다. 창은 초기 이벤트 큐에서 표시되므로 플랫폼 시작 이벤트도 같은 경로를 사용합니다. 외부 열기 중에는 세션 변경 전에 환영 항목을 숨겨 첫 프레임이나 세션 교체 때 환영 페이지가 번쩍이는 것을 방지합니다. 취소·실패 시 환영 화면을 복원하고 성공 시 실제 목록을 표시합니다. [로딩 상태](../ux/welcome-opening.md)를 참조하세요.

macOS 네이티브 시작은 AppKit의 NSApplicationDidFinishLaunchingNotification도 별도로 기다리고 숨겨진 상태에서 해당 대기 Qt 파일 이벤트 묶음을 처리한 뒤 창을 표시합니다. 알림과 대기 콜백은 메인 창 수명으로 제한되며 Qt 앱 대리자를 대체하거나 고정 밀리초 지연을 사용하지 않습니다. 실제 Launch Services 회귀에서 이전에 환영 내용 한 프레임이 발견되었으며 수정된 요구 사항은 0입니다. check-macos-open.py는 로컬과 Mac CI에서 argv와 네이티브 열기 경로를 실행하고 JSON·로그만 유지하며 앱 패키지를 생성하거나 업로드하지 않습니다.

## 자동 검증과 한계

세 플랫폼의 CTest는 시작 전 파일 이벤트, 비로컬 URL 거부와 읽기 중 큐 처리를 확인합니다. 명시적 XDZIP_ASSOCIATION_PROBE와 격리 설정은 CI 전용입니다. 일반 열기 흐름으로 자체 100,000개 항목 ZIP을 읽고 개수와 빌드 ID를 유지한 뒤 소스 경로를 기록하거나 추출하지 않고 정상 종료합니다.

현재 연결 프로브는 최소 한 프레임의 실제 표시를 기다리고 프레임별 환영 내용 가시성을 기록합니다. 현재 Windows ShellExecuteEx와 Linux GIO 제품 요구 사항은 presented_frames가 0보다 크고 welcome_frames가 0인 것입니다. 고정된 이전 제품은 과거 프로브 능력에 따라 테스트하며 오래된 바이너리가 새로운 첫 프레임 개수를 가진 것처럼 표시하지 않습니다.

Windows 네이티브 CI는 QA 변형 1을 설치하고 2로 업그레이드한 뒤 1로 롤백하고 실제 전달 패키지를 설치합니다. ShellExecuteEx는 중국어, 공백과 앰퍼샌드가 포함된 경로를 자체 등록 클래스로 열고 기본값과 UserChoice 불변, 사용 중 교체 거부, 제거 시 설정·추가 파일·다른 후보 유지를 확인합니다. Linux는 SDK, 시스템 Qt, 컴파일러와 네트워크가 없는 컨테이너에서 같은 dpkg 수명 주기를 실행합니다. UID 1000 제품이 GIO 데스크톱 실행기를 통해 실제로 100,000개 항목을 열며 MIME, 기본값, 잠금, 제거와 purge를 확인합니다.

Linux 릴리스 검증은 격리 컨테이너 사용자 프로필에서 MIME 기본값도 변경하고 GIO가 샘플 콘텐츠 형식을 xdzip으로 해석하면 `gio open`을 사용합니다. 선언된 모든 MIME 기본값을 확인하고 별도 처리기를 시험한 뒤 원래 기본값을 복원합니다. Linux와 macOS의 자동 파일 열기 검사는 입력 없이 열 수 있는 샘플을 사용합니다. AES Crypt 브라우저 테스트는 열기 비밀번호 요청과 인증된 읽기를 검증합니다. 시스템 설정과 다른 데스크톱 환경은 별도 검증이 필요합니다.

QA 변형은 현재 검증된 바이너리를 공유하고 설치 메커니즘 테스트를 위해 테스트 패키지 버전과 디렉터리만 변경합니다. 릴리스 아티팩트가 아니며 서로 다른 제품 버전 사이의 실제 데이터 마이그레이션을 가장하지 않습니다. 설정 스키마는 현재 1이며 설치 메타데이터가 두 플랫폼 모두에서 이를 기록합니다. 오래된 설치 프로그램은 더 새로운 스키마를 선언한 설치를 덮어쓰지 않으며 향후 마이그레이션은 계약을 업데이트해야 합니다.

후속 장애 주입은 QA 변형 2에서만 허용되며 postinst configure 중 한 번 실패합니다. 프로덕션에는 이 분기가 없어야 합니다. 컨테이너는 반구성 상태, 게이트로 차단된 시작과 유지된 설정을 실제 검증하고 dpkg --configure -a로 복구한 뒤 GIO를 통해 실제 파일을 엽니다. 이는 재현 가능한 구성 실패이며 장치 정전이 아닙니다. 별도 설정 호환성 검사는 유지된 원본과 현재 설정 컴포넌트를 컴파일하고 같은 스키마의 구성을 왕복합니다. 원본 입력은 오래된 main 기록과 독립적으로 파일별 해시로 고정되며 완전한 설치 데이터 마이그레이션을 가장하지 않습니다.

완전한 제품 호환성은 prepare-previous-product.py가 일회용 Windows/Linux CI 호스트에서 고정 호환 스냅샷(원래 4cf0f8c에서 유래)의 전체 제품 소스를 빌드하여 별도로 검증합니다. 분리된 worktree는 실제 깨끗한 ID를 유지하며 현재 고정 SDK로 만료되는 오래된 Actions 바이너리를 다운로드하지 않고 과거 제품 QA 설치 프로그램을 빌드합니다. 과거 제품을 설치해 실제로 열고 현재 제품을 설치해 revision 변경과 구성 바이트 유지를 확인한 뒤 롤백하여 과거 제품을 다시 열고 데이터를 확인합니다. 이는 두 완전한 소스 커밋 간 같은 스키마 호환성을 입증하며 과거에 서명된 원본 패키지, 존재하지 않는 스키마 2 마이그레이션 또는 장치 재시작을 가장하지 않습니다. 과거 설치 프로그램은 QA 전용이며 제품 에셋으로 업로드하지 않습니다. 영수증은 두 바이너리와 설치 프로그램의 SHA-256을 연결합니다.

windows-install-lifecycle.json, linux-install-lifecycle.json과 패키지 해시가 실제 증거입니다. 실제 Explorer/Finder 이중 클릭, 기본 선택 UI, 재시작 후 서비스·캐시, 정식 서명 부재 경고, GPU와 데스크톱 검사는 수동 체크리스트를 따릅니다. 자동 스크립트가 모든 INSTALL-01/ASSOC-01을 스스로 통과로 표시하면 안 됩니다. 기존 수동 결론은 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)에 있으며 보류 템플릿 재생성은 수용을 다시 열지 않습니다. 향후 스키마 마이그레이션에는 추가 실제 호환성 사례가 필요합니다.

사용 중 교체 보호를 위한 Windows 무작업 프로브는 트레이를 비활성화한 격리 설정을 사용하여 해당 구성에서 `CloseMainWindow`가 정상 종료되도록 합니다. 다른 연결 열기 프로브는 기본 트레이 설정을 유지하고 명시적 종료 장벽을 통해 끝납니다. 설치 보호 프로브는 숨겨진 창을 프로세스 종료 실패로 오인하거나 일반 경로를 강제 종료해 종료 검증을 우회하면 안 됩니다.

Windows 업데이트 프로토콜 픽스처와 실제 Qt 업데이트 픽스처에는 연결 실행기와 문서 ICO 9개가 모두 포함됩니다. 필수 페이로드가 없는 설치 프로그램은 검사를 통과할 수 없습니다. 프로토콜 픽스처의 작은 C 메인 프로그램과 연결 명령은 업데이트 전달만 검증합니다. 내장 앱 리소스, 네이티브 후보와 실제 창은 완전한 Qt 제품을 통해 별도로 검사하며 결론을 섞지 않습니다.

<a id="兼容基线的长期输入"></a>

## 장기 호환성 기준 입력

실제 업그레이드·롤백은 다시 쓴 main 기록이 아니라 고정된 이전 전체 제품 소스를 사용합니다. `prepare-previous-product.py`는 독립 `codex/installer-compat-baseline` 브랜치에 단일 루트 소스 스냅샷을 고정합니다. 이 브랜치는 빌드와 회귀에 필요한 원본 소스와 에셋만 포함하며 이전 main 부모, 실행 보고서 또는 워크플로는 포함하지 않습니다. 스냅샷의 `COMPAT-BASELINE.json`은 원래 소스 커밋과 트리 ID를 기록합니다. main으로 다시 병합하거나 릴리스 패키지 또는 현재 제품 수용 결론으로 취급하면 안 됩니다.

빌드 후 CLI ID는 스냅샷 자체의 고정 SHA와 일치해야 하며 영수증은 원래 소스 ID를 별도로 기록하고 원래 출시 바이너리를 가장해서는 안 됩니다. 설정 컴포넌트도 작은 원본 소스 파일 3개를 영구 테스트 입력으로 유지하고 파일별 해시로 고정합니다. 양방향 읽기·쓰기와 미래 스키마 거부는 네트워크나 과거 커밋에 의존하지 않습니다. 현재 구현에 맞춰 이 회귀 입력을 변경하지 말고 새 기준은 별도 검토하며 필요할 때 출처를 유지하세요.

고정 스냅샷은 독립 업데이트 도우미 이전 것이므로 도우미 소스나 바이너리가 없습니다. `prepare-previous-product.py`만 `--qa-historical-revision`을 전달합니다. 배포·패키징 스크립트는 일회용 CI, 완전한 고정 SHA와 실제 깨끗한 제품 ID가 동시에 일치할 것을 요구합니다. Linux는 추가로 과거 버전 모드를 요구하고 현재 QA 변형과의 혼합을 금지합니다. Windows는 이전 제품의 버전 명령을 직접 실행해 ID를 확인하고 Linux는 독립 배포 감사의 ID와 전체 파일 해시를 사용합니다. 영수증은 과거 소스에 도우미가 없음을 명시하고 현재 도우미를 주입하지 않으며 존재하지 않는 Linux 도우미의 polkit 인증을 설치하지 않습니다. 일반 제품과 현재 QA 변형은 계속 독립 도우미를 전달·검증해야 하며 `--qa-legacy-version`만으로 누락을 허용하지 않습니다.

## Debian 버전 마이그레이션

첫 공개 프리릴리스부터 Debian epoch 1을 사용합니다. 형식은 `1:<semantic-version mapping>+git<commit>`, 예를 들어 `1:0.1.0~alpha.1+git...`입니다. 과거 `0.1.0~dev.9`는 epoch가 없으면 dpkg 규칙에서 alpha.1보다 뒤에 정렬되므로 이전 매핑을 그대로 유지할 수 없습니다. epoch는 시스템 패키지 정렬 전용이며 제품 UI, 버전 태그와 release.json은 원래 시맨틱 버전을 계속 사용합니다. 후속 Debian 패키지도 epoch 1을 유지하고 alpha, beta, rc와 stable 순서는 계속 업스트림 버전으로 표현합니다. [Debian Policy](https://www.debian.org/doc/debian-policy/ch-controlfields.html#version)를 참조하세요.

실제 과거 제품 픽스처는 CI 전용 `--qa-legacy-version`으로 epoch가 없는 원래 ID를 유지합니다. 업그레이드 문제를 숨기려고 이전 패키지의 정렬을 바꾸지 마세요. 현재 패키지는 실제 dpkg 업그레이드, 롤백과 설치 수명 주기 수용을 통과해야 합니다. 사용자 구성과 외부 기본 연결은 원래 계약을 유지합니다.

## 설정의 기본 앱

설정은 생성된 형식 데이터로 연결 확장자를 그룹화합니다. 편집 가능한 스위치는 플랫폼 서비스와 설치 ID에 따라 달라집니다. Windows는 변경을 시스템 기본 앱에 위임하고 UserChoice를 쓰지 않습니다. MSIX는 스위치를 비활성화한 채 패키지의 시스템 설정 항목을 사용합니다. Linux는 지원 확장자를 MIME 형식에 매핑하고 사용자 기본값에 `xdg-mime`을 사용하며 기본값 제거 시 설치된 대체 앱을 제안합니다.

macOS의 설정은 설치된 번들의 Viewer/Alternate 선언에 포함되는 시스템 형식마다 스위치를 활성화합니다. 활성 스위치는 이전 기본값이 없던 형식에도 xdzip을 기본값으로 요청할 수 있습니다. macOS가 확인을 요청할 수 있습니다. 콜백 성공 후 설정은 Launch Services가 새 기본값을 보고할 짧은 시간을 허용합니다. 시스템 조회에서 xdzip이 확인되면 앱은 이전 앱의 번들 ID를 시스템 콘텐츠 형식별로 로컬에 기록합니다. 스위치를 끌 때 macOS가 이전 앱을 여전히 호환 대상으로 나열하면 먼저 해당 앱을 요청합니다. 저장된 호환 앱이 없거나 복원에 실패하면 설정은 설치된 호환 대안을 제안합니다. 대안이 없으면 스위치 옆에 설명합니다. 취소하면 현재 선택을 유지하고 표시 상태는 새 시스템 조회를 따릅니다. Viewer/None 빌드는 변경을 비활성화합니다.

자동 언어 모드에서 Windows Setup은 현재 사용자의 시스템 UI 언어로 연결 설명을 선택합니다. 중국어 번체를 선호하면 번체를 사용하고 지원하지 않는 언어에는 영어(미국)를 사용합니다. 지원되는 수동 선택이 우선합니다. Setup은 자동 모드를 유지하여 앱이 시스템 언어를 해석하고 시작 시 자체 설명을 동기화할 수 있도록 합니다.
