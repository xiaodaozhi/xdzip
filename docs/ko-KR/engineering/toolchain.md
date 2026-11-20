<a name="xdzip-toolchain-and-platform-freeze"></a>

# 도구 체인과 플랫폼 고정

[English](../../engineering/toolchain.md) | [简体中文](../../zh-CN/engineering/toolchain.md) | [繁體中文](../../zh-TW/engineering/toolchain.md) | [日本語](../../ja-JP/engineering/toolchain.md) | 한국어 | [Français](../../fr-FR/engineering/toolchain.md) | [Deutsch](../../de-DE/engineering/toolchain.md) | [Español](../../es-ES/engineering/toolchain.md) | [Português (Brasil)](../../pt-BR/engineering/toolchain.md) | [Русский](../../ru-RU/engineering/toolchain.md)

개발 도구 버전은 rust-toolchain.toml, Cargo.lock, CMake와 CI에서 고정하며 지원 범위는 [지원 정책](support-policy.md)을 따릅니다. 이 표는 업그레이드와 릴리스의 검증을 정의합니다. 구체적인 관찰은 issue나 Actions에 남기며 한 기기의 도구 목록을 릴리스 의존성 잠금으로 삼지 않습니다.

| 항목 | 현재 방향 | 고정에 필요한 증거 |
| --- | --- | --- |
| Rust | rust-toolchain.toml에서 1.99.0, edition 2024, rust-version 1.99 고정 | 업그레이드 시 의존성 MSRV, 세 대상 테스트와 Clippy를 확인하고 도구 체인과 Cargo 설정을 일치시킴 |
| Qt | SDK 6.8.2 고정. QtNetwork는 공식 보안 패치로 파생한 `6.8.2-xdzip-network.1` 사용. 로컬 SDK는 `build/toolchain/Qt/6.8.2/<platform-subdirectory>`, 동적 배포 | 고정된 출처와 해시, 모듈 라이선스 및 세 플랫폼 기본 실행 확인. 릴리스마다 보안 유지보수를 확인하고 업그레이드 시 소스와 교체를 재검증 |
| C++ | C++17과 Qt에 맞는 컴파일러 | macOS clang, Windows MSVC, Linux 컴파일러 버전 |
| CMake | 루트는 3.24 이상과 Ninja 요구 | 실제 빌드 버전, 생성 헤더 의존성 및 깨끗한 설정 확인. 워크플로에서 CI 도구 버전 고정 |
| Python | 공통 데스크톱 진입점은 3.10+ 요구. 패키지 검증 도구는 격리 환경 사용 | 후속 Python 스크립트에 같은 인터프리터를 쓰고 단계별 고정 도구 의존성 확인. 브라우저 디자인 도구는 제품 빌드 의존성이 아님 |
| cxx | 1.0.205. 빌드 시 브리지 생성, 안정적인 바이너리 ABI 없음 | crate와 생성기 버전 일치, 종료 경합 및 예외 경계 회귀 확인 |
| 시스템 | macOS 14+ arm64, 유지보수 중인 Windows 11 x64, Ubuntu 24.04 X11 x64 | 승인된 지원 정책에 따라 실제 환경을 확인하고 CI 호스트 증거를 모든 시스템 버전의 보장으로 확장하지 않음 |

Windows에서 데스크톱 빌드, Qt 소스 빌드와 제공 소스 재빌드는 MSVC/Ninja용 UTF-8 콘솔 코드 페이지를 임시 사용하고 이후 원래 값을 복원합니다. 콘솔이 없으면 프로세스와 자식용 임시 숨김 콘솔을 만들며 외부 프로세스는 연결해서는 안 됩니다. 기존 콘솔을 공유하는 보호 빌드는 연결 프로세스 목록으로 직렬화하고 독립 콘솔은 병렬 빌드할 수 있습니다. 보호 중 프로세스는 연결을 유지해야 합니다. 프로세스 목록이 유효하지 않으면 빌드를 중단하고 30초 기다린 후 재시도를 요청합니다. 창 핸들이 없는 이전 스크립트는 호환되지 않는 잠금을 사용하며 이 조정은 다른 프로그램의 콘솔 설정 변경을 막지 못합니다. 빌드 전후에 컴파일된 객체의 Ninja 의존성 기록을 확인합니다. 직접 따옴표 include 또는 같은 디렉터리의 꺾쇠 include가 있는 소스에 의존성 기록이 없으면 중단하고 이전 디렉터리를 보존하며 객체를 자동 삭제하거나 재사용하지 않고 새 빌드 디렉터리를 요청합니다. Ninja가 시스템 헤더를 의도적으로 생략할 수 있으므로 시스템 헤더만 쓰는 소스의 의존성이 0개일 수 있습니다. 다른 검색 경로로 해결되는 꺾쇠 include나 모든 암묵적 컴파일러 입력은 검사하지 않고 전처리 조건도 평가하지 않아 비활성 include를 보수적으로 거부할 수 있습니다. CMake/Ninja 직접 실행은 이 진입점 보호를 우회합니다. 컴파일러 출력이 Ninja 접두사와 맞는지 확인하고 코드 페이지나 도구 체인을 바꾼 후 실제 헤더 변경이 재컴파일을 유발하는지 검증합니다.

