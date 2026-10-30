"""Generate the author capability map from source, the original dispatch table and the ledger.

Rows are derived on each run: every original newland arm with its interpreter routes
and the diagnostics its handlers construct; every authorable record accessor with its
offset, width and engine readers; and the nesting compositions guarded by reentrancy
diagnostics. Hand judgments (native verification, policies, operand domains) live in
evidence/capability-map-notes.json keyed by row id. Supplied-scenario absence never
lowers a row: the map describes what the 1.3.2 engine accepts, not what one scenario uses.
"""
import argparse
import json
import re
import sys
from collections import Counter

import engine_inventory
from repository import ROOT

MAP_PATH = ROOT / "evidence/capability-map.json"
NOTES_PATH = ROOT / "evidence/capability-map-notes.json"
DOC_PATH = ROOT / "docs/capability-map.md"
LEDGER_PATH = ROOT / "evidence/unsupported-ledger.json"
STATUSES = ("native-verified", "source-ported", "policy", "unsupported")

# Interpreters that dispatch authored programs, with the files that own their routes.
CONTEXTS = {
    "exploration": ("crates/core/src/exploration/script_execution",),
    "battle_round": ("crates/core/src/combat/round_stories.rs", "crates/core/src/combat/round_stories"),
    "enemy_turn": ("crates/core/src/combat/final_turn_stories.rs",),
    "combat_story": ("crates/core/src/story/combat.rs",),
    "death_script": ("crates/core/src/combat/death_scripts.rs", "crates/core/src/combat/death_scripts",
                     "crates/core/src/story_death.rs", "crates/core/src/death_story"),
    "getup": ("crates/core/src/combat/runtime/out_of_combat_getup.rs", "crates/core/src/combat/runtime/out_of_combat_getup"),
    "nested_callback": ("crates/core/src/combat/npc_return", "crates/core/src/story_callbacks.rs",
                        "crates/core/src/exploration/callbacks"),
}
# Dependency keywords that place a diagnostic in a calling context.
CONTEXT_WORDS = {
    "death_script": ("death-script", "death script", "death story", "immediate deaths", "death callback"),
    "getup": ("getup",),
    "nested_callback": ("nested", "callback", "suspended", "another modal", "another NPC caller"),
    "battle_round": ("battle round", "round-slot", "round slot", "round getup"),
    "enemy_turn": ("turn story", "enemy turn"),
}
# Composition rules: (parent, child, keywords). A composition inherits every diagnostic
# whose dependency contains all keywords of one alternative.
COMPOSITIONS = [
    ("death_script", "death_script", [("multiple immediate deaths",)]),
    ("death_script", "battle_round", [("battle round", "death-script")]),
    ("death_script", "enemy_turn", [("turn story", "death-script")]),
    ("death_script", "child_battle", [("child battle",)]),
    ("death_script", "story_page", [("death", "dialogue"), ("death", "story page")]),
    ("nested_callback", "child_battle", [("child battle", "suspended")]),
    ("child_battle", "rewards", [("combat cleanup", "callback"), ("reward", "after combat teardown")]),
    ("getup", "rewards", [("retained Getup return during another modal",), ("reward return after combat teardown",)]),
    ("getup", "npc_return", [("nested NPC Getup return",)]),
    ("enemy_turn", "selected_npc_change", [("selected NPC changed",)]),
    ("battle_round", "round_jump", [("before any Getup round scan",)]),
    ("exploration", "calendar", [("calendar child",), ("calendar destination",)]),
    ("combat_teardown", "reaction", [("reaction loop after combat teardown",), ("warrior walk after combat teardown",)]),
    ("combat_teardown", "getup", [("getup after combat teardown",)]),
]
# Authorable record views: family -> (file, impl names).
RECORDS = {
    "monster": ("crates/content/src/records/monster.rs", ("EnemyRecord",)),
    "npc": ("crates/content/src/combat.rs", ("MonsterRecord",)),
    "battle": ("crates/content/src/combat.rs", ("BattleRecord",)),
    "item": ("crates/content/src/records/item.rs", ("ItemDefinition",)),
    "spell": ("crates/content/src/records/attack.rs", ("SpellInfoRecord",)),
    "terrain": ("crates/content/src/tile_sets.rs", None),
    "encounter": ("crates/content/src/records/typed_encounter.rs", None),
    "extra": ("crates/content/src/records/exploration_extras.rs", None),
    "region": ("crates/content/src/records/exploration_tables.rs", None),
    "story": ("crates/content/src/records/story.rs", None),
    "schedule": ("crates/content/src/records/time_schedule.rs", None),
    "travel": ("crates/content/src/records/travel_events.rs", None),
    "shop": ("crates/content/src/records/shop.rs", None),
}
# Authored data read through plain struct fields, which the accessor scan does not see:
# (family, file, view, field, offset, width, type, pattern naming its maintained readers).
SURFACES = [
    ("terrain", "crates/content/src/tile_sets.rs", "TileSet", "properties", None, 2, "i16",
     r"\b(?:map_stats|tile_word|tactical_movement_property|movement_blocker|movement_sound)\b"),
    ("battle", "crates/content/src/combat.rs", "BattleRecord", "battle_grid", None, 2, "i16", r"\.battle_grid\b"),
    ("item", "crates/content/src/records/equipment.rs", "EquipmentModifiers", "attributes", 46, 2, "i16",
     r"\.attributes\b"),
]
OPERATION = re.compile(r"Operation::(\w+)")
FUNCTION = re.compile(r"\bfn (\w+)\s*(?:<[^>{]*>)?\s*\(")
CALL = re.compile(r"\b([a-z_][a-z0-9_]*)\s*(?:::<[^>]*>)?\(")
CALL_DEPTH = 2
# Helpers called from nearly every handler; following them attributes unrelated diagnostics.
# Notes key listing L01/L02 diagnostics whose work has a known startup value, so only an
# older save or native import reaches them; each entry gives the reason.
LEGACY_KEY = "legacy_state_diagnostics"
LEGACY_GROUPS = ("L01", "L02")
# Notes key listing open diagnostics no authored scenario data reaches; each gives the reason.
UNAUTHORED_KEY = "unauthored_diagnostics"
# L01/L02 dependencies that name state only an older save or native import can lack.
LEGACY_WORDS = ("legacy", "unreconstructable")
COMMON = {"unsupported", "unavailable", "validate", "new", "from", "into", "ok_or", "map_err", "clone",
          "get", "len", "iter", "push", "word", "span", "apply", "resolve", "execute", "run", "next", "step",
          "finish", "commit", "with_context", "some", "ok", "err", "unwrap", "expect", "default", "from_bytes",
          "record", "record_mut", "into_iter", "extend", "map", "contains", "is_some", "is_none", "take"}
