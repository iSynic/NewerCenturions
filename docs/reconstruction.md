# Source translation workflow

The maintained runtime and desktop are organized by game subsystem and screen.
See [Maintaining the runtime](maintainability.md) for ownership, the complete
combat-click path, save contracts and change guides. The refactor preserves
expanded translation libraries and their existing compiler profiles.

Desktop integration is tracked separately from source translation. Recovered
routine coverage does not mean that every original menu or control is connected.
The runtime correction pass connects original editor feedback, paid/free upgrades,
warrior information, appearance selection, music playlist toggles and typed combat
refusals. Native refusal receipts cover the bounded ranged-selection branch;
ordinary map-click routing remains source-derived until the complete original
event loop is observed. Live desktop comparison is still required for visual
parity. Creation, roster, mission-ending and preference handlers are connected;
their existence does not establish complete interactive or original-game parity.

## Local implementation checkpoint

The maintained runtime keeps the compiler split and shared command boundary.
Schema 12 saves retain readers for versions 1–11. Persistent round/death caller
ownership, retained death queues and script pauses are validated before replacing
a session; older formats cannot silently admit those new contexts. Version 10 also
retains outdoor camp/search state, clinic access and interrupted control actions.
Old saves initialize these new contexts as inactive.

The current repairs connect authored credit chests, retained equipment and NPC
work, conditioned battle entry, large allies, friendly rifle turns, ordinary
click reload, natural combat effects, the previously omitted learned powers,
encounter item returns and original text conversion. The About menu uses its
original resources, sounds and delay. Creature labels respect their original
Pascal lengths and the supplied bestiary order.

The exploration panel uses original PICT 176 and its original control rectangles.
Inventory, trade, Psi, camp, automatic/active search, rest, healing and contextual
crate/shop/clinic controls connect shared actions. Reopening shop/clinic dialogs
uses their retained ownership checks rather than requiring an active script.
Held control loops are bounded and stop retrying after refusals. These connections
are source-backed compositions of independently compared rule primitives; full
button-level native comparison and user-operated visual acceptance remain separate.
Mechanical capture now has shared entry, target, cancellation and saved-state commands. Bounded original transactions compare records, costs, RNG and feedback; whole-click and live presentation parity remain separate.

Six assisted command routes exercise all six Thunder stages, the authored
conclusion and mission-one reset. Each recording is independently replayed and
compared with saved suffixes. Assistance is explicit; these checks test
functionality without requiring unaided campaign completion.

This does not certify every alternative or obtainable feature. Remaining work
is tracked in one reconciled completion inventory: complete original caller
comparisons, broader combat/Psi/actor qualification, remaining branch coverage,
Hive Leader fault qualification, and live presentation and interaction
acceptance. All 35 historical case identities now have executed current ownership or an
exact independent superseding receipt. Ten original ray-dispatch and six movement-spell controls now compare actual
commands and saved replay at explicit native service boundaries. Four historical
occupied-refill controls also run through Move with documented NPC health
assistance. The source-domain script-entry audit classifies all 26 previously
unqualified identities without using campaign route absence as proof.
The two nested-caller cases compare the
complete captured three-window sequence, retained records, queue/header bytes,
RNG and ordered feedback. Their controlled service boundaries remain explicit.

Current enforced accounting is 26,849 test code lines / 108,920 maintained
production code lines (24.650202%, ceiling 25%). Physical totals are 28,084 test
and 112,232 production lines. Test helpers and verification scripts are counted;
expanded bodies, generated catalogs, fixtures, vendor code and recovery tools
are excluded from maintained production code. All frozen native references
remain intact. New bounded native cases compare natural effects and all 256
original MacRoman case-fold outputs. A complete original opcode-8 header copy
now agrees with the actual movement/save path. Original script replacement
retains the full footer; Click Mouse pauses persist and resume through the
desktop feedback handler. These are distinct from live observations.

Fresh cold, representative maintained-runtime edit and no-change release
measurements, compiler version, hardware and memory peaks are recorded in the
local checkpoint provenance. The gates remain ten minutes cold and two minutes
incremental. These do not predict hosted runner times. Compiler priority and
memory guards remain in place.

