# 로컬 macOS 개발용 DMG

[English](../../engineering/macos-deployment.md) | [简体中文](../../zh-CN/engineering/macos-deployment.md) | [繁體中文](../../zh-TW/engineering/macos-deployment.md) | [日本語](../../ja-JP/engineering/macos-deployment.md) | 한국어 | [Français](../../fr-FR/engineering/macos-deployment.md) | [Deutsch](../../de-DE/engineering/macos-deployment.md) | [Español](../../es-ES/engineering/macos-deployment.md) | [Português (Brasil)](../../pt-BR/engineering/macos-deployment.md)

전용 Release 워크플로는 네이티브 배포, 필요한 ad-hoc 서명, app.zip 및 DMG 패키징과 업로드를 허용합니다. 요청 시 실행하는 Qt 워크플로의 macOS 작업은 SDK 검증을 유지합니다. 두 릴리스 패키지에 배포·서명·검증된 동일한 .app을 사용하며, ZIP을 푼 앱도 위치를 옮긴 뒤 시작을 검증합니다. [릴리스 프로토콜](release-protocol.md)을 참조하세요.

로컬 개발용 DMG는 허용되는 개발 산출물입니다. 요청 시 실행하는 Qt 워크플로는 Apple Silicon 네이티브 빌드와 시작을 검증하지만 Mac 앱을 패키징·서명·업로드하지 않습니다. 로컬 스크립트는 애플리케이션을 설치하거나 기본 연결을 바꾸거나 파일을 업로드하거나 릴리스 자격 증명을 읽지 않습니다.

## 빌드와 검증

환경 사전 검사와 단계 옵션은 [개발자 빌드 가이드](../../engineering/developer-build.md)를 따릅니다. 패키징 도구를 준비하고 깨끗한 커밋에서 별도의 Release 주 앱을 빌드한 다음 패키징하고 검증합니다.

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

각 단계는 앞 단계를 암묵적으로 다시 실행하지 않습니다. 기본 Qt SDK는 `build/toolchain/Qt/6.8.2/macos`에 있습니다. 다른 위치를 사용하려면 각 명령에 `--qt-prefix`를 전달합니다. 일반 개발·테스트 빌드는 기본적으로 `None`을 사용하고 위 주 앱은 사용자 기본값을 바꾸지 않고 명시적으로 `Alternate`를 선택합니다. 로컬 macOS 패키지는 `all`만 지원합니다. 요청 시 실행하는 Qt 워크플로는 Mac 앱을 패키징하거나 업로드하지 않습니다. 전용 Release 예외는 릴리스 프로토콜을 참조하세요.

macdeployqt 전에 공통 패키지 검사는 네이티브 arm64, 시스템 SDK, 배포 도구, Rust PATH와 고정 메타데이터, Python 검증 의존성과 Qt 라이선스·스키마 자료를 검사합니다. build와 test는 커밋하지 않은 소스를 허용하지만 package는 깨끗한 커밋과 일치하는 실제 CLI 식별 정보를 요구하며 컴파일 캐시로 이를 우회할 수 없습니다. 시스템 컴파일러, SDK, lipo, codesign, hdiutil은 시스템 도구 체인에서 제공하며 prepare가 설치하지 않습니다.

macdeployqt는 이미지 작업 프로세스도 재배치합니다. Qt 6.8의 기본 플랫폼 플러그인 선택에는 Cocoa만 포함되므로 패키징 스크립트는 앱 라이브러리를 검사하기 전에 offscreen을 명시적으로 복사하여 해당 Qt 프레임워크 의존성을 처리합니다. 플러그인이나 작업 프로세스가 없으면 패키징을 거부합니다. 무결성 서명과 가져오기 감사를 마친 후 Qt/QML/DYLD 검색 재지정을 지우고 실제 동봉된 이미지 작업 프로세스의 형식 목록과 자체 PNG 디코딩을 검증합니다. 기록은 `image-preview-deployed.json`입니다. 스크립트 단위 검사는 배포된 패키지의 승인을 증명하지 않습니다.