ARM_TAIL = re.compile(r"(?:\([^()]*\))?\s*(?:\|\s*Operation::\w+(?:\([^()]*\))?\s*)*")
IMPL = re.compile(r"^impl(?:<[^>]*>)?\s+(\w+)(?:<[^>]*>)?\s*\{", re.M)
ACCESSOR = re.compile(r"pub fn (\w+)\(&self(?:, (\w+): [\w&\[\]]+)?\)\s*->\s*([^{]+)\{(.*?)\n    \}", re.S)
OFFSET = re.compile(r"(word|long|i32_at|self\.0\[|put_word)\W*(?:self\.0,\s*)?(0x[0-9a-fA-F]+|\d+)")


def rust_files(paths):
    for name in paths:
        path = ROOT / name
        if path.is_file():
            yield path
        elif path.is_dir():
            yield from sorted(path.rglob("*.rs"))


def blocks(text, start):
    """The brace-delimited block opening at or after start."""
    open_at = text.find("{", start)
    if open_at < 0:
        return start, start
    depth = 0
    for index in range(open_at, len(text)):
        depth += {"{": 1, "}": -1}.get(text[index], 0)
        if depth == 0:
            return open_at, index
    return open_at, len(text)


def line_of(text, index):
    return text.count("\n", 0, index) + 1


