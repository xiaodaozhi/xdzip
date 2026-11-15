#!/usr/bin/env python3
"""Regression checks for the M10 candidate source and license gates."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
ENGLISH = ROOT / "docs/engineering/m10-candidate-source-gates.md"
CHINESE = ROOT / "docs/zh-CN/engineering/m10-candidate-source-gates.md"
REGISTRY = ROOT / "specs/engineering/archive-formats.json"

CANDIDATES = ("DAA 1.0", "ISZ", "ZPAQ", "PEA", "BOND", "J2J")
BLOCKED_CANDIDATES = ("DAA 1.0", "ISZ", "ZPAQ", "BOND", "J2J")


def section(document: str, heading: str) -> str:
    marker = f"## {heading}\n"
    start = document.find(marker)
    if start < 0:
        raise AssertionError(f"missing section: {heading}")
    end = document.find("\n## ", start + len(marker))
    return document[start:] if end < 0 else document[start:end]


class M10SourceGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.english = ENGLISH.read_text(encoding="utf-8")
        cls.chinese = CHINESE.read_text(encoding="utf-8")
        cls.registry = json.loads(REGISTRY.read_text(encoding="utf-8"))

    def test_bilingual_candidate_sections_have_required_evidence_fields(self):
        for candidate in CANDIDATES:
            with self.subTest(candidate=candidate):
                english = section(self.english, candidate)
                self.assertIn("**Public specification:**", english)
                self.assertTrue(
                    "**Reference implementation:**" in english
                    or "**Reference implementations:**" in english
                )
                self.assertIn("**License and redistribution:**", english)
                self.assertIn("**Complete samples:**", english)
                self.assertIn("**Clean-room feasibility:**", english)
                self.assertIn("**Capability boundary:**", english)
                self.assertIn("**Conclusion:**", english)

                chinese = section(self.chinese, candidate)
                for field in ("**公开规范**", "**参考实现**", "**许可和再分发**",
                              "**完整样本**", "**净室可行性**", "**能力边界**", "**结论**"):
                    self.assertIn(field, chinese)

    def test_blocked_candidates_have_explicit_blocked_or_unconfirmed_conclusions(self):
        for candidate in BLOCKED_CANDIDATES:
            with self.subTest(candidate=candidate):
                english = section(self.english, candidate)
                self.assertRegex(english, r"\*\*Conclusion:\*\* \*\*(?:Blocked|Blocked pending)")
                chinese = section(self.chinese, candidate)
                self.assertRegex(chinese, r"\*\*结论\*\*：\*\*(?:阻塞|等待)")

    def test_pea_has_an_explicit_admitted_profile(self):
        self.assertIn("**Conclusion:** **Admitted as a conservative read-only profile.", section(self.english, "PEA"))
        self.assertIn("**结论**：**已准入保守只读配置", section(self.chinese, "PEA"))

    def test_blocked_candidates_are_absent_from_product_registry(self):
        serialized = json.dumps(self.registry, ensure_ascii=False).lower()
        for candidate in ("daa", "isz", "zpaq", "bond", "j2j"):
            with self.subTest(candidate=candidate):
                self.assertNotIn(f'"id": "{candidate}"', serialized)


if __name__ == "__main__":
    unittest.main()
