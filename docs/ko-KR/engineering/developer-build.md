# 개발 빌드와 플랫폼 검증

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | [繁體中文](../../zh-TW/engineering/developer-build.md) | [日本語](../../ja-JP/engineering/developer-build.md) | 한국어 | [Français](../../fr-FR/engineering/developer-build.md) | [Deutsch](../../de-DE/engineering/developer-build.md) | [Español](../../es-ES/engineering/developer-build.md) | [Português (Brasil)](../../pt-BR/engineering/developer-build.md)

이 문서는 세 플랫폼에서 환경을 준비, 빌드, 테스트, 패키징하고 검증하는 통합 진입점입니다. 공통 조정은 [scripts/desktop.py](../../../scripts/desktop.py)에서 구현합니다. 플랫폼 문서는 별도의 일상 명령을 유지하지 않고 배포 구조, 도구 동작과 시스템 제한을 보충합니다.

독립 CLI는 Qt, X11 또는 그래픽 세션 없이 `python3 scripts/cli.py build`, `test`, `package`를 사용합니다. Linux 컴파일에는 `libssl-dev pkg-config`도 필요하며 실행 시 시스템 OpenSSL과 인증서를 사용합니다. 데스크톱 빌드는 비공개 CLI 디렉터리도 자동 생성합니다. 명령, 설치와 신뢰할 수 있는 업데이트 경계는 [CLI 계약](command-line.md)을 참조하세요.

개발 빌드는 실험적이며 공식 Release 설치 프로그램이 아닙니다. 기능과 제한은 루트 [README](../../../README.ko-KR.md), 지원 범위는 [지원 정책](support-policy.md)을 참조하세요. 과거 `v0.1.0` 릴리스 검토는 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)와 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)에 있으며 향후 버전은 해당 후보별 새 검토 기록이 필요합니다.

<a id="environment-and-entry-points"></a>

## 환경과 진입점

먼저 Python 3.10 이상, Git, CMake 3.24 이상, Ninja, rustup과 플랫폼 컴파일러/SDK를 설치합니다. Rust 버전은 루트 rust-toolchain.toml에서 가져오고 Qt는 6.8.2로 고정합니다. 의존성과 업데이트 규칙은 [도구체인 정책](toolchain.md)을 참조하세요. 명령은 저장소 루트에서 실행합니다. Windows에서는 예시의 `python3`을 사용 가능한 `python`으로 바꾸고 x64 MSVC 개발 환경을 사용합니다.

## 로컬 도구체인 디렉터리

Visual Studio PowerShell 환경을 로드할 때 `USERPROFILE`, `LOCALAPPDATA`, `APPDATA`의 Unicode 값을 보존하고 로더가 인코딩을 변경하면 복원합니다. Qt는 기본 캐시와 구성 경로에 이 위치를 사용하므로 값이 손상되면 컴파일러가 작동하더라도 파일 시스템 오류가 발생할 수 있습니다.

`prepare`는 기존 SDK에서 CVE-2026-116337과 CVE-2026-105946에 대한 보안 패치가 적용된 QtNetwork를 빌드하고 검증합니다. 다시 빌드한 라이브러리로 다른 런타임 모듈을 교체하지 않습니다. Perl, make(Windows에서는 nmake), OpenSSL 명령줄 도구도 필요합니다. Ubuntu에는 `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`가 필요합니다. 소스, 생성된 선언과 빌드 캐시는 `build/` 아래에 두며 기본적으로 `build/toolchain/qt-network-security/<platform>`을 재사용합니다. 입력과 실제 라이브러리 다이제스트가 일치하지 않으면 제품 빌드와 패키징이 즉시 실패하므로 `prepare`를 다시 실행하세요. 출처, ABI, TLS 기능과 실패 복구는 [패치 유지 관리 노트](../../../patches/qt/README.md)를 참조하세요.

통합 진입점의 로컬 의존성은 Git이 무시하고 커밋하지 않는 `build/toolchain/`만 사용합니다.

