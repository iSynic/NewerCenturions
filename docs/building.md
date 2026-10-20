# Building and testing

Use stable Rust and Python 3.11 or newer. Original game content is needed to
play, but the public source and its asset-independent checks build without it.
See [installation notes](installation.md) for running a package and
[the maintainer guide](maintainability.md) for code ownership and change guides.

## Windows

Install the Rust MSVC toolchain, Visual Studio C++ build tools and PowerShell 7.
From the repository root:

```powershell
./tools/Build.ps1
```

This runs source accounting, Python checks, formatting, Rust tests, Windows
keyboard compatibility checks, Clippy and a release build. Without private
original data it selects only asset-independent tests.

To build an asset-free runtime folder and ZIP under `dist`:

```powershell
./tools/Build.ps1 -Package -Timings
```

Compiler work uses available CPUs at BelowNormal priority, with a 16 GiB
aggregate compiler-memory limit and a ten-minute limit per build command.
Test processes have separate CPU, memory and timeout limits.

## Linux and macOS

On Ubuntu, install build dependencies and the isolated file-picker test tools:

```sh
sudo apt-get install pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev zenity xvfb xdotool
```

On macOS, install the Xcode command-line tools. Build on the target operating
system and architecture:

```sh
python3 tools/build_portable.py
```

The script runs source checks, asset-independent Python and Rust checks,
formatting, Clippy and a timed release build, then packages the native binaries.
Compiler processes run at reduced priority with a 16 GiB aggregate memory
ceiling, lowered on smaller machines. Test processes have a 512 MiB resident
memory limit. Linux file-picker verification runs on an isolated virtual display.

## Focused checks

For a source-only checkout:

```sh
python -m unittest discover -s tests/python -p test_extract.py
python -m unittest discover -s tests/python -p test_public.py
python -m unittest discover -s tests/python -p test_translation_plan.py
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
```

Use the build scripts for bounded compiler and test execution. Direct Cargo
commands do not apply those process limits.

## Original-data checks

The full suite requires the private prepared game data and native fixtures.
Those inputs are not included in the public repository. In a prepared local
development checkout:

```powershell
python -m unittest discover -s tests/python
./tools/Build.ps1 -OriginalFixtures -ExtendedScenarios -Timings
```

The `original-fixtures` Cargo feature enables independent native-reference cases.
Essential tests stay independent of the complete translated registry. Local
packaging with prepared originals also requires the extended campaign replays.
These assisted replays check functionality and save/resume behavior; they do not
establish complete original-game parity or live desktop acceptance.

Preserve native fixture bytes and case identities. Keep test code and helpers at
or below 25% of maintained production code, as counted by
`python tools/source_audit.py`. Expanded instruction bodies and generated catalogs
are excluded from the production denominator. Retain only distinct critical
coverage; prefer existing command/replay groups.

## Public source and GitHub builds

Export an allowlisted snapshot into a fresh directory outside the private checkout:

```sh
python tools/export_source.py <fresh-directory>
```

The exporter uses `docs/public-readme.md` as the public README and excludes private
history, original assets, native fixtures and generated analysis. In a clean
public checkout, stage the snapshot and run `python tools/audit_public.py` before
pushing. Never push the private development history.

The **Source build** GitHub Actions workflow validates Windows x86_64,
Linux x86_64, macOS Apple Silicon and macOS Intel. Beta tags prepare a draft
release after all platform builds pass. Verify its archives before publishing.
The **Experimental platforms** workflow is also available for manual builds.
Packages contain executables, documentation and licenses, with no original game
assets. macOS bundles use an ad hoc signature, without Developer ID signing or
notarization.
