#!/usr/bin/env python3
"""Build the HTML review font from a verified upstream source; never download it here."""

import argparse
import hashlib
import json
import re
from pathlib import Path

import brotli
from fontTools import __version__ as fonttools_version
from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parents[2]
OUTPUT = DIRECTORY / "xdzip-design-sans-sc.woff2"
MANIFEST = DIRECTORY / "manifest.json"
UPSTREAM_COMMIT = "a85815a42757630ce188fdad368c2dfc444d4773"
UPSTREAM_SHA256 = "a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def current_characters() -> set[int]:
    """Include fixture text, UI labels, ASCII and a small set of stable punctuation."""
    codepoints = set(range(0x20, 0x7F))
    codepoints.update(map(ord, "\u00a0\u00b7×–—‘’“”…‰′″›←↑→↓−"))
    for path in sorted((ROOT / "design/html").iterdir()):
        if path.suffix not in {".html", ".css", ".js"} or path.name == "assets.js":
            continue
        source = path.read_text(encoding="utf-8")
        codepoints.update(ord(character) for character in source if ord(character) >= 0x20)
        codepoints.update(int(value, 16) for value in re.findall(r"\\u([0-9a-fA-F]{4})", source))
    # Visible QML/C++ labels share the same metrics as HTML; do not rely on platform
    # fallback for newly implemented operation windows.
    paths = list((ROOT / "apps/desktop/ui").glob("*.qml")) + [ROOT / "apps/desktop/BrowserController.cpp"]
    for path in paths:
        source = path.read_text(encoding="utf-8")
        for label in re.findall(r'(?:qsTr|\btr)\("((?:[^"\\]|\\.)*)"\)', source):
            codepoints.update(ord(c) for c in label if ord(c) >= 0x20)
    for path in sorted((ROOT / "resources/i18n").glob("*.json")):
        catalog = json.loads(path.read_text(encoding="utf-8"))
        for value in catalog["messages"].values():
            texts = value.values() if isinstance(value, dict) else [value]
            for label in texts:
                codepoints.update(ord(c) for c in label if ord(c) >= 0x20)
    return codepoints


def verify(font: TTFont) -> None:
    missing = sorted(current_characters() - set(font.getBestCmap()))
    if missing:
        descriptions = ", ".join(f"U+{value:04X} ({chr(value)})" for value in missing)
        raise SystemExit(f"Missing HTML glyphs; regenerate the subset: {descriptions}")
    axes = [(axis.axisTag, axis.minValue, axis.defaultValue, axis.maxValue) for axis in font["fvar"].axes]
    if axes != [("wght", 400.0, 400.0, 750.0)]:
        raise SystemExit(f"Unexpected variable font axes: {axes}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, help="Pinned NotoSansSC[wght].ttf source")
    parser.add_argument("--check", action="store_true", help="Validate coverage, axis and recorded hashes")
    arguments = parser.parse_args()
    if arguments.check:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        if sha256(OUTPUT) != manifest["subset"]["sha256"]:
            raise SystemExit("The subset differs from its recorded SHA-256")
        if sha256(DIRECTORY / "OFL.txt") != manifest["license"]["sha256"]:
            raise SystemExit("The OFL license differs from its recorded SHA-256")
        verify(TTFont(OUTPUT))
        print(f"HTML font validated: {len(current_characters())} requested characters, {OUTPUT.stat().st_size} bytes")
        return
    if arguments.source is None:
        parser.error("A verified upstream source font is required")
    if sha256(arguments.source) != UPSTREAM_SHA256:
        raise SystemExit("Source font SHA-256 does not match the pinned upstream font")

    font = TTFont(arguments.source, recalcTimestamp=False)
    version = font["name"].getDebugName(5)
    options = subset.Options()
    options.notdef_outline = True
    options.recommended_glyphs = True
    options.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14, 16, 17, 25]
    options.name_legacy = True
    options.name_languages = [0x409]
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=current_characters())
    subsetter.subset(font)
    # Limit the already-small subset; this preserves all CSS weights currently used.
    font = instantiateVariableFont(font, {"wght": (400, 400, 750)}, inplace=True)

    # The derivative has a distinct family name; the upstream copyright and OFL remain.
    names = {1: "Xdzip Design Sans SC", 2: "Regular", 3: "XdzipDesignSansSC-HTML-v1",
             4: "Xdzip Design Sans SC", 6: "XdzipDesignSansSC", 16: "Xdzip Design Sans SC",
             17: "Regular", 25: "XdzipDesignSansSC"}
    for identifier, value in names.items():
        font["name"].setName(value, identifier, 3, 1, 0x409)
    font["OS/2"].usWeightClass = 400
    font.flavor = "woff2"
    font.save(OUTPUT)
    verify(TTFont(OUTPUT))
    manifest = {
        "purpose": "Shared HTML and Qt UI font metrics; native TTF exported without outline or metric changes",
        "upstream": {
            "repository": "https://github.com/google/fonts",
            "commit": UPSTREAM_COMMIT,
            "path": "ofl/notosanssc/NotoSansSC[wght].ttf",
            "sha256": UPSTREAM_SHA256,
            "version": version,
            "original_source_repository": "https://github.com/notofonts/noto-cjk",
            "original_source_commit": "523d033d6cb47f4a80c58a35753646f5c3608a78",
        },
        "license": {
            "spdx": "OFL-1.1",
            "path": "OFL.txt",
            "sha256": sha256(DIRECTORY / "OFL.txt"),
            "copyright": "(c) 2014-2021 Adobe (http://www.adobe.com/), with Reserved Font Name 'Source'.",
            "derivative_family": "Xdzip Design Sans SC",
        },
        "subset": {
            "path": OUTPUT.name,
            "sha256": sha256(OUTPUT),
            "bytes": OUTPUT.stat().st_size,
            "format": "WOFF2",
            "fonttools_version": fonttools_version,
            "brotli_version": brotli.__version__,
            "weight_axis": {"min": 400, "default": 400, "max": 750},
            "unicode_count": len(font.getBestCmap()),
            "requested_codepoints": [f"U+{value:04X}" for value in sorted(current_characters())],
        },
    }
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Generated {OUTPUT.relative_to(ROOT)}: {OUTPUT.stat().st_size} bytes")


if __name__ == "__main__":
    main()
