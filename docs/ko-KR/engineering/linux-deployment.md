<a name="xdzip-linux-standalone-runtime-directory-and-verification"></a>

# Linux 독립 실행 디렉터리와 검증

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | [繁體中文](../../zh-TW/engineering/linux-deployment.md) | [日本語](../../ja-JP/engineering/linux-deployment.md) | 한국어 | [Français](../../fr-FR/engineering/linux-deployment.md) | [Deutsch](../../de-DE/engineering/linux-deployment.md) | [Español](../../es-ES/engineering/linux-deployment.md) | [Português (Brasil)](../../pt-BR/engineering/linux-deployment.md) | [Русский](../../ru-RU/engineering/linux-deployment.md)

현재 개발 배포 기준은 Ubuntu 24.04, x86_64, X11입니다. 목표는 개발용 Qt SDK 없이 현재 제품을 실행하는 것이며 다른 배포판, 더 오래된 glibc, Wayland 또는 완전한 설치 프로그램 지원을 약속하지 않습니다. macOS CI는 네이티브 빌드, 테스트, 시작 검증을 유지합니다. 로컬 개발 DMG는 [Mac 배포 안내서](macos-deployment.md)를 따르며 이 절차의 영향을 받지 않습니다.

<a name="xdzip-layout-and-generation"></a>

## 구성과 생성

`scripts/deploy_linux.py`는 신뢰할 수 있는 프로젝트 소유 ELF 파일과 명시적으로 지정된 Qt 6.8.2 SDK만 받아 새로운 build/ 또는 dist/ 디렉터리를 만듭니다.

- `xdzip`은 실행기이며 개발 SDK 검색 경로를 상속하지 않고 로컬 라이브러리, 플러그인, QML을 사용합니다.
- `bin/xdzip`은 제품 본체이고 옆의 `qt.conf`는 상대 Qt 경로를 사용합니다.
- `lib/`에는 실제로 가져오는 Qt/SDK 공유 라이브러리가 있고 `qml/`에는 Basic Controls 스타일과 필요한 구현 하위 모듈을 유지합니다. 다른 Controls 스타일, NativeStyle, Effects, 스타일별 대화 상자 변형, 디자이너 파일과 QML 형식 메타데이터는 네이티브 의존성 수집 전에 제거하여 그 전용 라이브러리를 배포하지 않습니다.
- `plugins/`에는 xcb, 오프스크린 검증, 이미지, 아이콘, 플랫폼 입력, xcb 그래픽 통합과 필수 `tls/libqopensslbackend.so`가 포함됩니다. OpenSSL 백엔드가 없으면 배포가 실패합니다. EGLFS, 프레임 버퍼, Wayland 플랫폼 플러그인을 검증된 제공 범위로 설명하지 않습니다.
- `translations/`는 Qt 표준 컨트롤 번역을 유지하며 `share/notices/`는 현재 고지와 소스 의존성 자료를 제공합니다.
- `share/deployment.json`은 QML, 시스템 런타임 의존성, 복사된 SDK 라이브러리와 범위를 기록하며 README.md에 사용법이 있습니다.

스캐너는 고정된 Qt의 [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html)를 사용합니다. Xdzip.Settings와 Xdzip.Native는 내장 등록 모듈이며 개발용 정적 라이브러리는 복사하지 않습니다. 일반 의존성 누락, SDK를 벗어나는 QML 경로, 같은 이름의 라이브러리 충돌, 알려지지 않은 외부 라이브러리는 실패합니다. 네이티브 가져오기의 재귀 수집은 SDK 라이브러리만 복사하고 호스트 glibc나 그래픽 드라이버는 복사하지 않습니다. 시스템 의존성은 실제 Debian 패키지 소유 관계에 따라 나열합니다.

