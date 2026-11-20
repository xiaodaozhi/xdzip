# Developer preparation guide

English | [简体中文](../zh-CN/engineering/development.md) | [繁體中文（台灣）](../zh-TW/engineering/development.md) | [日本語](../ja-JP/engineering/development.md) | [한국어](../ko-KR/engineering/development.md) | [Français](../fr-FR/engineering/development.md) | [Deutsch](../de-DE/engineering/development.md) | [Español](../es-ES/engineering/development.md) | [Português (Brasil)](../pt-BR/engineering/development.md) | [Русский](../ru-RU/engineering/development.md)

Use [Build and contribute](../contributing/README.md) to choose a build and verification path for the change. See the [system architecture](../design/system-architecture.md) and [repository layout](repository-layout.md) for the Qt, Rust runtime and codec boundaries.

## Obtain and modify

Start from current `main` and preserve other work in the checkout. Follow the [contribution policy](../../CONTRIBUTING.md) for scope and review and the [repository delivery guide](../../developers/engineering/repository-hygiene.md) for authorized integration. When a contract changes, update its language editions and the affected verification cases. Use `specs/visual` for colors and geometry.

## Documentation builds

Maintain a corresponding root README and docs page for each supported interface language. The default English pages are at `README.md` and `docs/`, and translations use `README.<locale>.md` and `docs/<locale>/`. The maintainer handbook uses US English and Simplified Chinese. Place a language selector below each title. Show the current language in plain text and link the other languages to the same topic. From a Simplified Chinese reader page, link to the Simplified Chinese maintainer page. From all other reader pages, use the default English maintainer page. Update indexes, incoming links and section anchors when adding or moving documents.

Documentation tools require Python 3.10+, lxml, Pandoc, and Graphviz; system CJK fonts are recommended. Run `python3 scripts/build-design-html.py` to generate 16 detailed software design documents and a single-file edition under build/docs-html, using the Chinese source in `docs/zh-CN/design/`. The downloadable bundle is written under dist/. Read other engineering and collaboration documentation through docs/README.md. HTML is not the product UI technology choice.

Run `python3 scripts/check-docs.py` to check translation coverage, reciprocal language selectors, reader link languages, Markdown heading anchors, JSON and design assets. This check runs offline. The HTML builder validates the links and structure of its generated pages. Run `python3 scripts/test-check-docs.py` when changing the checker.

## Product toolchain preparation

Prepare Xcode Command Line Tools and Qt on macOS; MSVC Build Tools, Windows SDK, and matching Qt on Windows; and a compiler, target glibc environment, and Qt Wayland/X11 dependencies on Linux. Current development versions are pinned by root toolchain files and CI. Record sources and licenses under the [toolchain freeze table](toolchain.md); do not rely on the developer machine's “latest version.” See the [support policy](support-policy.md) for official OS support and maintenance; use the release license-review issue for specific provenance/licensing review.

Commands to build and run from a clean checkout are in the [Qt development application](../../apps/desktop/README.md), including CMake switches, macOS architecture and deployment targets, and offscreen-test boundaries. Reverify paths with spaces/Chinese when build or installation logic changes. The current personal project does not require paid signing identity; see the [build guide](developer-build.md) for local DMG ad-hoc integrity and the CI boundary prohibiting Mac app deployment.

## Independent engineering probes

The [toolchain probe](../../tools/toolchain-probe/README.md) provides a minimal build and CTest commands for a standalone Qt Quick window; it is not the desktop product. See its README for version and test boundaries. Actual conclusions remain in the [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1).

The [unified CMake/Cargo bridge probe](../../tools/bridge-probe/README.md) provides a root build entry point, locked dependencies, and lifecycle checks. Dependency records are in [cxx diagnostic admission](cxx-admission.md).