내부 [build-macos-dmg.py](../../../scripts/build-macos-dmg.py)와 [verify-macos-dmg.py](../../../scripts/verify-macos-dmg.py)는 이미지와 네이티브 검증의 세부 사항을 유지합니다. 별도의 일상 절차를 유지하기보다 문제 해결에 `--help`를 사용하세요. 내부 출력 경로는 build/ 안의 새 경로여야 합니다. DMG 입력의 모든 문서 유형은 `Viewer/Alternate`를 선언해야 합니다. 그렇지 않으면 원래 번들의 plist나 서명을 고쳐 쓰지 말고 다시 빌드합니다. macdeployqt는 실제 QML을 검사해 Qt 프레임워크, 플러그인과 QML을 배포합니다. Qt 시스템 컨트롤 번역, 프로젝트 고지, Rust 상류 고지, 글꼴 라이선스, 소스 SBOM과 고정된 Qt 라이선스 자료를 추가합니다. 프레임워크 내부 심볼릭 링크는 유지하고 앱 밖으로 나가는 링크는 거부합니다. 개발자 컴퓨터의 절대 RPATH를 제거하고 모든 네이티브 가져오기가 앱 또는 시스템에서만 해결되는지 재귀적으로 검사합니다. 실제 고지를 연결하려면 빌드 식별 정보가 변경되었거나 불명이어서는 안 되며 현재의 깨끗한 커밋과 일치해야 합니다.

Qt의 선택적 QML LocalStorage는 QtSql을 불러오므로 배포 시 데이터베이스 드라이버도 복사합니다. 프로젝트에는 서버 데이터베이스 기능이 없으므로 이미지에 PostgreSQL, ODBC, Mimer 등 SDK 외부 클라이언트가 들어가지 않게 내장 SQLite 이외의 드라이버를 제거합니다. 제거 내역을 기록하고 해결되지 않은 가져오기를 통과로 숨기지 않습니다.

## 아키텍처와 스타일 축소

개발용 DMG는 Apple Silicon `arm64`만 제공합니다. 주 프로그램의 CMake 아키텍처 설정은 macdeployqt가 복사하는 Qt SDK 파일을 제한하지 않으므로 프레임워크, 동적 라이브러리, 플러그인, QML 네이티브 플러그인을 포함해 앱의 모든 Mach-O를 재귀적으로 검사합니다. 두 아키텍처 파일을 `arm64`만 남기도록 축소하는 작업은 새로 만든 패키징 복사본에서만 수행합니다. `arm64` 누락, 인식되지 않는 아키텍처 또는 추가 슬라이스 잔존 시 패키징에 실패하며, 주 실행 파일의 아키텍처가 내부 검사 전체를 대신하지 않습니다. RPATH 재작성과 재귀적 ad-hoc 서명 전에 축소합니다.

제품은 Qt Quick Controls `Basic`을 사용합니다. 배포는 명시적으로 열거한 비 Basic 스타일과 전용 가져오기 또는 구현 플러그인만 제거하고 Basic, 공유 컨트롤, Templates와 실제 QML 의존성을 유지합니다. 알 수 없는 모듈을 삭제하여 축소하지 않습니다. 예상과 다른 스타일 배치, 유지해야 할 의존성 누락 또는 사용하지 않는 스타일의 잔존은 감사 실패입니다. 제품 스타일이나 Qt SDK가 바뀌면 축소 규칙과 테스트도 함께 갱신해야 합니다. 크기 축소는 Qt 라이선스, 대응 소스 자료, 시스템 번역 또는 필요한 런타임 플러그인을 생략하지 않습니다.

범위가 제한된 `build/` 아래에 새로 준비한 앱만 최적화하고 입력 주 앱, Qt SDK 또는 소스는 최적화하지 않습니다. 기록에는 아키텍처·스타일 검사, 축소된 바이너리와 제거 목록, 최적화 전후 실제 앱의 논리 바이트 수를 프레임워크, 플러그인, QML과 기타 범주별로 남깁니다. 바이트는 실제 파일 내용으로 계산하며 심볼릭 링크를 중복 계산하지 않습니다. Finder 디스크 사용량이나 UDZO 이미지 크기가 아닙니다. 최종 DMG 크기는 별도로 기록합니다. 버전별 비교, 검증 결론과 남은 작업은 issue에 기록하고 원시 측정값은 로컬 산출물에 보관합니다.

