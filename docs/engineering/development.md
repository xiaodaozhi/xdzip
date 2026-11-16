# Developer preparation guide

First read the product README, AGENTS, software design 01/02/10, and the current work package. The root Cargo workspace already includes domain, formats, I/O, runtime, and bridge. CMake builds the real Qt application, component preview, and independent probes. See [directory responsibilities](repository-layout.md) for existing modules; the root README is the entry point for actual capabilities and limits.

## Obtain and modify

Clone the repository with Git, inspect the working tree and remote main, and preserve existing changes. Before committing or pushing, complete local builds or verification appropriate to the change. Maintainers may create temporary branches for short development cycles, integrate into main after relevant checks pass, use PRs as needed, and clean merged branches and the cycle's `build/` temporary directories. External contributors submit PRs under the [contributing guide](../../CONTRIBUTING.md). Check for duplicates and create or reuse an issue for new work; do not add stage results to docs. Markdown is the specification source. Colors and dimensions are centralized in specs/visual/tokens.json. Update the corresponding design and acceptance IDs when adding/changing contracts. Use [repository cleanup rules](repository-hygiene.md#研发周期工件收尾) for build cleanup; do not recursively delete all of `build/`.

## Documentation builds

Documentation tools require Python 3.10+, lxml, Pandoc, and Graphviz; system CJK fonts are recommended. Run `python3 scripts/build-design-html.py` to generate 16 detailed software design documents and a single-file edition under build/docs-html, using the Chinese source in `docs/zh-CN/design/`. The downloadable bundle is written under dist/. Read other engineering and collaboration documentation through docs/README.md. HTML is not the product UI technology choice.

Run `python3 scripts/check-docs.py` to check repository Markdown file targets, JSON, and design assets. The builder separately checks HTML internal links and structure. The former does not check Markdown heading anchors or remote URLs. Passing documentation checks does not establish actual product acceptance.

## Product toolchain preparation

Prepare Xcode Command Line Tools and Qt on macOS; MSVC Build Tools, Windows SDK, and matching Qt on Windows; and a compiler, target glibc environment, and Qt Wayland/X11 dependencies on Linux. Current development versions are pinned by root toolchain files and CI. Record sources and licenses under the [toolchain freeze table](toolchain.md); do not rely on the developer machine's “latest version.” See the [support policy](support-policy.md) for official OS support and maintenance; use the release license-review issue for specific provenance/licensing review.

Commands to build and run from a clean checkout are in the [Qt development application](../../apps/desktop/README.md), including CMake switches, macOS architecture and deployment targets, and offscreen-test boundaries. Reverify paths with spaces/Chinese when build or installation logic changes. The current personal project does not require paid signing identity; see the [build guide](developer-build.md) for local DMG ad-hoc integrity and the CI boundary prohibiting Mac app deployment.

## Independent engineering probes

The [toolchain probe](../../tools/toolchain-probe/README.md) provides a minimal build and CTest commands for a standalone Qt Quick window; it is not the desktop product. See its README for version and test boundaries. Actual conclusions remain in the [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1).

The [unified CMake/Cargo bridge probe](../../tools/bridge-probe/README.md) provides a root build entry point, locked dependencies, and lifecycle checks. Dependency records are in [cxx diagnostic admission](cxx-admission.md).
