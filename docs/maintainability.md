# Maintaining the runtime

The maintained runtime is the editing surface for game behavior. The Centurions
1.3.2 binary remains the behavioral authority. Native receipts, source-derived
implementations and live visual observations establish different kinds of
evidence; a structural refactor does not improve their qualification by itself.

## Start here

### Crates

- `crates/content` (`centurions-content`): opens the original installation and
  its resource forks, and exposes original records through borrowed views in
  `records`, plus graphics, audio, interface resources and mission data.
- `crates/core` (`centurions-core`): game rules and `Session` state. Gameplay
  changes enter through `Command` and `apply` and produce `Event`s. State is saved
  by `persistence` or imported from original saves by `original_save`.
- `crates/desktop` (`centurions-desktop`): the playable application. Screens,
  input, rendering and platform services (file dialogs, audio, windows) turn
  player actions into the same commands.
- `crates/tools` (`centurions-tools`): command-line `inspect` (the default),
  `replay`, `audit-resources`, `audit-import` and `convert-assets`; `--help`
  lists their arguments. `main.rs` routes actions; `replay.rs`,
  `convert_assets.rs` and `acceptance/` own the larger ones.

### Running from source

The desktop and tools read an existing New Centurions installation (see
[installation notes](installation.md) for its layout). Pass the folder as the
first argument, or set `CENTURIONS_ASSET_ROOT`; otherwise the executable's own
folder is searched, which is not an installation when running from `target/`:

```sh
cargo run -p centurions-desktop --release -- "<installation folder>"
cargo run -p centurions-tools -- inspect "<installation folder>"
cargo run -p centurions-tools -- replay "<installation folder>" commands.json --trace
```

### Reading order

1. This section, then [building and testing](building.md) for the checks and
   test layout.
2. The rest of this guide: ownership, the combat-click path and change guides.
3. [Source translation workflow](reconstruction.md) for how recovered routines
   become Rust, and [the reconstruction toolkit](toolkit.md) for comparisons
   and acceptance records.
4. In a development checkout, `docs/README.md` indexes the subsystem notes,
   the unsupported-behavior backlog and the capability map.

### Adding a feature

1. **Record view.** Read and write original fields through a view in
   `crates/content/src/records/`. Native offsets and integer widths stay there.
2. **Rule owner.** Put arithmetic, RNG draws and effect order in the subsystem
   owner listed below. A native global that outlives its caller belongs in
   `Globals` (`crates/core/src/state/globals.rs`).
3. **Command.** Reach the rule through a `Command` (`crates/core/src/command.rs`)
   dispatched by `commands` and entered through `apply`. An unsupported path
   returns a diagnostic without changing state. A new saved field or
   continuation raises `CURRENT_SCHEMA_VERSION` in `persistence`.