`scripts/test-macos-bundle.py`는 플랫폼 공통 CI의 작은 프로젝트 소유 픽스처로 macOS 배포 도구 없이 아키텍처 인식, 복사본만 처리하는 동작, 심볼릭 링크 경계, 스타일 유지·제거와 실패를 검증합니다. 최종 패키지 검증을 대체하지 않습니다. DMG 검사는 마운트한 이미지에서 복사한 후 모든 네이티브 파일에 `arm64`만 있는지와 스타일이 계약에 맞는지 재귀적으로 확인하고 아래 SDK 검색 격리와 Cocoa/Metal 흐름을 실행해야 합니다.

## 이미지와 런타임 검증

UDZO 압축 이미지에는 xdzip.app, Applications 링크와 설치 안내가 들어 있습니다. hdiutil verify를 실행합니다. Applications로 드래그하기 전에 이전 버전을 정상 종료하세요. 이 과정에는 설치 중 업데이터나 작업 이전이 없습니다. 설정과 최근 항목은 사용자 디렉터리에 남으며 앱을 제거할 때 자동으로 삭제되지 않습니다.

배포 검증 JSON과 파일 단위 CycloneDX SBOM을 이미지 옆에 보관하고 동일 커밋의 소스 목록, 최종 네이티브 파일 SHA-256과 가져오기 관계에 연결합니다. SBOM은 여전히 불완전합니다. 파일 열거는 정확한 구성 요소 귀속이나 라이선스 승인이 아닙니다. 재귀적 자체 해시를 피하려고 목록은 이미지 밖에 둡니다.

새로 준비된 `xdzip.app`은 배포, 서명, 이미지 생성과 검증 중에만 존재하며 성공과 실패 모두에서 자동으로 회수됩니다. DMG, JSON, SBOM과 안내는 남고 입력 주 앱은 변경되지 않습니다. 시스템이 다시 발견할 수 있는 느슨한 준비 번들은 남지 않습니다.

수정한 ARM64 Mach-O 파일에는 유효한 코드 디렉터리가 필요하므로 복사와 재작성 후 로컬 ad-hoc 서명을 적용하고 내부 코드를 검증합니다. **이는 Developer ID 서명이나 공증이 아니며 Gatekeeper 신뢰를 뜻하지 않습니다.** [개인 오픈소스 릴리스 정책](../../../developers/engineering/release-runbook.md)에 따라 정식 서명과 공증은 필수 릴리스 조건이 아니지만 의존성, 라이선스, 보안과 실제 패키지 검증은 계속 필요합니다. SIP를 비활성화하거나 전역 사용자 보안 정책을 삭제하지 마세요.

검증 스크립트는 이미지를 읽기 전용으로 마운트하고 Applications 링크를 검사한 뒤 중국어 문자와 공백이 포함된 격리 경로에 복사하여 모든 네이티브 파일과 번들의 arm64, Basic 스타일과 엄격한 서명을 재귀적으로 검사합니다. SDK/QML/플러그인 검색 변수를 지우고 Cocoa/Metal로 시작과 종료를 수행합니다. 이미지의 실제 제품으로 자체 생성한 ZIP을 열어 검색, 정렬, 추출과 종료를 수행하고 Unicode, 빈 파일과 디렉터리 및 모든 출력 바이트를 독립적으로 검증하며 실제 첫 프레임에서 빌드 식별 정보, 아키텍처와 Metal 백엔드를 확인합니다. 이어서 검증 복사본에서만 QtCore Mach-O 라이브러리 식별 정보를 바꾸고 ad-hoc으로 다시 서명한 후 아키텍처, 스타일, 서명과 시작을 재검사합니다. 이는 배포된 수정·교체가 가능함을 증명하지만 대응 소스에서 Qt를 재빌드했다는 것, 임의의 ABI 호환성 또는 법적 준수 승인을 증명하지는 않습니다.

app.zip 검증도 제품의 기본 동명 폴더 옵션으로 앱 전체를 추출하고 파일 바이트, 링크 대상과 안전한 권한을 `ditto`와 항목별로 비교한 다음 추출된 실제 앱의 심층 엄격 서명 검사와 Cocoa/Metal 시작을 실행합니다. 바깥 디렉터리는 `.app`으로 끝나면 안 됩니다. 검사 중 새로 만든 임시 앱과 등록은 이후 정리합니다. 서명 신뢰나 공증의 경계는 바뀌지 않습니다.