| 내용 | 고정 디렉터리 | 참고 |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>`는 `macos`, `msvc2022_64`, `gcc_64` 중 하나이며 qmllint, qmake, CMake 구성과 Qt 런타임은 이곳에서 가져옴 |
| Python 도구 환경 | `build/toolchain/desktop-tools` | aqtinstall과 패키징 및 검증 스크립트용 고정 가상 환경 |
| Qt 다운로드 준비 영역 | `build/toolchain/qt-install-tool` | Qt 설치 도구 다운로드와 임시 입력만 보관 |

`scripts/desktop.py prepare --install-qt`, `doctor`, `build`, `test`는 기본적으로 이 위치를 공유합니다. 개발 파이프라인마다 별도의 `.cache/Qt`, `.cache/desktop-tools` 또는 유사한 사본을 만들지 않습니다. 다른 Qt 버전, 플랫폼 또는 실험 빌드를 격리할 때만 `--qt-prefix`, `--qt-root`, `--tools-dir`을 명시합니다. CI는 러너 임시 디렉터리를 명시적으로 지정할 수 있지만 CI 폴백 경로가 로컬 기본값을 변경하지는 않습니다. CMake, Ninja, rustup, Rust와 플랫폼 C++ 컴파일러는 호스트 도구체인에 속하며 환경 사전 검사 요구 사항에 따라 설치하고 호출합니다. Qt 도구 디렉터리에서 복사하지 않습니다.

개발 주기와 브랜치 및 worktree 정리가 끝나면 해당 주기에 `build/` 아래에 만든 임시 디렉터리와 파일을 제거합니다. `build/toolchain/` 같은 재사용 가능한 공통 도구체인은 유지하고 해당 주기 전용 빌드, 진단과 다운로드 준비 영역은 제거합니다. 규칙과 수동 보완 방법은 [저장소 내용과 기록 관리](../../../developers/engineering/repository-hygiene.md#研发周期工件收尾)를 참조하세요.

| 플랫폼 | 먼저 준비할 시스템 환경 | Qt SDK 하위 디렉터리 |
| --- | --- | --- |
| macOS | 네이티브 Apple Silicon 터미널, Xcode Command Line Tools, 유효한 macOS SDK | macos |
| Windows | MSVC 2022 x64 C++ 도구와 Windows SDK, x64 Developer PowerShell/명령줄 | msvc2022_64 |
| Ubuntu 24.04 | x86_64 C++ 컴파일러, OpenGL/EGL/xkbcommon 개발 패키지, X11 실행 라이브러리 | gcc_64 |

macOS에서는 `xcode-select --install`로 명령줄 도구를 준비합니다. Ubuntu 기본 개발 패키지에는 `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config`가 포함됩니다. 패키징에는 patchelf, binutils(readelf와 strip), Debian 패키지 도구도 필요하고 격리된 실행 검증에는 접근 가능한 Docker가 필요합니다. Windows에서 Visual Studio만 설치하고 개발 환경에 진입하지 않으면 cl, link, rc 또는 dumpbin을 사용할 수 없을 수 있습니다. 사전 검사는 실제로 없는 도구를 식별합니다. 공통 진입점은 시스템 컴파일러, SDK 또는 관리자 권한이 필요한 시스템 패키지를 자동 설치하지 않습니다.

Windows 개발자는 [무료 Community 라이선스 경로](windows-licensing.md)에 따라 해당 버전의 조건을 읽고 동의한 뒤 도구를 준비할 수 있습니다. Build Tools 또는 VC_redist만으로는 자격이 입증되지 않습니다. GitHub 표준 호스팅 러너의 기업 사용과 관련 재배포에는 별도의 공개 권한이 있습니다. 유지 관리자는 호스팅 빌드를 위해 Enterprise를 구매하거나 로컬에 Community IDE를 설치할 필요가 없으며 그 권한은 자체 호스팅 기기에 적용되지 않습니다. 라이선스 검토를 위해 기존 2019 도구를 제거할 필요는 없습니다. 로컬 대안과 현재 2022 릴리스 SDK의 호환성은 별도로 검증해야 합니다.

모든 명령은 다음 옵션을 공유합니다.

| 옵션 | 기본값과 목적 |
| --- | --- |
| `--qt-prefix` | 먼저 `XDZIP_QT_PREFIX`, 없으면 `build/toolchain/Qt/6.8.2/<platform-subdirectory>`. bin이 아닌 SDK 접두사를 가리킴 |
| `--build-dir` | `build/desktop`, CMake 빌드와 테스트 디렉터리 |
| `--output-dir` | `build/package`, 제공 아티팩트와 검증 출력 디렉터리 |
| `--tools-dir` | `build/toolchain/desktop-tools`, 격리된 Python 도구와 준비 자료 |
| `--configuration` | `Release`. 다른 개발 구성을 명시적으로 선택할 수 있음 |

build-dir과 output-dir은 기본 패키징 및 검증 출력 경계에 맞게 저장소 build/ 안의 독립 하위 디렉터리여야 합니다. 한 실행의 각 단계에서 같은 옵션을 유지합니다. 명시적 `--qt-prefix`가 환경 변수보다 우선하며 전역 CMAKE_PREFIX_PATH 변경은 필요하지 않습니다. 진입점은 하위 프로세스의 Qt와 도구 검색 환경을 준비합니다. 일반 데스크톱 실행에는 오프스크린 테스트 변수를 상속하지 않아야 합니다. 생성 출력은 무시되는 build/, target/, dist/ 또는 도구 캐시에만 두고 커밋하지 않습니다.

## 단계별 실행

단계는 독립적으로 실행할 수 있으며 이전 준비, 빌드 또는 테스트를 암묵적으로 다시 실행하지 않습니다. 사전 조건이 없으면 오류가 발생하므로 해당 단계를 실행하고 다시 시도합니다. 각 단계는 작업 전에 필요한 환경을 확인하며 `doctor`는 빌드나 패키징 없이 문제를 미리 보여줍니다.

보호된 Windows 빌드는 기존 콘솔 안에서 프로세스를 조정합니다. 별도 콘솔은 병렬 실행할 수 있으며 같은 기존 콘솔을 공유하는 빌드는 순차 실행합니다. 빌드 중 다른 코드가 보호된 프로세스를 해당 콘솔에서 분리하거나 다시 연결해서는 안 됩니다. 콘솔이 없는 프로세스가 만든 임시 콘솔은 그 프로세스와 하위 프로세스용이며 외부 프로세스는 연결하지 않아야 합니다. 오래된 스크립트는 창 핸들이 없는 콘솔에 공통 폴백 잠금을 사용하고 새 스크립트와 조정할 수 없습니다. 오래된 빌드가 끝나기 전에 같은 창 없는 콘솔에서 새 버전 빌드를 시작하지 않습니다. 프로세스 목록을 확인할 수 없으면 코드 페이지를 변경하지 않고 중지합니다.

비대화식 빌드는 stdin, stdout, stderr를 모두 리디렉션한 독립 콘솔 래퍼를 명시적으로 사용할 수 있습니다. PowerShell 예시:

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

래퍼는 명령마다 독립적인 숨김 콘솔을 만들고 시간 초과 또는 감독자 종료 시 하위 프로세스를 종료합니다. 완료 신호가 없으면 실패합니다. 병렬 빌드는 여전히 별도의 `--build-dir` 값이 필요하며 래퍼는 빌드 디렉터리를 잠그지 않습니다. 기존 desktop-build와 `windows_ninja_safety.py` 의존성 검사는 계속 실행되지만 다른 명령을 래핑해도 자동으로 추가되지는 않습니다. 이 진입점은 비대화식 명령 전용이며 프롬프트, 암호 입력과 의사 콘솔 상호작용을 지원하지 않습니다.

`prepare`는 현재 제품 형식 소스 목록을 검증합니다. ZIP, TAR/GZIP, 7z, RAR, CAB와 광학 컨테이너는 독립 Rust를 사용하며 공식 제품 소스 목록은 비어 있고 공식 UnRAR/7-Zip 엔진을 다운로드하거나 컴파일하지 않습니다. 독립 테스트 참조는 각자의 고정 소스에서 준비하고 제품 실행 의존성과 분리합니다. 제한된 RAR 매핑의 원본 소스 공지는 패키징과 대응 소스 제공물에 유지합니다. [라이선스 계약](licensing.md)을 참조하세요. `build`는 준비된 입력만 사용하고 변동 버전을 다운로드하지 않습니다. Rust 전용 테스트는 먼저 `xdzip-native-codec`을 빌드하고 `XDZIP_NATIVE_CODEC`이 해당 작업 프로세스를 가리키게 한 뒤 워크스페이스 테스트를 실행합니다. 네이티브 백엔드 작업 흐름은 추가로 `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored`를 실행하며 통합 `test`와 Foundation CI 모두 요구합니다. ignored 상태는 형식 검사 통과가 아닙니다. 소스, 스트림 인가와 라이선스 경계는 [네이티브 백엔드 계약](native-codecs.md)을 참조하세요.

| 명령 | 목적과 사전 조건 |
| --- | --- |
| `doctor --for-stage build` | PATH, 고정 Rust, Qt, 컴파일러, SDK와 버전 확인. 기본적으로 빌드 환경 검사 |
| `prepare` | 공통 도구와 의존성 준비. Qt, Rust 도구체인과 패키징 자료 다운로드/설치에는 아래 명시 옵션 필요 |
| `build` | 실제 제품, 테스트와 진단을 구성하고 빌드. 준비된 환경이 필요하며 미커밋 변경 허용 |
| `test` | 기본적으로 Rust 워크스페이스 테스트, qmllint, CTest 실행. 같은 구성의 완료된 빌드가 필요하며 미커밋 변경 허용 |
| `package` | 런타임 배포, 라이선스/출처 자료 수집, 배포 감사와 선택 아티팩트 생성. 깨끗한 소스, 같은 커밋의 빌드/테스트 회신과 패키징 도구 필요 |
| `verify` | 재빌드, 제품 설치 또는 기본 연결 변경 없이 기존 런타임/패키지 검증. 아티팩트와 플랫폼 검증 환경 필요 |

사전 검사는 필요한 도구의 부재, 버전 불일치와 현재 단계에 불필요한 도구를 구분합니다. Docker나 Inno Setup이 없어도 일반 개발 빌드는 차단되지 않습니다. 패키징 사전 검사는 Rust/Cargo 호출과 잠금된 의존성 메타데이터를 실제로 확인해 PATH나 의존성 캐시 문제가 라이선스 수집 단계에서야 발견되지 않게 합니다. 실패한 명령과 검증 제한은 로컬/CI 출력에 남기며 부분 완료를 통과로 보고하지 않습니다.

먼저 고정 Qt와 Rust 도구체인을 준비합니다.

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust`는 설치된 rustup으로 저장소에서 고정한 도구체인을 준비하며 rustup 자체는 설치하지 않습니다. `--install-qt`는 기본적으로 고정 Qt SDK를 `build/toolchain/Qt`에 명시적으로 다운로드합니다. 기존 SDK는 다시 다운로드할 필요가 없으며 각 단계에 해당 경로를 전달합니다.

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

