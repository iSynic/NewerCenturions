# Reconstruction toolkit

Run `python tools/toolkit.py --help` from the checkout. The five tools share
content identities and private report storage, while their implementations stay
separate under `tools/reconstruction_toolkit`. They never push, publish, change
the original installation or update backlog acceptance automatically.

Generated outputs must be new files or directories under this checkout's ignored
`analysis/`. Inputs are read-only. JSON receipts contain input SHA-256 values;
HTML reports work offline and include filters and downloadable JSON. Captured
text is rendered as text, including text that contains HTML or script syntax.
Reports can contain original data and local paths: keep them private.

## Native/source comparison

```powershell
python tools/toolkit.py compare --native evidence/fixtures/retained-getup-field.json --case field-native-status --source analysis/backlog-31/release-controls-02/field-native-status.stdout --out analysis/my-review/field
```

`--case` accepts a unique kind/name or zero-based index. Retained captures with
`warriors_after`, `npcs_after`, field/map/packet arrays and work globals are
adapted directly. Source input can be the existing replay command's JSONL
output, a saved document, or a canonical comparison document. The native
projection chooses which source work fields are compared; omitted native data
is not inferred from the source. Unknown bytes remain compared.

Differences include record index, byte offset, native address, known field name,
width and signed value. Map rows include native X/Y coordinates. Full event and
RNG sequences are compared when both inputs capture them; retained receipts
without a native draw sequence qualify aggregate seed/draw count only. The
retained event projection compares sound/result/status/Flash requests, excluding
uncaptured amounts and source-only presentation events explicitly. The first
difference is the first in the report's record/work/RNG/event ordering, not an
inferred causal explanation across those independent sections. At most500 rows
are stored; total differences and truncation remain visible.

Canonical documents have this shape:

```json
{
  "schema": "comparison-1",
  "records": {"warriors": "0000"},
  "work": {"ready": 0},
  "rng": {"seed": 1, "draws": 0},
  "event_projection": "full",
  "events": []
}
```

That short byte string illustrates the encoding, not a valid warrior record.
Record bytes can also be byte arrays. Add `rng.trace` entries with `call` and
`result` to compare draw order. A failed or truncated probe remains inspectable
but is not usable equality evidence. Exit codes are0 for a usable match,1 for
differences/unusable evidence and2 for invalid inputs or execution errors.

## Reusable native probes

`ToolkitProbeHarness.java` shares initialization, bounded stepping, explicit
service interception and capture formatting on the existing ActionOracleHarness.
`VerifyToolkitProbe.java` runs declarative case sets in an existing read-only
Ghidra project. Earlier probes and the frozen SANE/FPU implementation are not
modified. The Ghidra project and reference definitions remain private inputs.

```powershell
python tools/toolkit.py probe --ghidra /path/to/ghidra --project analysis/ghidra/project/CenturionsMapped --spec analysis/my-review/cases.json --out analysis/my-review/probe
```

A minimal original `getmovecost153de6` case:

```json
{
  "schema_version": 1,
  "cases": [{
    "name": "movement-cost", "entry": "0x153de6",
    "stack": "0x500000", "sentinel": "0x5ffff0",
    "maximum_instructions": 200, "compare_registers": ["D0"],
    "values": [
      {"address": "0x43167e", "width": 2, "value": 8},
      {"address": "0x442736", "width": 1, "value": 1},
      {"address": "0x442737", "width": 1, "value": 1}
    ],
    "stack_values": [{"offset": 4, "width": 2, "value": 1000}],
    "captures": [{"name": "direction", "address": "0x442736", "length": 2}]
  }]
}
```

`memory` entries supply `address` and exactly one of `hex` or a repository-relative
`file` with byte `offset` and `length`. Scalar values use widths1/2/4. Registers
accept D0-D7, A0-A6 and the condition flags. Extra `stops` are explicit native
addresses. Each case has its own emulator and cannot alter the imported program.

Each optional `services` entry needs a unique `id`, exactly one `entry` address
or `opcode`, and a `kind`: `return`, `trap`, `random_bound` or `sane_model`.
`arguments` record named stack offsets/widths. `writes` declare literal emulated
memory writes; `registers` declare returned register values. `return` defaults to
C caller cleanup; `argument_bytes` declares additional callee cleanup. A `trap`
requires its exact `stack_delta`. `random_bound` controls the existing bounded
Rand primitive, reading its WORD argument at `argument_offset` (default4).
`sane_model` is allowed only forA9EB and delegates to the unchanged model.
There is no blanket unknown-trap or unknown-call success handler.

Failures retain PC/opcode, registers, the last64 instruction locations and native
call targets, the stack window, declared memory watches and up to4,096 service
calls. Call-log truncation is explicit.
Instruction budgets stop runaway cases; uninitialized-memory diagnostics make
the receipt fail. Provenance includes the specification, helper definitions,
input files, Ghidra installation identity and initialized program-memory blocks.
The receipt and HTML viewer retain completed and failed cases separately. A
completed probe proves only its bounded original instruction segment with its
declared controls; it does not qualify original ROM/FPU, UI or a whole caller.

## Validation and preservation batches

Prepare a JSON array containing every literal file intended for the commit.
Initialize only after finishing source edits. Use an existing reviewed
preservation baseline containing the frozen-file hash map.