def ledger_index():
    ledger = json.loads(LEDGER_PATH.read_text(encoding="utf-8"))
    status = {g["id"]: g["status"] for g in ledger["groups"]}
    # A closed group's active diagnostics are boundaries the runtime still refuses.
    active = {d for g in ledger["groups"] for d in g.get("active_diagnostics") or []}
    rows = {}
    for ident, entry in ledger["diagnostics"].items():
        rows[ident] = {"group": entry.get("group"), "category": entry.get("category"),
                       "group_status": status.get(entry.get("group")), "active": ident in active}
    return rows


def is_open(entry):
    return entry.get("active") or entry.get("group_status") not in (None, "closed")


def diagnostic_sites():
    rows = [r for r in engine_inventory.diagnostics() if r["layer"] == "maintained"]
    return rows, {(s["file"], s["line"]): r["id"] for r in rows for s in r["sites"]}


def function_index(sites):
    """Every maintained core function: its body's direct diagnostics and called names."""
    index = {}
    for path in rust_files(("crates/core/src",)):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8")
        for match in FUNCTION.finditer(text):
            start, end = blocks(text, match.end())
            first, last = line_of(text, start), line_of(text, end)
            body = text[start:end]
            entry = index.setdefault(match[1], {"diagnostics": set(), "calls": set(), "definitions": 0})
            entry["definitions"] += 1
            entry["diagnostics"].update(ident for (file, line), ident in sites.items()
                                        if file == rel and first <= line <= last)
            entry["calls"].update(CALL.findall(body))
    return index


def reachable(calls, index, depth=CALL_DEPTH):
    """Diagnostics constructed by uniquely named functions called within depth.

    A name defined more than once cannot be resolved without types and is not followed."""
    found, seen, frontier = set(), set(), set(calls)
    for _ in range(depth):
        nxt = set()
        for name in frontier - seen - COMMON:
            seen.add(name)
            entry = index.get(name)
            if entry and entry["definitions"] == 1:
                found |= entry["diagnostics"]
                nxt |= entry["calls"]
        frontier = nxt
    return found


def operation_arms(sites, index):
    """For each Operation name, the context files and diagnostics inside its match arms."""
    arms = {}
    for context, paths in CONTEXTS.items():
        for path in rust_files(paths):
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8")
            for match in OPERATION.finditer(text):
                # A match arm (the name, an optional payload and further or-patterns),
                # or an `== Operation::Name {` test guarding its own block.
                arrow = text.find("=>", match.end())
                brace = text.find("{", match.end())
                if arrow >= 0 and ARM_TAIL.fullmatch(text[match.end():arrow].strip()):
                    start, end = blocks(text, arrow)
                elif (brace >= 0 and not text[match.end():brace].strip()
                      and text[:match.start()].rstrip().endswith("==")):
                    start, end = blocks(text, brace)
                else:
                    continue
                first, last = line_of(text, start), line_of(text, end)
                found = {ident for (file, line), ident in sites.items()
                         if file == rel and first <= line <= last}
                found |= reachable(CALL.findall(text[start:end]), index)
                entry = arms.setdefault(match[1], {}).setdefault(context, {"files": set(), "diagnostics": set()})
                entry["files"].add(rel)
                entry["diagnostics"].update(found)
    return arms


def literal_routes():
    """Opcodes the enemy-turn and combat-story interpreters handle locally by literal arm."""
    routes = {}
    for context, path in (("enemy_turn", "crates/core/src/combat/final_turn_stories.rs"),
                          ("combat_story", "crates/core/src/story/combat.rs")):
        text = (ROOT / path).read_text(encoding="utf-8")
        found = set()
        for match in re.finditer(r"match opcode \{", text):
            _, end = blocks(text, match.start())
            body = text[match.end():end]
            for arm in re.findall(r"^\s*((?:-?\d+\s*\|\s*)*-?\d+)\s*=>", body, re.M):
                found.update(int(v) for v in re.findall(r"-?\d+", arm))
        routes[context] = found
    text = (ROOT / "crates/core/src/combat/round_stories.rs").read_text(encoding="utf-8")
    start = text.index("fn shared_operation")
    _, end = blocks(text, start)
    routes["battle_round_local"] = set(OPERATION.findall(text[start:end]))
    return routes


