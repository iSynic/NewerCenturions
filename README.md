# Newer Centurions

I'm rebuilding **The New Centurions** in Rust so it can run on modern computers.
The original Mac version, 1.3.2, is the reference. I'm trying to keep it as close
to the original game as possible.

It's playable, but it's still a beta. Expect bugs and unfinished behavior.

## Download and play

[Download the latest build](https://github.com/iSynic/NewerCenturions/releases/latest).
Windows is the main platform. Linux and macOS builds are available, but they're
experimental. Mac downloads are separate for Apple Silicon and Intel.

You need your own copy of the original game. Extract the download into its
folder, beside the old Mac application, then run **The New Centurions.exe** on
Windows or **The New Centurions.app** on macOS. On Linux, run
**The New Centurions**.

Keep the original files and their resource forks intact. The download doesn't
include game data, and the new executable reads your existing installation.
You don't need Rust, Python or any development tools to play.

See [installation notes](docs/installation.md) if you need help getting it running.

## Screenshots

<img width="802" height="632" alt="The New Centurions screenshot" src="https://github.com/user-attachments/assets/c4dd1720-2598-4bdf-92d0-d091ed59266e" />
<img width="800" height="625" alt="image" src="https://github.com/user-attachments/assets/39bbd5aa-1229-415a-bca3-773cfdf73a4a" />
<img width="798" height="627" alt="image" src="https://github.com/user-attachments/assets/aa37ebf1-a1e2-4617-a305-60b80f53deed" />


## Working on the code

[Building and testing](docs/building.md) · [Code guide](docs/maintainability.md) ·
[Recovery notes and known limitations](docs/reconstruction.md)

Built with Codex Astra 6 and Sol 6.1.

The reconstruction code is [MIT licensed](LICENSE). The original game belongs
to its respective owners. See [THIRD_PARTY.md](THIRD_PARTY.md) for bundled code
and licenses.