삭제 전에 복사된 QML 트리, 필요한 Basic 플러그인, 유지되는 강한 가져오기와 알려진 스타일 구성을 검증합니다. 예상하지 못한 링크, 알 수 없는 스타일이나 대화 상자 선택기, 생략한 모듈에 대한 강한 의존성은 실패합니다. 기본 대화 상자, 입력기, 접근성, 이미지 플러그인, TLS, 시스템 번역은 기존 규칙을 유지합니다. `share/deployment.json`은 제거된 QML 파일과 바이트 수를 기록합니다. 비공개 `cli/`는 고정된 컨텍스트 경로로 `bin/xdzip-native-codec`를 공유하며 독립 CLI 패키지는 계속 자체 완결됩니다.

HTTPS에는 Ubuntu의 `libssl3t64`와 `ca-certificates`도 필요합니다. `libssl.so.3`과 `libcrypto.so.3`의 시스템 패키지 소유 관계와 비어 있지 않은 시스템 CA 신뢰 저장소를 확인합니다. Qt OpenSSL 백엔드는 이 라이브러리를 실행 중 로드할 수 있으므로 ldd만으로 의존성 완전성을 입증할 수 없습니다. 배포 매니페스트는 설치된 바이너리 패키지 버전, 소스 패키지와 그 버전을 기록합니다. 설치 프로그램은 두 의존성을 선언하며 실제 파일 SBOM은 시스템 라이브러리를 앱에 포함하지 않고 필수 OS 패키지로 기록합니다.