def split_open(found, ledger):
    """Open diagnostics an authored scenario can meet, and those only legacy/imported state reaches."""
    open_diags = [d for d in found if is_open(ledger.get(d, {}))]
    legacy = [d for d in open_diags if ledger[d]["group"] in LEGACY_GROUPS
              and (ledger[d].get("legacy") or any(w in d for w in LEGACY_WORDS))]
    return [d for d in open_diags if d not in legacy], legacy


def contexts_for(dependency):
    lower = dependency.lower()
    return sorted(c for c, words in CONTEXT_WORDS.items() if any(w.lower() in lower for w in words))


def opcode_rows(ledger, sites, notes):
    dispatch = json.loads((ROOT / "evidence/original-dispatch.json").read_text(encoding="utf-8"))
    known = engine_inventory.story_operations()
    arms = operation_arms(sites, function_index(sites))
    routes = literal_routes()
    rows = []
    rows.append({"id": "opcode:default", "kind": "opcode", "value": None, "operation": "Unarmed",
                 "contexts": {}, "diagnostics": [], "open_diagnostics": [], "operand": None,
                 "status": "source-ported", "behavior": dispatch["newland"]["default"]})
    finish(rows[-1], notes, ledger)
    for op in dispatch["newland"]["arms"] + [0]:
        name = known.get(op)
        contexts = {}
        for context, entry in sorted(arms.get(name, {}).items()):
            contexts[context] = {"route": "handler", "files": sorted(entry["files"]),
                                 "diagnostics": sorted(entry["diagnostics"])}
        if name is not None:
            contexts.setdefault("exploration", {"route": "shared", "files": [], "diagnostics": []})
            contexts.setdefault("battle_round", {
                "route": "round_local" if name in routes["battle_round_local"] else "shared",
                "files": [], "diagnostics": []})
            for context in ("death_script", "getup", "nested_callback"):
                contexts.setdefault(context, {"route": "shared", "files": [], "diagnostics": []})
        for context in ("enemy_turn", "combat_story"):
            if op in routes[context]:
                contexts.setdefault(context, {"route": "literal", "files": [], "diagnostics": []})
                contexts[context]["route"] = "literal"
        diagnostics = sorted({d for c in contexts.values() for d in c["diagnostics"]})
        open_diags, legacy = split_open(diagnostics, ledger)
        row = {"id": f"opcode:{op}", "kind": "opcode", "value": op, "operation": name,
               "contexts": contexts, "diagnostics": diagnostics, "open_diagnostics": open_diags,
               "legacy_state_diagnostics": legacy, "operand": None}
        if name is None:
            row["status"] = "unsupported"
            row["diagnostics"] = row["open_diagnostics"] = ["diagnostic:story|unknown opcode"]
        else:
            row["status"] = "source-ported"
        finish(row, notes, ledger)
        rows.append(row)
    return rows


def record_rows(ledger, notes):
    readers = {}
    for path in rust_files(("crates/core/src",)):
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(ROOT).as_posix()
        for name in set(re.findall(r"\.(\w+)\(", text)):
            readers.setdefault(name, set()).add(rel)
    rows = []
    for family, (path, impls) in RECORDS.items():
        text = (ROOT / path).read_text(encoding="utf-8")
        starts = [(m.start(), m[1]) for m in IMPL.finditer(text)] + [(len(text), None)]
        for (start, owner), (end, _) in zip(starts, starts[1:]):
            if impls is not None and owner not in impls or owner is None or owner.endswith("Mut"):
                continue
            for match in ACCESSOR.finditer(text[start:end]):
                name, index, returns, body = match[1], match[2], match[3].strip(), match[4]
                offset = OFFSET.search(body)
                width = None
                if offset:
                    width = {"word": 2, "put_word": 2, "long": 4, "i32_at": 4}.get(offset[1], 1)
                kind = re.search(r"\b([iu](?:8|16|32)|bool)\b", returns)
                files = sorted(readers.get(name, ()))
                row = {"id": f"field:{family}:{owner}.{name}", "kind": "record_field", "family": family,
                       "view": owner, "accessor": name, "indexed": index is not None,
                       "offset": int(offset[2], 0) if offset else None, "width": width,
                       "type": kind[1] if kind else None, "readers": files,
                       "status": "source-ported" if files else "policy",
                       "domain": None, "diagnostics": [], "open_diagnostics": []}
                if not files:
                    row["policy"] = "Preserved bytes: no maintained engine path reads this accessor."
                finish(row, notes, ledger)
                rows.append(row)
    for family, path, view, field, offset, width, kind, pattern in SURFACES:
        if not re.search(rf"pub {field}:", (ROOT / path).read_text(encoding="utf-8")):
            raise ValueError(f"{path} has no field {view}.{field}")
        files = sorted(source.relative_to(ROOT).as_posix() for source in rust_files(("crates/core/src",))
                       if re.search(pattern, source.read_text(encoding="utf-8")))
        row = {"id": f"field:{family}:{view}.{field}", "kind": "record_field", "family": family, "view": view,
               "accessor": field, "indexed": True, "offset": offset, "width": width, "type": kind,
               "readers": files, "status": "source-ported" if files else "policy",
               "domain": None, "diagnostics": [], "open_diagnostics": []}
        finish(row, notes, ledger)
        rows.append(row)
    return rows