Native file dialogs, fullscreen restoration, held movement and targeting,
inventory/trade/shops, music/audio, original visuals/help and actual desktop
resource use require user-operated acceptance. Complete populated Mac save
import, other genuine scenarios and publishing remain separate work. Experimental
Linux and macOS builds are available; live acceptance on those platforms remains
separate from compilation and save roundtrip checks.
No automated desktop control or public release is part of this pass.

## Recovery workflow

Recover application routines from the mapped Ghidra project, split their
control flow into ordered action blocks, translate related behavior into Rust,
and compile the batch. Native comparisons refine fidelity after implementation;
they do not decide whether a recovered action may be implemented.

The exporter `tools/ghidra/ExportActionInventory.java` processes CODE 1
application routines using the existing project with `-readOnly -noanalysis`.
It writes decompiled C, actual function ranges, direct call dependencies,
basic blocks, ordered instructions, and width-preserving p-code to a fresh
ignored analysis directory. Runtime/math segments and imported thunks are
excluded from the application count. A failed decompilation retains the
instruction/control-flow export for translation.

Build the private action plan and public-safe compiled catalog with:

```powershell
python tools/plan_translation.py analysis/reconstruction/application-actions-1/actions.json
cargo test -p centurions-core --lib
cargo build --workspace --release
./target/release/centurions-tools.exe translation-report
./target/release/centurions-tools.exe translation-report calcstats
```

The report needs no game installation. Its generated catalog contains only
routine names, addresses, counts, dependencies and Rust source ownership.
Decompiled C, instructions, executable bytes and game data remain outside the
public source snapshot. The private plan retains ordered action references
and dependency-first layers; mutually dependent routines share a layer.
Indirect calls and operating-system services still need explicit adapters.

The three translation labels mean:

- **Recovered:** control flow/actions are available; no Rust behavior is claimed.
- **Partial:** selected original branches have Rust implementations. This does
  not imply that the entire original routine has been translated.
- **SourcePort:** the routine's rule logic has a direct source-derived Rust port
  with explicit inputs/outputs replacing original globals and record selection.
  Unknown or unsafe input/service behavior may return a diagnostic. This label
  makes no claim of native comparison or desktop integration.

Comparison coverage is recorded separately. Existing native cases certify
their declared boundaries; a routine owning a tested action is not thereby
certified in full. Recovered blocks, metadata entries and declarations never
increase the count of implemented behavior.

The initial broad batch implements derived character statistics, upgrade
record mutations, strength/agility adjustments, attribute saving throws,
pooling/sharing credits, item charge totals, signed clamping, spell counts
and selection indexing. Statistics and credit actions use the shared atomic
command boundary. Upgrade point costs and eligibility belong to the native
upgrade caller and remain separate from its record mutation helper.

The following geometry/economy batch adds short/long map visibility sweeps,
body rectangles, item-cost scaling and ordered experience awards. Existing
line-of-sight sampling is reused. Geometry returns ordered writes through
explicit map/body adapters; its original window/caller integration remains
separate. Economy uses a qualified staged binary32 arithmetic model and
diagnoses floating-point conversion exceptions; SANE status is not yet modeled.

Source recovery also extends body-field restoration to the original signed-size
conditions and composes it into enemy destruction. The former size-zero-only
destruction gate is removed. This translates the record/map/order/target cleanup;
the destruction counter, centering and result-drawing service effects remain
separate recovery work. It does not certify the full destruction routine.

Additional batches translate the item and spell scratch loaders, target clearing,
candidate selection, resistance saves, displacement, range and closest-body
queries, mechanical-unit checks, layout selection, numeric and dialog text,
version strings, and weapon/missile preparation. Each exposes the original
record writes or presentation requests through explicit table/service inputs.
The native random-range helper and complete tile movement-cost lookup are used
by existing gameplay paths. Pack7 text conversion, visibility, layout reads,
and native drawing remain explicit service boundaries; source coverage does not
claim those services have been compared to the original.

