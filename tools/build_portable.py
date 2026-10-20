"""Bounded native Linux/macOS validation and experimental packaging."""
from __future__ import annotations

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "target/platform-validation.json"
METRICS = []


def run(arguments, *, seconds=600, memory=16384, test=False, capture=False):
    def setup():
        os.nice(10)
        if test and hasattr(os, "sched_setaffinity"):
            os.sched_setaffinity(0, {min(os.sched_getaffinity(0))})
    started = time.monotonic()
    logfile = ROOT / "target/platform-command.log"
    logfile.parent.mkdir(exist_ok=True)
    peak = 0
    failure = None
    # Redirect build JSON to a file so Cargo cannot block on a full output pipe.
    with logfile.open("w+") as output:
        process = subprocess.Popen(arguments, cwd=ROOT, start_new_session=True,
                                   preexec_fn=setup, stdout=output if capture else None)
        try:
            while process.poll() is None:
                snapshot = subprocess.check_output(["ps", "-axo", "pgid=,rss="], text=True)
                resident = sum(int(row[1]) for line in snapshot.splitlines()
                               if len(row := line.split()) == 2 and int(row[0]) == process.pid)
                peak = max(peak, resident)
                if time.monotonic() - started > seconds or resident > memory * 1024:
                    failure = f"Resource limit exceeded: {arguments}"
                    break
                time.sleep(0.25)
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        output.seek(0)
        result = output.read() if capture else ""
    METRICS.append(dict(command=arguments, seconds=round(time.monotonic() - started, 3),
                        peak_memory_mib=round(peak / 1024, 2), exit_code=process.returncode))
    REPORT.write_text(json.dumps(dict(platform=sys.platform, compiler=subprocess.check_output(
        ["rustc", "-Vv"], text=True), cpus=os.cpu_count(), commands=METRICS), indent=2) + "\n")
    if failure or process.returncode:
        raise SystemExit(failure or f"Command failed ({process.returncode}): {arguments}")
    return result


def main():
    if sys.platform not in ("linux", "darwin"):
        raise SystemExit("Use tools/Build.ps1 on Windows")
    os.chdir(ROOT)
    # Leave at least 1 GiB for the OS on smaller hosted runners.
    physical_bytes = (int(subprocess.check_output(["sysctl", "-n", "hw.memsize"])) if sys.platform == "darwin"
                      else os.sysconf("SC_PHYS_PAGES") * os.sysconf("SC_PAGE_SIZE"))
    physical = physical_bytes // (1024 * 1024)
    memory = min(16384, max(512, physical - 1024))
    run([sys.executable, "tools/source_audit.py"])
    run([sys.executable, "tools/audit_public.py"])
    for pattern in ("test_extract.py", "test_public.py", "test_translation_plan.py"):
        run([sys.executable, "-m", "unittest", "discover", "-s", "tests/python", "-p", pattern])
    run(["cargo", "fmt", "--check"])
    cargo = ["--workspace", "--locked", "--jobs", str(os.cpu_count() or 1)]
    records = run(["cargo", "test", *cargo, "--no-run", "--message-format=json"],
                  memory=memory, capture=True)
    executables = set()
    for line in records.splitlines():
        record = json.loads(line)
        if record.get("reason") == "compiler-artifact" and record.get("profile", {}).get("test"):
            if executable := record.get("executable"):
                executables.add(executable)
    for executable in sorted(executables):
        run([executable, "--test-threads=1"], memory=512, test=True)
        if sys.platform == "linux" and Path(executable).name.startswith("centurions_desktop-"):
            run(["xvfb-run", "-a", sys.executable, "tests/python/native_dialog_driver.py", executable],
                seconds=120, memory=512, test=True)
    run(["cargo", "clippy", *cargo, "--all-targets", "--", "-D", "warnings"], memory=memory)
    run(["cargo", "build", *cargo, "--release", "--timings"], memory=memory)
    run([sys.executable, "tools/package.py"])


if __name__ == "__main__":
    main()
