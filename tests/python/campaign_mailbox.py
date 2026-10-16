"""Client for guarded headless recordings with explicit test-assistance receipts."""
import json
from contextlib import contextmanager, suppress
import os
from pathlib import Path
import struct
import subprocess
import time


@contextmanager
def record_candidate(root, prefix, seed=12, target_directory=None, assistance_steps=()):
    root = Path(root).resolve()
    directory = root / "analysis/thunder-completion" / f"candidate-{time.time_ns()}"
    directory.mkdir(parents=True)
    environment = dict(os.environ, CENTURIONS_CAMPAIGN_DRIVER=str(directory),
                       CENTURIONS_CAMPAIGN_SEED=str(seed))
    target_directory = Path(target_directory) if target_directory else root / "target/campaign-recording"
    arguments = [
        "pwsh", "-NoProfile", "-File", str(root / "tools/Run-Cargo.ps1"),
        "-Action", "Run", "-Package", "centurions-core", "-Release",
        "-Binary", "centurions-replay-recorder", "-MaximumTestSeconds", "120",
        "-TargetDirectory", str(target_directory),
        "-MetricsPath", str(directory / "metrics.json"),
    ]
    with (directory / "process.log").open("wb") as output:
        process = subprocess.Popen(arguments, cwd=root, env=environment, stdout=output,
                                   stderr=subprocess.STDOUT, creationflags=0x08004000)
        box = None
        try:
            while not (directory / "response-0.json").exists():
                if process.poll() is not None:
                    raise RuntimeError(f"Recorder stopped during startup: {directory}")
                time.sleep(0.05)
            box = Mailbox(directory, process)
            cursor = 0
            for step in assistance_steps:
                boundary = step["command_index"]
                if not cursor <= boundary <= len(prefix):
                    raise ValueError("Assistance boundaries must be ordered within the recording")
                response = box.send(prefix[cursor:boundary], timeout=110)
                if response.get("error"):
                    raise RuntimeError(f"Recorded prefix failed: {response['error']}")
                box.assistance = step["settings"]
                cursor = boundary
            response = box.send(prefix[cursor:], timeout=110)
            if response.get("error"):
                raise RuntimeError(f"Recorded prefix failed: {response['error']}")
            yield box
        finally:
            try:
                if box is not None and process.poll() is None:
                    with suppress(TimeoutError, subprocess.TimeoutExpired, RuntimeError):
                        box.send([], finish=True, timeout=15)
                        process.wait(timeout=15)
            finally:
                if process.poll() is None:
                    # Closing the guard owner also closes its kill-on-close child job.
                    process.terminate()
                    process.wait(timeout=5)


class Mailbox:
    def __init__(self, directory, process=None):
        self.directory = Path(directory)
        self.process = process
        self.assistance = []
        self.owner = json.loads((self.directory / "response-0.json").read_text())["owner"]
        request = self.directory / "request.json"
        previous = json.loads(request.read_text()) if request.exists() else {}
        self.id = previous.get("id", 0) if previous.get("owner") == self.owner else 0

    def receive(self, timeout=30):
        path = self.directory / f"response-{self.id}.json"
        deadline = time.monotonic() + timeout
        while True:
            try:
                response = json.loads(path.read_text())
                break
            except (FileNotFoundError, PermissionError):
                if self.process is not None and self.process.poll() is not None:
                    raise RuntimeError(f"Owned recorder stopped; inspect {self.directory / 'process.log'}")
            if time.monotonic() >= deadline:
                raise TimeoutError(f"Request {self.id} is still pending; inspect its owned process")
            time.sleep(0.02)
        assert response["owner"] == self.owner and response["id"] == self.id
        return response

    def send(self, commands, finish=False, timeout=30):
        previous = self.receive(timeout)
        if previous.get("error") and not finish:
            raise RuntimeError("Candidate failed; preserve its diagnostic and valid prefix")
        self.id += 1
        request = {"owner": self.owner, "id": self.id, "commands": commands, "finish": finish,
                   "assistance": self.assistance}
        temporary = self.directory / "request.tmp"
        temporary.write_text(json.dumps(request), encoding="utf-8")
        deadline = time.monotonic() + 30
        while True:
            try:
                temporary.replace(self.directory / "request.json")
                break
            except PermissionError:
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.02)
        return self.receive(timeout)

    def commands(self):
        rows = (self.directory / f"commands-{self.owner}.jsonl").read_text().splitlines()
        commands = []
        for line in rows:
            row = json.loads(line)
            if "error" in row:
                raise RuntimeError("Candidate contains a failed command; not an accepted recording")
            commands.append(row["command"])
        return commands

    def continue_dialogues(self):
        response = self.receive()
        for _ in range(32):
            position = response["session"].get("exploration")
            presentation = position.get("presentation") if position else None
            if (response.get("error") or not isinstance(presentation, dict)
                    or not any(key in presentation for key in ("Dialogue", "Messages"))):
                return response
            response = self.send(["Continue"])
        raise RuntimeError("Dialogue continuation budget exhausted")

    def collect_rewards(self):
        response = self.receive()
        word = lambda raw, offset: struct.unpack_from(">h", bytes(raw), offset)[0]
        for _ in range(64):
            state = response["session"]
            if response.get("error"):
                break
            promotion = state.get("promotion")
            if promotion:
                command = ({"PromotionAnswer": {"yes": True}} if promotion.get("question")
                           else {"PromotionControl": {"item": 1}})
            elif state.get("survivors"):
                command = "Continue"
            elif state.get("exploration") and state["exploration"]["presentation"] == "Loot":
                existing = {
                    word(warrior["bytes"], 0x132 + slot * 22)
                    for warrior in state["warriors"]
                    for slot in range(word(warrior["bytes"], 0x1A))
                }
                entry = next(((i, item) for i, item in enumerate(state["booty_items"])
                              if word(item, 0)), None)
                if entry is None:
                    command = "Continue"
                else:
                    slot, item = entry
                    ident = word(item, 0)
                    command = ({"TakeLoot": {"warrior": 0, "slot": slot}}
                               if 800 <= ident < 900 and ident not in existing
                               else {"StoreLoot": {"slot": slot}})
            else:
                break
            response = self.send([command])
        return response

