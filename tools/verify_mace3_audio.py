"""Capture independent FFmpeg MACE3 PCM references from the extracted sound corpus."""

import argparse
import json
from pathlib import Path
import random
import struct
import subprocess
import tempfile

from repository import data_digest as digest


def chunk(tag, data):
    return tag + struct.pack(">I", len(data)) + data + (b"\0" if len(data) & 1 else b"")


def container(payload, rate):
    comm = struct.pack(">HIH", 1, len(payload) * 3, 8) + rate + b"MAC3\x04MACE\0"
    body = b"AIFC" + chunk(b"FVER", struct.pack(">I", 0xA2805140))
    body += chunk(b"COMM", comm) + chunk(b"SSND", bytes(8) + payload)
    return b"FORM" + struct.pack(">I", len(body)) + body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--resources", type=Path, default=Path("analysis/extracted/resources"))
    parser.add_argument("--ffmpeg", default="ffmpeg")
    parser.add_argument("--output", type=Path, default=Path("evidence/fixtures/sound-mace3.json"))
    args = parser.parse_args()
    version = subprocess.run([args.ffmpeg, "-version"], check=True, capture_output=True, text=True).stdout.splitlines()[0]
    cases = []
    corpus = []
    for path in sorted(args.resources.rglob("736e6420_*.bin")):
        resource = path.read_bytes()
        if len(resource) < 42:
            continue
        fmt = int.from_bytes(resource[:2], "big")
        command = 12 if fmt == 1 else 6
        offset = int.from_bytes(resource[command + 4:command + 8], "big")
        if len(resource) <= offset + 20 or resource[offset + 20] != 0xFE:
            continue
        header = resource[offset:offset + 64]
        channels = int.from_bytes(header[4:8], "big")
        compression = int.from_bytes(header[56:58], "big", signed=True)
        entry = {"path": path.as_posix(), "sha256": digest(resource), "channels": channels,
                 "compression_id": compression, "format": header[40:44].decode("ascii", errors="replace")}
        corpus.append(entry)
        if channels != 1 or compression != 3:
            continue
        frames = int.from_bytes(header[22:26], "big")
        payload = resource[offset + 64:offset + 64 + frames * 2]
        cases.append({"name": path.stem, "input": resource.hex(), "input_sha256": digest(resource),
                      "rate_fixed": int.from_bytes(header[8:12], "big"), "packets": frames,
                      "payload": payload.hex(), "rate80": header[26:36].hex()})
    for value in range(256):
        cases.append({"name": f"cold-{value:02x}", "payload": bytes([value, value ^ 0xA5]).hex()})
    streams = {f"repeat-{value:02x}": bytes([value]) * 4096 for value in [0, 0xFF, 0x55, 0xAA, 0x6D, 0x92]}
    streams["ascending"] = bytes(range(256)) * 16
    streams["descending"] = bytes(range(255, -1, -1)) * 16
    source = random.Random(10121)
    streams["mixed"] = bytes(source.randrange(256) for _ in range(8192))
    for name, payload in streams.items():
        cases.append({"name": name, "payload": payload.hex()})
    with tempfile.TemporaryDirectory(prefix="mace-reference-") as directory:
        directory = Path(directory)
        for case in cases:
            payload = bytes.fromhex(case["payload"])
            rate = bytes.fromhex(case.get("rate80", "400cac44000000000000"))
            source_path, pcm_path = directory / "reference.aifc", directory / "reference.pcm"
            source_path.write_bytes(container(payload, rate))
            subprocess.run([args.ffmpeg, "-hide_banner", "-loglevel", "error", "-y", "-i", str(source_path),
                            "-f", "s16le", str(pcm_path)], check=True, capture_output=True)
            pcm = pcm_path.read_bytes()
            if len(pcm) != len(payload) * 6:
                raise ValueError(f"unexpected PCM size: {case['name']}")
            case["pcm_le"] = pcm.hex()
            case["pcm_le_sha256"] = digest(pcm)
            case["samples"] = len(payload) * 3
    fixture = {"reference": version, "output": "mono signed 16-bit little-endian PCM",
               "corpus": corpus, "cases": cases}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")
    print(f"Captured {len(cases)} cases; {len(corpus)} compressed resources; {args.output}")


if __name__ == "__main__":
    main()