4. **Test.** Prefer a command/replay case in a `crates/core/tests/command_*.rs`
   file, using the helpers in `tests/support`. A new file is declared as a
   module of `tests/native_reference.rs` (see [test layout](building.md#test-layout)).
5. **Verify** with the commands listed in `AGENTS.md`:
   `python -m unittest discover -s tests/python`, `cargo fmt --check`,
   `cargo test --workspace --features original-fixtures`,
   `cargo clippy --workspace --all-targets --features original-fixtures -- -D warnings`
   and `cargo build --workspace --release`.

## Ownership and dependencies

| Responsibility | Owner | Calls into |
| --- | --- | --- |
| Command admission and transactions | core `commands` | maintained game subsystems |
| Native records and saved session representation | core `state` | content records and explicit defaults |
| Session validation | core `session` | subsystem validation |
| Save encoding, schema check and atomic writes | core `persistence` | session validation and filesystem |
| Click routing and occupied-cell continuations | core combat `interaction`, `occupied_cell` | record views, movement and shooting |
| Attack, reaction, turn and round execution | core combat `runtime` submodules | recovered arithmetic and record adapters |
| Spell selection, restrictions and effects | core combat `spells`, `player_psi` | packet, mask and native record rules |
| Shared spell body execution | combat `spells::support` | ordered resistance, field usage, damage, immediate conditions and health owners |
| Equipment and shops | core `equipment`, `shop` | record rules, inventory transactions and catalog |
| Exploration and scripts | core `exploration` | movement, choices, script execution and return ownership |
| Desktop screen lifecycle | desktop `application` submodules | the shared commands and screen-specific views |
| Rendering and hit rectangles | desktop `presentation`, `layout`, individual screens | original resources and input availability |
| Input and command presentation | desktop `input`, `targeting`, `view_actions`, `command_result` | the shared commands |
| Native dialogs, storage discovery, windows and audio | desktop platform services | operating-system and presentation services |

Desktop and replay both enter `apply`. The preserved reference layer is a
separate, private Cargo workspace under `archive/`, outside the public source:

- **`centurions-reference`** (`archive/crates/reference`) holds the recovered
  execution bodies (`recovered_*`) and the routine catalog (`reconstruction`). It
  depends on core; core and desktop do not depend on it.
- **The translated crates** (`archive/crates/translated*`) build on it. The
  composition crate combines the expanded translations in separate production
  libraries. Its compatibility entry points are not the ordinary gameplay interface.

Tests that compare against recovered code live in `archive/crates/reference/tests` and
reuse core's shared test helpers through `#[path]`. The archive workspace keeps the
reference and translated crates' own optimizer settings.

Public imports are forwarded from the original module paths. Internal helpers
shared by extracted sibling modules are visible only to their owning subtree.
Record algorithms use explicit imports so their dependencies can be read locally.

## Following a combat click

1. The canvas converts the window pointer to logical coordinates. The combat
   camera maps the logical map point to a tactical position. Capture and target
   selection use that same camera conversion.
2. Input produces an existing serialized command. The desktop classifies its
   original identity for gesture and presentation consequences.
3. `apply` resolves the content context, unwraps command preferences and validates
   the session in the established order. Combat click resolution is read-only.
4. Routing checks the actor and map location before capture, then firing-mode
   occupied targets, then a single movement step. Clicking the actor requests no
   action. A movement request during capture becomes a capture target.
5. Execution classification uses the normalized command. Admission and dispatch
   operate on a candidate session. Ordinary errors discard it; the existing
   `RefusedAfterEffects` variant validates and commits deliberate partial effects.
6. Successful targeting-only actions validate and commit immediately. Other
   actions process medical readiness, movement/script continuation, retained work
   context and message presentation in their original order before final commit.
7. Desktop command-result handling updates targeting and repeat state, prepares
   round notices, enqueues feedback, observes application events and plays sounds
   in their existing order. Failed automatic actions stop scheduling retries.

Native body values, coordinate arithmetic and record storage have not changed.
`CombatBodyId`, `CombatPosition` and borrowed warrior/enemy views provide names at
the interaction boundary. In particular, an active byte being nonzero and being
exactly one are distinct original checks. Unknown record bytes remain untouched.

## Changing a feature

### Record interpretation and semantic boundaries

Ordinary command handlers and the click-to-movement/firearm workflow use borrowed
views in `centurions_content::records`. For example, a movement-point check reads
`warrior.record().movement_points()`; it does not know the warrior byte offset.
Writes use `record_mut()` and affect only the original field. No view is saved,
and none replaces the original record bytes. Warrior and enemy durations retain
their different signed widths. Active flags remain values: a caller requiring
exactly one must not use another caller's nonzero test.

The following boundaries are intended editing surfaces:

| Change | Start here | Native interpretation |
| --- | --- | --- |
| Ground/capture/firearm click precedence | combat `interaction::resolve_click` | `CombatBodyId`, `CombatPosition` and record views |
| Occupied cells and movement | combat `movement`, `occupied_cell` | warrior/enemy views; named terrain properties |
| Firearm selection, cost and cancellation | combat `shooting`, runtime `shooting` | inventory views and `AttackPacket` |
| Reload and ammunition selection | `ammunition::refill`, `ammunition::fitting` | retained item/ammo/weapon definitions and inventory views |
| Firearm packet composition and resolution | combat `firearm::preparation`, `firearm::resolution` | `WeaponAttackParameters` and warrior/enemy views |
| Damage rolls, shields, armor and resistances | combat `damage` and `damage::defenses` | `DamageDefinition`, `DamageDefenses`, `DamageTargetRecord` |
| Player melee and reflection | combat `attack::warrior_enemy`, `warrior_attack`, `attack::melee_accuracy` | warrior/enemy conditions, weapon definitions and statistics |
| Target hit/damage estimates | combat `estimates`, item display `estimate` | read-only game calculations and original panel drawing commands |
| Actor choice and turn scheduling | runtime `enemy_actions`, `actor_turns`, `turns` | signed actor fields and native turn primitives |
| General enemy turn story return | runtime `turn_stories` | shared initial/advance callers, accepted chance prefix and deferred NPC initialization |
| NPC random weapon initialization | combat `monster_weapons` | recovered tables and caller-specific first/last range match |
| Scripted tactical spell origins | story spell `targets` | native caller parameter and represented position aliases |
| Equip/transfer/buy/sell | equipment `transactions`, shop `transactions` | `ItemDefinition`, `ShopRecord`, equipment `record_rules` |
| Enhanced shop stock | shop `generated_stock` | named instance modifiers and bounded RNG draws |
| Script operation and resumption | exploration `script_execution`, `ScriptState::restart_from` | named `Operation`, `StoryRecord`, operation-specific EXTRA views |
| Death and battle-round stories | runtime `death`, combat `round_stories` | the same `Operation` names, each interpreter's own subset |
| Globals that outlast a battle | state `Globals` | each field documents its global, reason and consumers |
| Encounter headers and shared labels | story `EncounterHeaders` | loader, file-copy, replacement, redraw and saveencounter adapters |
| Upgrade selection and drawing | desktop editor `controls`, warrior upgrade view | original DITL IDs and base/derived/purchase fields |
| Selected-battle reward windows | combat runtime `selected_rewards` | newland48's separate ordinary and additional loot returns |
| Save acceptance | `persistence::load` | `CURRENT_SCHEMA_VERSION`, then `Session::validate` |

The round interpreter keeps its specialized NPC and slot operations. At another
recognized story operation it delegates the current action and remaining script
to exploration's shared interpreter, just as the original round loop calls
`newland` in mode -1. `RoundDispatch` holds the suspended caller until all modal
pages and script replacements finish; returning advances the round slot once.
These suspended callers are saved.

The clinic introduction's pending entry is saved. Credits remain carried until
its original textbox returns.

Native arithmetic is still implemented by bounded routines such as firearm
resolution, turn initialization and equipment record rules. Their purpose is to
interpret original records and reproduce original arithmetic. They are not an
alternative application controller: keep input, screen state, command dispatch,
filesystem access and turn scheduling out of those adapters. Expanded translation
crates remain a separate recovery surface, not the normal place to edit a feature.

An uncertain field must keep an honest context-specific name. For example,
item word 32 (`ammunition_caliber`) is the caliber only for weapons; other
item classes give the word other meanings, so it is not a universal item
statistic. The owned-firearm health-context ID restriction is documented at its
caller and is not generalized by this refactor. Broadening such qualifications
requires a behavioral change with independent evidence.

### Following and changing a workflow

For a combat click, start with `Command::CombatClick` and follow normalization to
`ClickIntent`: no action, capture, fire at a body, or one movement step. Capture
comes first. An occupied firing-mode click selects a target; selection and attack
execution are separate. Movement then owns friendly exchange/confirmation,
occupied-cell reload, reaction processing, terrain cost and the final map update.
Firearm execution owns payment and ordered impacts. Cancellation restores the
native first inventory work record but deliberately does not refund the separate
load field. Do not replace that behavior with a seemingly cleaner full rollback.

Reload first refills existing magazines, then considers an empty magazine's
compatible ammunition. These are separate passes with different exact-stock and
reserve-debit behavior. `AmmunitionOutcome` names the result used by command and
occupied-cell callers; the historical integer return remains for reference
comparisons. Battery re-wearing uses `equipment::retained` and `equipment::modifiers`.
These operate on named `EquipmentStat` fields and distinguish the selected
inventory slot from the caller's retained modifiers. Removal is not an inverse
of wearing: modifier sources, clamps, condition limits and warning order differ.
The editor uses the same operations while preserving its level-check exception.
Creation's initial equipment installs only fields written by the original routine.
`item_modifiers` applies capacity and statistic changes in order, retaining partial
effects on a shield-efficiency refusal. `shop::generated_stock` owns enhancement
rolls and preserves the final fresh-install suffix. These paths no longer allocate
or execute native-address memory images.

Weapon preparation combines weapon and ammunition fields into an existing
attack template. It preserves byte narrowing, burst multiplication and float
rounding. Shared damage rules then apply rolls, armor filtering, shields, armor
and resistance in order. The record codec owns the original byte layout; those
rules operate on named values. Melee reflection retains caller-specific RNG
ordering and damage attribution. Do not unify these paths based on similarity.

Attack estimates are a different original algorithm from actual resolution.
They consume no RNG and change no session state. The desktop receives the same
visible drawing operations through `item_display::output::PanelPlan`, which also
carries the three retained definitions after calculation; they are never read
to execute rules. The old memory-based estimate is retained
only in the comparison harness and reference implementation. Passing that
comparison establishes reconstruction equivalence, not a new native receipt.

For a story, inspect the named operation before its numeric native identity. The
instruction cursor advances before execution. Branch replacement restarts the
instruction body while retaining its eight-byte header. A pause retains the
session's existing continuation; presentation does not own the script. Fatigue
percentage arithmetic divides before multiplying with word wrapping, even when
a more conventional percentage calculation would produce a different result.

For a save, `persistence::load` checks the size limit, parses the document,
refuses any version other than `CURRENT_SCHEMA_VERSION` with `unsupported save
version`, then runs `Session::validate` before the loaded session replaces the
current one. Saves from earlier reconstruction schemas are not migrated.
Original 1.3.2 game files are imported separately through `original_save`.

**Combat:** start at command routing, then the relevant combat owner. Attack
resolution, reactions, actor turns, round effects, medical work, shooting and
battle returns have separate implementations. Preserve integer widths and RNG
draw order. Do not merge apparently similar costs or active-state checks without
native evidence.

**Inventory:** desktop inventory selection owns members, scrolling, dragging and
detail caches. View and detail modules render that state. Equipment inspection,
record rules and transactions are separate. Shop access owns the retained caller;
the catalog supplies stock and prices; transactions commit purchases, sales and
reloads through existing commands.

**Stories:** the bounded interpreter advances the script cursor. Operation
adapters return Continue or Pause explicitly. Choices, typed menus, item use,
loot, battle entry and script return ownership have separate modules. The
derived script owner retains round-before-death precedence and introduces no
new saved fields. Do not infer exclusivity between stored contexts where the
original handlers did not enforce it.

**Screens:** use the application action entry point. Character dialogs, squad
selection and introductions own independent private state. Settings, information,
menus, input, rendering and combat notices have named handlers. Early handler
returns explicitly bypass the outer observer and preference writes exactly as
before extraction. A common ancestor must not become the implementation site for
each child screen.

**Saves:** keep serialized field/variant names, defaults and order stable during a
structural refactor. Only `CURRENT_SCHEMA_VERSION` loads; earlier reconstruction
saves are refused rather than migrated. Newly required persistence belongs in a
separate feature change that raises that constant. `Globals` is
flattened into `Session`, so its saved keys keep their names. A loaded session
is validated before replacement; original assets and imported saves remain
read-only.

Some saved fields need context:

- `party_mask` and `health_context` live in `Globals`.
- `random_battles` is BYTE402f42.
- `saved_position` (opcode 70) is omitted while zero.
- The script fields `caller_action` (newland's encounter-caller action, used by
  opcode 34) and `chained` (an event started by opcode 20 keeps the party in
  place) are omitted while unset.
- `coordinates_hidden` (WORD4354fe, opcode 71) is omitted while zero and is read
  from original saves.
- A running encounter's `eliminated` branches (opcode 44) are omitted while
  zero. The multi-member `PartyPick` presentation serves opcodes 14 and -14.
- The camping flag (WORD4394b4) is deliberately not saved, as in the original
  save, so a loaded session allows camping.

**Controls:** drawing and hit testing consume the same Control records and
original resource rectangles. Mouse and keyboard eligibility remains in the
existing shared policy. Preserve explicit modal precedence and deliberate
differences between gestures. Operating-system file panels and audio services
remain outside rule execution.

### Retained work ownership

Native work globals that outlive a caller have one owner each. A missing value
is unknown history; do not fabricate it from a default, the current roster or a
parent's copy.

- **Turn story returns:** saved NPC turn frames distinguish initial entry from
  advancement; initial entry suppresses the extra `TurnStarted` that advancement
  emits. Saved item selections admit the original signed item mode and WORD power
  domain.
- **Health work:** `story_health` and `Globals`. Fresh work copies
  initialized DATA. Session consumers use the fallible retained accessor; the
  public record-only helper keeps its existing default contract. A command
  epilogue may refresh known work but must not turn absent work into a
  fabricated origin.
- **Calendar recovery:** food consumption and noon/midnight recovery belong to
  `time/recovery.rs`; clock and schedule routing stay in `time.rs`. Recovery
  emits deterministic original status feedback and returns immediately after
  mission defeat. Keep native presentation addresses in the feedback owner and
  preserve heal/partyloss effect order.
- **Zero-time calendar returns:** `time::zero` owns them. The parent cursor and
  locals survive while its instruction record is replaced by the final shared
  child DOOR. The ordinary positive-time recovery continuation stays separate.
- **Ready:** `Globals::in_spell` records supported outdoor writers;
  unrecovered tactical histories are unknown. While combat is live,
  `Combat::ready` owns WORD438760. Session-level writers use `set_ready`;
  record, turn and cast owners update the live copy at their recovered write
  boundaries. Teardown carries that global value instead of a parent snapshot.
  Presentation reads the same admission as replay.
- **Death scripts:** they run in the shared exploration interpreter;
  `runtime::death` owns their queue and return frames. A death caller saves only
  DOOR actions and parameters, so its suspended record follows the child's
  shared header and level byte. Keep the loop and its early and normal return
  retention points together.
- **Death processing bytes:** the literal queue and callback bytes belong to
  `Globals::death_processing`, not to the lifetime of `DeathStory` or
  `DeathQueue::processing`. New queue calls and resumed local scans have
  different writer boundaries. `story_combat` reads the actual queue byte before
  selecting a live or retained origin; keep missing history explicit.
- **Death callback refresh:** `runtime::death_return` owns it. Its rule-side item
  loads and firing-mode correction read current initiative, not the saved
  attacker. `combat::retained::InitiativeWork` holds the native order and cursor
  after teardown; a live `Combat` owns those WORDs while present. Keep
  compatibility-only record helpers separate from Session callers that have
  literal processing and Ready work.
- **Neighbors:** shared neighbor rows and the body-list/party-mask alias
  belong to `combat::neighbor_scan`. The caller owns its local loop count and
  cursor; `Globals` keeps them through teardown. Session
  entry points supply the selected warrior and shared mask explicitly.
  Record-only helpers keep their existing contracts. Do not reconstruct a
  before-move row from the current map or restore a parent copy after a child
  call.

## Code hygiene

### Code style

The code should read as the game's own source would: written by someone who
designed the rules, not by someone describing a binary.

- **Names.** Use the original author's vocabulary from the [glossary](glossary.md):
  Realmz names for shared concepts (`stamina`, `stamina_max`, `battle_order`,
  `killbody`, `getup`, `booty`), adopted only where the 1.3.2 binary confirms the
  meaning, and plain game terms for Centurions-only concepts. Avoid
  engine-emulation words such as "native", "legacy", "WORD", "alias" or "work"
  in identifiers when a game term exists. Serialized names stay fixed; rename
  through `#[serde(rename = "...")]`.
- **Comments.** State the rule and its intent in the present tense: what the game
  does and why. Don't narrate how it was recovered. Use comments sparingly; a
  well-named function rarely needs more than a sentence.
- **Provenance.** A function that follows an original routine ends its doc comment
  with one line, `Original: <routine>` (for example `Original: getbattleorder`).
  Addresses, WORD/BYTE globals, register notes and decompilation detail belong in
  the subsystem notes under `docs/`, in `evidence/` and in fixture provenance, not
  in code.
- **Deliberate oddities.** When the game does something surprising on purpose, or
  the reconstruction keeps an original quirk, say so plainly and point to the note
  that records the evidence, for example: "A warrior still on automatic keeps an
  initiative slot (see docs/enemy-turn-dispatch.md)."

Before:

```rust
// combatsetup counts each newly created nonzero-faction NPC; getbattleorder
// then clears the defeated WORD. This is an entry writer, not a live recount.
```

After:

```rust
/// Counts the monsters that join the battle; the battle order starts with none
/// defeated.
///
/// Original: combatsetup
```

### Replacing execution machinery

Ordinary attacks and reloads use game values and record codecs. The primary
owners are ammunition `selection`, `refill`, `fitting`; firearm `preparation`,
`resolution`, `reload`; attack `melee_accuracy`, `warrior_enemy`, `natural`;
enemy-weapons `warrior_melee`, `enemy_melee`; and natural-condition
`attack_effects`. The allied natural caller shares the accuracy rules while
retaining its own death, sound and retained-actor ordering.

Equipment `retained` owns replacement and silent wear/removal; `modifiers`
applies ordered stat changes. Creation, item upgrading and enhanced shop stock
do not construct emulated memory. Do not merge their instance-copy rules:
creation deliberately leaves bytes 11/12 intact, and generated shop stock copies
only the original installitem suffix, not the generated ammunition or quantity.

Character information and inventory row/detail presentation calculate fields
directly, then emit original text and drawing operations. Inventory panels and
estimates emit `item_display::output` effects; `recovered_detail_ui::legacy_plan`
and the effect conversions express them as `DetailPlan` and `ItemUiEffect` for
reference comparison, including the three address-labelled definition receipts.
Their address-labelled numeric receipts are not a memory executor. Private
scratch writes and simulated Toolbox observations are not part of the desktop
rendering contract.

The expanded translations and recovered execution bodies remain available for
reference comparisons. Do not make ordinary commands depend on them. Random
ranges and movement cost live in `rules`; shop price rounding lives in
`shop/pricing`; ordered credit payment lives in `story_inventory/payment`.
Historical adapters preserve their reference API and receipt representation.

Creation and promotion use borrowed race/profession profiles and development
fields. Party filtering and scripted inventory changes share record views with
ordinary commands while retaining their caller-specific rules. Spell fields
remain the same saved 20-byte records; `records/spell_field` owns their signed
coordinates, duration and bounds. `medical/preparation` composes weapon and
ammunition parameters without CPU registers or a simulated address space.

Use `records/warrior`, `monster`, `development`, `equipment`, `inventory_slots`
and `attack` for stored fields. Rules own arithmetic, RNG and ordering; views
only expose the original representation. Do not eagerly decode a whole record
when a caller's narrow read determines its validation order. For example, story
selection deliberately reads one slot beyond the inventory count. The fixed
29-slot drop shift is different from ordinary occupied-slot removal. Outdoor
and collision spell callers consume duration draws differently. Do not merge
these distinctions as a cleanup.

A change to combat should start at `combat/interaction` and the command
transaction, then follow the selected attack/reload owner and its ordered events.
For a story item, start at `exploration/item_use`, then `story_item_use` or
`story_inventory`, and follow the existing script continuation. For character
creation, follow `creation` through the profile views and initial inventory;
for promotion, follow `story_promotion` and `story_promotion/scans`. Session and
save types are still the persistence authority; none of these views is serialized.

Source-level completion and comparison evidence are reconciled in
`evidence/semantic-gameplay.md`. Original unknown fields, the qualified Hive
policy and live acceptance retain their existing qualifications.

- Give each maintained module one game or presentation responsibility. Review
  orchestration modules approaching 600 physical lines; split by responsibility,
  rather than by arbitrary line ranges. Keep native record algorithms bounded by
  their original routine or transaction.
- A facade owns contracts and routing. Avoid placing arithmetic, rendering,
  filesystem handling and campaign rules into the same controller.
- Use named imports and small boundary types. Introduce abstractions only where
  they replace repeated knowledge; avoid a universal engine context or generic
  dialog framework.
- Keep native offsets inside documented record views and native record/rule
  adapters. Recovered compatibility bodies and mechanically expanded routines
  retain their source shape; moving their callers does not turn them into
  idiomatic domain code.
- Retain native address/resource comments where they explain a behavioral
  constraint. Use maintainer-facing names for ordinary workflow code.
- Preserve independent native cases. Reuse command/replay groups for critical
  changes and keep all test helpers under test directories. Count verification
  code under the maintained or reference layer it serves (see `docs/building.md`).
  Maintained test lines (excluding native-evidence comparisons) are ratcheted and
  may not grow; their ratio against the 25% target is reported.

## Diagnosing a failure

Replay the commands that reach the failure with the opt-in trace:

```sh
centurions-tools replay . commands.json --from save.json --trace
```

- `--from` starts from a saved session, so a player's save can be replayed
  directly. Without it, replay starts from a new game.
- `--save OUTPUT` writes a versioned save through the production save writer.
  It also saves the actual valid state at a rejected command boundary, including
  deliberate partial effects, while retaining the nonzero process exit. With
  this option that rejected state is the last stdout JSON line. Without it,
  existing rejection output is unchanged. JSON parsing and input-save loading
  failures create no output save; missing or duplicate output arguments and
  filesystem write failures remain errors.
- Each command prints its trace entries, then its events. The final session
  follows as JSON. Trace entries are:
  - `draw`: every RNG draw, labelled with the call site, its bound or range, and
    the result;
  - `story`: story instructions, with interpreter, script, cursor and opcode;
  - `damage`: damage channels after rolling and after every defense;
  - `declined`: original-behavior returns that produced nothing.
- Errors carry context. Unsupported story operations name the script, cursor and
  opcode; occupied-cell refusals name the failing predicate.

The three examples below were traced from a saved state of the first Thunder
battle (assisted campaign route 12/0, before command 10). Two of them use
constructed states, which are labelled. Call-site line numbers change as code
moves; follow the function names.

**Occupied-cell reload refusal.** Warrior 4 is in firing mode beside the enemy,
with an empty magazine and no reserves. It moves into the enemy's cell.

- `Command::Move` reaches `combat::movement::move_warrior`. Moving into an
  opposing occupied cell in firing mode calls
  `occupied_cell::reload_at_occupied_cell`.
- Eligibility failures there return `Unsupported` with the context
  `occupied cell {target}: {predicate}`.
- An eligible warrior with rounds left prepares its burst with
  `firearm::prepare`. Every eligible warrior then refills through
  `ammunition::refit_with_record`.
- When nothing can be loaded, the original refusal is `Event::Sound { ERROR_SOUND }`
  then `Event::CollectionRejected { OUT_OF_AMMUNITION_WARNING }`, with automatic
  mode cleared. The trace shows no draws: the refusal consumes no RNG.

```text
[{"Sound":{"resource_id":6000}},{"CollectionRejected":{"warning":25}}]
```

**An enemy firearm damage number.** This is a constructed state: the battle's
natural attacker is replaced by Thunder MD 7 (firearm 70), keeping the replaced
enemy's 26/26 health because battle setup normally rolls it. After five
`EndTurn`s, `RunEnemyTurn` acts for body 10.

- `runtime::actor_turns` passes `globals.loaded_item_field32` as the
  work-item context. That is the last item a *player* loaded, and
  `enemy_weapons::ammo_id` uses it to choose the enemy's ammunition.
- `runtime::enemy_actions` draws against `firearm_chance` (`enemy_actions.rs`
  draw, result 13 against bound 100) and calls `runtime::enemy_fire`.
- `spells::selection` draws the target area. For each of the three shots,
  `enemy_fire` and `firearm::resolution` draw the shot's rolls.
- `damage::resolve_damage_with_amount` runs `roll_damage`, `filter_armor`,
  `absorb_shield`, `absorb_armor` and `apply_resistances`. Its `damage` entry
  shows every channel after rolling and after defenses. In this replay all of
  them are zero.

```text
{"kind":"draw","site":"crates/core/src/combat/runtime/enemy_actions.rs:131","call":{"bound":100},"result":13}
{"kind":"draw","site":"crates/core/src/combat/spells/selection.rs:263","call":{"bound":7},"result":7}
{"kind":"draw","site":"crates/core/src/combat/spells/selection.rs:371","call":{"bound":11},"result":9}
...
{"kind":"draw","site":"crates/core/src/combat/runtime/enemy_fire.rs:186","call":{"range":[0,0]},"result":0}
{"kind":"draw","site":"crates/core/src/combat/firearm/resolution.rs:33","call":{"bound":7},"result":6}
{"kind":"draw","site":"crates/core/src/combat/firearm/resolution.rs:40","call":{"bound":100},"result":77}
...
{"kind":"damage","rolled":[0,0,0,0,0,0,0,0,0],"components":[0,0,0,0,0,0,0,0,0],"total":0}
[{"Sound":{"resource_id":390}},{"AttackResolved":{"attacker":10,"target":2,"damage":0,"defeated":false}}, ...]
```

**A Continue that does nothing.** This is a constructed state: messages are
pending during the combat phase.

- `Command::Continue` (`commands/world.rs`) checks its owners in a fixed order:
  1. a pending round program (`runtime::continue_round`);
  2. loot;
  3. survivors;
  4. messages (`exploration::continue_messages`);
  5. a round dispatch that owns the script;
  6. a waiting death story (`runtime::continue_death`);
  7. otherwise `exploration::continue_story`.
- `continue_messages` presents the next page. When the pages run out during
  combat, it returns nothing, as the original does. The trace names the owner
  that declined.

```text
{"kind":"declined","site":"exploration.continue_messages","owner":"combat","reason":"messages ended during combat"}
[]
```

## Local acceptance

The [reconstruction toolkit](toolkit.md) combines native/source comparison,
bounded probe execution, validation receipts, scenario/backlog navigation and
build-bound acceptance records. Generated reports remain private under analysis;
tool output does not change the distinction between source, native and observed
acceptance evidence.

Headless comparisons cover state, RNG, ordered events, save/load continuations
and handler behavior. The following checks require a user-operated desktop:

- Click ground, nearby and distant actors in movement and firing modes; try a
  blocked or ammunition-limited action and verify original feedback.
- Enter and cancel capture, firearm and Psi targeting; change target quantity
  and return to ordinary movement.
- Dismiss round/story feedback and confirm that movement and the next turn resume.
- Open inventory, transfer/equip an item, close it and return to the prior screen.
- Save a named file, reload it and resume a pending selection where permitted.
- Open character information, creation/editor and preferences; cancel and confirm
  their child dialogs, then return to gameplay.
- Check native file panels, Alt+Enter, held movement, music and original visuals on
  each available platform. Compilation alone does not close live acceptance.
- Changes in this cycle to check live:
  - error messages reworded for equipment wear and generated stock;
  - heat, cold, psionic and atomic natural attacks against warriors;
  - the desktop `--audit-log` trace output.
