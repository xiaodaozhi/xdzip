#!/usr/bin/env python3
"""Guard the checked Bandizip inventory and its bilingual assessment tables."""

from collections import Counter
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
ENGLISH = ROOT / "docs/engineering/bandizip-format-assessment.md"
CHINESE = ROOT / "docs/zh-CN/engineering/bandizip-format-assessment.md"
REGISTRY = ROOT / "specs/engineering/archive-formats.json"

EXPECTED_ROWS = [
    "7Z",
    "ACE",
    "AES",
    "ALZ",
    "ARJ",
    "BH",
    "BIN",
    "BR",
    "BZ / BZ2",
    "CAB",
    "Compound(MSI)",
    "DAA(1.0)",
    "DEB",
    "EGG",
    "GZ",
    "IMG",
    "ISO",
    "ISZ",
    "LHA / LZH",
    "LZ",
    "LZMA",
    "PMA",
    "RAR / RAR5",
    "SFX(EXE)",
    "TAR",
    "TBZ / TBZ2",
    "TGZ",
    "TLZ",
    "TXZ",
    "UDF",
    "WIM",
    "XPI",
    "XZ",
    "Z",
    "ZIP",
    "ZIPX",
    "ZPAQ",
    "PEA",
    "UU / UUE",
    "XXE",
    "ASAR",
    "ZSTD",
    "NSIS",
    "LZ4",
]

EXPECTED_STATUS_COUNTS = {
    "Admitted": 30,
    "Alias/profile": 7,
    "Blocked": 0,
    "Candidate": 2,
    "Excluded": 5,
}

EXPECTED_STATUSES = {
    "7Z": "Admitted",
    "ACE": "Excluded",
    "AES": "Admitted",
    "ALZ": "Admitted",
    "ARJ": "Admitted",
    "BH": "Admitted",
    "BIN": "Candidate",
    "BR": "Admitted",
    "BZ / BZ2": "Admitted",
    "CAB": "Admitted",
    "Compound(MSI)": "Admitted",
    "DAA(1.0)": "Excluded",
    "DEB": "Admitted",
    "EGG": "Admitted",
    "GZ": "Admitted",
    "IMG": "Candidate",
    "ISO": "Admitted",
    "ISZ": "Excluded",
    "LHA / LZH": "Admitted",
    "LZ": "Admitted",
    "LZMA": "Admitted",
    "PMA": "Admitted",
    "RAR / RAR5": "Admitted",
    "SFX(EXE)": "Alias/profile",
    "TAR": "Admitted",
    "TBZ / TBZ2": "Alias/profile",
    "TGZ": "Alias/profile",
    "TLZ": "Alias/profile",
    "TXZ": "Alias/profile",
    "UDF": "Admitted",
    "WIM": "Excluded",
    "XPI": "Alias/profile",
    "XZ": "Admitted",
    "Z": "Admitted",
    "ZIP": "Admitted",
    "ZIPX": "Alias/profile",
    "ZPAQ": "Excluded",
    "PEA": "Admitted",
    "UU / UUE": "Admitted",
    "XXE": "Admitted",
    "ASAR": "Admitted",
    "ZSTD": "Admitted",
    "NSIS": "Admitted",
    "LZ4": "Admitted",
}


def _table_rows(path: Path, header: str):
    text = path.read_text(encoding="utf-8")
    start = text.find(header)
    if start < 0:
        raise AssertionError(f"missing inventory table header in {path}")
    end = text.find("\n\n", start)
    if end < 0:
        raise AssertionError(f"unterminated inventory table in {path}")
    lines = text[start:end].splitlines()
    if len(lines) < 3 or not lines[1].startswith("| ---"):
        raise AssertionError(f"invalid inventory table separator in {path}")
    rows = []
    for line in lines[2:]:
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.split("|")]
        if len(cells) < 4:
            raise AssertionError(f"invalid inventory row in {path}: {line}")
        rows.append(cells[1:4])
    return rows


class BandizipInventoryTests(unittest.TestCase):
    def test_english_inventory_is_complete_and_ordered(self):
        rows = _table_rows(ENGLISH, "| Bandizip name | xdzip status |")
        self.assertEqual([row[0] for row in rows], EXPECTED_ROWS)
        self.assertEqual({row[0]: row[1] for row in rows}, EXPECTED_STATUSES)
        self.assertEqual(Counter(row[1] for row in rows), Counter(EXPECTED_STATUS_COUNTS))
        for name, status, reason in rows:
            self.assertTrue(status)
            self.assertTrue(reason, name)

    def test_chinese_inventory_has_the_same_number_of_rows(self):
        rows = _table_rows(CHINESE, "| Bandizip 名称 | xdzip 状态 |")
        self.assertEqual(len(rows), len(EXPECTED_ROWS))
        self.assertEqual([row[0] for row in rows], EXPECTED_ROWS)
        for name, status, reason in rows:
            self.assertTrue(status)
            self.assertTrue(reason, name)

    def test_blocked_and_ambiguous_names_have_no_registry_entry(self):
        registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
        registered = {
            extension["suffix"]
            for item in registry["formats"]
            for extension in item["extensions"]
        }
        for suffix in (".ace", ".daa", ".isz", ".wim", ".zpaq", ".nsis", ".bin", ".img"):
            self.assertNotIn(suffix, registered, suffix)
        nsis = next(item for item in registry["formats"] if item["id"] == "nsis")
        self.assertEqual(nsis["extensions"], [{
            "suffix": ".exe",
            "drop": False,
            "shell": False,
            "windows_icon": None,
            "macos_icon": None,
        }])
        self.assertIn(".pea", registered)


if __name__ == "__main__":
    unittest.main()