빌드 후 전체 `native_backend_tests` 묶음에 필요한 공개 테스트 입력을 명시적으로 준비합니다. CAB, ISO/UDF와 기본 RAR 픽스처는 프로젝트 소스의 `tests/fixtures/cab-iso/`와 `tests/fixtures/native-archives/`에 유지합니다. 7z 테스트는 새로 빌드한 Rust 작업 프로세스로 자체 압축 파일을 만듭니다. 이 런타임 묶음은 공식 7-Zip/UnRAR 도구나 마운트된 이미지를 필요로 하지 않습니다. 다른 Foundation 상호 운용 참조는 [네이티브 백엔드 계약](native-codecs.md)을 따릅니다.

고정된 공개 RAR PPM, 기존 암호와 기타 입력을 처음 가져올 때는 명시적으로 실행합니다.

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

크기와 SHA-256으로 검증한 기존 고정 캐시가 있으면 캐시 경로를 바꾸고 매니페스트의 `junrar/` 하위 디렉터리 구조를 유지하면서 오프라인 모드를 명시적으로 사용합니다.

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

고정 소스와 공지는 `scripts/rar-rust-fixtures.json`과 공개 호환성 목록에 있습니다. 출력 `sources.json.environment`는 소스 루트 기준 상대 경로를 사용합니다. 현재 대응 소스 제공물에는 이 공개 캐시 파일 일곱 개가 없으며 Cargo `--offline`도 제공하지 않습니다. 준비기는 셸 환경 변수를 내보내지 않습니다. 전체 `test` 전에 아래 명시적인 전달로 하위 프로세스를 시작합니다.

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