The inventory batch supplies complete bank insertion, equipped-record sorting,
weight accumulation and token spending. It preserves the native record stride,
scratch counters and wrapping arithmetic. Timing helpers recover the unsigned
strict elapsed comparison, default-duration selection and mouse interruption
order through an explicit TickCount/Button adapter. The original exceptional
or nonterminating paths remain qualified diagnostics where supplied data or
service replies cannot establish an outcome.

Body actions now include placement, ground restoration/display, monster display
and complete destruction control flow, including the wrapping killed WORD and
ordered center/results calls. Their native LONG origins and presentation child
services are explicit inputs. Audio control includes queue retries, cycle
playback, resource lifetime, immediate commands and packed stereo volume; it
consumes observed service replies without changing context on unavailable data.
Attack preparation and damage calculation preserve scratch records, instance
adjustments, attack scaling and source write order. Native entry/frame values
that the original leaves uninitialized are required inputs. SANE uses the same
qualified staged binary32 arithmetic model as the economy batch; status and
nondefault rounding behavior remain parity work.

Further source batches cover filename construction, Finder metadata, profile
loading, preference resources and indexed-string replacement. Paths and strings
remain raw bytes; file and resource operations consume actual service
observations. Preference creation composes its save/load/open routines, and
seed bookkeeping uses the existing RNG without adding a registration gate.

Item installation now includes the complete recovered random-item child and
caller suffix. One speculative result preserves prefix writes, interleaved
random draws and child writes, and final inventory writes. Native addressed
memory retains aliases; unavailable spans or an exhausted draw budget diagnose
without committing memory or RNG. The separate pending-child API remains
available for caller inspection and does not itself certify completed gameplay.

Drawing and interface ports include tile/building copies, color icons, spell
lines, tier and credit displays, cursor conversion, window checks, button
bevels, light indicators and menu-bar control. Gamma routines preserve the
original raw-byte arithmetic, cache values, header quirks and linked-list
control through explicit device/trap observations. Their source coverage does
not certify graphics, devices or original handle lifetime, and desktop caller
integration remains separate.

Further batches translate complete visibility sampling and movement-record
bookkeeping, coordinate and shop displays, NPC menu reset/update, combat body
sprites, queued field drawing and cursor selection. The fade caller composes
the recovered gamma children. Toolbox arguments retain their Pascal push
order, and allocator/graphics observations include their actual memory and
register results; service names do not imply reconstructed OS behavior.

CODE 1 runtime helpers include relocation decoding, transfer-stack mutation,
unsigned LONG math, random scaling, pixel-handle maintenance and buffer copies.
Destructor-list traversal calls explicitly registered Rust handlers within the
candidate state. Unknown callback targets return an atomic diagnostic; generic
callback dispatch does not establish coverage of an unrecovered callback body.

The display families now include full pack/item/spell panels, dialog updates,
combat indicators, target and damage drawing, range/rectangle flashes and
clock-driven icon updates. They compose the recovered record, text, geometry,
drawing, filename and timing children. Toolbox/stdio/string observations and
SANE arithmetic qualifications remain explicit; this source coverage does not
establish original pixels or desktop caller integration.

Battle ordering retains raw caller-frame bytes, BYTE counters, placement
retries, adjacent comparison passes and final warrior positions. Warrior
verification preserves both the checksum writer/reader difference and the
ordered shadow-field copies. The two registration routines deliberately
select their recovered registered branch under the existing always-registered
policy; they do not implement the original rejected-registration restrictions.

Inventory eligibility, upgrades and equipment removal now compose their full
recovered record, profile, movement and warning children. Character statistics,
upgrade dialogs, restriction lists, specialty updates, item details and combat
odds retain the original record widths and caller-frame data. The shared warning,
button and message routines include their cleanup and event/timing branches.
Observed SANE, Toolbox and file services remain explicit boundaries; damage
calculation retains its qualified binary32 model. These source ports do not
establish native UI fidelity or complete application integration.