로컬 컴퓨터에는 여전히 SDK가 있습니다. 가져오기 검사와 환경 변수 정리는 별도의 깨끗한 Mac을 대체하지 않습니다. Finder로 끌어오기, 시스템의 다음으로 열기, 기본 선택, Gatekeeper, 실제 재시작, 서명·공증은 [수동 체크리스트](../../../developers/qa/private-test-manual.md)를 따릅니다.

네이티브 메뉴 편집 명령과 Qt 표준 컨트롤에는 시스템 로캘의 qtbase 번역이 필요합니다. 패키징은 `Contents/Resources/translations/qtbase_*.qm` 복사에 더해 `Contents/Resources/qt.conf`의 `[Paths]`에 `Translations = Resources/translations`를 설정하고 기존 플러그인/QML 경로를 유지하며 무결성 서명 전에 파일을 씁니다. 재배치된 시작 검사는 네이티브 편집 응답자와 앱 메뉴 역할을 모두 검사합니다. SDK 개발 디렉터리에서만 검사하는 것으로는 충분하지 않습니다.

## 로컬 Finder Sync 서명과 활성화

Finder Sync는 같은 앱 안의 네이티브 appex와 독립 릴레이를 사용하며 서비스 도우미도 유지됩니다. 완전한 서명 순서는 네이티브 이미지, 프레임워크와 전용 도우미, 정확한 샌드박스 권한의 appex, 릴레이 번들, 외부 앱이며 그 뒤 내부 번들을 엄격히 검증합니다. 서명된 appex를 나중에 권한이 없는 일반 서명으로 덮어쓰지 마세요. 일반 CI 빌드는 확장을 암묵적으로 서명하거나 활성화하지 않습니다.

로컬 Finder 검증을 위해 기본 네이티브 빌드를 마친 후 다음을 명시적으로 실행합니다.

```sh
python3 scripts/sign-macos-finder.py --app build/desktop/apps/desktop/xdzip.app --output build/review/finder-signing.json
build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip --register-shell-integration
```

서명 스크립트는 프로젝트 소유 앱과 저장소의 `build/` 아래 출력만 허용하며 Qt를 배포하거나 DMG를 만들거나 릴리스 자격 증명을 읽지 않습니다. 기록은 ad-hoc 무결성과 엄격한 내부 검증만 증명하며 Developer ID, 공증, Gatekeeper 신뢰 또는 릴리스 라이선스 승인을 증명하지 않습니다. 앱은 원래 위치에 있어야 합니다. 시스템 설정의 로그인 항목과 확장 프로그램에서 xdzip Finder 확장을 활성화하세요. 설정 경로와 메뉴는 OS 버전에 따라 다를 수 있으며 등록 성공은 메뉴나 동작 승인을 증명하지 않습니다.

일시 검증을 끝내기 전에 같은 앱의 `--remove-shell-integration`으로 소유한 서비스와 확장 등록을 해제합니다. 초기 상태를 확인한 후 이 작업이 만든 설정과 임시 LaunchServices 등록만 정리합니다. 다른 앱이나 기본 연결은 변경하지 않습니다. Finder가 앱을 로드한 경우 업데이트 후 Finder를 재시작하지 말고 시스템 설정에서 소유 확장을 비활성화했다가 다시 활성화합니다. 전달과 롤백의 경계는 [파일 관리자 작업](shell-actions.md)을 참조하세요.

## 개발·검증 복사본의 다음으로 열기 등록

LaunchServices는 이전에 연 앱 번들을 기억하므로 빌드 트리에서 같은 번들 ID의 복사본 여러 개가 시스템 또는 Codex의 다음으로 열기 메뉴에 나타날 수 있습니다. 이 때문에 시스템 데이터베이스를 삭제하거나 Finder를 재시작하거나 검증 패키지를 삭제하거나 사용자 기본값을 바꾸지 마세요.

