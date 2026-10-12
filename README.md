# Newer Centurions

A work-in-progress native Windows Rust reconstruction of The New Centurions 1.3.2.
The original binary is the behavior authority. The current build covers
selected mission routes, combat, persistent maps, earned equipment, learned
psi, versioned saves and the original startup/menu presentation. Complete
campaign reconstruction remains unfinished.

Built with Codex Astra 6 and Sol 6.1.

<img width="802" height="632" alt="image" src="https://github.com/user-attachments/assets/c4dd1720-2598-4bdf-92d0-d091ed59266e" />
<img width="802" height="632" alt="image" src="https://github.com/user-attachments/assets/dfba2cc2-34f4-4e30-b7e9-385d32a3f442" />
<img width="802" height="632" alt="image" src="https://github.com/user-attachments/assets/c9540a6f-94a6-43d8-aca0-d0c694dc343b" />


## Source and content

This repository contains reconstruction code, test source, extraction and
analysis tools, and licensed third-party code. It does not contain the original
game executable, graphics, fonts, audio, maps, scenarios, warriors, manuals,
native comparison fixture data, screenshots or local emulator media. Public
Git history is separate from the private development history.

You need your own original game content to run the application. Registration
is assumed satisfied: the reconstruction does not ask for a serial number or
warrior code and does not gate story progression or promotion on registration.

## Build

Install stable Rust with the MSVC toolchain and Python 3 on Windows, then run:

```powershell
./tools/Build.ps1
```

Without private fixture data, this runs the synthetic resource-parser tests,
asset-independent Rust tests, Windows keyboard compatibility checks, formatting,
Clippy and the release build. Original-data parity tests are a separate optional
suite; a public source-only test pass does not establish campaign parity.

```powershell
cargo test --workspace
cargo build --workspace --release --locked
```

The four crates separate content readers, deterministic rules, inspection/replay
tools and native presentation. Desktop and replay use the same atomic command
boundary. Unsupported gameplay reports its missing dependency without changing
the represented state or random stream.

## Install the Windows beta

Download the Windows x86_64 ZIP from the GitHub Releases page and extract its
contents into your existing New Centurions game folder. The Windows executable
sits beside the original Mac application:

```text
TheNewCenturions/
  New Centurions                 # original Mac application
  New Centurions.rsrc            # preserved resource fork, where applicable
  The New Centurions.exe         # Windows reconstruction
  centurions-tools.exe           # optional inspection/replay tool
  Data Files/
  Scenarios/
  Warriors/
  Save/                         # original saves remain untouched
```

Run `The New Centurions.exe`. No `NewCenturions` wrapper directory, extraction
manifest, mapping receipt, Python or Ghidra is required to play. Original content
must be supplied by the player; the beta ZIP contains no game assets.
The Windows build uses the Microsoft Visual C++ runtime; install that runtime
if Windows reports a missing `VCRUNTIME140.dll`.

Macintosh resource forks must be preserved. The Windows loader supports raw
`.rsrc` sidecars, adjacent AppleDouble `._filename` sidecars, and AppleSingle
files containing both forks. A copy containing only data forks is insufficient.
The loader reads the original installation directly without modifying it or
writing extracted files into the game folder.

An explicit folder argument or `CENTURIONS_ASSET_ROOT` overrides discovery
beside the executable. The inspection tool also defaults to its own folder:

```powershell
./centurions-tools.exe inspect
```

Versioned reconstruction saves go to
`%LOCALAPPDATA%/NewCenturions/saves/session.json`. Original warriors and Mac saves
remain separate. The prepared extraction layout remains supported for existing
local development and native-comparison workflows. Installers and archives must
be extracted with resource forks preserved before running the Windows binary.

## Playable scope

After the logo, select Mission → Begin New Mission, add the six supplied warriors
and choose Done. Normal difficulty is supported. The selection order becomes
the squad order. The briefing scrolls, supports dragging and closes on
double-click. Open menus track the pointer across menu-bar headings.

Arrow/numpad keys move; Enter/Space advances ordinary story pages. Equipment,
ammunition, psi and medical actions are available for the recovered contracts.
Unavailable original menu actions remain disabled. Original file dialogs,
music, full text styling and complete campaign behavior remain unfinished.

## Original-data verification and packaging

Native fixture source is retained under `crates/*/tests` and `tools/ghidra`.
The `original-fixtures` feature enables those tests only when the corresponding
private fixtures and prepared game data are available locally. Keep them out
of public Git history:

```powershell
./tools/Build.ps1 -OriginalFixtures
cargo test --workspace --features original-fixtures
```

`./tools/Build.ps1 -Package` creates an asset-free Windows runtime folder and ZIP.
It contains the game and inspection executables, instructions and licenses.
It does not copy game data, fixtures, screenshots or extracted resources.

`python tools/export_source.py <fresh-directory>` prepares a clean source
snapshot. `python tools/audit_public.py` checks the public tracked files and
history before a push. Original source data and generated analysis remain
outside that snapshot.

## License

Reconstruction code uses the [MIT license](LICENSE). Original game content
retains its separate terms. Third-party licenses and decoder attribution are
listed in [THIRD_PARTY.md](THIRD_PARTY.md).
