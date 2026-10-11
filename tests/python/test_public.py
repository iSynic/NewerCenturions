import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.audit_public import check_repository
from tools.export_source import export


class PublicSourceTests(unittest.TestCase):
    def git(self, root, *args):
        return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL)

    def repository(self, root):
        root.mkdir()
        self.git(root, "init", "-b", "main")
        self.git(root, "config", "user.name", "Publication Test")
        self.git(root, "config", "user.email", "test@example.invalid")
        for name in ("Cargo.toml", "Cargo.lock", "LICENSE", "THIRD_PARTY.md",
                     "docs/public-readme.md", "tools/Build.ps1", "tools/audit_public.py",
                     "README.md", ".gitignore"):
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("Synthetic source\n", encoding="utf-8")
        self.git(root, "add", ".")
        self.git(root, "commit", "-m", "Synthetic source")

    def test_export_excludes_assets_and_history(self):
        with tempfile.TemporaryDirectory() as directory:
            root, output = Path(directory) / "private", Path(directory) / "public"
            self.repository(root)
            (root / "evidence").mkdir()
            (root / "evidence/fixture.json").write_bytes((root / "Cargo.lock").read_bytes())
            self.git(root, "add", "evidence")
            self.git(root, "commit", "-m", "Private data")
            count = export(root, output)
            self.assertEqual(count, 9)
            self.assertFalse((output / "evidence").exists())
            self.assertFalse((output / ".git").exists())
            self.assertEqual((output / "README.md").read_bytes(),
                             (root / "docs/public-readme.md").read_bytes())
            self.git(output, "init", "-b", "main")
            self.git(output, "add", ".")
            self.assertEqual(check_repository(output)[0], count)

    def test_audit_rejects_deleted_private_history(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "repository"
            self.repository(root)
            (root / "evidence").mkdir()
            (root / "evidence/fixture.json").write_bytes((root / "Cargo.lock").read_bytes())
            self.git(root, "add", "evidence")
            self.git(root, "commit", "-m", "Private data")
            self.git(root, "rm", "evidence/fixture.json")
            self.git(root, "commit", "-m", "Remove private data")
            with self.assertRaisesRegex(ValueError, "Excluded public history"):
                check_repository(root)

    def test_export_rejects_private_paths_before_writing(self):
        with tempfile.TemporaryDirectory() as directory:
            root, output = Path(directory) / "private", Path(directory) / "public"
            self.repository(root)
            private_path = "/".join(("C:", "Users", "Private", "source"))
            (root / "tools/local.py").write_text(f'path = "{private_path}"')
            self.git(root, "add", "tools/local.py")
            with self.assertRaisesRegex(ValueError, "Private path"):
                export(root, output)
            self.assertFalse(output.exists())
            with self.assertRaisesRegex(ValueError, "fresh directory"):
                export(root, root / "public")


if __name__ == "__main__":
    unittest.main()
