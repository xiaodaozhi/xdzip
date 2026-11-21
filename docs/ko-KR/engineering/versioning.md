<a name="xdzip-product-version-and-build-identity"></a>

# 제품 버전 및 빌드 식별자

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | [繁體中文](../../zh-TW/engineering/versioning.md) | [日本語](../../ja-JP/engineering/versioning.md) | 한국어 | [Français](../../fr-FR/engineering/versioning.md) | [Deutsch](../../de-DE/engineering/versioning.md) | [Español](../../es-ES/engineering/versioning.md) | [Português (Brasil)](../../pt-BR/engineering/versioning.md) | [Русский](../../ru-RU/engineering/versioning.md)

선택 항목인 `prerelease_number`는 사전 릴리스 순번과 빌드 번호를 구분합니다. 예를 들어 alpha.1은 빌드 14를 쓰고 이후 alpha.2에서도 빌드 번호를 계속 올릴 수 있습니다. 값이 없으면 기존의 `channel.build` 해석을 유지합니다. 릴리스 매니페스트와 배포 파일 이름은 [릴리스 프로토콜](release-protocol.md)에서 정의합니다.

[VERSION.json](../../../VERSION.json)은 제품 버전, 채널, 빌드 번호를 관리하는 유일한 원본입니다. 빌드는 현재 값을 읽습니다. 버전 변경은 [변경 기록](../../../CHANGELOG.md), 제공 및 검증은 [프로젝트 이슈](https://github.com/xiaodaozhi/xdzip/issues)를 참조하세요. 버전 번호를 올리는 것만으로 릴리스가 승인되지는 않습니다.

이전 버전 호환성 검사는 고정된 소스 기준과 현재 빌드로 업그레이드, 이전 버전 복귀 및 설정 호환성을 검사합니다. [이전 제품 준비 스크립트](../../../scripts/prepare-previous-product.py)는 전체 제품 기준을, [설정 호환성 스크립트](../../../scripts/check-settings-compat.py)는 설정 기준을 관리합니다. 바뀌는 커밋과 버전 값은 여기에서 반복하지 않습니다. 설정 스키마와 제품 버전은 별도로 관리합니다.

- `version`은 앞에 0이 없는 `major.minor.patch`입니다. 비호환 변경, 호환되는 기능, 수정에는 해당 버전 구간을 사용합니다. 0.x에서도 호환성의 경계를 설명해야 합니다.
- `channel`은 development, alpha, beta, rc, stable을 지원합니다. 안정 버전 전에는 `version-channel-abbreviation.build`로 표시하고 development를 dev로 줄입니다. stable은 기본 버전만 표시합니다.
- `build`는 1부터 65535까지의 정수입니다. 외부 제공용 새 빌드에 대해 올립니다. 로컬 컴파일마다 자동으로 올리거나 태그나 릴리스를 자동으로 만들지 않습니다. Windows 리소스의 숫자 버전 각 부분도 65535 이하입니다.
- CMake는 JSON을 직접 읽습니다. `python3 scripts/sync-version.py`는 파생된 Cargo 작업 공간 버전을 갱신합니다. CI에서는 `--check`를 사용합니다. 내부 Rust 패키지는 작업 공간 버전을 상속합니다. 동기화 후 Cargo.lock을 갱신하고 `cargo check --workspace --locked`를 실행하세요.
- Windows 네 부분 숫자 리소스는 기본 버전과 빌드 번호를 결합하며 텍스트 버전에는 채널을 유지합니다. macOS에서 `CFBundleShortVersionString`은 기본 버전, `CFBundleVersion`은 빌드 번호이며 긴 버전 표기에는 채널이 남습니다. [Apple 문서](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion)를 따르세요. 플랫폼이 허용하는 숫자 형식을 사용하고 시스템 숫자 필드에 Git 커밋을 넣지 않습니다.
- Windows 작업 관리자 등에서 쓰는 `FileDescription`은 `xdzip Archive Manager`입니다. `ProductName`은 `xdzip`을 유지합니다. 실행 파일 이름과 연결 ID는 표시 이름 변경에 영향을 받지 않습니다. 빌드 검사는 실제 EXE 리소스를 읽습니다.
- 데스크톱 빌드는 짧은 Git 커밋과 커밋되지 않은 변경 표시를 생성합니다. Git이 없으면 알 수 없는 값으로 표시합니다. 기기의 경로, 사용자 이름 또는 고정되지 않은 타임스탬프는 포함하지 않습니다. 내용이 바뀔 때만 헤더를 갱신합니다. 커밋 후 다시 빌드하면 변경이 없는 올바른 커밋 ID가 생성됩니다.

QML 모듈, 브리지 프로토콜 및 설정 스키마 버전은 제품 버전과 별도로 관리합니다.

`xdzip --version`, 설정 → xdzip 정보 및 미리보기 앱은 같은 빌드 정보를 읽습니다. CLI는 GUI를 생성하기 전에 버전 인수를 처리합니다. 정보 페이지는 버전, 빌드, 커밋, 실제 Qt 버전, 시스템 및 아키텍처를 표시하며 개발 채널과 커밋되지 않은 변경을 명시합니다. 복사한 JSON에는 개인 파일, 경로, 기기 이름 또는 자격 증명이 포함되지 않습니다. 프로젝트 홈페이지 단추는 사용자가 조작했을 때만 브라우저를 엽니다.

정보 페이지의 큰 아이콘은 플랫폼용 512 px PNG를 사용합니다. HTML은 같은 원본의 벡터를 사용하고 72 px UI 아이콘을 확대하지 않습니다.

정보 페이지는 설정 창의 논리 크기를 쓰며 바닥글이 없습니다. [포함된 인터페이스 언어](../ux/settings-localization.md), 밝은 테마와 어두운 테마, 100%, 125%, 150% 글자 크기를 지원합니다. 정보 설정 디자인을 참조하세요. 네이티브 Qt 또는 커밋 값을 얻을 수 없으면 알 수 없는 상태로 표시합니다.

실제 CLI 및 네이티브 버전 리소스 검사는 `scripts/check-build-version.py`, `scripts/test-version.py`, `BrowserTest::versionAboutWindow`를 사용합니다. 스크린샷과 측정값은 `build/review/about/`에 보관합니다. 소스, 시스템 메타데이터 및 실제 실행 버전은 일치해야 합니다. 버전 번호만으로 서명, 수동 인수 또는 호환성을 입증할 수 없습니다.
