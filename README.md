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

The workspace separates content readers, deterministic rules, inspection/replay
tools, native presentation and three expanded translation libraries. A composition
crate owns the full routine registry; essential tests depend on the small runtime.
Desktop and replay use the same atomic command
boundary. Unsupported gameplay reports its missing dependency without changing
the represented state or random stream.

## Experimental Linux and macOS builds

The **Experimental platforms** GitHub Actions workflow produces downloadable
Windows x86_64, Linux x86_64, macOS Apple Silicon and macOS Intel artifacts.
These are experimental packages, not published releases. Original game content
is never included. A successful build does not establish original-game or live
desktop parity.

On Linux, extract the ZIP beside your original installation and run
`./The\ New\ Centurions`. Install `zenity` or `kdialog` for Save/Open dialogs;
the application invokes these system file pickers without a shell. Ubuntu
build dependencies are `pkg-config libx11-dev libxi-dev libgl1-mesa-dev
libasound2-dev`; runtime needs the corresponding X11, OpenGL and ALSA libraries.
The validation script also needs `xvfb` and `xdotool` to exercise file pickers
on an isolated virtual display.
An X11 session or XWayland is required by the current desktop backend.
If your extractor drops executable permissions, run
`chmod +x 'The New Centurions' centurions-tools`.

On modern macOS, put **The New Centurions.app** beside the original Mac
application and the Data Files, Scenarios and Warriors folders. Finder launch
discovers content beside the bundle, independently of the working directory.
Save/Open uses the macOS file panels through the system scripting host. The
experimental bundle has an ad-hoc signature, not Developer ID signing or
notarization; macOS may require an explicit approval in Privacy & Security.
Use the Apple Silicon or Intel package matching your Mac.

Resource forks can be preserved as `.rsrc`, AppleDouble or AppleSingle on either
platform; macOS also reads native resource forks without modifying originals.
Keep the same content and fork representation when moving reconstructed saves
between installations, because saves validate the installation identity.

Saves, preferences, created warriors and logs use persistent per-user storage:
`$XDG_DATA_HOME/NewCenturions` (default `~/.local/share/NewCenturions`) on Linux,
`~/Library/Application Support/NewCenturions` on macOS, and the existing
`%LOCALAPPDATA%/NewCenturions` on Windows. Fast Save retains the selected named
file. Cancelling a picker or loading a malformed save leaves the session intact.
The `--save-path`, `--preferences-path` and `--roster-path` overrides remain available.

To build and validate on Linux/macOS with stable Rust and Python 3.11 or newer:

```sh
python3 tools/build_portable.py
```

This checks the source ceiling and public-source boundary, runs the essential
asset-independent checks, Clippy and a timed release build, and packages the
native binaries. Compiler jobs use available CPUs at reduced priority with a
16 GiB aggregate ceiling (lower on small runners); tests run separately with
a 512 MiB resident-memory limit. Private native references remain a local suite.

For acceptance on each platform, exercise named Save/Open, cancellation and
Fast Save; Alt+Enter and window restoration; held movement and targeting;
inventory/shop dialogs; music/audio; and comparison with the original visuals.
Interactive acceptance remains pending until someone actually observes it.

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

Save Current Game and Load A Saved Game open Windows file dialogs, allowing
multiple named, versioned reconstruction saves. The initial directory is
`%LOCALAPPDATA%/NewCenturions/saves`; Fast Save reuses the last successfully
saved or loaded file. Cancellation leaves the session unchanged. Existing Mac
saves and original data directories cannot be overwritten. `--save-path` keeps
the fixed-file workflow for command-line use. Original warriors and Mac saves
remain separate. The prepared extraction layout remains supported for existing
local development and native-comparison workflows. Installers and archives must
be extracted with resource forks preserved before running the Windows binary.

Created warriors and mission-end warrior updates are stored separately in
`%LOCALAPPDATA%/NewCenturions/warriors.json`. The Character menu connects the
original creation dialogs, initial Psi selection, starting equipment, squad
ordering and squad modification. Ending a mission returns to the title screen
after preserving its warriors. UI preferences are saved in
`%LOCALAPPDATA%/NewCenturions/ui.json`; original files remain read-only.

Windows starts in a movable window. Alt+Enter toggles fullscreen, including
while a dialog is open; holding the shortcut does not repeat. The original
Hide The Desktop control changes fullscreen for the current session. Changing
the 640×480/800×600 preference returns to windowed mode at the chosen size.

## Playable scope

After the logo, select Mission → Begin New Mission, add the six supplied warriors
and choose Done. All five original difficulty levels can be selected. The selection order becomes
the squad order. The briefing scrolls, supports dragging and closes on
double-click. Open menus track the pointer across menu-bar headings.

Arrow/numpad keys move; Enter/Space advances ordinary story pages. Equipment,
ammunition, psi and medical actions are available for the recovered contracts.
The Warrior Editor uses the original activation/deactivation messages. Warrior
information connects to paid upgrades and free editor upgrades; portraits and
tactical icons can be changed through their original selection dialog. Music
uses the original sound resources and playlist menu. Sound effects and music
have separate, persisted volume controls, including mute. Technical failures
appear in a separate error dialog, not the scrolling story text.

Archives, Allies, Maps/Notes, character creation, squad management and mission
ending now have connected handlers. Original Mac save import has a read-only
foundation, but complete compatibility with populated original saves is not
certified. Additional mission-pack compatibility remains unfinished.

The current functionality checkpoint passes six assisted Thunder routes across
all six stages, both ending branches and the mission reset. These checks use
explicit setup and battle cleanup to test functionality; they do not establish
that every combat, ability, equipment or story branch works.

Known gaps include a failure when collecting authored credit chests, omitted
learned Psi choices and encounter effects, restricted occupied-target reloads,
conditioned party entry, some enemy weapons and recruited allies, retained
equipment/death-script contexts, original text input and the About menu.
Native-reference coverage and live Windows visual, audio, input and resource-use
acceptance remain separate. This beta is not certified for reliable complete
campaign play. See [the current limitations](docs/reconstruction.md#current-functionality-checkpoint).

## Original-data verification and packaging

Native reference runners are retained under `crates/*/tests`. Maintained capture
support is under `tools/ghidra` and `tests/native`; superseded capture generators
are retired while their historical provenance and private reference data remain
preserved.
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