```powershell
python tools/toolkit.py batch init --dir analysis/my-batch --allow analysis/my-allow.json --protected-from analysis/backlog-31/baseline.json
python tools/toolkit.py batch run --dir analysis/my-batch
python tools/toolkit.py batch verify --dir analysis/my-batch
python tools/toolkit.py batch stage --dir analysis/my-batch
```

The runner executes the seven checks required by AGENTS.md using the guarded
Cargo runner, one build job and the existing test bounds. `--checks python format`
runs a subset during investigation; staging still requires all seven. Resuming
reuses successful checks only when source files, the complete source file set,
allowlisted documentation/metadata, toolchain and recorded logs still match.
Missing or changed inputs require a new batch rather than refreshing old success.
Interrupted runs retain authoritative process IDs; another run refuses to
restart while a recorded process remains alive. Attempts have separate logs.
A process exit0 becomes reusable only after post-check identity verification
succeeds. Interrupted or invalidated attempts retain their logs without a cached
success status.

The protected set includes all existing checkout files outside the allowlist,
original installation files and the supplied frozen map. It is checked before
and after checks and again before staging. Receipts stay under analysis. Staging
uses a literal pathspec, checks whitespace and the complete staged path set with
rename detection disabled, and records the reviewed tree. It requires an empty
index and refuses missing files rather than implicitly staging deletions.
There is deliberately no commit, reset, cleanup, push or publish command.

After reviewing the staged diff and making a local commit, use
`batch verify --dir analysis/my-batch --after-commit`. This admits only a direct
child of the checkpoint with the exact reviewed tree and an empty index.
Public source-only builds still use default Cargo features; this private batch
plan intentionally requires original fixtures and the private archive.

## Scenario-to-backlog navigation

```powershell
python tools/toolkit.py navigate --out analysis/my-review/routes
```

`--originals` selects a different input root. `--scenario` selects the scenario
level folder relative to it; the default remains the supplied Thunder data.
The existing route catalogue now accepts this optional selection without
changing its earlier callers. Reports join authored operations, item classes
and spell packets to source references, current diagnostics and backlog groups.

Typed Operation references and legacy numeric dispatch arms are shown
separately. A diagnostic in the same source file is a coarse investigation link,
not proof that an opcode reaches it. Item/packet group links are labelled workflow
planning links. Conservative potential nodes, conditional edges and unqualified
nodes retain the existing route analysis's limitations.

`--observations replay.jsonl` adds executed source opcode/spell observations.
Without a capture envelope containing `build_sha256`, these are explicitly
unbound source observations. Source traces never become original-game
observations, and absent observations never establish unreachability. No ledger
or status file is changed.

## Product acceptance kit

```powershell
python tools/toolkit.py acceptance inventory --input NewCenturions --out analysis/my-review/available
python tools/toolkit.py acceptance run --manifest analysis/my-review/acceptance.json --build target/release/centurions-tools.exe --assets . --out analysis/my-review/acceptance-run
```

Inventory identifies scenario, resource-fork and Data I1 save candidates; their
provenance still needs review. A manifest uses schema_version1 and a `cases`
array with unique filename-safe `id`, `kind`, optional `gates` and `provenance`.
Paths inside it are relative to the manifest, unless absolute:

```json
{
  "schema_version": 1,
  "cases": [
    {"id": "resources", "kind": "resources", "gates": ["P05", "P06"]},
    {"id": "saved-return", "kind": "replay", "input": "save.json",
     "commands": "commands.json", "gates": ["P01"], "provenance": "reviewed input description"},
    {"id": "original-import", "kind": "import", "input": "original-save/Data I1",
     "commands": "commands.json", "gates": ["P01"], "provenance": "reviewed original save origin"}
  ]
}
```

The resources adapter calls the runtime's actual PICT/cicn/ppat and sound
decoders. It records source and decoded checksums, dimensions, channel/rate/bit
metadata, rejected forms and known forms without adapters. It exports no decoded
assets. Decode success does not establish original visual or playback quality.

The import adapter uses native_save::import, saves a separate schema-versioned
JSON file under the new analysis run, reloads it, checks equality, and runs the
supplied commands through the same apply boundary as desktop/replay. Errors and
whether they changed state remain visible. Reconstruction replay uses the
existing CLI with tracing. Replay and import cases emit a build-bound
`observations.json` envelope for the navigator; its source-output checksum is
verified when read. An optional `expected` JSON file requests an exact
structured output comparison. Per-case timeout defaults to120 seconds.

Runs fingerprint the executable, installation/extraction inputs, manifest,
commands, saves and expected output, checking them again after execution. An
executable checksum identifies the build; it does not by itself prove which
source produced an externally supplied executable. Cases retain stdout/stderr,
status and checksums. The six product gates collect evidence and remain
`review required`, including when every headless case passes.

After actually observing a workflow, record an attributed observation:

```powershell
python tools/toolkit.py acceptance observe --receipt analysis/my-review/acceptance-run/receipt.json --build target/release/centurions-tools.exe --gate P02 --observer maintainer --workflow "save and return" --status pass --note "Observed result" --evidence analysis/my-review/screenshot.png
```

Observations bind the executable, host, observer, workflow, note and optional
evidence hashes. A different build is rejected. Later observations produce new
HTML revisions. This records an attestation; the tool does not claim to have
performed the observation. Missing genuine saves/scenarios and unavailable
platforms stay open rather than being replaced with constructed evidence.