def values(pattern):
    found = set()
    for part in pattern.split("|"):
        part = part.strip()
        if part.endswith(".."):
            # An open range runs to the top of the signed selector WORD.
            part += "=32767"
        bounds = [int(v) for v in part.split("..=")]
        found.update(range(bounds[0], bounds[-1] + 1))
    return found


def admitted_set(path, anchor):
    """The union of the effect sets in matches! on an effect within the function at anchor."""
    text = (ROOT / path).read_text(encoding="utf-8")
    start, end = blocks(text, text.index(anchor))
    found = set()
    for match in re.finditer(r"matches!\(\s*([^,]*?),\s*([-\d\s|.=]+?)\s*\)", text[start:end], re.S):
        if "effect" in match[1]:
            found |= values(match[2])
    if not found:
        raise ValueError(f"no effect set after {anchor} in {path}")
    return found


def value_rows(ledger, notes):
    """Spell-effect selectors and natural attack conditions an author can store."""
    support = admitted_set("crates/core/src/combat/spells/restrictions.rs", "pub(crate) fn is_support")
    # story_spell/outdoors.rs::resolve refuses only resolvespell's arms that read the
    # tactical target cell: summon always, movement when WORD438786 is set.
    outdoor = set(range(0, 141)) - {58}
    target_cell = {55, 56, 58}
    drain = {60, 61}
    # resolvespell's 0x3a arm: party casts copy MONSTER work (combat/spell_summon.rs).
    effects = (ROOT / "crates/core/src/combat/player_psi/effects.rs").read_text(encoding="utf-8")
    summon = {58} if "spell_summon::summon" in effects else set()
    summon_diags = sorted(d for d in ledger if d.startswith("diagnostic:combat.spell.summon|")) + [
        "diagnostic:combat.spell.mask|rotated shapes or summon targeting"]
    rows = []
    for effect in range(0, 141):
        contexts = {"combat": "support" if effect in support else "drain_category5" if effect in drain
                    else "summon" if effect in summon else None,
                    "outdoor": "support" if effect in outdoor else None}
        supported = effect in support or effect in drain or effect in summon
        diags = [] if supported else ["diagnostic:combat.spell.support|movement, summon or category-5 drain packet in a record-only body resolver"]
        if effect in summon:
            diags += summon_diags
        if effect in target_cell:
            diags.append("diagnostic:story.spell|newland18 combat cast of a movement or summon packet without a fresh WORD443722")
        open_diags, legacy = split_open(diags, ledger)
        row = {"id": f"spell_effect:{effect}", "kind": "spell_effect", "value": effect, "contexts": contexts,
               "diagnostics": diags, "open_diagnostics": open_diags, "legacy_state_diagnostics": legacy,
               "status": "source-ported" if supported else "unsupported"}
        finish(row, notes, ledger)
        rows.append(row)
    inventory = [r for r in engine_inventory.original_arms() if r["kind"] == "natural_condition"]
    for arm in inventory:
        row = {"id": arm["id"], "kind": "natural_condition", "value": arm["condition"],
               "target": arm["id"].split(":")[1], "admitted_by": arm["admitted_by"],
               "diagnostics": [], "open_diagnostics": [], "legacy_state_diagnostics": [],
               "status": {"yes": "source-ported", "partial": "source-ported", "no": "unsupported"}[arm["implemented"]]}
        if arm["implemented"] == "partial":
            row["open_diagnostics"] = ["partial: admitted by only some attack callers"]
        finish(row, notes, ledger)
        rows.append(row)
    return rows


