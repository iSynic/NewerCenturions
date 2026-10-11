import struct
import tempfile
import unittest
from pathlib import Path
from tools.extract import fork_bytes, resources, safe_input, span


class ExtractionTests(unittest.TestCase):
    def test_wrapper_truncation_and_duplicate_fork(self):
        header = struct.pack(">II16sH", 0x51607, 0x20000, b"", 1)
        wrapper = header + struct.pack(">III", 2, 38, 3) + b"abc"
        self.assertEqual(fork_bytes(wrapper), b"abc")
        with self.assertRaises(ValueError):
            fork_bytes(wrapper[:-1])
        with self.assertRaises(ValueError):
            fork_bytes(header[:24] + b"\x00\x00")

    def test_resource_offsets_cannot_escape_declared_areas(self):
        with self.assertRaises(ValueError):
            resources(struct.pack(">IIII", 256, 512, 10, 28))
        for offset, size in ((-1, 1), (0, -1), (3, 2), (99, 0)):
            with self.assertRaises(ValueError):
                span(b"abcd", offset, size)

    def test_manifest_path_traversal(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in ("../outside", "/outside", "C:/outside", "a\\b"):
                with self.assertRaises(ValueError):
                    safe_input(root, path)


if __name__ == "__main__":
    unittest.main()
