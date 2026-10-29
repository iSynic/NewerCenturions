"""Record assisted campaign functionality, with every setup change in the journal."""
import hashlib
import json
import os
from pathlib import Path

from campaign_mailbox import record_candidate


def setup(box, steps, settings, command):
    base = box.assistance
    box.assistance = base + settings
    steps.append({"command_index": len(box.commands()), "settings": box.assistance})
    response = box.send([command])
    if response.get("error"):
        raise RuntimeError(response["error"])
    box.assistance = base
    steps.append({"command_index": len(box.commands()), "settings": base})
    return response


def drain(box, steps):
    for _ in range(128):
        response = box.receive()
        if response.get("error"):
            raise RuntimeError(response["error"])
        state = response["session"]
        position = state.get("exploration")
        page = position["presentation"] if position else None
        if state.get("promotion") or state.get("survivors") or page == "Loot":
            box.collect_rewards()
        elif state["phase"] == "Combat" and page is None:
            setup(box, steps, ["DefeatEnemies"], "EndTurn")
        elif isinstance(page, dict) and any(key in page for key in ("Question", "StageChange")):
            box.send([{"Choose": {"answer": True}}])
        elif isinstance(page, dict) and "WarriorChoice" in page:
            box.send([{"ChooseWarrior": {"index": 0}}])
        elif isinstance(page, dict) and any(key in page for key in ("Dialogue", "Messages", "Prologue")):
            box.send(["Continue"])
        else:
            return response
    raise RuntimeError("Functional continuation budget exhausted")


def route(root, seed, difficulty, target):
    commands = [{"SetDifficulty": {"level": difficulty}}, "ImportStarters",
                {"EnterMission": {"name": "The Thunder of Guns"}}]
    with record_candidate(root, commands, seed, target) as box:
        steps = [{"command_index": len(commands), "settings": ["Health", "Ammunition", "Psi"]}]
        box.assistance = steps[0]["settings"]

        def enter(map_id, x, y, dx, dy, extra=()):
            setup(box, steps, [{"Position": {"map": map_id, "x": x, "y": y}}] + list(extra),
                  {"Move": {"dx": dx, "dy": dy}})
            return drain(box, steps)

        enter(5, 4, 85, -1, -1)
        enter(1, 7, 3, 1, 0, [{"QuestItems": list(range(804, 810))}])
        enter(3, 58, 39, 1, 0, [{"Region": {"slot": 6, "chance": 0}}] if seed == 1 else [])
        box.send([{"SubmitText": {"text": "wildriver"}}])
        drain(box, steps)
        box.send(["CancelMenu"])
        drain(box, steps)
        enter(3, 57, 41, 1, 0)
        box.send([{"Move": {"dx": 0, "dy": 1}}])
        drain(box, steps)
        enter(8, 11, 28, 1, 0)
        enter(8, 22, 15, 1, 0)
        enter(4, 35, 16, 1, 0)
        response = enter(9, 75, 73, 0, 1, [
            {"Region": {"slot": 5, "chance": 10000}},
            {"Flag": {"index": 52, "value": int(seed == 1)}},
        ])
        assert response["session"].get("mission_stage", 0) == 0
        directory = Path(box.directory)
        rows = [json.loads(line) for line in next(directory.glob("commands-*.jsonl")).read_text().splitlines()]
        output = root / "analysis/thunder-completion"
        name = f"functional-{seed}-{difficulty}.json"
        raw = (json.dumps(box.commands()) + "\n").encode()
        (output / name).write_bytes(raw)
        events = json.dumps([row["events"] for row in rows], sort_keys=True,
                            separators=(",", ":"), ensure_ascii=False).encode()
        return {"seed": seed, "difficulty": difficulty, "commands": name,
                "commands_sha256": hashlib.sha256(raw).hexdigest(),
                "events_sha256": hashlib.sha256(events).hexdigest(),
                "session_sha256": rows[-1]["state_sha256"], "assistance": steps}, response, directory


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[2]
    output = root / "analysis/thunder-completion"
    routes = []
    for seed, difficulty in [(12, 0), (1, 0), (12, -2), (12, -1), (12, 1), (12, 2)]:
        receipt, response, directory = route(root, seed, difficulty,
                                             os.environ.get("CENTURIONS_CAMPAIGN_TARGET"))
        routes.append(receipt)
        print(seed, difficulty, directory, flush=True)
        manifest = {"verification_kind": "assisted_functional",
                    "qualification": "Explicit branch prerequisites, positions and battle cleanup setup; no acquisition, navigation, combat-math or unaided completion claim.",
                    "content_identity": response["session"]["content_identity"],
                    "source_sha256": json.loads((directory / "response-0.json").read_text())["source_sha256"],
                    "routes": routes}
        (output / "campaign-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
