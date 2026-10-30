"""Extract original dispatch domains from private Ghidra exports.

Writes evidence/original-dispatch.json with the newland opcode arms and the
attack2 natural-condition arms for warrior and NPC targets. Only case values,
saving-throw indices and result codes are recorded, never decompiled source.
resolvespell and castspell dispatch through range guards on reused temporaries,
so their domains are not generated here.
"""
import argparse
import json
import re
from pathlib import Path

from repository import ROOT, data_digest

NEWLAND = "analysis/reconstruction/application-actions-1/001400e4.c"
ATTACK2 = "analysis/ghidra/m6-exports/attack2.c"
CASE = r"^{indent}case (\w+):"


def signed(text):
    value = int(text, 0)
    return value - 0x10000 if value >= 0x8000 else value


def source(path):
    data = (ROOT / path).read_bytes()
    return data.decode("utf-8"), {"path": path, "sha256": data_digest(data)}


def switch_arms(lines, start, indent):
    """Case values and bodies of the switch opening at lines[start]."""
    case = re.compile(CASE.format(indent=indent))
    close = indent + "}"
    arms, current = [], None
    for line in lines[start + 1:]:
        if line == close:
            break
        match = case.match(line)
        if match:
            current = {"value": signed(match[1]), "body": []}
            arms.append(current)
        elif line.startswith(indent + "default:"):
            current = None
        elif current is not None:
            current["body"].append(line)
    return arms


def newland():
    text, origin = source(NEWLAND)
    lines = text.splitlines()
    start = next(i for i, line in enumerate(lines) if line.startswith("  switch((&DAT_00479eb2)"))
    arms = sorted(arm["value"] for arm in switch_arms(lines, start, "  "))
    return {"function": "newland", "entry": "0x1400e4", "source": origin, "arms": arms,
            "default": "values without an arm leave the switch without effect"}


def attack2():
    text, origin = source(ATTACK2)
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines) if line == "        switch(iVar5) {"]
    if len(starts) != 2:
        raise SystemExit(f"{ATTACK2}: expected two condition switches, found {len(starts)}")
    tables = {}
    for target, start in zip(("warrior", "npc"), starts):
        rows = []
        for arm in switch_arms(lines, start, "        "):
            body = "\n".join(arm["body"])
            save = re.search(r"savevs\((\w+),", body)
            rows.append({"condition": arm["value"], "saving_throw": int(save[1], 0) if save else None,
                         "results": sorted({signed(code) for code in
                                            re.findall(r"showresults\(\w+,(0x[0-9a-f]+),", body)})})
        tables[target] = rows
    return {"function": "attack2", "entry": "0x101eb6", "source": origin,
            "selector": "natural attack condition byte (attack slot byte 3)", "targets": tables}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "evidence/original-dispatch.json")
    options = parser.parse_args()
    dispatch = {"schema_version": 1,
                "authority": "Centurions 1.3.2 decompiled exports; values only, no source text.",
                "newland": newland(), "attack2": attack2(),
                "not_generated": {"resolvespell": "effect dispatch is range guards in if-chains",
                                  "castspell": "target-mode dispatch is range guards in if-chains"}}
    options.output.write_text(json.dumps(dispatch, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"newland_arms": len(dispatch["newland"]["arms"]),
                      **{f"attack2_{t}": len(rows) for t, rows in dispatch["attack2"]["targets"].items()}}))


if __name__ == "__main__":
    main()
