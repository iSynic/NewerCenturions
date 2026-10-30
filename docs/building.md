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

## Test layout

`crates/core/Cargo.toml` sets `autotests = false` and declares one integration
test binary, `native_reference` (`tests/native_reference.rs`), with
`required-features = ["original-fixtures"]`. That binary holds the native
comparisons and also the maintained command and campaign tests, so a default
`cargo test --workspace` builds none of core's integration tests. Run them with
`--features original-fixtures`.

Under `crates/core/tests/`:

- `command_*.rs`, `source_*.rs`, `campaign.rs` and `enemy_callers.rs`:
  command/replay and campaign tests, declared as modules of `native_reference.rs`.
- `native/`: comparisons against original-binary fixtures, listed in
  `native/modules.rs` and included by `native_reference.rs`.
- `support/`, plus `prelude.rs`, `native_common.rs`, `snapshots.rs` and
  `hex.rs`: shared helpers for those tests, including command replay, campaign
  assistance and native save receipts. `support/replay_recorder.rs` is also the
  `centurions-replay-recorder` example.
- `unit/`: tests and helpers compiled into the library itself.

Tests that need private items are included from the source module they test
with `#[path]`, for example `src/time.rs` includes `tests/unit/time_reference.rs`
and `src/exploration.rs` includes `tests/native/regional_reference.rs`. In core
these modules are gated on `all(test, feature = "original-fixtures")`.
`crates/content` also sets `autotests = false` and gates its declared tests on
the same feature. The desktop includes its `tests/unit` files the same way;
some of those run without the feature.

## Test-code accounting

Preserve native fixture bytes and case identities. `python tools/source_audit.py`
counts two layers separately:

- **Maintained:** maintained tests against maintained production code.
- **Reference:** the archive's recovered execution bodies and routine catalog,
  against the probes, harnesses and tests that compare with them or with the
  original binary. The expanded translations are not counted.

Native-evidence comparisons, which test maintained code against original-binary
fixtures (`crates/core/tests/native/` and their dedicated helpers), are a third
category. They are reported with the fixture count and grow with native evidence.
The reported target for the remaining maintained test code is 25% of maintained
production code. The gate is a ratchet on the count of maintained test code lines, which may
not exceed `MAINTAINED_TEST_CEILING` in the tool:

- Lower the ceiling whenever consolidation removes lines.
- Never raise it to admit new tests; offset new coverage by consolidating.
- A count rather than a ratio, so that simplifying production code never fails
  the gate.

Both ratios, and the combined historical one, are still reported. Retain only distinct critical coverage, and prefer
the existing command/replay groups.

## Private development checkout

The material in this section exists only in the private development checkout.
It is excluded from the public source snapshot.

### Reference archive

The instruction-level reference is a separate Cargo workspace under `archive/`:
the routine catalog and recovered execution bodies (`archive/crates/reference`),
the expanded translations (`archive/crates/translated*`) and
`centurions-reference-tools`, which prints the routine catalog report. It builds
against the maintained crates in the root workspace.

```powershell
cargo test --manifest-path archive/Cargo.toml --features original-fixtures
cargo clippy --manifest-path archive/Cargo.toml --all-targets --features original-fixtures -- -D warnings
cargo run --manifest-path archive/Cargo.toml -p centurions-reference-tools
```

### Ghidra and reference tools

The Ghidra probes in `tools/ghidra/` and the menu-boundary check over their
fixtures, the Ghidra project setup scripts, the translation generator and the
original-game reference drivers are private.

`python tools/original_dispatch.py` extracts the original `newland` opcode arms
and the `attack2` natural-condition arms for warrior and NPC targets from the
private Ghidra exports. It writes values only to `evidence/original-dispatch.json`.
The inventory lists each original arm against the current Rust: a named story
operation, or the admission guards for natural conditions. `resolvespell` and
`castspell` branch on range guards and are not extracted.

### Local package with the original files

`python tools/package_local.py` builds a standalone local checkpoint that bundles the
supplied original game files beside the release binaries, after checking every
packaged fork against the extraction manifest. It is for local development only;
public packages never contain the originals.

### Engine inventory

`python tools/engine_inventory.py` regenerates `evidence/engine-inventory.json` from
the current source, the routine catalog and the supplied data. It covers unsupported
diagnostics, catalog routines (each binding labelled maintained or reference) and
the dispatch-domain values the supplied scenario uses. Hand judgments live only in
`evidence/engine-inventory-notes.json`, keyed by row id. `source_audit.py` runs the
same check as part of the build gate, and skips it in a checkout without the private
evidence and archive: a maintained diagnostic without a note, or a
catalog binding to a missing file, fails the build. `--check` also reports rows
marked `unreviewed`. A value absent from the supplied scenario is not
evidence that the original binary cannot reach it. Regenerate after changing a
diagnostic, a catalog binding or a dispatch arm.

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