GitHub 호스팅 Apple Silicon 작업은 macOS 15 러너를 사용하지만 제품 배포 대상은 macOS 14.0으로 유지됩니다. 러너 업그레이드는 macOS 14 실제 기기 검증을 입증하지 않으며 플랫폼 호환성과 수동 승인을 별도로 기록합니다.

일상적인 준비, 검사, 실행은 [개발자 빌드 안내](developer-build.md)를 참조하세요. `scripts/desktop.py doctor`는 기본적으로 build를 확인하고 `doctor --for-stage package`는 전체 패키징 체인을 미리 확인합니다. prepare의 `--install-rust`, `--install-qt`, `--packaging`은 각 도구와 고정된 자료를 명시적으로 준비합니다. 시스템 컴파일러와 SDK는 개발자가 설치하며 사전 검사는 암묵적인 설치 프로그램이 아닙니다.

고정된 로컬 도구 체인 디렉터리는 `build/toolchain/`입니다. Qt SDK, `qmllint`, `qmake`는 `build/toolchain/Qt/6.8.2/<platform-subdirectory>`를 사용하고 `aqtinstall`과 공통 Python 도구는 `build/toolchain/desktop-tools`, Qt 다운로드 준비는 `build/toolchain/qt-install-tool`을 사용합니다. 공통 진입점은 빌드, 테스트, 패키지 또는 개별 Qt 검사용 별도 캐시 대신 이 디렉터리를 사용합니다. 러너 임시 디렉터리를 쓰는 CI는 로컬 기본 구조를 바꾸지 않고 명시적 매개변수를 전달해야 합니다.

사전 검사는 PATH에서 실제 사용되는 Rust/Cargo를 실행하고 고정 버전과 네이티브 호스트, CMake, Ninja, Qt 접두사 및 플랫폼 도구를 확인합니다. 설치되어 있어도 현재 셸에서 사용할 수 없는 컴파일러는 없는 것으로 간주합니다. 패키징은 잠긴 Rust 의존성 메타데이터, Qt SPDX 및 라이선스 자료, 스키마 캐시도 확인합니다. 이후 모든 프로세스는 같은 검증 환경을 사용하여 빌드 성공 후 라이선스 수집에서 rustc/cargo가 누락되는 일을 막습니다. Docker나 Inno Setup이 없어도 필요한 단계에만 영향을 주며 일반 개발 빌드를 막지 않습니다.

각 기록에는 버전, 다운로드 출처, SHA-256, 라이선스, 대상, 담당자, 검증 커밋과 날짜가 포함됩니다. Rust/Qt 보안 업데이트는 재검증을 유발하며 유지보수되지 않는 패치를 무기한 보존하지 않습니다. 최소 시스템 후보는 공식 지원 약속이 아닙니다.

시스템 유지보수, Qt 보안 업데이트 경로와 업그레이드 검증 책임은 [지원 정책](support-policy.md)을 따릅니다. 출처와 라이선스 결론은 [릴리스 검토 절차](../../../developers/qa/release-review.md)에 기록합니다.

Windows 기본 창의 클라이언트 영역 확장은 WindowsFrame이 공개 Qt 창 인터페이스와 Win32로 구현하며 GuiPrivate에 링크하지 않습니다. 일반 창에 처음 바인딩할 때 현재 QWindow 클라이언트 논리 크기, DPR 및 네이티브 테두리 측정값으로 프레임을 결정하며 이후 테마 또는 DPI 재적용은 테두리만 갱신합니다. 로컬 배율 재정의가 없고 Qt PassThrough 반올림을 사용하는 일반 PMv2 창에서 WM_GETDPISCALEDSIZE는 대기 중 SIZE의 배율 적용 전 조정을 보존하고 클라이언트 영역을 조정한 뒤 대상 DPI의 실제 테두리를 더합니다. 잘못된 측정값, 오버플로 또는 다른 상태에서는 SIZE를 변경하지 않고 Qt에 제어를 돌려줍니다. 빌린 네이티브 메시지와 쓰기 가능한 구조체는 동기 콜백 중에만 사용합니다. Qt 업그레이드나 SDK 교체 시 영향을 받는 클라이언트 영역, 테두리와 창 상태를 재검증합니다. 물리적 듀얼 화면 끌기와 시각 확인은 필요에 따라 수행하며 프로그램 결과로 수행하지 않은 물리 검증을 대신하지 않습니다.

일일 CI는 rust-toolchain.toml에 고정된 Rust 버전, rustfmt, clippy를 러너 임시 디렉터리에 독립 설치하고 실제 컴파일러가 그곳에 있는지 확인한 후 Rust 서식을 검사하여 불완전한 호스트 rustup 도구 체인을 피합니다. 전체 Rust 및 서명 도구 테스트는 변경에 따라 로컬 또는 관련 주문형 워크플로에서 실행합니다.
