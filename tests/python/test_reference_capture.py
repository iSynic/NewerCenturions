import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import capture_reference


class ReferenceCaptureTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.previous = Path.cwd()
        os.chdir(self.directory.name)
        Path("reference").mkdir()
        Path("analysis/native").mkdir(parents=True)
        Path("reference/basilisk.pid").write_text("17")
        Path("analysis/native/reference-workaround.json").write_text(json.dumps({
            "pid": 17, "patch": {"code1_offset": "0x3ebde", "original": "4e56ffee", "replacement": "70004e75"}
        }))
        self.code = "4e56ffee"
        self.seed_reads = 0
        self.changing_seed = False

    def tearDown(self):
        os.chdir(self.previous)
        self.directory.cleanup()

    def read(self, address, length):
        if address == 0x904:
            value = "00020000"
        elif address == 0x20000:
            value = "00005000"
        elif address == 0x20000 - 0x1de2:
            value = "00021000"
        elif address == 0x21000 + 0x3ebde:
            value = self.code
        elif address == 0x5000 - 0x7e:
            self.seed_reads += 1
            value = "00000002" if self.changing_seed and self.seed_reads > 1 else "00000001"
        else:
            value = "00" * length
        return {"address": address, "length": length, "hex": value}

    def test_reloaded_original_code_does_not_inherit_stale_patch_label(self):
        with patch.object(capture_reference.reference, "read_region", side_effect=self.read):
            state = capture_reference.capture()
        self.assertNotIn("reference_modification", state)
        self.assertEqual(state["presentation_code"]["hex"], "4e56ffee")

    def test_actual_patch_is_recorded(self):
        self.code = "70004e75"
        with patch.object(capture_reference.reference, "read_region", side_effect=self.read):
            state = capture_reference.capture()
        self.assertEqual(state["reference_modification"]["replacement"], self.code)

    def test_rng_change_rejects_incoherent_capture(self):
        self.changing_seed = True
        with patch.object(capture_reference.reference, "read_region", side_effect=self.read):
            with self.assertRaisesRegex(RuntimeError, "RNG changed during capture"):
                capture_reference.capture()
