"""Capture state through Basilisk's guest test interface; mark modified-reference observations."""
import argparse
import json
from pathlib import Path
import reference


def capture(combat=False):
    a5 = int(reference.read_region(0x904, 4)["hex"], 16)
    quickdraw = int(reference.read_region(a5, 4)["hex"], 16)
    state = dict(schema_version=1, a5=a5, random_seed=int(reference.read_region(quickdraw-0x7e, 4)["hex"], 16),
                 globals=reference.read_region(a5+0x42720, 200),
                 fatigue=reference.read_region(a5+0x42578, 2),
                 warriors=[reference.read_region(a5+0x48aa0+i*3138,3138) for i in range(6)])
    state["ammunition"]=reference.read_region(a5+0x2dbe4,800)
    state["loot"]=reference.read_region(a5+0x1fc74,440)
    state["schedule_file"]="original authored Data TD3; no active entries in these live cases"
    if Path("analysis/native/reference-state-input.json").exists():
        receipt=json.loads(Path("analysis/native/reference-state-input.json").read_text())
        if receipt["pid"]==int(Path("reference/basilisk.pid").read_text()):
            state["controlled_state_inputs"]=receipt["controlled_state_inputs"]
    if combat:
        state["combat"] = {name: reference.read_region(a5+offset, length) for name, offset, length in (
            ("map", 0x85b8, 16200), ("enemies", 0x39d3c, 28800),
            ("squad_positions", 0x4709a, 24), ("enemy_positions", 0x46f0a, 400),
            ("order", 0x470b8, 220), ("cursor_enemy_index", 0x47194, 4),
            ("counts", 0x435c4, 16), ("origin", 0x42780, 8),
            ("under_tiles", 0x37a6e, 12), ("combat_flags", 0x3874a, 46))}
    if Path("analysis/native/reference-workaround.json").exists():
        receipt = json.loads(Path("analysis/native/reference-workaround.json").read_text())
        if receipt["pid"] == int(Path("reference/basilisk.pid").read_text()):
            code = int(reference.read_region(a5-0x1de2, 4)["hex"], 16)
            patch = receipt["patch"]
            replacement = patch["replacement"]
            actual = reference.read_region(code+int(patch["code1_offset"], 16), len(replacement)//2)["hex"]
            if actual == replacement:
                state["reference_modification"] = patch
            elif actual != patch["original"]:
                raise RuntimeError("reference presentation code matches neither original nor recorded patch")
            state["presentation_code"] = dict(code1=code, offset=patch["code1_offset"], hex=actual)
    if Path("analysis/native/reference-input.json").exists():
        receipt = json.loads(Path("analysis/native/reference-input.json").read_text())
        if receipt["pid"] == int(Path("reference/basilisk.pid").read_text()):
            state["controlled_input"] = receipt["controlled_input"]
    if int(reference.read_region(quickdraw-0x7e, 4)["hex"], 16) != state["random_seed"]:
        raise RuntimeError("reference RNG changed during capture; stop music and capture a settled player turn")
    if combat and reference.read_region(a5+0x47194, 4)["hex"] != state["combat"]["cursor_enemy_index"]["hex"]:
        raise RuntimeError("reference turn changed during capture; capture a settled player turn")
    return state


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("path",type=Path)
    parser.add_argument("--combat", action="store_true")
    args = parser.parse_args()
    args.path.parent.mkdir(parents=True,exist_ok=True)
    args.path.write_text(json.dumps(capture(args.combat),indent=2)+"\n",encoding="utf-8")
    print(f"Captured reference state: {args.path}")
