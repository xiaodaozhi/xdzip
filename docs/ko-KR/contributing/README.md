# XDZip 빌드와 기여

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | [繁體中文](../../zh-TW/contributing/README.md) | [日本語](../../ja-JP/contributing/README.md) | 한국어 | [Français](../../fr-FR/contributing/README.md) | [Deutsch](../../de-DE/contributing/README.md) | [Español](../../es-ES/contributing/README.md) | [Português (Brasil)](../../pt-BR/contributing/README.md) | [Русский](../../ru-RU/contributing/README.md)

[기여 정책](../../../CONTRIBUTING.md)을 읽고 변경 사항에 맞는 이슈를 찾거나 새로 만드세요. 예상 동작, 영향받는 플랫폼, 검증할 수 있는 작은 재현 예제를 적어 주세요. 프로젝트가 소유하거나 재배포할 수 있는 테스트 파일을 사용하세요. [저장소 구조](../engineering/repository-layout.md)에 구현의 각 부분이 어디에 있는지 나와 있습니다.

<a name="xdzip-build-the-part-you-need"></a>

## 필요한 부분 빌드

저장소 루트에서 명령을 실행하세요. Windows는 x64 MSVC 개발 환경에서 `python`을 사용합니다. macOS는 네이티브 Apple Silicon 터미널과 Xcode Command Line Tools를 사용합니다. Ubuntu는 해당 플랫폼의 종속성 목록을 따릅니다. [빌드 안내](../engineering/developer-build.md#environment-and-entry-points)에는 필수 조건과 저장소에 고정된 Rust 및 Qt 도구 체인이 나와 있습니다.

첫 데스크톱 빌드에서는 아래 명령을 실행합니다.

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

준비 옵션은 기존 rustup으로 고정된 도구 체인을 설치하고 필요한 경우 고정된 Qt SDK를 내려받습니다. 공유 `build/toolchain/` 종속성이 이미 준비되어 있다면 다시 사용하세요. 기본 경로 밖에 호환되는 Qt SDK가 있으면 관련 단계에 동일한 `--qt-prefix`를 전달하세요. 데스크톱 테스트가 필요한 변경은 같은 구성으로 빌드가 끝난 결과를 대상으로 `python3 scripts/desktop.py test`를 실행하세요. 전체 워커 테스트는 [테스트 입력 준비](../engineering/developer-build.md)를 참조하세요.

독립 CLI를 작업할 때는 CLI와 워커를 빌드하고 테스트하세요.

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

기본 출력 위치는 `build/cli`입니다. CLI 빌드에는 고정된 Rust 도구 체인, CMake 및 플랫폼 C++ 도구 체인이 필요합니다. Ubuntu에서는 `libssl-dev`와 `pkg-config`도 필요합니다. 플랫폼별 세부 사항은 [CLI 빌드 규칙](../engineering/command-line.md#xdzip-build-and-command-installation)을 참조하세요.

<a name="xdzip-verify-the-affected-behavior"></a>

## 영향을 받는 동작 검증

조건에 맞는 기존 빌드가 있으면 다시 사용하세요. 문서만 변경했다면 아래 검사부터 실행하고 영향을 받는 생성기의 출력을 검증하세요. 변경이나 필수 검사가 요구할 때만 전체 데스크톱 빌드를 수행합니다.

```sh
python3 scripts/check-docs.py
git diff --check
```

| 변경 사항 | 코드와 규칙 | 실행하거나 검증할 항목 |
| --- | --- | --- |
| 문서와 예제 | 해당 번역, 링크, 앵커를 갱신하세요. 루트 README와 `docs/`에는 10개 언어 버전이 있습니다. `developers/`의 안내서는 미국 영어와 중국어 간체를 사용합니다. 관련 구현 규칙에 따라 직접 만든 작은 테스트 파일로 바뀐 명령을 확인하세요. | 위 문서 검사를 실행합니다. 변경된 섹션 링크를 열고 생성기가 문서를 입력으로 쓰는 경우 생성된 출력을 검증합니다. |
| CLI | `crates/xdzip-cli`에서 공유 Rust 런타임을 따라갑니다. [CLI 규칙](../engineering/command-line.md) | 빌드한 독립 CLI를 검사하세요. 명령이나 출력이 바뀌면 stdout, stderr, 종료 코드와 기록된 파일을 검증합니다. 관련 실패 사례도 포함하세요. |
| Rust 도메인, 형식 또는 압축 파일 작업 | `xdzip-domain`, 관련 코덱, `xdzip-runtime` 또는 `xdzip-io`에서 시작합니다. 데스크톱 명령은 `BrowserController`와 `RuntimeConnection`을 따라갑니다. [형식 목록](../engineering/format-registry.md), [런타임](../engineering/product-runtime.md), [파일 안전성](../design/file-security.md) | `cargo fmt --all -- --check`와 영향받는 crate 테스트를 실행합니다. 실제 런타임 경로와 관련 실패, 취소, 최종 저장 동작을 확인하세요. 네이티브 형식 워커를 사용하면 [워커 테스트 진입점](../../../developers/qa/runtime-worker-tests.md)에서 검증합니다. 목록이 바뀌면 `python3 scripts/generate-format-registry.py`로 다시 생성하고 `--check`로 확인한 뒤 `python3 scripts/test-format-registry.py`를 실행합니다. |
| QML 창, 컨트롤 또는 현지화 | `apps/desktop/ui`, `resources/i18n`, `specs/visual`에서 시작합니다. 창을 편집하기 전에 [시각 사양](../visual/design-standard.md)과 [디자인 절차](../visual/design-workflow.md)를 읽고 [창 레지스트리](../../../specs/visual/window-registry.json)에 계약을 기록하세요. | 창 변경은 `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` 및 해당하는 두 언어와 여러 글자 크기의 레이아웃 테스트를 실행합니다. 현지화는 `python3 scripts/check-ui-i18n.py`을 실행합니다. 필요한 경우 관련 플랫폼에서 네이티브 상호작용이나 화면을 확인하세요. |
| 설치, 셸 통합 또는 업데이트 | [빌드 및 배포 안내](../engineering/developer-build.md), [파일 연결 규칙](../engineering/installers-associations.md), [업데이트 규칙](../engineering/software-update.md)을 사용합니다. | 영향을 받는 설치 또는 등록 경로, 실제 결과물 식별 정보, 오류 및 복구를 확인합니다. 해당 플랫폼 워크플로는 [CI 정책](../../../developers/engineering/ci-policy.md)을 참조하세요. |

<a name="xdzip-describe-the-evidence"></a>

## 검증 결과 작성

PR에 바뀐 동작, 검증한 명령과 플랫폼, 남은 검사를 적어 주세요. 화면 없이 수행한 테스트, 네이티브 데스크톱, 실제 설치 결과를 명시하세요. [검증 안내](../../../developers/qa/README.md)에서 각 검증의 적용 범위를 설명합니다. 하위 시스템 규칙은 [기술 문서](../reference/README.md), 릴리스 절차는 [유지보수 안내서](../../../developers/README.md)를 참조하세요.