The complete map composition, combat panel and character panel parents now
execute their recovered drawing, visibility, record and damage children.
Equipment dropping, wear, replacement and refit compose their inventory,
eligibility, warning, movement and random bodies. Land, picture, door and
extra-code loading retain short-read data and explicit fatal-dialog control.
Native source quirks remain represented; bounded alias diagnostics and typed
child views are documented separately from native fidelity.

Combat view centering now includes both viewport sizes, capture overlays,
range drawing, actor cycling and capture exit. Character/portrait refresh,
shop and status panels compose their recovered children. Scenario storage,
bestiary/map menus and mission-name/input dialogs retain their full control
flow, observed file/Toolbox effects and source frame offsets. Captured drawing,
record and numeric child domains remain qualified separately from parent
raw-memory observation and native fidelity.

Damage resolution, resistance, result messages and spell/missile animation now
compose their recovered rule, character, rendering, timing and prompt children.
Inventory editing and creation preserve the native dispatch, eligibility and
installation order. Prompt/music/text control, world allocation, startup state,
logo display and order-form flow retain their observed external calls and native
caller storage. Spell/creature/survivor selection, character creation and level
selection, inventory/squad selection, character file operations and field
control now compose the recovered children with observed service boundaries.
Loot collection, item information and transfer, ammunition interaction,
character upgrades and viewing, party selection, generation, scenario
registration, confirmation dialogs, save and quit control now also compose
their recovered children. These fifteen source ports cover 14,631 exported
native instructions and preserve the registered-build policy. Test execution
uses one logical CPU and one test thread, below-normal priority, resident-memory
and elapsed-time limits; a resource violation fails the check.

Main-screen dispatch, menu handling and preferences, combat setup and turn advancement,
monster construction and casualty queues, spell resolution, target selection,
healing and condition decay now also have direct source-derived bodies. These
fifteen parents cover another 9,239 exported instructions and share one candidate,
call-depth limit and RNG budget. Concrete existing child adapters are connected;
remaining application children diagnose explicitly rather than return invented
results. Hidden native stack saves, complete child CPU outputs and some imported
arithmetic remain qualified projections. See
[application control evidence](../evidence/source-application-control-actions.md).

All 344 cataloged application routines now have provisional source-derived
Rust bodies; none remain partial or recovery-only in the source catalog.
The final twelve gameplay parents cover 23,391 of the 101,054 canonical exported
instructions. Complete source ownership does not establish a connected
application. All ninety previously identified direct child targets in that
final family are now connected through the shared candidate memory/RNG
boundary. Further dependencies remain, including two children of the newly
connected drawing routine. See
[gameplay control evidence](../evidence/source-gameplay-control-actions.md).
The compiled `items` dispatcher also includes a separately recovered
2,224-instruction continuation, qualified outside that immutable inventory.
Its 709-instruction canonical prefix is now a source port; the inventory
instruction total remains unchanged.
These counts describe source coverage, not native comparison or full desktop
and campaign integration.

Continue with dependency-related families, preserve widths and ordered effects,
document uncertain fields/services, and keep the existing regressions passing.
The full application and campaign remain under reconstruction.

Verification retains critical state transitions, save/replay integrity, atomic failures and independent native edge cases. Cosmetic traces, equality-only checks and duplicate test harnesses have been removed or consolidated. The default release workflow uses one integrated mission/save/replay gate; smaller route diagnostics remain available explicitly. The full original-fixture workspace passed before this test cleanup; changed feature, replay and desktop lifecycle tests were rerun afterward, alongside formatting, Python checks, Clippy and the release build.

The application-control batch added one critical queue transaction test, reusing
an existing observation fixture. It checks successful clearing and atomic failure
on exhausted processing limits or an unavailable reached child input. No
per-getter, constant-return, framework or cosmetic tests were added for this batch.

The complete Items source translation composes 45 distinct child targets across
191 call sites, including shared spell and main-screen execution. Its symbolic
entry saves retain known masks and diagnose reached unknown aliases. One new
critical test covers symbolic entry and bounded atomic failure; the queue
transaction case was rerun because the shared application boundary changed.
See [Items source evidence](../evidence/source-items-control-actions.md).

