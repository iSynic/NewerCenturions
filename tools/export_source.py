"""Export an allowlisted source snapshot without copying Git history or assets."""
from __future__ import annotations

import argparse
from pathlib import Path
import subprocess

try:
    from .audit_public import check_text, public_path
except ImportError:
    from audit_public import check_text, public_path


PUBLIC_IGNORE = """/NewCenturions/
/assets/
/analysis/
/reference/
/evidence/
/target/
/dist/
__pycache__/
*.tmp
*.rsrc
*.hfv
*.ROM
*.sit
*.sitx
/vendor/miniquad/Cargo.lock
"""


def export(source: Path, destination: Path) -> int:
    source, destination = source.resolve(), destination.resolve()
    if destination.exists() or destination == source or destination.is_relative_to(source):
        raise ValueError("Destination must be a fresh directory outside the source checkout")
    tracked = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=source).decode().split("\0")
    deleted = set(subprocess.check_output(["git", "ls-files", "--deleted", "-z"], cwd=source).decode().split("\0"))
    selected = [name for name in tracked if name and name not in deleted and public_path(name)]
    required = {"Cargo.toml", "Cargo.lock", "LICENSE", "THIRD_PARTY.md", "docs/public-readme.md", "tools/Build.ps1", "tools/audit_public.py"}
    if not required.issubset(selected):
        raise ValueError(f"Missing tracked public inputs: {sorted(required.difference(selected))}")
    pending = {}
    for name in selected:
        file = source / name
        if file.is_symlink():
            raise ValueError(f"Source symlink is not permitted: {name}")
        raw = file.read_bytes()
        if name == "README.md":
            raw = (source / "docs/public-readme.md").read_bytes()
        elif name == ".gitignore":
            raw = PUBLIC_IGNORE.encode()
        check_text(name, raw)
        pending[name] = raw
    destination.mkdir(parents=True)
    for name, raw in pending.items():
        file = destination / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_bytes(raw)
    return len(pending)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    count = export(Path(__file__).resolve().parents[1], args.destination)
    print(f"Exported {count} source files to {args.destination.resolve()}; no Git history or game assets copied")