patchelf는 각 ELF에 상대 `$ORIGIN` RPATH를 설정하고 각 플러그인에서 lib/까지의 상대 거리를 계산합니다. Qt 경로 재지정은 [공식 qt.conf 규칙](https://doc.qt.io/qt-6.8/qt-conf.html)을 따라 전체 디렉터리를 이동할 수 있습니다. 라이브러리는 동적 링크와 교체 가능 상태를 유지하며 호환되는 Qt 교체를 제한하지 않습니다. 일반적인 범위는 [Qt Linux 배포](https://doc.qt.io/qt-6.8/linux-deployment.html)를 참조하세요.

상대 RPATH를 다시 쓴 뒤 배포는 불필요한 네이티브 심볼 테이블을 제거하고 모든 DT_NEEDED 및 RPATH/RUNPATH 항목이 변하지 않았는지 검증합니다. 각 ELF의 도구 버전, 전후 해시와 바이트 수를 `symbol_stripping`에 기록하며 SDK 입력과 원본 빌드 바이너리는 건드리지 않습니다. 진단 복사본에는 하위 배포 스크립트가 `--keep-symbols`를 허용합니다. 최종 SDK 귀속과 애플리케이션 해시는 변환된 파일을 설명합니다. 이는 패키지 크기를 줄이는 것이며 측정된 메모리 또는 처리량 개선을 뜻하지 않습니다.

일상적인 생성과 검증에는 [통합 빌드 진입점](developer-build.md)을 단계별로 사용합니다. 먼저 같은 구성의 빌드, 테스트, 패키징 준비를 마친 다음 실행합니다.

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

기본 빌드 디렉터리는 build/desktop, 제공 출력은 build/package, Qt SDK는 `build/toolchain/Qt/6.8.2/gcc_64`입니다. 다른 위치는 각 단계에서 공통 옵션을 전달합니다. `package --kind runtime`은 런타임 디렉터리와 감사 자료를 생성하며, 이어 같은 output-dir에서 `package --kind installer`를 실행하면 tar.gz와 .deb가 생성됩니다. 기본 `all`은 두 단계를 순서대로 수행합니다. 하위 [deploy_linux.py](../../../scripts/deploy_linux.py), [package_linux.py](../../../scripts/package_linux.py), [verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py)는 플랫폼 로직을 유지하므로 문제 해결 시 각 `--help`를 참조하세요.

배포는 네이티브 Linux x86_64에서만 실행하며 기존 디렉터리를 덮어쓰지 않습니다. 패키징 사전 검사는 qmlimportscanner, ldd, patchelf, dpkg-query와 설치 프로그램에 필요한 dpkg-deb를 확인합니다. 이러한 빌드 도구는 제품에 포함되지 않습니다. 컨테이너 런타임 검증에는 Docker와 접근 가능한 데몬도 필요하며 빌드는 고정된 Ubuntu 이미지와 기록된 OS 패키지로 제한됩니다. Docker가 없어도 일반 빌드와 테스트는 차단하지 않지만 verify는 명시적으로 실패하며 실행하지 않은 격리 검사를 통과로 기록하지 않습니다. 일반 verify는 호스트에 .deb를 설치하거나 파일 연결을 변경하지 않습니다. 설치 수명 주기는 일회용 CI에서 검사합니다.

<a name="xdzip-experimental-strict-snap"></a>

## 실험적인 strict Snap

strict confinement의 `core24` Snap은 별도의 실험적 배포입니다. 제한된 애플리케이션은 호스트의 XKB 설정을 `$SNAP/usr/share/X11/xkb`에 상속하지 않습니다. 실행기가 `QT_XKB_CONFIG_ROOT`를 이 경로로 설정하면 Snap에 고정된 Ubuntu 빌드 입력의 전체 XKB 트리를 포함해야 합니다. 이를 생략하면 Qt Wayland 플러그인이 로드되어 컴포지터에 연결되더라도 키보드 컨텍스트를 만들 때 `xkbcommon` 포함 경로 오류로 실패합니다. `xkb-data` 저작권 고지를 페이로드에 유지하고 패키지 버전, 결정적인 디렉터리 해시, 파일 수, 고지 해시를 후보 기록에 남깁니다. 완성된 Snap을 추출하여 검증한 후 오프스크린 및 Wayland 스모크 테스트를 각각 실행합니다. 이 검사는 실제 strict Snap 데스크톱 세션에서의 설치와 상호작용 검사를 대신하지 않습니다.

<a name="xdzip-automated-evidence-without-the-sdk"></a>

## SDK 없는 자동화 증거

컨테이너는 다이제스트로 고정된 Ubuntu 24.04를 사용하고 실제 OS 런타임 의존성과 Xvfb, 글꼴, 로캘 테스트 지원을 설치하지만 Qt, Rust, 컴파일러는 설치하지 않으며 그 부재를 다시 확인합니다. 개발 SDK나 전체 소스 트리는 마운트하지 않고 읽기 전용 제품 디렉터리, 별도의 프로젝트 소유 100,000개 항목 ZIP, 검증 스크립트만 마운트합니다. 압축 파일 런타임 검증은 네트워크와 권한 상승을 비활성화하며 임시 설정 디렉터리만 쓸 수 있습니다.

공백이 포함된 새 경로에 애플리케이션을 마운트합니다. 개발 검색 변수를 지운 다음 실행기로 다음을 검사합니다.

1. 실제 `--version`이 실행됩니다.
2. X11에서 실제 QML 창이 시작되고 안전하게 닫힙니다.
3. 네이티브 제품 프로브가 첫 프레임, 100,000개 항목 읽기, 활성 요청 취소, 종료를 완료하며 수와 취소 결과가 계약을 충족합니다.
4. 호스트 개발 변수를 지운 뒤 SDK 폴백 없이 배포 모드에서 패키징된 모든 ELF를 검증합니다. 실제 파일 SBOM은 동일한 버전과 커밋 식별 정보를 유지하고 공식 스키마를 통과합니다.
5. 같은 SDK 없는 이미지의 새 프로세스에서 네트워크를 켜고 격리된 `XDZIP_SETTINGS_PATH`로 제품의 `--update-probe`를 실행하여 GitHub HTTPS에 실제 요청합니다. 사용 가능한 OpenSSL 백엔드를 보고하고 업데이트 검사를 완료해야 하며 `.https.json`과 `.https.log`를 별도로 보존합니다. 네트워크, 인증서, TLS 실패는 검증을 차단합니다.

HTTPS 프로브는 OpenSSL 백엔드, 시스템 CA 파일, 실제 GitHub 요청 결과를 별도로 기록합니다. 실제 요청 성공만으로 서명 검증이나 네이티브 업데이트 설치를 입증하지 않습니다. 서명 구문 분석과 SDK 없는 업데이트 워크플로는 [별도 업데이트 회귀 워크플로](../../../.github/workflows/update-regression.yml)에서 검증하며 이 증거들은 서로 대체할 수 없습니다.

각 프로브 시도의 원시 로그와 구조화된 보고서를 `.https.attempt-N.log`와 `.https.attempt-N.json`으로 보존하고 마지막 시도의 `.https.log`도 보존합니다. 보고서 누락이나 오류, 시간 초과, 프로세스 종료는 실패 상태를 유지합니다. 제품 컨트롤러가 익명 GitHub 릴리스 목록의 속도 제한을 확인하고 상태, 요청 단계, 응답 헤더, 진단 정보가 완전한 경우에만 재시도합니다. 서버 기한까지 기다리거나 제한된 백오프를 사용하며 macOS 수용 프로브와 마찬가지로 최대 세 번 시도하고 총 대기는 3,700초 이내입니다. 기한이 남은 예산을 초과하거나 대기가 중단되거나 TLS, 네트워크 또는 기타 오류가 발생하면 즉시 실패합니다. 통과하려면 실제 업데이트 요청을 완료해야 합니다. 각 대기 결정과 실제 시간을 `.https-waits.json`에 기록합니다.

출력 경로를 재사용하기 전에 이전 보고서와 로그를 같은 상위 디렉터리에서 보고서 이름을 딴 `.history` 디렉터리로 옮기고 실행마다 별도 하위 디렉터리를 둡니다. 새 실행이 중간에 실패하면 이전 성공 보고서가 현재 출력 경로에 남지 않아 혼동을 방지합니다. 과거 산출물은 별도로 확인할 수 있습니다.

linux-runtime.json은 실제 이벤트와 실행 파일 SHA-256을 기록합니다. Xvfb/Qt Quick 소프트웨어 렌더링은 런타임 흐름만 입증하며 실제 GPU, 중국어 후보 창, 실제 화면 읽기 프로그램, 완전한 데스크톱 설치를 입증하지 않습니다.

<a name="xdzip-download-delivery-and-remaining-gates"></a>

## 다운로드 제공과 남은 조건

CI는 xdzip-linux-x86_64.tar.gz를 생성하고 SHA-256과 실행 파일 해시를 기록합니다. tar.gz는 실행 권한을 유지하여 Actions ZIP 압축 해제 시 실행기 권한이 사라지는 일을 방지합니다. 아카이브 경로, UID/GID, 타임스탬프는 개인 디렉터리 없이 정규화합니다. 다운로드 후 압축을 풀고 디렉터리 안에서 `./xdzip`을 실행하며 bin/xdzip만 옮기지 않습니다. 원시 linux/ 디렉터리는 다시 업로드하지 않습니다.

여기서 생성한 `build/package` 런타임 디렉터리, tar.gz와 .deb는 실험적 개발 제공물이며 자동 설치하거나 기본 연결을 변경하지 않습니다. 공식 Release용 Ubuntu tar.gz와 .deb는 전용 Release 워크플로에서 생성하고 [릴리스 프로토콜](release-protocol.md)에 따라 제공합니다. 플랫폼 범위는 [지원 정책](support-policy.md)을 따르며 다른 배포판과 데스크톱, Wayland 및 관련 확장은 별도 issue가 필요합니다. 기존 데스크톱 수용은 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)에 남습니다. 이후 변경은 영향을 받는 실제 IME와 화면 읽기 프로그램, 클린 설치·업그레이드·제거 검사를 다시 검증하며 디렉터리 재빌드만으로 기존 수용을 반복할 필요는 없습니다. 과거 `v0.1.0` 구성 요소 검토는 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)와 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)에 있으며 디렉터리 실행 성공이 향후 릴리스 조건을 자동으로 완료하지는 않습니다.

Ubuntu .deb 설치, 업그레이드, 롤백, 제거, 런타임 잠금과 데스크톱 MIME 자동화는 [설치와 연결](installers-associations.md)을 참조하세요. 런타임 컨테이너 통과는 실제 데스크톱 검사나 릴리스 라이선스 검토를 대신하지 않습니다.
