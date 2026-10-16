"""Check public tracked files and reachable history for excluded local content."""
from __future__ import annotations

from pathlib import Path, PurePosixPath
import re
import subprocess
import sys


TOP_LEVEL = {".gitattributes", ".gitignore", "AGENTS.md", "Cargo.toml", "Cargo.lock", "README.md", "LICENSE", "THIRD_PARTY.md"}
PRIVATE_TOOLS = {"tools/package_local.py", "tools/registration_status.py"}


def public_path(name: str) -> bool:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name:
        return False
    if name in TOP_LEVEL or name in {"docs/public-readme.md", "docs/reconstruction.md", ".github/workflows/build.yml"}:
        return True
    if name in PRIVATE_TOOLS:
        return False
    if name.startswith("crates/"):
        return path.name == "Cargo.toml" or path.suffix == ".rs"
    if name.startswith("tools/"):
        return path.suffix in {".py", ".ps1", ".java", ".ldefs", ".slaspec"}
    if name.startswith("tests/python/"):
        return path.suffix == ".py"
    if name.startswith("tests/native/"):
        return path.suffix == ".java"
    if name.startswith("vendor/"):
        return path.name in {"LICENSE", "LICENSE-MIT", "LICENSE-APACHE"} or path.suffix in {".rs", ".toml", ".md"}
    return False


def check_text(name: str, raw: bytes) -> None:
    text = raw.decode("utf-8")
    if "\0" in text:
        raise ValueError(f"Binary payload in {name}")
    patterns = [
        r"(?i)[a-z]:[\\/]+Users[\\/]+[^\\/\s]+",
        r"(?i)[a-z]:[^\n\r\"']*Codex",
        r"\bgh[pousr]_[A-Za-z0-9]{20,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{20,}\b",
        r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
    ]
    if any(re.search(pattern, text) for pattern in patterns):
        raise ValueError(f"Private path or credential pattern in {name}")
    if re.search(r"\b[0-9a-fA-F]{256,}\b", text):
        raise ValueError(f"Embedded bulk hex payload in {name}")


def check_repository(root: Path) -> tuple[int, int]:
    records = subprocess.check_output(["git", "ls-files", "--stage", "-z"], cwd=root).split(b"\0")
    blobs = {}
    files = 0
    for record in records:
        if not record:
            continue
        header, name_bytes = record.split(b"\t", 1)
        mode, oid, stage = header.decode().split()
        name = name_bytes.decode("utf-8")
        if not public_path(name) or mode not in {"100644", "100755"} or stage != "0":
            raise ValueError(f"Excluded public index entry: {name}")
        blobs.setdefault(oid, name)
        files += 1
    commits = subprocess.check_output(["git", "rev-list", "--all"], cwd=root, text=True)
    for commit in commits.splitlines():
        tree = subprocess.check_output(["git", "ls-tree", "-r", "-z", "--full-tree", commit], cwd=root)
        for record in tree.split(b"\0"):
            if not record:
                continue
            header, name_bytes = record.split(b"\t", 1)
            mode, kind, oid = header.decode().split()
            name = name_bytes.decode("utf-8")
            if not public_path(name) or kind != "blob" or mode not in {"100644", "100755"}:
                raise ValueError(f"Excluded public history entry: {name}")
            blobs.setdefault(oid, name)
    requests = ("\n".join(blobs) + "\n").encode()
    batch = subprocess.check_output(["git", "cat-file", "--batch"], cwd=root, input=requests)
    cursor = 0
    for oid, name in blobs.items():
        end = batch.index(b"\n", cursor)
        found, kind, size = batch[cursor:end].decode().split()
        if found != oid or kind != "blob":
            raise ValueError(f"Unexpected Git object for {name}")
        cursor = end + 1
        check_text(name, batch[cursor:cursor + int(size)])
        cursor += int(size) + 1
    return files, len(blobs)


if __name__ == "__main__":
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]
    count, versions = check_repository(root)
    print(f"Public audit passed: {count} tracked files, {versions} source blobs; no original assets, fixture data or private paths")
