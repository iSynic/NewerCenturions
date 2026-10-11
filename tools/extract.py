"""Extract Classic resource forks without modifying the input tree."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import struct


def span(data: bytes, offset: int, size: int) -> bytes:
    if offset < 0 or size < 0 or offset > len(data) or size > len(data) - offset:
        raise ValueError(f"range {offset}+{size} outside {len(data)} bytes")
    return data[offset:offset + size]


def fork_bytes(data: bytes) -> bytes:
    if data[:4] not in (b"\x00\x05\x16\x07", b"\x00\x05\x16\x00"):
        return data
    header = span(data, 0, 26)
    count = struct.unpack_from(">H", header, 24)[0]
    descriptors = span(data, 26, count * 12)
    found = []
    for i in range(count):
        ident, offset, size = struct.unpack_from(">III", descriptors, i * 12)
        value = span(data, offset, size)
        if ident == 2:
            found.append(value)
    if len(found) != 1:
        raise ValueError("AppleSingle/Double must contain exactly one resource fork")
    return found[0]


def resources(data: bytes) -> list[dict]:
    data = fork_bytes(data)
    do, mo, dl, ml = struct.unpack(">IIII", span(data, 0, 16))
    area = span(data, do, dl)
    mapping = span(data, mo, ml)
    type_offset, name_offset = struct.unpack(">HH", span(mapping, 24, 4))
    count_minus_one = struct.unpack(">H", span(mapping, type_offset, 2))[0]
    count = 0 if count_minus_one == 0xFFFF else count_minus_one + 1
    types = span(mapping, type_offset + 2, count * 8)
    result = []
    seen = set()
    for i in range(count):
        raw_type, n, refs = struct.unpack_from(">4sHH", types, i * 8)
        n += 1
        references = span(mapping, type_offset + refs, n * 12)
        for j in range(n):
            ref = span(references, j * 12, 12)
            ident, name = struct.unpack_from(">hH", ref)
            attributes = ref[4]
            offset = int.from_bytes(ref[5:8], "big")
            length = struct.unpack(">I", span(area, offset, 4))[0]
            payload = span(area, offset + 4, length)
            key = (raw_type, ident)
            if key in seen:
                raise ValueError(f"duplicate resource {key}")
            seen.add(key)
            resource_name = None
            if name != 0xFFFF:
                pos = name_offset + name
                length_name = span(mapping, pos, 1)[0]
                resource_name = span(mapping, pos + 1, length_name).decode("mac_roman")
            result.append(dict(type=raw_type.decode("mac_roman"), type_hex=raw_type.hex(),
                               id=ident, name=resource_name, attributes=attributes,
                               fork_offset=do + offset + 4, size=len(payload),
                               sha256=hashlib.sha256(payload).hexdigest(), payload=payload))
    return result


def safe_input(root: Path, relative: str) -> Path:
    parts = PurePosixPath(relative).parts
    if not parts or any(p in ("..", "/") or ":" in p or "\\" in p for p in parts):
        raise ValueError(f"unsafe manifest path: {relative!r}")
    path = (root / Path(*parts)).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError("manifest path escapes input root")
    return path


def extract(root: Path, output: Path) -> dict:
    root, output = root.resolve(), output.resolve()
    if output == root or output.is_relative_to(root):
        raise ValueError("output must be outside the original input tree")
    mapping = json.loads((root / "packsmith-mapping.json").read_text(encoding="utf-8"))
    report = dict(schema_version=1, input_manifest_sha256=hashlib.sha256(
        (root / "packsmith-mapping.json").read_bytes()).hexdigest(), forks=[], resources=[])
    # Validate every input before writing any outputs.
    pending = []
    for entry in mapping["entries"]:
        for kind, hash_key, path_key in (("data", "data_sha256", "output"),
                                         ("resource", "resource_sha256", "sidecar")):
            if hash_key not in entry:
                continue
            path = safe_input(root, entry[path_key])
            raw = path.read_bytes()
            data = fork_bytes(raw) if kind == "resource" else raw
            digest = hashlib.sha256(data).hexdigest()
            if digest != entry[hash_key]:
                raise ValueError(f"checksum mismatch: {entry[path_key]}")
            report["forks"].append(dict(original=entry["original"], path=entry[path_key],
                                        kind=kind, size=len(data), sha256=digest))
            if kind == "resource":
                for res in resources(data):
                    payload = res.pop("payload")
                    destination = f"resources/{entry['id']}/{res['type_hex']}_{res['id']}.bin"
                    res.update(original=entry["original"], input_path=entry[path_key],
                               output=destination)
                    report["resources"].append(res)
                    pending.append((destination, payload))
    output.mkdir(parents=True, exist_ok=True)
    for destination, payload in pending:
        path = output / destination
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)
    (output / "manifest.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path("NewCenturions"))
    parser.add_argument("--output", type=Path, default=Path("analysis/extracted"))
    args = parser.parse_args()
    result = extract(args.input, args.output)
    print(f"Verified {len(result['forks'])} forks; extracted {len(result['resources'])} resources")