def composition_rows(ledger, diagnostics, notes):
    active = {r["id"] for r in diagnostics}
    rows = []
    for parent, child, alternatives in COMPOSITIONS:
        def matches(ident):
            dependency = ident.split("|", 1)[1].lower()
            return any(all(w.lower() in dependency for w in alt) for alt in alternatives)
        found = sorted(r["id"] for r in diagnostics if matches(r["id"]))
        retired = sorted(i for i in ledger if i not in active and matches(i))
        open_diags, _ = split_open(found, ledger)
        row = {"id": f"composition:{parent}>{child}", "kind": "composition", "parent": parent, "child": child,
               "diagnostics": found, "retired_diagnostics": retired, "open_diagnostics": open_diags,
               "status": "unsupported" if open_diags else "source-ported"}
        if not found and not retired:
            # No guard names this nesting; it runs through the shared owners untested.
            row["open_diagnostics"] = ["untested: no diagnostic or reference case covers this nesting"]
        finish(row, notes, ledger)
        rows.append(row)
    return rows


def finish(row, notes, ledger):
    note = notes.get(row["id"], {})
    for key in ("status", "operand", "domain", "policy", "evidence"):
        if key in note:
            row[key] = note[key]
    # Diagnostics that callers reach from this row's authored data beyond the
    # generated attribution; each note entry records the traced route.
    attached = sorted(set(note.get("diagnostics", {})) - set(row["diagnostics"]))
    if attached:
        open_diags, legacy = split_open(attached, ledger)
        row["diagnostics"] = sorted(row["diagnostics"] + attached)
        row["open_diagnostics"] = row["open_diagnostics"] + open_diags
        if legacy:
            row["legacy_state_diagnostics"] = row.get("legacy_state_diagnostics", []) + legacy
    if row["status"] not in STATUSES:
        raise ValueError(f"{row['id']}: unknown status {row['status']}")
    # A diagnostic an operand rule blocks in the editor no longer warns on the whole row.
    ruled = {rule["diagnostic"] for rule in row.get("operand") or [] if rule.get("editor") == "block"}
    if ruled:
        row["operand_blocked"] = sorted(d for d in row.get("open_diagnostics", []) if d in ruled)
        row["open_diagnostics"] = [d for d in row["open_diagnostics"] if d not in ruled]
    if row["status"] == "unsupported":
        row["editor"] = "block"
    elif row.get("open_diagnostics"):
        row["editor"] = "warn"
    else:
        row["editor"] = "allow"
    row["editor"] = note.get("editor", row["editor"])


def build():
    notes = json.loads(NOTES_PATH.read_text(encoding="utf-8")) if NOTES_PATH.is_file() else {}
    ledger = ledger_index()
    legacy = notes.pop(LEGACY_KEY, {})
    for ident in legacy:
        if ledger.get(ident, {}).get("group") not in LEGACY_GROUPS:
            raise ValueError(f"legacy-state note {ident} is not an L01 or L02 diagnostic")
        ledger[ident]["legacy"] = True
    unauthored = notes.pop(UNAUTHORED_KEY, {})
    for ident in unauthored:
        if ident not in ledger:
            raise ValueError(f"unauthored note {ident} is not in the ledger")
    diagnostics, sites = diagnostic_sites()
    rows = (opcode_rows(ledger, sites, notes) + value_rows(ledger, notes) + record_rows(ledger, notes)
            + composition_rows(ledger, diagnostics, notes))
    for row in rows:
        for ident in row.get("diagnostics", []):
            if ident in ledger:
                continue
            if not ident.startswith(("partial:", "untested:")):
                raise ValueError(f"{row['id']}: {ident} is not in the ledger")
    unused = sorted(set(notes) - {r["id"] for r in rows})
    if unused:
        raise ValueError(f"Notes without rows: {unused[:5]}")
    by_diag = {}
    for row in rows:
        for ident in row.get("diagnostics", []):
            by_diag.setdefault(ident, []).append(row["id"])
    # Legacy-only and unauthored diagnostics carry a recorded reason instead of a row.
    unmapped = sorted(r["id"] for r in diagnostics if r["id"] not in by_diag
                      and r["id"] not in legacy and r["id"] not in unauthored
                      and is_open(ledger.get(r["id"], {})))
    return {"schema_version": 1,
            "authority": "Centurions 1.3.2. Rows are derived from current source, the original newland dispatch "
                         "and the unsupported ledger; statuses other than the generated defaults come only from "
                         "evidence/capability-map-notes.json.",
            "contexts": sorted(CONTEXTS),
            "summary": summary(rows),
            "unmapped_open_diagnostics": unmapped,
            "legacy_only_diagnostics": sorted(legacy),
            "unauthored_diagnostics": sorted(unauthored),
            "rows": rows}


