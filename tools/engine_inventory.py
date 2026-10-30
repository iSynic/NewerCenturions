"""Generate the engine capability inventory from source, catalog and supplied data.

Rows are derived on each run: unsupported diagnostics, catalog routines and the
dispatch-domain values used by the supplied scenario. Hand judgments live in
evidence/engine-inventory-notes.json, keyed by row id; line numbers are never
stored. Absence from the supplied scenario is not evidence that the original
binary cannot reach a value.
"""
import argparse
import json
import re
import struct
from collections import Counter, defaultdict
from pathlib import Path

import scenario_route
import source_audit
import engine_diagnostics
from repository import ROOT

SOURCES = ("crates/content/src", "crates/core/src", "crates/desktop/src", "archive/crates/reference/src")
LITERAL = re.compile(r'Unsupported\s*\{\s*action:\s*"([^"]*)",\s*dependency:\s*"([^"]*)"')
DEFAULT_ACTION = re.compile(
    r'fn unsupported\(dependency: &\'static str\)[^{]*\{\s*RuleError::Unsupported\s*\{\s*action:\s*"([^"]*)"')
# rustfmt leaves a trailing comma after arguments it wraps onto their own lines.
ONE_ARG = re.compile(r'\bunsupported\(\s*"([^"]*)",?\s*\)')
TWO_ARG = re.compile(r'\bunsupported\(\s*"([^"]*)",\s*"([^"]*)",?\s*\)')
ROUTINE = re.compile(r'Routine \{ name: "([^"]+)", entry: 0x([0-9a-f]+), decompiled: (\w+), blocks: (\d+), '
                     r'instructions: (\d+), translation: Translation::(\w+), rust: &\[([^\]]*)\]')
OPERATION = re.compile(r'^\s*(\w+) = (-?\d+) => \w+,', re.M)
ADMITTED = re.compile(r'!matches!\(\s*[^,]*condition,\s*([\d\s|.=]+)\)')
# Rust sites that admit natural-attack conditions, per original attack2 target table.
CONDITION_GUARDS = {"warrior": ("crates/core/src/combat/attack/natural.rs",
                                "crates/core/src/combat/enemy_weapons/warrior_melee.rs"),
                    "npc": ("crates/core/src/combat/ally_turns/natural.rs",)}


def relative(path):
    return path.relative_to(ROOT).as_posix()


def diagnostics():
    """Unsupported (action, dependency) pairs with the files that construct them."""
    raw = {relative(p): p.read_text(encoding="utf-8")
           for name in SOURCES for p in sorted((ROOT / name).rglob("*.rs"))}
    raw = {p:t for p,t in raw.items() if not p.endswith('/catalog.rs')}
    rows, unresolved = engine_diagnostics.scan(raw)
    missing = [site for site in unresolved if not site['file'].startswith('archive/')]
    if missing:
        raise ValueError(f'Unresolved maintained diagnostic builders: {missing}')
    return [{"id": f"diagnostic:{action}|{dependency}", "kind": "diagnostic", "action": action,
             "dependency": dependency, "files": sorted({s['file'] for s in sites}),
             "sites": sorted(sites, key=lambda s:(s['file'],s['line'])),
             "layer": "reference" if all(source_audit.layer(Path(p), "production") == "reference"
                                         for p in {s['file'] for s in sites}) else "maintained"}
            for (action, dependency), sites in sorted(rows.items())]


def routines():
    text = (ROOT / "archive/crates/reference/src/reconstruction/catalog.rs").read_text(encoding="utf-8")
    rows = []
    for name, entry, decompiled, blocks, instructions, translation, rust in ROUTINE.findall(text):
        paths = re.findall(r'"([^"]+)"', rust)
        rows.append({"id": f"routine:{name}", "kind": "routine", "entry": f"0x{entry}",
                     "translation": translation, "instructions": int(instructions),
                     "maintained": sorted(p for p in paths
                                          if source_audit.layer(Path(p), "production") == "maintained"),
                     "reference": sorted(p for p in paths
                                         if source_audit.layer(Path(p), "production") == "reference"),
                     "missing": sorted(p for p in paths if not (ROOT / p).is_file())})
    return rows


def story_operations():
    text = (ROOT / "crates/core/src/exploration/script_execution/operation.rs").read_text(encoding="utf-8")
    body = text[text.index("operations! {"):]
    return {int(opcode): name for name, opcode in OPERATION.findall(body)}


def admitted(path):
    # Prepared attack2 kernels also admit the BYTE switch default after its draw.
    source = (ROOT / path).read_text(encoding="utf-8")
    match = ADMITTED.search(source)
    domain = match[1] if match else ("0..=255" if re.search(r'\b(?:resolve_prepared|admit_weapon_condition)\b', source) else "")
    ranges = [[int(v) for v in part.split("..=")] for part in domain.split("|")]
    return {v for r in ranges for v in range(r[0], r[-1] + 1)}


