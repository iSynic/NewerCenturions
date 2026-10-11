"""Export embedded Metrowerks names and reproducible 68k import seeds."""
import json
from pathlib import Path
import re


def prepare():
    root = Path("analysis/extracted")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    rows = []
    symbols = []
    for resource in manifest["resources"]:
        if resource["original"] != "TheNewCenturions/New Centurions" or resource["type"] != "CODE" or resource["id"] == 0:
            continue
        segment = resource["id"]
        data = (root / resource["output"]).read_bytes()
        last_end = 4
        for match in re.finditer(rb"\x4e\x75([\x81-\xbf])([A-Za-z_][A-Za-z_0-9]{0,62})", data):
            length = match[1][0] & 0x7F
            name_bytes = match[2]
            if len(name_bytes) != length:
                continue
            name = name_bytes.decode("ascii")
            # The trailer name is exact; LINK-based function starts are seeds,
            # checked against Ghidra flow rather than assumed complete boundaries.
            start = data.find(b"\x4e\x56", last_end, match.start())
            if start != -1 and start % 2 == 0:
                rows.append(f"{segment:x}\t{start:x}\t{match.start()+2:x}\t{name}")
                symbols.append(dict(segment=segment, resource_offset=start,
                                    end_offset=match.start()+2, name=name,
                                    name_offset=match.start()+3, start_evidence="LINK seed after previous name trailer"))
            last_end = match.start() + 3 + length
            last_end += last_end % 2
    output = Path("analysis/ghidra")
    output.mkdir(parents=True, exist_ok=True)
    (output / "seeds.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
    (output / "symbols.json").write_text(json.dumps(symbols, indent=2) + "\n", encoding="utf-8")
    print(f"Recovered {len(symbols)} named function seeds")


if __name__ == "__main__":
    prepare()