Windows에서는 작업 프로세스 경로에 `.exe`를 추가합니다. 사용자 지정 SDK, build-dir 또는 tools-dir을 사용하면 프로세스 경로 뒤에 같은 옵션을 추가합니다. 예를 들어 `--qt-prefix /path/to/Qt/6.8.2/gcc_64`를 지정하고 프로세스 경로를 실제 빌드 디렉터리에 맞춥니다. `XDZIP_CAB_BACKEND=rust`는 고정 Rust CAB 검사를 선택하며 `password`는 테스트 하위 프로세스에만 전달하는 기존 공개 픽스처 테스트 값입니다. 이 전달은 기본 워크스페이스 테스트와 전체 ignored 묶음을 유지하며 입력을 다운로드하거나 작업 프로세스를 암묵적으로 빌드하지 않습니다.

`test`는 하위 프로세스에만 오프스크린/소프트웨어 모드를 활성화하며 이후 데스크톱 명령을 오염하지 않습니다. 일상 변경은 영향에 따라 다시 빌드하고 검증합니다. Rust 코어 전용 검증은 Qt 없이 영향을 받은 크레이트에서 `cargo test --locked`를 실행할 수 있습니다. `test --skip-rust`는 qmllint와 Qt CTest를 실행하지만 Rust 워크스페이스와 전체 네이티브 백엔드 테스트를 건너뛰며 회신에 Rust가 `not-run`으로 표시됩니다. Rust에 대한 결론이 필요하면 별도로 실행하고 실제 결과를 기록합니다.

브라우저 테스트는 기존 스위트 진입점을 통해 직접 또는 그룹별로 실행할 수 있습니다. 각 스위트 프로세스는 프로젝트 `build/test-tmp/` 아래에 고유한 짧은 임시 루트를 만듭니다. 종료 시 테스트 객체를 파괴한 후 Qt 애플리케이션을 파괴하기 전에 TEMP, TMP와 TMPDIR을 복원하고 정리합니다. 기본 설정 파일도 격리합니다. Unix는 임시 루트와 기존 IPC 이름의 바이트 길이가 실제 sun_path 용량에 맞는지 검사합니다. 경로가 너무 길면 픽스처 사전 조건이 명시적으로 실패합니다. 프로젝트 밖으로 옮기거나 제품 IPC를 변경하지 말고 체크아웃 경로를 줄이세요. 그룹의 보조 프로세스는 단일 인스턴스 IPC 디렉터리 신원을 유지하도록 부모 임시 루트를 상속하고 비공개 사전 검사 프로브를 사용하며 부모가 정리하기 전에 종료합니다. 서로 다른 그룹은 루트를 공유하지 않습니다. 언어 재시작과 열기 방식 하위 프로세스에 명시적으로 전달한 테스트 구성은 새 빈 구성 대신 계속 공유합니다. 실행 전에 하위 파일 생성, 잠금, 임시 배치, 원자적 커밋과 설정 읽기를 실제로 검증합니다. 실패 시 `Fixture precondition`과 0이 아닌 종료를 보고합니다. 테스트를 건너뛰거나 창 숨김 단언을 약화하지 않습니다. 디렉터리 존재와 QTemporaryDir::isValid만으로 쓰기 가능성이 입증되지 않으며 픽스처 통과를 위해 시스템 권한을 변경하지 않습니다.