def original_arms():
    """Original dispatch arms from evidence/original-dispatch.json, against current admission."""
    dispatch = json.loads((ROOT / "evidence/original-dispatch.json").read_text(encoding="utf-8"))
    known = story_operations()
    rows = [{"id": f"newland_arm:{op}", "kind": "newland_arm", "value": op, "operation": known.get(op),
             "implemented": "yes" if op in known else "no"} for op in dispatch["newland"]["arms"]]
    for target, arms in dispatch["attack2"]["targets"].items():
        sites = {path: admitted(path) for path in CONDITION_GUARDS[target]}
        for arm in arms:
            paths = sorted(p for p, values in sites.items() if arm["condition"] in values)
            rows.append({"id": f"natural_condition:{target}:{arm['condition']}", "kind": "natural_condition",
                         **arm, "admitted_by": paths,
                         "implemented": "yes" if len(paths) == len(sites) else "partial" if paths else "no"})
    return rows


def supplied_opcodes(originals):
    """Authored opcode uses in the supplied Thunder data (level folder 0 holds all of it)."""
    catalogue = scenario_route.catalogue(originals)
    actions = [a for m in catalogue["maps"] for e in m["events"] for a in e["actions"]]
    actions += [a for s in catalogue["scripts"] for a in s]
    actions += [a for m in catalogue["menus"] for c in m["choices"] for a in c["actions"]]
    return Counter(op for op, _ in actions)


def domains(originals):
    known = story_operations()
    rows = [{"id": f"story_opcode:{op}", "kind": "story_opcode", "value": op,
             "operation": known.get(op), "supplied_uses": count,
             "implementation": "named" if op in known else "unknown_opcode_diagnostic"}
            for op, count in sorted(supplied_opcodes(originals).items())]
    data = originals / "TheNewCenturions/Data Files"
    spells = (data / "Data S").read_bytes()
    effects, targets = Counter(), Counter()
    for school in range(3):
        for slot in range(49):
            packet = spells[school * 2842 + slot // 7 * 406 + slot % 7 * 58:][:58]
            if any(packet):
                effects[struct.unpack_from(">h", packet, 40)[0]] += 1
                targets[packet[1]] += 1
    rows += [{"id": f"spell_effect:{v}", "kind": "spell_effect", "value": v, "supplied_packets": n}
             for v, n in sorted(effects.items())]
    rows += [{"id": f"spell_target:{v}", "kind": "spell_target", "value": v, "supplied_packets": n}
             for v, n in sorted(targets.items())]
    items = (data / "Data ID").read_bytes()
    categories = Counter(abs(struct.unpack_from(">h", items, i * 150 + 26)[0]) for i in range(len(items) // 150))
    rows += [{"id": f"item_category:{v}", "kind": "item_category", "value": v, "supplied_definitions": n}
             for v, n in sorted(categories.items()) if v]
    monsters = (originals / scenario_route.SCENARIO / "Data MD").read_bytes()
    conditions = Counter()
    for index in range(len(monsters) // 288):
        record = monsters[index * 288:(index + 1) * 288]
        if any(record):
            conditions.update(record[0x96 + attack * 4] for attack in range(6))
    rows += [{"id": f"monster_attack_condition:{v}", "kind": "monster_attack_condition", "value": v,
              "supplied_attacks": n} for v, n in sorted(conditions.items())]
    return rows


def build(originals):
    notes_path = ROOT / "evidence/engine-inventory-notes.json"
    notes = json.loads(notes_path.read_text(encoding="utf-8")) if notes_path.is_file() else {}
    rows = diagnostics() + routines() + original_arms()
    if (originals / "packsmith-mapping.json").is_file():
        rows += domains(originals)
    for row in rows:
        row["notes"] = notes.get(row["id"])
    return {"schema_version": 1,
            "authority": "Centurions 1.3.2. Rows are derived from current source, the routine catalog and "
                         "supplied data; evidence levels and dispositions come only from the notes file.",
            "rows": rows}


def problems(inventory):
    """Maintained diagnostics without a judgment and catalog bindings to missing files."""
    found = [f"Unclassified diagnostic {row['id']}" for row in inventory["rows"]
             if row["kind"] == "diagnostic" and row["layer"] == "maintained" and not row["notes"]]
    found += [f"Catalog binding missing: {row['id']} {path}" for row in inventory["rows"]
              if row["kind"] == "routine" for path in row["missing"]]
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--originals", type=Path, default=ROOT / "NewCenturions")
    parser.add_argument("--output", type=Path, default=ROOT / "evidence/engine-inventory.json")
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    inventory = build(options.originals)
    found = problems(inventory)
    if not options.check:
        options.output.write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    summary = Counter(row["kind"] for row in inventory["rows"])
    unreviewed = sum(1 for row in inventory["rows"] if (row["notes"] or {}).get("disposition") == "unreviewed")
    print(json.dumps({"rows": dict(summary), "unreviewed": unreviewed, "problems": len(found)}, indent=2))
    for problem in found:
        print(problem)
    return 1 if found else 0


if __name__ == "__main__":
    raise SystemExit(main())
