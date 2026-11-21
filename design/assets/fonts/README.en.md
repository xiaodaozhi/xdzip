# Shared HTML and Qt interface font

English | [简体中文](README.md)

Xdzip Design Sans SC is a variable subset of official Noto Sans SC 2.004-H2 for fixed interface text and HTML designs, with weights 400–750. Qt uses a lossless TTF export of the same WOFF2, preserving glyphs, weights and text metrics. Filenames, directories, paths and other user data still prefer the system family captured at startup, with Qt/platform fallback for missing glyphs.

## Automatic generation

The repository retains only the [pinned source and generation configuration](../../../specs/dependencies/ui-font.json), CSS, tool requirements and OFL license. The complete upstream font is cached at `build/toolchain/fonts/<SHA-256>/source.ttf`; generated TTF, WOFF2 and result manifests remain in the build directory and are never committed.

`python scripts/desktop.py prepare` prepares the shared Python tools, pinned FontTools/Brotli versions and SHA-256 verified upstream font. `python scripts/desktop.py build` then generates the subset offline using that environment. For direct CMake use, select the same tool environment through `Python3_EXECUTABLE`; every `cmake --build` checks current text, including added and removed source files. Qt outputs are in `<build-directory>/generated/fonts/`, retaining the resource URL `:/xdzip/fonts/xdzip-design-sans-sc.ttf`.

The generator collects language catalogs under `resources/i18n/`, fixed QML/C++ interface text, HTML/CSS/JavaScript design text, ASCII and explicitly retained characters. Ordinary copy changes edit only their original source, without maintaining another character list. Source font, algorithm, configuration, tool versions and the sorted character set determine generation identity. An unchanged character set reuses output without rewriting fonts or triggering extra resource compilation.

Prepare before building when the source cache is missing; remove a corrupt `source.ttf` before preparing again after a hash mismatch. Unsupported upstream characters fail the build with their codepoints instead of silently falling back. Downloads are permitted during preparation only. Build and packaging do not download fonts; packaging validates the matching build manifest without regenerating the subset.

## HTML preview and offline export

HTML-only review uses the same `build/toolchain/desktop-tools` environment. If it does not exist, first run `python3 -m venv build/toolchain/desktop-tools`, then run from the repository root:

```sh
build/toolchain/desktop-tools/bin/python -m pip install -r design/assets/fonts/requirements.txt
python3 scripts/ui-fonts.py prepare
build/toolchain/desktop-tools/bin/python scripts/ui-fonts.py check
```

To import an existing complete upstream font, use `python scripts/ui-fonts.py prepare --source /path/to/NotoSansSC.ttf`; the fixed hash remains mandatory. Use `prepare --offline` to check the source cache. On Windows, replace `bin/python` with `Scripts/python.exe` and `python3` with `python`.

## Source and license

The source is [Google Fonts](https://github.com/google/fonts/tree/a85815a42757630ce188fdad368c2dfc444d4773/ofl/notosanssc) commit `a85815a42757630ce188fdad368c2dfc444d4773`. Upstream `NotoSansSC[wght].ttf` has SHA-256 `a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da`. The pinned configuration records original provenance, version and tools; each build's `manifest.json` records generated hashes and actual requested characters.

Copyright is Adobe 2014–2021, with Reserved Font Name `Source`; the derivative uses a distinct family name. Fonts and derivatives use [OFL-1.1](OFL.txt), not the project's Apache-2.0 license. Offline HTML and product notices preserve complete copyright and license text. A shared font does not establish identity with unknown fonts in historical images; platform antialiasing and user-data glyph rendering still vary.