def summary(rows):
    counts = {}
    for row in rows:
        counts.setdefault(row["kind"], Counter())[row["status"]] += 1
        counts[row["kind"]]["editor:" + row["editor"]] += 1
    return {kind: dict(sorted(c.items())) for kind, c in sorted(counts.items())}


def render(data):
    lines = ["# Author capability map", "",
             "Generated by `python tools/capability_map.py --refresh` from current source, "
             "`evidence/original-dispatch.json` and `evidence/unsupported-ledger.json`. "
             "The machine-readable map is `evidence/capability-map.json`; hand judgments are in "
             "`evidence/capability-map-notes.json`. The 1.3.2 binary is the authority; a row's status is "
             "independent of whether the supplied scenario uses it.", "",
             "Statuses: native-verified (independent native evidence), source-ported (translated from "
             "the original source), policy (documented compatibility treatment), unsupported (an atomic "
             "diagnostic). Editor treatment: allow, warn (some operands or contexts diagnose), block.", "",
             "The backlog is complete when every row is native-verified, source-ported or policy, and the "
             "editor warns on or blocks every remaining row. Clearing the unsupported ledger alone is not "
             "sufficient.", "",
             "## Summary", "", "| Kind | " + " | ".join(STATUSES) + " | allow | warn | block |",
             "|---|" + "---|" * (len(STATUSES) + 3)]
    for kind, counts in data["summary"].items():
        cells = [str(counts.get(s, 0)) for s in STATUSES]
        cells += [str(counts.get("editor:" + e, 0)) for e in ("allow", "warn", "block")]
        lines.append(f"| {kind} | " + " | ".join(cells) + " |")
    lines += ["", f"Open diagnostics not yet attached to a row: {len(data['unmapped_open_diagnostics'])}. "
              f"Reached only from older saves or native imports: {len(data['legacy_only_diagnostics'])}. "
              f"Reached by no authored data: {len(data['unauthored_diagnostics'])}. Each of these carries "
              "its reason in the notes.", "",
              "## Opcodes", "", "| Opcode | Operation | Status | Editor | Routes | Open diagnostics |",
              "|---|---|---|---|---|---|"]
    for row in (r for r in data["rows"] if r["kind"] == "opcode"):
        routes = ", ".join(f"{c}:{v['route']}" for c, v in row["contexts"].items())
        lines.append(f"| {'other' if row['value'] is None else row['value']} | {row['operation'] or '—'} | {row['status']} | {row['editor']} | "
                     f"{routes or '—'} | {len(row['open_diagnostics'])} |")
    ruled = [r for r in data["rows"] if r.get("operand")]
    if ruled:
        lines += ["", "## Value rules", "", "| Row | Record | Field | Domain | Editor |", "|---|---|---|---|---|"]
        for row in ruled:
            for rule in row["operand"]:
                lines.append(f"| {row['id']} | {rule['record']} | {rule['field']} | {rule['domain']} | "
                             f"{rule['editor']} |")
    lines += ["", "## Compositions", "", "| Parent | Child | Status | Editor | Open diagnostics |",
              "|---|---|---|---|---|"]
    for row in (r for r in data["rows"] if r["kind"] == "composition"):
        diags = "<br>".join(d.removeprefix("diagnostic:") for d in row["open_diagnostics"]) or "—"
        lines.append(f"| {row['parent']} | {row['child']} | {row['status']} | {row['editor']} | {diags} |")
    lines += ["", "## Record fields", "", "| Family | Fields | Read by the engine | Unread (preserved) |",
              "|---|---|---|---|"]
    families = Counter(r["family"] for r in data["rows"] if r["kind"] == "record_field")
    read = Counter(r["family"] for r in data["rows"] if r["kind"] == "record_field" and r["readers"])
    for family, count in sorted(families.items()):
        lines.append(f"| {family} | {count} | {read[family]} | {count - read[family]} |")
    lines += ["", "Field rows carry offset, width, type and the maintained files that call each accessor; "
              "value domains are filled from notes as they are recovered.", "",
              "## Spell effects and natural conditions", "",
              "| Kind | Values | Allow | Warn | Block | Blocked values |", "|---|---|---|---|---|---|"]
    for kind in ("spell_effect", "natural_condition"):
        rows = [r for r in data["rows"] if r["kind"] == kind]
        tally = Counter(r["editor"] for r in rows)
        blocked = compress(r["value"] for r in rows if r["editor"] == "block") or "—"
        lines.append(f"| {kind} | {len(rows)} | {tally['allow']} | {tally['warn']} | {tally['block']} | {blocked} |")
    lines += ["", "## How rows are derived", "",
              "- Opcode rows cover every arm of the original `newland` switch plus its no-effect default. "
              "Routes come from the interpreters that dispatch authored programs; diagnostics come from the "
              f"handler arm and from uniquely named functions it calls within {CALL_DEPTH} levels. A name "
              "defined more than once is not followed, so attribution is a lower bound. The native-verified "
              "status comes only from notes.",
              "- Notes attach further diagnostics to a row, each with the traced route from the row's authored "
              "data to the diagnostic site; this covers combat paths no opcode handler calls by a unique name.",
              "- Record-field rows also cover authored data read through plain struct fields: tile set "
              "property rows, battle grids and item attribute selectors.",
              "- L01 and L02 diagnostics that only an older save or native import can reach are listed separately "
              "as `legacy_state_diagnostics` and do not set the editor treatment.",
              "- Composition rows match reentrancy diagnostics by keyword. A nesting that no diagnostic or "
              "reference case covers is marked untested and warns.",
              "- Spell effect rows cover selectors 0..140 against the support, Psi-drain and outdoor guards.", "",
              "## Editor integration (proposed, not yet made)", "",
              "The scenario editor gates steps with a fixed list in "
              "`crates/centurions-format/src/scripts.rs::supported`. Proposal: the editor's staging tool copies "
              "`evidence/capability-map.json` into its pinned inputs, and the fixed list is replaced by each "
              "row's `editor` treatment:", "",
              "- `block`: the step, field value or nesting cannot be saved; show the ledger diagnostic.",
              "- `warn`: the step can be saved; list `open_diagnostics` with their calling contexts.",
              "- `allow`: no diagnostic is known on this route.", "",
              "Existing operand checks (Data SD2 rows, EXTRA rows) stay; operand domains recorded in notes "
              "extend them. The editor rejects an unknown `schema_version`.", ""]
    return "\n".join(lines)


def compress(values):
    numbers = sorted(values)
    spans = []
    for value in numbers:
        if spans and spans[-1][1] + 1 == value:
            spans[-1][1] = value
        else:
            spans.append([value, value])
    return ", ".join(f"{a}" if a == b else f"{a}..{b}" for a, b in spans)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = build()
    text = json.dumps(data, indent=2) + "\n"
    doc = render(data)
    if args.refresh:
        MAP_PATH.write_text(text, encoding="utf-8", newline="\n")
        DOC_PATH.write_text(doc, encoding="utf-8", newline="\n")
    if args.check:
        current = MAP_PATH.read_text(encoding="utf-8") if MAP_PATH.is_file() else ""
        if current != text or (DOC_PATH.read_text(encoding="utf-8") if DOC_PATH.is_file() else "") != doc:
            sys.exit("capability map is stale; run python tools/capability_map.py --refresh")
    print(json.dumps(data["summary"], indent=2))


if __name__ == "__main__":
    main()