새 개발 번들은 생성된 Info.plist에서 기본적으로 `LSHandlerRank=None`을 사용하며 선택한 주 앱과 배포 패키지만 `Alternate`를 유지합니다. CMake 옵션 `XDZIP_MACOS_HANDLER_RANK`는 이 두 값만 엄격히 허용합니다. 오프라인 소스 재빌드, 소스 Qt 제품 검증과 Mac 제품 CI는 `None`을 명시적으로 전달하여 향후 기본값 변경이 테스트 복사본을 후보로 다시 도입하지 않도록 합니다. [Apple의 `None` 정의](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)는 번들을 해당 형식의 열기 처리기로 선택하지 않으면서 파일 드래그는 유지합니다. 앱 내부 파일 선택이나 명시적인 앱 열기에는 영향을 주지 않으며 기본값을 설정하지 않습니다.

Mac 제품 CI는 `scripts/check-macos-registration.py`를 실행하여 네이티브 `NSWorkspace`를 통해 새 번들의 파일 후보를 조회합니다. 후보 수와 번들 ID로 열거한 등록은 별도로 평가하며 후자는 실제 사용자에게 표시되는 다음으로 열기 후보를 대신하지 않습니다. 일시 등록 중 기존 기본 처리기는 유지되어야 합니다. 이전 처리기가 없던 유형은 일시적으로 새 후보로 해결될 수 있지만 등록 해제 후 테스트한 모든 유형의 원래 처리기와 후보 상태가 복구되어야 합니다. 현재 사용자의 `LSHandlers` 설정은 등록 전, 중, 후 메모리 스냅샷으로만 비교하고 내용은 출력하지 않습니다. 다른 설정과 시스템 수준 규칙은 이 비교의 범위 밖입니다.

`scripts/macos_launch_services.py`는 `org.xdzip.desktop`만 인식합니다. DMG 재배치, 오프라인 소스 재빌드, 소스 Qt 제품 검증, QtCore 교체와 시스템 파일 열기 검증은 제한된 `build/` 안의 실행 전 등록을 기록하고 완료 또는 실패 시 새로 추가한 테스트 복사본만 등록 해제합니다. 기존 주 앱과 다른 제품은 유지합니다. 정리 실패는 조용히 성공으로 보고하지 않고 검증 실패로 처리합니다.

DMG 재배치와 QtCore 교체는 배포 번들의 원래 문서 선언과 검증을 유지하지만 `build/` 아래 새롭고 존재하지 않는 `.app` 경로에만 복사합니다. 필요한 해시를 해당 범위 안에 확정하고 검증 후 또는 실패 후 새 번들을 자동 회수합니다. JSON, 로그, 교체 전 주 번들, DMG와 Qt 소스는 남습니다. 정리는 기존 앱이나 심볼릭 링크를 허용하지 않고 과거 검증 번들을 소급 삭제하지 않습니다. 등록만 해제한 오래된 `Alternate` 번들은 시스템이나 다른 앱이 재발견하고 등록할 수 있으므로 영구적인 재발 방지는 보장할 수 없습니다. 오래된 번들을 보존해야 한다면 명시적 허가 아래 느슨한 복사본을 처리하기 전에 검증·복원 가능한 백업에 둘 수 있습니다.

다음 도구로 과거 잔여물을 미리 확인한 다음 명시적으로 적용합니다. `--keep`은 현재 개발 빌드 또는 Applications의 설치 중 올바르게 식별된 기존 주 앱을 지정해야 합니다.

```sh
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app --apply
```

이 도구는 이 저장소의 `build/` 안에 있는 다른 xdzip 복사본과 같은 식별 정보의 무효 경로만 등록 해제합니다. 번들을 삭제하거나 서명을 바꾸거나 기본 ZIP 앱을 설정하지 않습니다. 다른 제품, Applications의 다른 설치와 빌드 트리 밖 경로는 자동 처리하지 않습니다. 심볼릭 링크를 통해 범위 밖으로 해결되는 경로는 정리하지 않습니다. 이후 등록을 다시 확인하고 지정한 주 개발 번들만 유지합니다. 빌드 트리 밖에 설치가 여러 개면 미처리 항목으로 별도 열거하고 전체 중복 제거를 주장하지 않습니다.

정리 진행 상황, 건수와 실행 결론은 [개발 복사본 관리 #7](https://github.com/xiaodaozhi/xdzip/issues/7)과 같은 관련 issue에 기록합니다. 생성된 JSON과 네이티브 프로브 결과는 로컬 또는 CI 산출물에만 보관합니다.
