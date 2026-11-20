# Build and contribute to XDZip

English | [简体中文](../zh-CN/contributing/README.md) | [繁體中文](../zh-TW/contributing/README.md) | [日本語](../ja-JP/contributing/README.md) | [한국어](../ko-KR/contributing/README.md) | [Français](../fr-FR/contributing/README.md) | [Deutsch](../de-DE/contributing/README.md) | [Español](../es-ES/contributing/README.md) | [Português (Brasil)](../pt-BR/contributing/README.md) | [Русский](../ru-RU/contributing/README.md)

Read the [contribution policy](../../CONTRIBUTING.md) and find or open an issue for the change. Describe the expected behavior, affected platforms and a small reproduction you can verify. Use project-owned or redistributable fixtures. The [repository layout](../engineering/repository-layout.md) shows where each part of the implementation lives.

## Build the part you need

Run the commands from the repository root. For Windows, use `python` from an x64 MSVC development environment. On macOS, use a native Apple Silicon terminal and the Xcode Command Line Tools. On Ubuntu, follow the platform dependency list. The [build guide](../engineering/developer-build.md#environment-and-entry-points) lists prerequisites and the repository-pinned Rust and Qt toolchains.

For a first desktop build:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

The setup switches install the pinned toolchain through existing rustup and download the pinned Qt SDK if needed. Reuse the shared `build/toolchain/` dependencies when already prepared. If using an existing compatible Qt SDK outside the default location, pass the same `--qt-prefix` to the relevant stages. For a change that needs the desktop test suite, run `python3 scripts/desktop.py test` against a completed build with the matching configuration. Follow the [test input preparation](../engineering/developer-build.md) for the full worker suites.

For standalone CLI work, build and then test the resulting CLI and worker:

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

The default output is `build/cli`. The CLI build needs the pinned Rust toolchain, CMake and the platform C++ toolchain. On Ubuntu, it additionally needs `libssl-dev` and `pkg-config`. See the [CLI build contract](../engineering/command-line.md#build-and-command-installation) for platform details.

## Verify the affected behavior

Reuse a matching build where available. For a documentation-only change, start with the checks below and validate the output of any affected generators. A whole desktop build is needed only when the change or required checks call for it.

```sh
python3 scripts/check-docs.py
git diff --check
```

| Change | Code and contract | What to run or verify |
| --- | --- | --- |
| Documentation and examples | Update the corresponding translations, links and anchors. The root README and `docs/` have ten language editions. The `developers/` guides use US English and Simplified Chinese. Use the relevant implementation contract and exercise changed commands with a small owned fixture. | Run the documentation check above. Open changed section links and check generated output when the page feeds a generator. |
| CLI | Start in `crates/xdzip-cli` and follow the shared Rust runtime. [CLI contract](../engineering/command-line.md) | Run the standalone CLI checks against your build. For command or output changes, verify stdout, stderr, exit status and any files written, including an applicable failure. |
| Rust domain, formats or archive tasks | Start with `xdzip-domain`, the relevant codec, `xdzip-runtime` or `xdzip-io`. For a desktop command, follow `BrowserController` and `RuntimeConnection`. [Format registry](../engineering/format-registry.md), [runtime](../engineering/product-runtime.md), [file security](../design/file-security.md) | Run `cargo fmt --all -- --check` and the affected crate tests. Exercise the actual runtime path and relevant failure, cancellation or commit behavior. If the path uses the native format worker, verify it with the [worker test entry points](../../developers/qa/runtime-worker-tests.md). On registry changes, regenerate with `python3 scripts/generate-format-registry.py`, then run it with `--check` and run `python3 scripts/test-format-registry.py`. |
| QML windows, controls or localization | Start in `apps/desktop/ui`, `resources/i18n` and `specs/visual`. Before editing a window, read the [visual specification](../visual/design-standard.md) and [design workflow](../visual/design-workflow.md), and record its contract in the [window registry](../../specs/visual/window-registry.json). | For window changes, run `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` and the affected bilingual multiple-font-size layout tests. Run `python3 scripts/check-ui-i18n.py` for localization. Validate native interaction or visual behavior on the relevant platform where required. |
| Installation, shell integration or updates | Use the [build and deployment guide](../engineering/developer-build.md), [association contract](../engineering/installers-associations.md) and [update contract](../engineering/software-update.md). | Check the affected install or registration path, actual artifact identity, errors and recovery. Use the [CI policy](../../developers/engineering/ci-policy.md) to find a matching platform workflow. |

## Describe the evidence

In your PR, identify the behavior changed, the commands and platforms tested, and any outstanding checks. Keep offscreen, native desktop and installation results explicit. The [verification guide](../../developers/qa/README.md) explains what each applies to. Find subsystem contracts in the [technical reference](../reference/README.md) and release procedures in the [maintainer handbook](../../developers/README.md).
