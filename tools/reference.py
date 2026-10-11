"""Use Basilisk's guest test interface without controlling Windows input."""
import argparse
import json
import os
from pathlib import Path
import subprocess


CLIENT = Path(os.environ.get("CENTURIONS_REFERENCE_CLIENT", "reference/bin/divinity-harness-client.exe"))


def request(*args):
    environment = os.environ.copy()
    environment["PATH"] = r"C:\msys64\mingw32\bin;" + environment["PATH"]
    result = subprocess.run([str(CLIENT), "--token-file", str(Path("reference/divinity-harness-token.txt").resolve()), *args],
                            env=environment, capture_output=True, text=True, check=True)
    response = json.loads(result.stdout)
    if not response.get("ok"):
        raise RuntimeError(response.get("error", "guest request failed"))
    return response


def read_region(address, length):
    parts = []
    for offset in range(0, length, 1024):
        parts.append(request("read-memory", "--address", hex(address + offset),
                             "--length", str(min(1024, length-offset)))["hex"])
    return dict(address=address, length=length, hex="".join(parts))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("info")
    memory = commands.add_parser("read")
    memory.add_argument("address")
    memory.add_argument("length", type=int)
    memory.add_argument("--output", type=Path)
    snapshot = commands.add_parser("snapshot")
    snapshot.add_argument("path", type=Path)
    click = commands.add_parser("click")
    click.add_argument("x", type=int)
    click.add_argument("y", type=int)
    key = commands.add_parser("key")
    key.add_argument("code")
    chord = commands.add_parser("chord")
    chord.add_argument("modifiers")
    chord.add_argument("code")
    text = commands.add_parser("text")
    text.add_argument("value")
    args = parser.parse_args()
    if args.command == "info":
        response = request("info")
    elif args.command == "read":
        response = read_region(int(args.address, 0), args.length)
    elif args.command == "snapshot":
        args.path.parent.mkdir(parents=True, exist_ok=True)
        response = request("snapshot-framebuffer", "--out", args.path.as_posix())
    elif args.command == "click":
        response = request("mouse-click", "--x", str(args.x), "--y", str(args.y))
    elif args.command == "key":
        response = request("press-key", "--mac-keycode", args.code)
    elif args.command == "chord":
        response = request("press-chord", "--modifiers", args.modifiers, "--mac-keycode", args.code)
    else:
        response = request("type-text", "--text", args.value)
    if getattr(args, "output", None):
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
    else:
        print(json.dumps(response, indent=2))
