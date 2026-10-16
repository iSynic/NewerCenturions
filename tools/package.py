"""Package the Windows runtime without original game content or fixture data."""
from __future__ import annotations

import hashlib
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tomllib
import zipfile


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    version = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", nargs="?", type=Path, default=root / "dist" / f"The New Centurions Windows {version} source-only")
    parser.add_argument("--build-directory", type=Path, default=root / "target/release")
    parser.add_argument("--provenance", type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    build = args.build_directory.resolve()
    archive = output.parent / (output.name + ".zip")
    if output.exists() or archive.exists():
        raise SystemExit("Package directory and archive must both be fresh")
    files = {
        build / "centurions-desktop.exe": "The New Centurions.exe",
        build / "centurions-tools.exe": "centurions-tools.exe",
        "LICENSE": "LICENSE",
        "THIRD_PARTY.md": "THIRD_PARTY.md",
        "docs/public-readme.md": "README.md",
        "docs/reconstruction.md": "docs/reconstruction.md",
        "vendor/miniquad/LICENSE-MIT": "licenses/miniquad-MIT.txt",
        "vendor/miniquad/LICENSE-APACHE": "licenses/miniquad-APACHE.txt",
        "vendor/miniquad/LOCAL_CHANGES.md": "licenses/miniquad-local-changes.md",
        "vendor/mace3/LICENSE": "licenses/mace3.txt",
        "vendor/mace3/LOCAL_CHANGES.md": "licenses/mace3-local-changes.md",
    }
    if args.provenance:
        files[args.provenance.resolve()] = "build-provenance.json"
    for source in files:
        if not (root / source).is_file():
            raise SystemExit(f"Missing package input: {source}")
    output.mkdir(parents=True)
    entries = []
    for source, destination in files.items():
        target = output / destination
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(root / source, target)
        raw = target.read_bytes()
        entries.append(dict(path=destination, size=len(raw), sha256=hashlib.sha256(raw).hexdigest()))
    receipt = dict(
        schema_version=1, application_version=version, contains_original_game_assets=False,
        source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        source_dirty=bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True).strip()),
        files=entries,
    )
    (output / "package-manifest.json").write_text(json.dumps(receipt, indent=2) + "\n")
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as package:
        for path in sorted(output.rglob("*")):
            if path.is_file():
                package.write(path, path.relative_to(output))
    print(f"Packaged {len(entries)} runtime/documentation files; no original game assets")
    print(f"Archive: {archive}")


if __name__ == "__main__":
    main()
