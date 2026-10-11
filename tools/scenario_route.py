"""Inspect authored route dependencies without editing the supplied scenario."""
import argparse
import json
from pathlib import Path
import struct
from collections import deque

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
        typed_encounters.append(dict(index=index, labels=labels, branches=branches))
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


def static_path(root, result, map_index, start, target, avoid_slots=(), floor_changes=()):
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
    while pending and tuple(target) not in parents:
        x, y = pending.popleft()
        for dx, dy in ((1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)):
            following = x + dx, y + dy
            if following not in parents and passable(*following):
                parents[following] = x, y
                pending.append(following)
    path = []
    if tuple(target) in parents:
        current = tuple(target)
        while current is not None:
            path.append(current)
            current = parents[current]
        path.reverse()
    return dict(authority="Static tile candidate; event actions, regional RNG and combat are not executed",
                map=map_index, start=start, target=target, avoid_slots=list(avoid_slots),
                floor_changes=list(floor_changes), steps=max(0, len(path) - 1), path=path,
                headers=[e for e in map_data["events"] if (e["x"], e["y"]) in path],
                regions=[r for r in map_data["regions"]
                         if any(r["bounds"][1] <= x <= r["bounds"][3] and
                                r["bounds"][0] <= y <= r["bounds"][2] for x, y in path)])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path("NewCenturions"))
    parser.add_argument("--script", type=int)
    parser.add_argument("--map", type=int)
    parser.add_argument("--typed", type=int)
    parser.add_argument("--summary", action="store_true")
    parser.add_argument("--path", type=int, nargs=5, metavar=("MAP", "X", "Y", "TO_X", "TO_Y"))
    parser.add_argument("--avoid-slot", type=int, action="append", default=[])
    parser.add_argument("--floor-change", type=int, nargs=3, action="append", default=[])
    args = parser.parse_args()
    result = catalogue(args.input)
    if args.path is not None:
        index, x, y, tx, ty = args.path
        result = static_path(args.input, result, index, (x, y), (tx, ty), args.avoid_slot, args.floor_change)
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