테스트 구성 경로는 `tests/test_environment` 범위 지정 저장/복원을 사용합니다. Windows는 와이드 문자 값을 유지하고 시스템 환경 블록에서 미설정과 명시적인 빈 항목을 구분하며 Unix는 원본 환경 바이트를 유지합니다. MSVC CRT 설정기는 빈 값을 삭제하므로 네이티브 환경 블록을 별도로 복원합니다. 빈 값 존재는 Qt CRT의 빈 값/null 구분을 보장하는 대신 도우미의 processValue 또는 QProcessEnvironment로 확인합니다. 이 도우미는 순차 테스트의 메인 스레드로 제한합니다. 관련 엔진/작업 프로세스를 만들기 전에 설정하고 파괴/join 후 복원합니다. 제품 코드나 동시 환경 변경에는 사용하지 않습니다. ASCII, 중국어, ANSI로 표현할 수 없는 문자는 원래 경로의 실제 파일, 언어 복원 하위 프로세스와 실패 시 창/초안 보존을 확인해야 합니다. 자동화 결과는 실제 사람이 수행한 설정 작업과 별도로 기록합니다.

실제 제품은 모든 플랫폼에서 `xdzip`이라는 이름을 사용합니다. 기본 빌드 진입점:

| 플랫폼 | 진입점 |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

`xdzip-ui-preview` 구성 요소 예시는 실제 파일 승인에 사용하지 않습니다. build-dir을 변경하면 경로를 조정합니다. 실제 빌드 신원에는 버전, 채널, 커밋과 변경 상태를 기록합니다. 모듈과 저수준 CMake 디버깅은 [Qt 개발 애플리케이션](../../../apps/desktop/README.md)을 참조하세요.

## UI 글꼴 생성

`prepare`는 공통 도구 환경에 고정 FontTools/Brotli를 설치하고 해시 검증된 업스트림 UI 글꼴을 캐시합니다. 각 데스크톱 빌드는 지원 언어 카탈로그와 고정 QML/C++ 텍스트를 자동 수집해 `<build-directory>/generated/fonts/`에 글꼴 부분 집합을 생성합니다. 문자를 추가해도 글꼴 바이너리를 커밋할 필요가 없습니다. 준비 후에는 오프라인 빌드가 가능하며 소스 캐시가 없거나 손상되면 명시적으로 실패합니다. 직접 CMake 빌드는 `Python3_EXECUTABLE`로 해당 환경을 선택해야 합니다.

HTML 전용 미리 보기, 오프라인 내보내기, 수동 소스 가져오기와 글꼴 검사는 [공통 글꼴 안내](../../../design/assets/fonts/README.en.md)를 참조하세요. 패키징은 일치하는 제품 빌드의 글꼴 매니페스트를 재생성하지 않고 검증합니다. 고정 UI 텍스트와 사용자 데이터의 시스템 글꼴 선택은 기존 역할을 유지합니다.

## 패키징과 검증

`test`와 `package`는 빌드 회신, 소스 지문, 구성, SDK 신원과 실제 제품 파일을 확인합니다. 소스나 구성을 변경한 후 먼저 다시 빌드합니다. 패키징 전 제공할 소스를 커밋한 뒤 깨끗한 해당 커밋에서 빌드하고 테스트합니다. package는 변경 상태를 무시하거나 아티팩트 신원을 변경하거나 오래된 보고서를 재사용해 검사를 우회해서는 안 됩니다. 라이선스 자료, Rust 메타데이터와 실제 CLI 버전은 해당 소스와 일치해야 합니다. 플랫폼 도구는 기존 패키지 디렉터리의 사용과 덮어쓰기 제한을 검사합니다. 오래된 파일이 섞이지 않도록 다시 빌드한 후보에는 새 output-dir을 권장합니다.

