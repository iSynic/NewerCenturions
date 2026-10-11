"""Validate and fingerprint independently generated mission fixtures."""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(root: Path, fixture: Path, toolchain: dict) -> dict:
    data = json.loads(fixture.read_text(encoding="utf-8"))
    declared = {**data, **data.get("identity", {}), **data.get("cache_identity", {})}
    java = {p.stem: p for p in (root / "tools/ghidra").glob("*.java")}
    sources = list(java.values())
    inputs = sorted(p for p in (root / "NewCenturions").rglob("*") if p.is_file())
    generated = sorted(p for p in (root / "evidence/fixtures").glob("*.json"))
    mapped = root / "analysis/ghidra/startup/code1-mapped.bin"
    auxiliary_code = root / "analysis/extracted/resources/106/434f4445_3.bin"
    candidates = sources + inputs + generated + [mapped, auxiliary_code]
    by_hash: dict[str, list[Path]] = {}
    for path in candidates:
        if path.is_file():
            by_hash.setdefault(digest(path), []).append(path)
    probe_hash = declared.get("probe_sha256")
    probes = [p for p in by_hash.get(probe_hash, []) if p.suffix == ".java"]
    if len(probes) != 1:
        raise ValueError(f"{fixture.name}: missing or changed probe definition")
    for key, value in declared.items():
        if key.endswith("_sha256") and value not in by_hash:
            raise ValueError(f"{fixture.name}: changed or unavailable {key}")
    if declared.get("ghidra", toolchain["ghidra"]) != toolchain["ghidra"]:
        raise ValueError(f"{fixture.name}: toolchain version differs")
    definitions: dict[str, str] = {}
    pending = probes.copy()
    while pending:
        path = pending.pop()
        relative = path.relative_to(root).as_posix()
        if relative in definitions:
            continue
        definitions[relative] = digest(path)
        text = path.read_text(encoding="utf-8")
        for name, dependency in java.items():
            if re.search(r"\b" + re.escape(name) + r"\b", text):
                pending.append(dependency)
    return {
        "executable": digest(mapped),
        "inputs": {p.relative_to(root).as_posix(): digest(p) for p in inputs},
        "probe_and_boundary_definitions": dict(sorted(definitions.items())),
        "fixture": digest(fixture),
        "toolchain": toolchain,
    }


def reusable(cached: dict, current: dict) -> bool:
    return cached == current


def main() -> None:
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("fixtures", nargs="+")
    parser.add_argument("--ghidra", required=True, type=Path)
    parser.add_argument("--receipt", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    properties = args.ghidra / "Ghidra/application.properties"
    text = properties.read_text(encoding="utf-8")
    version = re.search(r"^application.version=(.+)$", text, re.MULTILINE).group(1)
    toolchain = {"ghidra": version, "application_properties": digest(properties), "boundary_receipt_schema": 1}
    entries = {name: identity(root, root / f"evidence/fixtures/{name}.json", toolchain) for name in args.fixtures}
    if args.receipt.exists():
        cached = json.loads(args.receipt.read_text(encoding="utf-8"))
        if not reusable(cached["entries"], entries):
            raise SystemExit("Fixture cache is stale; regenerate affected original probes before replacing the receipt")
    else:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps({"schema_version": 1, "entries": entries}, indent=2) + "\n", encoding="utf-8")
    print(f"Validated executable, original inputs, probe/boundary definitions and toolchain for {len(entries)} fixtures")


if __name__ == "__main__":
    main()