Startup, the outer event loop, menu initialization and scenario availability,
scenario selection/addition/removal, game loading, map display, warrior editing,
party loss, About, secret discovery and temple control now also have compiled
source-derived bodies. Their 4,768 native rows are lifted into fixed Rust control
blocks using the same known-register and symbolic-save model as Items. New
stack/register observations are limited to actual Toolbox and imported library
boundaries; application logic continues through recovered Rust bodies or a
reached diagnostic. Default native observations remain unavailable until an
adapter supplies actual outputs. These are source ports, not native UI parity.
See [system control evidence](../evidence/source-system-control-actions.md).

Attack, secondary attack, casting, enemy detection, collision, combat and combat
selection, encounter selection, monster movement, area transitions, buttons and
time advancement now have direct source-derived bodies. Their 4,022 fixed Rust
blocks retain every canonical native instruction, including the area opcode
tables. Registration opcodes continue with registered behavior; scenario
admission cannot require reading or entering a code. This batch adds no unit
tests for individual instruction translations or configuration. Its regression
scope reuses critical shared-boundary cases and the command/save/replay gate.

The shared child adapter batch also connects inventory and squad selection,
field and encounter control, text/dialogs, menus, loading/storage, combat display,
cursor/window, record and profile/filename children. Existing addressed Items
adapters are reused without entering Items. Native working registers that their
typed plans do not establish remain unavailable. This adds no routine-coverage
credit and does not establish complete application integration. See
[shared gameplay child evidence](../evidence/source-gameplay-child-adapters.md).

Thirty-two additional native bodies now retain 2,920 instructions in 426 fixed
Rust control blocks for spatial/range checks, body and monster handling, sound,
spell and missile helpers. Existing typed owners and reference cases remain in
the catalog. Indexed address scales recovered from native instructions correct
the earlier combat, movement, area and temple translations. These connections
bring the shared dispatcher to 220 entries; they do not establish a completed
application or native fidelity. See
[native gameplay leaf evidence](../evidence/source-gameplay-leaf-actions.md).
Expanded instruction compilation is split across the internal gameplay,
system and application libraries. The composition library supplies the complete
routine registry; the maintained runtime receives an explicit object-safe
resolver and service references. No third-party dependency was added for this
split. Expanded libraries use release optimization level 1 with incremental
compilation and 256 code-generation units, and have no unit-test or doctest
targets. Maintained runtime release settings remain unchanged.

The consolidated native-reference runner depends only on the maintained runtime
and content library. Separate execution-limit tests use four synthetic routines,
without the production registry. `tools/source_audit.py` enforces the project
ceiling of 25 percent test code relative to maintained runtime code, including
test helpers, optional native harnesses and test scripts. Expanded instruction
bodies, generated catalogs, fixture data and build/recovery code are excluded
from the production denominator. `tools/Build.ps1` runs this gate before compiling.

Campaign checks exercise functionality rather than unaided adventure completion.
The headless recorder keeps explicit test-only assistance receipts for health,
ammunition, Psi, positions, branch prerequisites and battle cleanup. Each route
is independently replayed, including saved continuations, and compares ordered
events, state and RNG. `-ExtendedScenarios` runs this check; local packages with
original inputs require it. Assistance does not establish acquisition, navigation,
combat calculations or native fidelity. Independent original-binary cases remain
the authority for their measured behavior. No assistance enters the desktop
runtime. Live presentation, audio and responsiveness require user observation.

The combat panel uses original pictures and native hit regions. Auto (including
per-warrior checkboxes), Delay, Escape, Undo, the weapon-name chooser and selected-
body portrait remain missing normal-combat entry points. The first four report
technical diagnostics without modifying the session; the latter two still lack
control bindings. Weapon-mode W and general inventory/Archives access are distinct
actions. Camera controls are
source-derived; inspected-body information and original range presentation still
require completion/comparison. The finite control audit is tracked alongside the
private completion checklist. Version 11 retains mechanical targeting; versions
1-10 initialize that context as inactive and cannot admit it when downgraded.