Windows/Linux에서는 패키징 도구를 준비하고 통합 단계를 실행합니다.

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging`은 격리된 Python 검증 의존성, 고정 라이선스 텍스트와 스키마 캐시, 플랫폼 패키징 자료를 준비합니다. Windows에는 고정 CRT와 Inno Setup이 포함됩니다. 개발자는 여전히 시스템 도구를 먼저 설치해야 합니다. `package --kind runtime`은 배포된 런타임 디렉터리와 감사 자료만 준비합니다. 이후 같은 output-dir에서 `package --kind installer`를 실행해 검증된 런타임으로 설치 프로그램을 만들 수 있습니다. Linux는 실행 권한을 보존하는 tar.gz도 만듭니다. 기본 `--kind all`은 한 명령으로 두 작업을 수행합니다. Windows 패키징은 Release 또는 RelWithDebInfo를 사용하며 Debug Qt 라이브러리는 배포하지 않습니다. Windows는 배포 디렉터리와 사용자별 설치 프로그램, Linux는 런타임 디렉터리, tar.gz와 .deb를 제공합니다. Windows 검증에는 같은 구성의 브라우저 테스트 실행 파일과 Qt Test/오프스크린 테스트 입력이 필요합니다. Linux 검증에는 빌드 테스트가 생성한 m2-flat.zip이 필요합니다. 테스트 파일은 제품 디렉터리에 들어가지 않습니다. verify `--kind runtime`은 런타임을 검증하고 기본 `--kind all`은 선택한 제공물을 검증합니다. 일반 검증은 설치, 업그레이드/롤백 또는 사용자 파일 연결 변경을 수행하지 않습니다.

macOS DMG에는 일반 개발/테스트 사본이 ‘다음으로 열기’에 들어가지 않도록 문서 처리 후보로 등록할 수 있는 별도로 선택한 주 빌드가 필요합니다.

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

일반 macOS 빌드의 처리기 순위는 기본 `None`이며 사용자 기본값을 변경하지 않습니다. `Alternate`는 오래 유지할 주 앱이나 DMG 입력에만 선택합니다. 로컬 macOS 패키지는 `all`만 지원하며 필요한 ad-hoc 무결성 서명을 적용한 arm64 개발 DMG를 생성합니다. verify는 재배치된 Cocoa/Metal과 실제 추출을 확인합니다. 일반 CI는 Mac package를 실행하거나 앱 패키지를 배포, 서명, 압축 또는 업로드해서는 안 됩니다. 전용 Release 예외는 [릴리스 프로토콜](release-protocol.md)을 참조하세요. 전체 제한과 사본 등록 유지는 [Mac 배포 안내](macos-deployment.md)를 참조하세요.

구조와 기본 도구는 [Linux 배포](linux-deployment.md), [Mac DMG](macos-deployment.md), [설치와 파일 연결](installers-associations.md)을 참조하세요. Windows/Linux 설치 프로그램은 정식 서명되지 않았으며 macOS에는 Developer ID 서명과 공증이 모두 없습니다. 설치/런타임 문제는 실제 제품과 해당 플랫폼 검증으로 진단합니다. SDK에서 실행되는 것만으로 독립 패키지 검증이 입증되지는 않습니다.

## CI와의 관계

커밋이나 푸시 전에 제안한 내용에 적합한 로컬 빌드 또는 검증을 실행합니다. 문서 변경은 `python3 scripts/check-docs.py`를 실행하고 영향을 받은 생성 출력을 확인합니다. Rust 변경은 서식을 확인하고 적어도 영향받은 크레이트를 빌드하거나 테스트합니다. 격리된 형식 백엔드 관련 테스트는 위에 설명된 실제 작업 프로세스와 필요한 픽스처를 준비합니다. C++, 브리지와 QML 변경은 `python3 scripts/desktop.py build`로 영향받은 제품을 빌드한 뒤 관련 테스트를 실행합니다. 모든 변경에 전체 형식 상호 운용 또는 브라우저 스위트 전체 10회 반복이 필요한 것은 아닙니다.

QML 창을 추가하거나 변경할 때 `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py`와 영향받은 두 언어 및 여러 글꼴 크기의 레이아웃 회귀를 실행합니다. 빌드 후 `python3 scripts/desktop.py test --skip-rust`는 전체 Qt 구성 요소 및 CTest 검사를 실행합니다. 또는 Qt SDK 경로와 오프스크린 환경을 구성한 뒤 `ctest --test-dir build/desktop -R qt_task_components --output-on-failure` 같은 기존 진입점으로 관련 사례를 실행할 수 있습니다. 라디오/체크박스 변경은 `accessibleRadioActionsCommitSelection`과 `accessibleCheckboxActionsCommitIntent` 등 변경에 적합한 QAccessibleActionInterface 회귀도 실행합니다.

[일반 CI](../../../.github/workflows/docs.yml)는 가벼운 정적 검사만 수행하며 로컬 제품 빌드를 대체하지 않습니다. [Foundation](../../../.github/workflows/foundation.yml)과 [Qt 워크플로](../../../.github/workflows/qt-components.yml)는 필요할 때 수동 실행합니다. Qt는 같은 desktop.py로 빌드, Qt 테스트와 Windows/Linux 패키징 및 실행 검증을 수행합니다. [ci-desktop.ps1](../../../scripts/ci-desktop.ps1)은 필요 시 반복, 시각, 상호 운용과 설치 호환성 검사를 유지하며 일상 로컬 검증에 필수는 아닙니다.

과거 전체 제품, 실제 설치 업그레이드/롤백과 연결 수명 주기는 일회용 Windows/Linux CI 호스트 또는 격리 컨테이너에서만 실행합니다. 통합 진입점은 이를 일반 로컬 verify의 기본 동작으로 만들지 않습니다. 필요 시 Qt 워크플로의 macOS 작업은 SDK 모드 배포 감사와 Cocoa 시작/파일 열기 검사를 사용하며 로그와 JSON만 보관합니다. 전체 검사와 증거 경계는 [CI 정책](../../../developers/engineering/ci-policy.md)을 참조하세요.

## 파일과 작업 경계

열기는 탐색만 하며 자동으로 추출하지 않습니다. 같은 이름의 일반 파일은 묻기, 건너뛰기, 이름 변경과 명시적인 교체를 지원합니다. 파일 전체는 내용 검증과 동기화 후에만 공개합니다. 커밋 전 취소 또는 쓰기 실패는 기존 파일을 보존하며 이미 커밋된 추출 항목은 남습니다. 작업 창을 닫으면 세부 정보만 숨깁니다. 종료는 활성 작업 결정과 비동기 종료 배리어를 사용합니다.

ZIP은 제한된 POSIX 조건에서 상대 심볼릭 링크와 실행 비트를 복원할 수 있지만 특권 비트, 확장 속성 또는 원래 수정 시간은 복원하지 않습니다. 이는 앱 번들 설치나 서명 검증을 입증하지 않습니다. 소스와 대상은 [안전한 추출 계약](zip-safe-extraction.md)을 충족해야 합니다. 충돌 후 임시 배치는 보고하고 검사용으로 보존할 뿐이며 알 수 없는 내용을 자동 복구하거나 삭제하지 않습니다.

## Microsoft Store MSIX

Store 제출은 [유지 관리자 절차](../../../developers/market/windows/build.md)를 따릅니다.

<a id="windows-msix-实验构建"></a>

## 실험적 Windows MSIX 빌드

MSIX는 같은 커밋의 Windows Release 데스크톱 빌드, 테스트와 감사된 런타임 디렉터리를 재사용합니다. 위와 같이 `prepare --packaging`, `build`, `test`를 완료한 뒤 같은 옵션으로 실행합니다.

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

예시 신원은 로컬 개발 전용입니다. 정식 Store 신원의 패키지 이름, 게시자와 게시자 표시 이름은 Partner Center와 일치해야 합니다. 출력 `build/package/xdzip-windows-x64.msix`는 서명되지 않은 실험 패키지이며 자동으로 배포 가능한 사이드로드 패키지나 Store 인증 패키지가 되지 않습니다. 일반 사이드로드에는 게시자와 일치하고 장치가 신뢰하는 서명이 필요합니다. Windows 11의 제한된 미서명 개발 모드에는 지정된 게시자 OID가 추가로 필요하고 실행 콘텐츠에는 보통 관리자 권한이 필요합니다. 예시 신원은 해당 모드를 충족하지 않습니다. 스크립트는 인증서 생성, 서명, 설치 또는 Store 제출을 하지 않습니다. 기술적 최소 요구 사항은 Windows 10 22H2 x64이며 지원 플랫폼 약속을 확대하지 않습니다. 버전 `1.0.<product build>.0`은 표시 버전과 별개이며 빌드 번호를 모두 사용하기 전에 Store 버전 정책을 다시 검토해야 합니다.

빌드에는 깨끗한 소스, 일치하는 빌드/테스트 회신, 완전히 감사된 런타임 디렉터리와 Windows SDK MakeAppx 및 MakePRI가 필요합니다. 스크립트는 런타임과 업데이트, 연결, 메뉴 기능 마커를 다시 검사하고 별도 임시 영역에서 매니페스트와 아이콘 리소스를 생성해 패키지의 각 파일을 비교하고 해시를 기록합니다. 기존 패키지나 임시 영역을 덮어쓰지 않습니다. 매니페스트는 형식 등록부의 기존 확장자, 완전 신뢰 데스크톱 진입점과 채널별 메뉴 클래스를 선언합니다.

앱 아이콘은 배율 변형과 정확한 대상 크기에 대해 기본, 어두운 배경용 무판, 밝은 배경용 무판 형식을 포함합니다. MakePRI는 이를 `resources.pri`에 인덱싱해 Windows가 시스템 바탕판을 추가하지 않고 원하는 크기를 선택할 수 있게 합니다. 파일 연결은 별도의 형식 로고를 선언하며 기존 Windows ICO의 16–256픽셀 PNG 프레임을 아트워크, 가시 크기, 여백이나 위치를 바꾸지 않고 재사용합니다. 리소스 정책은 [msix-icons.json](../../../specs/visual/msix-icons.json)에 있습니다. 이는 설치 패키지 리소스이며 Partner Center 스토어 목록 로고와 별개입니다. `verify --kind msix`는 크기, 전체 변형, 정규 형식 이미지와 리소스 인덱스 존재를 포함해 새 아이콘 회신을 실제 패키지와 대조합니다. 변경은 업데이트된 패키지를 빌드하고 제공한 뒤 설치 사용자에게 적용됩니다.

예시는 기본 `--msix-channel sideload`를 사용해 Windows 11 클래식 메뉴만 등록합니다. Store 업로드에는 정확한 Partner Center 신원과 함께 `package --kind msix --msix-channel store`를 사용합니다. 이 경우 현대식과 클래식 메뉴를 모두 선언하지만 현대식 항목은 패키지에 실제 Store 시스템 서명이 생길 때까지 숨겨집니다. 채널 마커는 패키징 시 추가하며 감사된 런타임을 다시 빌드할 필요가 없습니다. 출력은 별도 디렉터리에 보관합니다. [메뉴 채널 정책](shell-actions.md#windows-패키지-경계)을 참조하세요. 라이선스, 실제 설치, 연결, 제거, Store 서명과 대상 시스템 실제 승인은 패키징 성공과 별개입니다.

MSIX 매니페스트는 공통 언어 등록부의 열 가지 언어를 선언해 내장 카탈로그와 일치하며 회신은 언어 목록을 기록하고 검증합니다. 이 MakeAppx 흐름은 Qt 번역 리소스를 패키지 언어 선언으로 자동 변환하지 않습니다. Partner Center의 ‘패키지에서 지원하는 언어’는 업로드된 패키지를 반영합니다. 소스 수정 후 다시 빌드하고 초안의 패키지를 교체해야 갱신됩니다. ‘추가 Store 목록 언어’는 별도로 구성하며 매니페스트 선언을 대체할 수 없습니다. [Microsoft 매니페스트 언어 문서](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region)를 참조하세요.

## 증거와 재검증

[공통 검증](../../../developers/qa/README.md)과 [필요 시 데스크톱 검증](../../../developers/qa/private-test-manual.md)에 따라 영향받은 검사를 선택합니다. 일상 작업에는 수동 승인 양식이 필요하지 않습니다. 빌드와 테스트 회신은 build-dir 아래 `desktop-build.json`과 `desktop-test.json`입니다. 준비, 패키징과 검증 회신은 output-dir 아래 `<stage>.json`이며 기본 증거에 연결합니다. 자동화 회신은 실제 단계와 소스를 기록합니다. 오프스크린/소프트웨어 렌더링, 네이티브 Cocoa 검사와 수행한 수동 검사는 별도로 기록하고 서로 대체하지 않습니다. 기존 승인은 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)에 있으며 후보 목록을 다시 생성했다는 이유만으로 반복하지 않습니다.

Windows/Linux 아티팩트 제공에는 같은 커밋의 소스와 라이선스 공지가 포함되며 Actions는 아티팩트를 14일간 보관합니다. 만료 후 신원과 고정 입력을 사용해 다시 빌드합니다. deployment-audit.json과 소스/파일 단위 CycloneDX 목록은 출처 검증을 돕지만 불완전할 수 있으며 라이선스 승인을 입증하지 않습니다. 소스 재빌드와 Qt 교체는 [소스 제공](source-delivery.md)과 [자동 검증 계약](../../../developers/engineering/candidate-verification.md)을 참조하세요.

빌드, 스크린샷, 측정값, 보고서와 패키지는 무시되는 로컬 또는 CI 아티팩트로만 보관합니다. 생성된 대기 양식, 오프스크린 검사 또는 부분 SBOM을 깨끗한 환경과 전체 릴리스 관문 통과로 설명하지 않습니다.

## macOS 업데이트 프레임워크

네이티브 빌드는 `prepare-sparkle.py`로 `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`를 준비합니다. 프레임워크 주 라이브러리는 고정 업스트림 소스와 프록시 패치에서 컴파일하며 설치 도우미, 리소스와 서명 도구는 공식 SDK 출처를 유지합니다. [의존성 계약](dependencies.md#macos-更新提供者)을 참조하세요. 유효한 빌드 회신은 재사용할 수 있지만 오래된 공식 SDK가 프록시 인터페이스가 있는 파생 프레임워크를 대체할 수는 없습니다. CI는 임시 SDK 디렉터리를 명시적으로 선택할 수 있습니다.

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`는 기존 Xcode Command Line Tools 또는 Xcode의 네이티브 Clang을 사용하고 별도 Qt SDK가 필요하지 않습니다. 원본 SDK와 소스 아카이브는 `build/source-cache/`에 캐시하며 오프라인 작업에는 `--archive`와 `--source-archive`를 별도로 전달할 수 있습니다. 빌드는 다운로드한 소스의 스크립트를 실행하거나 시스템 프록시를 변경하거나 프로덕션 서명 키에 접근하거나 프로젝트 소스 제공 아카이브를 수집하지 않습니다.

`check-macos-proxy.py`는 기본적으로 프로젝트 소유 루프백 다운로더 검사만 수행합니다. `--system-conflicts`는 전용 워크플로의 일회용 호스팅 Apple Silicon 러너로 제한됩니다. 제한된 도우미가 시스템 프록시와 고정 테스트 도메인의 hosts 설정을 준비, 복원하고 검증합니다. 이 옵션을 로컬에서 추가하지 않습니다.
