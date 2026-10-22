import tempfile
import unittest
from pathlib import Path

from tools import source_audit


class AccountingLayerTests(unittest.TestCase):
    def test_reference_code_and_verification_are_counted_apart_from_maintained_code(self):
        cases = {
            ("archive/crates/reference/src/recovered_drawing.rs", "production"): "reference",
            ("crates/core/src/reconstruction/mod.rs", "production"): "reference",
            ("archive/crates/translated/src/children.rs", "production"): "reference",
            ("crates/core/src/ammunition/refill.rs", "production"): "maintained",
            ("tools/ghidra/VerifyMovementPsi.java", "tests"): "reference",
            ("archive/crates/reference/tests/item_display_reference.rs", "tests"): "reference",
            ("crates/core/tests/native/binary_firearm.rs", "tests"): "native",
            ("crates/core/tests/native_common.rs", "tests"): "native",
            ("crates/core/tests/support/mod.rs", "tests"): "maintained",
        }
        for (path, role), expected in cases.items():
            self.assertEqual(source_audit.layer(Path(path), role), expected, path)

    def test_gate_caps_maintained_test_lines_not_the_ratio(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, text in {
                "crates/core/src/rules.rs": "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\n",
                "crates/core/src/recovered_x.rs": "fn r() {}\n" * 40,
                "crates/core/tests/rules.rs": "fn t() {}\n",
            }.items():
                (root / name).parent.mkdir(parents=True, exist_ok=True)
                (root / name).write_text(text)
            (root / "crates/content/src").mkdir(parents=True)
            (root / "crates/desktop/src").mkdir(parents=True)
            totals, _ = source_audit.accounting(root)
        self.assertEqual(totals["maintained"]["ratio"], 0.25)
        self.assertLess(totals["ratio"], 0.03)
        self.assertEqual(totals["passed"], 1 <= source_audit.MAINTAINED_TEST_CEILING)
