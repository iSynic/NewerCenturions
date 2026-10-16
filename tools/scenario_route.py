"""Inspect authored route dependencies without editing the supplied scenario."""
import argparse
import json
from pathlib import Path
import struct
from collections import deque
import re

SCENARIO = Path("TheNewCenturions/Scenarios/The Thunder of Guns/0")


def words(raw, count, offset=0):
    return list(struct.unpack_from(f">{count}h", raw, offset))


def script(raw):
    return [[op, arg] for op, arg in zip(words(raw, 8, 8), words(raw, 8, 24)) if op]


def catalogue(root):
    base = root / SCENARIO
    read = lambda name: (base / name).read_bytes()
    mission, doors, code = read("Data Mission"), read("Data ED3"), read("Data EDCD")
    headers, regions, land, menus = read("Data DD"), read("Data RD"), read("Data LD"), read("Data ED")
    typed = read("Data ED2")
    dialogue = read("Data SD2")
    text = lambda ident: dialogue[abs(ident) * 256 + 1:abs(ident) * 256 + 1 + dialogue[abs(ident) * 256]].decode("mac_roman")
    extra = [words(code, 5, i) for i in range(0, len(code) - 9, 10)]
    scripts = [script(doors[i:i + 42]) for i in range(0, len(doors) - 41, 42)]
    stages = []
    for i in range(0, len(mission) - 463, 464):
        raw = mission[i:i + 464]
        if raw[6]:
            stages.append(dict(index=i // 464, name=raw[7:7 + raw[6]].decode("mac_roman"),
                               start=words(raw, 3), start_script=words(raw, 1, 262)[0]))
    maps = []
    for index in range(len(land) // 64800):
        header = headers[index * 4200:(index + 1) * 4200]
        region = regions[index * 644:(index + 1) * 644]
        events = []
        for slot in range(100):
            raw = header[slot * 42:(slot + 1) * 42]
            key = struct.unpack_from(">i", raw)[0]
            if key >= 0:
                events.append(dict(slot=slot, key=key, x=key % 100, y=key // 100 % 100,
                                   destination=dict(map=struct.unpack_from("b", raw, 4)[0],
                                                    x=struct.unpack_from("b", raw, 5)[0],
                                                    y=struct.unpack_from("b", raw, 6)[0],
                                                    level=struct.unpack_from("b", raw, 40)[0]),
                                   chance=struct.unpack_from("b", raw, 7)[0], actions=script(raw)))
        regional = []
        for slot in range(20):
            bounds = words(region, 4, slot * 8)
            if any(bounds):
                regional.append(dict(slot=slot, bounds=bounds, chance=words(region, 1, 160 + slot * 2)[0],
                                     battles=words(region, 2, 200 + slot * 4),
                                     scripts=words(region, 3, 280 + slot * 6),
                                     script_chances=words(region, 3, 400 + slot * 6)))
        maps.append(dict(index=index, terrain=region[520], events=events, regions=regional))
    authored_menus = []
    # newland 0x142b8c seeks menu * 426, reads 106-byte header and 4 * 80-byte labels.
    for index in range(len(menus) // 426):
        raw = menus[index * 426:(index + 1) * 426]
        choices = []
        for choice in range(4):
            if raw[96 + choice]:
                label = raw[106 + choice * 80:106 + (choice + 1) * 80]
                choices.append(dict(index=choice, label=label[1:1 + min(label[0], 79)].decode("mac_roman"),
                                    actions=[[raw[choice * 8 + j], words(raw, 1, 32 + choice * 16 + j * 2)[0]]
                                             for j in range(8) if raw[choice * 8 + j]]))
        if choices:
            authored_menus.append(dict(index=index, dialogue=words(raw, 1, 104)[0], choices=choices))
    typed_encounters = []
    # newland 0x142de4 seeks encounter * 516; labels are nine 40-byte strings.
    for index in range(len(typed) // 516):
        raw = typed[index * 516:(index + 1) * 516]
        labels = []
        for label_index in range(9):
            label = raw[156 + label_index * 40:156 + (label_index + 1) * 40]
            labels.append(label[1:1 + min(label[0], 39)].decode("mac_roman"))
        branches = []
        for branch in range(4):
            branches.append([[raw[branch * 8 + j], words(raw, 1, 32 + branch * 16 + j * 2)[0]]
                             for j in range(8) if raw[branch * 8 + j]])
        typed_encounters.append(dict(
            index=index, labels=labels, branches=branches,
            action_selector=raw[96], password_selector=raw[97],
            action_requirements=list(raw[98:106]),
            spells=[dict(id=value, selector=raw[126 + slot])
                    for slot, value in enumerate(words(raw, 10, 106)) if value],
            items=[dict(id=value, selector=raw[146 + slot])
                   for slot, value in enumerate(words(raw, 5, 136)) if value],
            attempts=struct.unpack_from(">b", raw, 152)[0]))
    endings = []
    for index, actions in enumerate(scripts):
        if any(op == 110 for op, _ in actions):
            endings.append(dict(script=index, actions=actions,
                                dialogue={str(arg): text(arg) for op, arg in actions if op == 1}))
    return dict(authority="Authored bytes; control-flow meanings require independent instruction verification",
                counts=dict(land_maps=len(land) // 64800, header_records=len(headers) // 4200,
                            region_records=len(regions) // 644),
                stages=stages, maps=maps, scripts=scripts, extra=extra, menus=authored_menus,
                typed_encounters=typed_encounters,
                stage_transitions=endings)


def static_path(root, result, map_index, start, target, avoid_slots=(), floor_changes=(), target_bounds=None):
    """Find a tile route; report event dependencies without executing them."""
    base = root / SCENARIO
    raw = (base / "Data LD").read_bytes()[map_index * 64800:(map_index + 1) * 64800]
    floor = words(raw, 8100)
    events = struct.unpack_from(">8100i", raw, 32400)
    terrain_names = (None, "P", "Swamp", "Snow", "Sub", "Desert", "Lava", "Castle", "Techno", "Biologic")
    map_data = result["maps"][map_index]
    properties = (root / "TheNewCenturions/Data Files" /
                  f"Data {terrain_names[map_data['terrain']]} BD").read_bytes()
    default = words(properties, 1)[0]
    for x, y, value in floor_changes:
        if not (0 <= x < 90 and 0 <= y < 90):
            raise ValueError("floor change outside map")
        floor[x * 90 + y] = value
    blocked = {(e["x"], e["y"]) for e in map_data["events"] if e["slot"] in avoid_slots}
    def passable(x, y):
        if not (0 <= x < 90 and 0 <= y < 90) or (x, y) in blocked:
            return False
        index = x * 90 + y
        tile = floor[index]
        prop = default if tile < 0 or abs(tile) > 200 else tile
        obstacle, water = words(properties, 1, 64 + prop * 40 + 4)[0], words(properties, 1, 64 + prop * 40 + 8)[0]
        return water != 1 and (events[index] > 0 or (obstacle == 0 and water != 2))
    parents = {tuple(start): None}
    pending = deque([tuple(start)])
    destination = None
    while pending:
        x, y = pending.popleft()
        reached = ((x, y) == tuple(target)) if target_bounds is None else (
            target_bounds[1] <= x <= target_bounds[3] and
            target_bounds[0] <= y <= target_bounds[2])
        if reached:
            destination = x, y
            break
        for dx, dy in ((1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)):
            following = x + dx, y + dy
            if following not in parents and passable(*following):
                parents[following] = x, y
                pending.append(following)
    path = []
    if destination is not None:
        current = destination
        while current is not None:
            path.append(current)
            current = parents[current]
        path.reverse()
    return dict(authority="Static tile candidate; event actions, regional RNG and combat are not executed",
                map=map_index, start=start, target=target if target_bounds is None else destination,
                target_bounds=target_bounds, avoid_slots=list(avoid_slots),
                floor_changes=list(floor_changes), steps=max(0, len(path) - 1), path=path,
                headers=[e for e in map_data["events"] if (e["x"], e["y"]) in path],
                regions=[r for r in map_data["regions"]
                         if any(r["bounds"][1] <= x <= r["bounds"][3] and
                                r["bounds"][0] <= y <= r["bounds"][2] for x, y in path)])


def story_dispatch_arms(project):
    sources = (
        ("exploration.rs", "shared story", "Continue / Choose / Move"),
        ("story_combat.rs", "combat body operation", "Continue"),
        ("combat/runtime.rs", "queued combat story", "Continue / RunEnemyTurn / EndTurn"),
        ("combat/round_stories.rs", "round script", "Continue / RunEnemyTurn / EndTurn"),
        ("combat/final_turn_stories.rs", "enemy turn script", "RunEnemyTurn / EndTurn"),
    )
    arms = {}
    for name, context, entry in sources:
        path = Path("crates/core/src") / name
        lines = (project / path).read_text(encoding="utf-8").splitlines()
        for start, line in enumerate(lines):
            if not re.fullmatch(r"\s*match opcode \{", line):
                continue
            indent = len(line) - len(line.lstrip())
            for index in range(start + 1, len(lines)):
                line = lines[index]
                if line.startswith(" " * indent + "}"):
                    break
                arm = re.match(rf"^ {{{indent + 4}}}(-?\d+(?:\s*\|\s*-?\d+)*)\s*(?:if [^\n]+)?\s*=>", line)
                if arm:
                    for opcode in map(int, re.findall(r"-?\d+", arm[1])):
                        arms.setdefault(opcode, []).append(dict(
                            path=path.as_posix(), line=index + 1, context=context, command_entry=entry))
    return arms


def completion_inventory(root, result):
    """Conservative authored dependencies; source coverage is not native agreement."""
    base = root / SCENARIO
    project = Path(__file__).resolve().parent.parent
    dispatch_arms = story_dispatch_arms(project)
    nodes, edges, uncertainties = {}, [], []

    def add(kind, identity, actions, source, root_entry=False):
        key = f"{kind}:{identity}"
        nodes[key] = dict(kind=kind, identity=identity, actions=actions,
                          source=source, root=root_entry)
        return key

    for index, actions in enumerate(result["scripts"]):
        add("script", index, actions, f"Data ED3+{index * 42}")
    for stage in result["stages"]:
        add("stage", stage["index"], [[39, stage["start_script"]]] if stage["start_script"] else [],
            f"Data Mission+{stage['index'] * 464}", True)
    for m in result["maps"]:
        for e in m["events"]:
            add("header", f"{m['index']}/{e['slot']}", e["actions"],
                f"Data DD+{m['index'] * 4200 + e['slot'] * 42}", True)
        for r in m["regions"]:
            actions = [[39, s] for s in r["scripts"] if s > 0]
            actions += [[2, b] for b in r["battles"] if b > 0]
            add("region", f"{m['index']}/{r['slot']}", actions,
                f"Data RD+{m['index'] * 644 + r['slot'] * 8}", True)
    for menu in result["menus"]:
        for choice in menu["choices"]:
            add("menu", f"{menu['index']}/{choice['index']}", choice["actions"], "Data ED", True)
    for encounter in result["typed_encounters"]:
        for index, branch in enumerate(encounter["branches"]):
            add("typed", f"{encounter['index']}/{index}", branch, "Data ED2", True)
    schedule, scheduled = (base / "Data TD3").read_bytes(), (base / "Data ED3").read_bytes()
    for index in range(len(schedule) // 40):
        record = schedule[index * 40:index * 40 + 40]
        identity = words(record, 1, 6)[0]
        if identity >= 0 and (identity + 1) * 42 <= len(scheduled):
            add("schedule", index, script(scheduled[identity * 42:(identity + 1) * 42]),
                f"Data TD3+{index * 40}; Data ED3+{identity * 42}", True)
        else:
            uncertainties.append(dict(source=f"Data TD3+{index * 40}", reason="schedule record reference requires original-reader qualification"))
    monster_bytes = (base / "Data MD").read_bytes()
    monsters = []
    for index in range(len(monster_bytes) // 288):
        record = monster_bytes[index * 288:(index + 1) * 288]
        spells = sorted({s for s in words(record, 10, 0x12) if s > 1100})
        monsters.append(dict(id=index, spells=spells, weapons=words(record, 2, 0x26),
                             potential_drops=words(record, 6, 0x26),
                             turn_chance=words(record, 1, 0x5c)[0],
                             turn_script=words(record, 1, 0x5e)[0],
                             death_script=words(record, 1, 0x60)[0]))
    # Keep every authored roster as potential until encounter/placement predicates are qualified.
    for monster in monsters:
        for kind, offset in (("turn", 0x5e), ("death", 0x60)):
            script_id = monster[f"{kind}_script"]
            if script_id > 0:
                add(kind, monster["id"], [[39, script_id]],
                    f"Data MD+{monster['id'] * 288 + offset}", True)
    battles = (base / "Data BD").read_bytes()
    battle_rosters, drop_sources = [], {}
    for index in range(len(battles) // 1372):
        record = battles[index * 1372:(index + 1) * 1372]
        node = add("battle", index, [[39, s] for s in words(record, 5, 1360) if s > 0],
                   f"Data BD+{index * 1372}", True)
        identities = sorted({abs(ident) for ident in words(record, 676) if ident})
        battle_rosters.append(dict(id=index, monsters=identities,
                                   source=f"Data BD+{index * 1372}",
                                   qualification="Potential roster before placement, capacity and level predicates"))
        for ident in identities:
            if ident >= len(monsters):
                uncertainties.append(dict(source=node, reason=f"monster record {ident} is outside Data MD"))
                continue
            monster = monsters[ident]
            for kind in ("turn", "death"):
                if monster[f"{kind}_script"] > 0:
                    edges.append(dict(source=node, target=f"{kind}:{ident}", relation="roster member"))
            for item in monster["potential_drops"]:
                if item:
                    drop_sources.setdefault(abs(item), set()).add(index)
    items = []
    for first, table in ((0, root / "TheNewCenturions/Data Files/Data ID"),
                         (800, base / "Data NI")):
        raw = table.read_bytes()
        for index in range(800 if first == 0 else 200):
            record = raw[index * 150:(index + 1) * 150]
            if not any(record):
                continue
            item = dict(id=first + index, category=words(record, 1, 0x1a)[0],
                        effect=words(record, 1, 0x2c)[0], use_script=words(record, 1, 0x32)[0],
                        battle_drop_candidates=sorted(drop_sources.get(first + index, ())),
                        source=f"{table.name}+{index * 150}", eligibility="potential; acquisition qualification pending")
            items.append(item)
            if abs(item["category"]) == 23 and item["use_script"] >= 0:
                add("item", item["id"], [[39, item["use_script"]]], item["source"], True)
    attacks = (root / "TheNewCenturions/Data Files/Data S").read_bytes()
    spells = []
    psi_source = (project / "crates/core/src/combat/player_psi.rs").read_text()
    def catalog(name):
        match = re.search(rf"pub const {name}:.*?= &\[(.*?)\];", psi_source, re.S)
        return set(map(int, re.findall(r"\b\d{4}\b", match[1]))) if match else set()
    admitted = catalog("COMBAT_SPELLS")
    outdoor = catalog("PARTY_SPELLS") | catalog("OUTDOOR_PARTY_SPELLS") | catalog("OUTDOOR_TARGET_SPELLS")
    for school in range(len(attacks) // 2842):
        for level in range(7):
            for column in range(7):
                offset = school * 2842 + level * 406 + column * 58
                record = attacks[offset:offset + 58]
                identity = (school + 1) * 1000 + (level + 1) * 100 + column + 1
                spells.append(dict(id=identity, source=f"Data S+{offset}", effect=words(record, 1, 40)[0],
                                   target=record[1], combat=bool(record[21]), outdoor=bool(record[22]),
                                   player_combat_catalogued=identity in admitted,
                                   player_combat_admitted=identity in admitted and bool(record[21]),
                                   player_outdoor_admitted=identity in outdoor and bool(record[22]),
                                   command_entry="CastPartyPsi / SelectOutdoorPsiTarget / CancelOutdoorPsi" if identity in outdoor else "CastPsi / BeginPsiSelection" if identity in admitted else None,
                                   authored=bool(any(record)),
                                   eligibility="potential; promotion/profile qualification pending" if school < 3 else "engine packet band; no creation MOS selects school four"))
    references = {}
    for key, node in nodes.items():
        for op, arg in node["actions"]:
            references.setdefault(op, []).append(dict(node=key, parameter=arg, source=node["source"]))
            extra = result["extra"][abs(arg)] if abs(arg) < len(result["extra"]) else None
            targets = [arg] if op in (8, 39) else []
            if extra:
                if op in (33, 38, 42, 46) and extra[2] == 0:
                    targets.append(extra[3])
                elif op == 7:
                    targets.append(extra[2])
                elif op in (4, 5, 31):
                    targets.extend(extra[3:5])
                elif op in (21, 76, 77, 85, 87, 126):
                    # Mode-dependent fields need original caller proof; retain candidates.
                    targets.extend(extra)
                    uncertainties.append(dict(source=key, opcode=op, parameter=arg,
                                              reason="conditional successor candidates; predicate qualification pending"))
            for target in targets:
                if f"script:{target}" in nodes:
                    edges.append(dict(source=key, target=f"script:{target}", opcode=op))
    pending = deque(k for k, node in nodes.items() if node["root"])
    potential = set(pending)
    adjacency = {}
    for edge in edges:
        adjacency.setdefault(edge["source"], []).append(edge["target"])
    while pending:
        for target in adjacency.get(pending.popleft(), []):
            if target not in potential:
                potential.add(target)
                pending.append(target)
    return dict(authority=result["authority"], complete=False,
                qualification="Conservative potential-use inventory. Unknown predicates and indirect references remain open; unvisited records are not declared unreachable.",
                stages=result["stages"], difficulty_levels=[-2, -1, 0, 1, 2],
                nodes=nodes, edges=edges, potential_nodes=sorted(potential),
                unqualified_nodes=sorted(set(nodes) - potential), uncertainties=uncertainties,
                source_coverage_qualification="Located dispatch arms are source evidence only; guards, delegated calls and caller contexts still require command/native verification.",
                opcodes={str(op): dict(source_arm_present=op in dispatch_arms,
                                      source_arms=dispatch_arms.get(op, []), references=refs)
                         for op, refs in sorted(references.items())}, monsters=monsters,
                battle_rosters=battle_rosters, items=items, spells=spells,
                typed_encounters=result["typed_encounters"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path("NewCenturions"))
    parser.add_argument("--script", type=int)
    parser.add_argument("--map", type=int)
    parser.add_argument("--typed", type=int)
    parser.add_argument("--summary", action="store_true")
    parser.add_argument("--completion-inventory", action="store_true")
    parser.add_argument("--path", type=int, nargs=5, metavar=("MAP", "X", "Y", "TO_X", "TO_Y"))
    parser.add_argument("--region-path", type=int, nargs=4, metavar=("MAP", "X", "Y", "REGION"),
                        help="Find a static entry into an authored region")
    parser.add_argument("--avoid-slot", type=int, action="append", default=[])
    parser.add_argument("--floor-change", type=int, nargs=3, action="append", default=[])
    args = parser.parse_args()
    result = catalogue(args.input)
    if args.completion_inventory:
        result = completion_inventory(args.input, result)
    elif args.path is not None:
        index, x, y, tx, ty = args.path
        result = static_path(args.input, result, index, (x, y), (tx, ty), args.avoid_slot, args.floor_change)
    elif args.region_path is not None:
        index, x, y, slot = args.region_path
        region = next((r for r in result["maps"][index]["regions"] if r["slot"] == slot), None)
        if region is None:
            parser.error("region is not authored on this map")
        result = static_path(args.input, result, index, (x, y), None, args.avoid_slot,
                             args.floor_change, region["bounds"])
    elif args.script is not None:
        actions = result["scripts"][args.script]
        result = dict(script=args.script, actions=actions,
                      extra={str(arg): result["extra"][arg] for op, arg in actions
                             if op not in (0, 1, 9, 10, 11, 24, 25, 26, 39, 109, 110) and 0 <= arg < len(result["extra"])})
    elif args.map is not None:
        result = result["maps"][args.map]
    elif args.typed is not None:
        result = result["typed_encounters"][args.typed]
    elif args.summary:
        result = {key: result[key] for key in ("authority", "counts", "stages", "stage_transitions")}
    print(json.dumps(result, indent=2, ensure_ascii=False))
