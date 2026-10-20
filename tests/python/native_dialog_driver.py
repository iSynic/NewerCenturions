"""Exercise the real Linux file picker on an isolated hosted-runner X display."""
import subprocess
import sys


def main():
    process = subprocess.Popen([sys.argv[1], "native_picker_save_open_and_cancel", "--ignored", "--nocapture"])
    try:
        for title, key in (("save", "alt+s"), ("open", "alt+o"), ("cancel", "Escape")):
            window = subprocess.check_output(["xdotool", "search", "--sync", "--onlyvisible",
                                              "--name", f"^Centurions picker {title}$"], timeout=30).decode().splitlines()[-1]
            subprocess.run(["xdotool", "windowfocus", "--sync", window], check=True, timeout=10)
            subprocess.run(["xdotool", "key", "--clearmodifiers", key], check=True, timeout=10)
        if process.wait(timeout=30):
            raise SystemExit("Native picker workflow failed")
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


if __name__ == "__main__":
    main()
