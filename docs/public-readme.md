# The New Centurions

A work-in-progress native Windows Rust reconstruction of Centurions 1.3.2.
The original binary is the behavior authority. The current build covers
selected mission routes, combat, persistent maps, earned equipment, learned
psi, versioned saves and the original startup/menu presentation. Complete
campaign reconstruction remains unfinished.

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

## Prepare local content and run

Keep original files in a separate local asset directory. The current loader uses
the fork-preserving extraction layout below; Macintosh resource forks must be
preserved. An ordinary copy containing only data forks is insufficient.

```text
<asset-root>/
  NewCenturions/
    packsmith-mapping.json
    TheNewCenturions/
      New Centurions
      New Centurions.rsrc
      Data Files/
      Scenarios/
      Warriors/
  analysis/extracted/
    manifest.json
    resources/
```

`packsmith-mapping.json` is local extraction metadata with relative file paths
and fork hashes. It is supplied by the fork-preserving extraction workflow,
not downloaded from this repository. Resource sidecars can be raw forks or
AppleSingle/AppleDouble containers. Prepare decoded resources locally:

```powershell
python tools/extract.py --input 'D:/Games/TNC-assets/NewCenturions' --output 'D:/Games/TNC-assets/analysis/extracted'
./target/release/centurions-desktop.exe 'D:/Games/TNC-assets'
./target/release/centurions-tools.exe inspect 'D:/Games/TNC-assets'
```

Alternatively set `CENTURIONS_ASSET_ROOT` to that directory. A positional asset
root takes precedence. If neither is given, the desktop looks beside its own
executable; the tools default to the current directory. Original files are
read-only. Versioned reconstruction saves go to
`%LOCALAPPDATA%/NewCenturions/saves/session.json`.

A direct installer into an existing classic game folder is planned. The current
runtime requires the prepared layout above; it does not yet extract arbitrary
classic installers or archives automatically.

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
It contains executables, preparation instructions, the extractor and licenses.
It does not copy game data, fixtures, screenshots or extracted resources.

`python tools/export_source.py <fresh-directory>` prepares a clean source
snapshot. `python tools/audit_public.py` checks the public tracked files and
history before a push. Original source data and generated analysis remain
outside that snapshot.

## License

Reconstruction code uses the [MIT license](LICENSE). Original game content
retains its separate terms. Third-party licenses and decoder attribution are
listed in [THIRD_PARTY.md](THIRD_PARTY.md).
