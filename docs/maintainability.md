# Maintaining the runtime

The maintained runtime is the editing surface for game behavior. The Centurions
1.3.2 binary remains the behavioral authority. Native receipts, source-derived
implementations and live visual observations establish different kinds of
evidence; a structural refactor does not improve their qualification by itself.

## Ownership and dependencies

| Responsibility | Owner | Calls into |
| --- | --- | --- |
| Command admission and transactions | core `commands` | maintained game subsystems |
| Native records and saved session representation | core `state` | content records and explicit defaults |
| Session validation | core `session` | subsystem validation |
| Save encoding, migrations and atomic writes | core `persistence` | session validation and filesystem |
| Click routing and occupied-cell continuations | core combat `interaction`, `occupied_cell` | record views, movement and shooting |
| Attack, reaction, turn and round execution | core combat `runtime` submodules | recovered arithmetic and record adapters |
| Spell selection, restrictions and effects | core combat `spells`, `player_psi` | packet, mask and native record rules |
| Equipment and shops | core `equipment`, `shop` | record rules, inventory transactions and catalog |
| Exploration and scripts | core `exploration` | movement, choices, script execution and return ownership |
| Desktop screen lifecycle | desktop `application` submodules | the shared commands and screen-specific views |
| Rendering and hit rectangles | desktop `presentation`, `layout`, individual screens | original resources and input availability |
| Input and command presentation | desktop `input`, `targeting`, `view_actions`, `command_result` | the shared commands |
| Native dialogs, storage discovery, windows and audio | desktop platform services | operating-system and presentation services |

Desktop and replay both enter `apply`. Core and desktop do not depend on the
complete translated registry. The composition crate combines expanded
translations in separate production libraries; its compatibility entry points
are not the ordinary gameplay interface. Their optimizer settings remain separate.

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
| Equip/transfer/buy/sell | equipment `transactions`, shop `transactions` | `ItemDefinition`, `ShopRecord`, equipment `record_rules` |
| Enhanced shop stock | shop `generated_stock` | named instance modifiers and bounded RNG draws |
| Script operation and resumption | exploration `script_execution`, `ScriptState::restart_from` | named `Operation`, `StoryRecord`, operation-specific EXTRA views |
| Upgrade selection and drawing | desktop editor `controls`, warrior upgrade view | original DITL IDs and base/derived/purchase fields |
| Save acceptance and restoration | persistence `compatibility::restore` | ordered compatibility steps for versions 1-12 |

Native arithmetic is still implemented by bounded routines such as firearm
resolution, turn initialization and equipment record rules. Their purpose is to
interpret original records and reproduce original arithmetic. They are not an
alternative application controller: keep input, screen state, command dispatch,
filesystem access and turn scheduling out of those adapters. Expanded translation
crates remain a separate recovery surface, not the normal place to edit a feature.

An uncertain field must keep an honest context-specific name. For example,
`retained_field32` is category-dependent native work data, not a universal item
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
visible drawing operations through the existing `DetailPlan` interface; its
estimate memory contains only compatibility receipts of the three retained
definitions, encoded after calculation; it is never read to execute rules. The old memory-based estimate is retained
only in the comparison harness and reference implementation. Passing that
comparison establishes reconstruction equivalence, not a new native receipt.

For a story, inspect the named operation before its numeric native identity. The
instruction cursor advances before execution. Branch replacement restarts the
instruction body while retaining its eight-byte header. A pause retains the
session's existing continuation; presentation does not own the script. Fatigue
percentage arithmetic divides before multiplying with word wrapping, even when
a more conventional percentage calculation would produce a different result.

For a save, read the compatibility pipeline in order: version/schedule, control
contexts, nested story contexts, actor defaults, campaign contexts, then map and
spell-memory restoration. Rejection and restoration are interleaved intentionally;
moving all defaults ahead of rejection would change which legacy files are accepted.

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
structural refactor. The structural refactor retained schema 12 and readers for 1–11. A subsequent
original-instruction correction introduced schema 13 for the retained natural-attack
index, with readers for 1–12 and explicit unknown legacy context. Preserve that
versioned boundary and the existing validation/rejection rules. Newly required persistence belongs in a separate, explicitly
versioned feature change. A loaded session is validated before replacement;
original assets and imported saves remain read-only.

**Controls:** drawing and hit testing consume the same Control records and
original resource rectangles. Mouse and keyboard eligibility remains in the
existing shared policy. Preserve explicit modal precedence and deliberate
differences between gestures. Operating-system file panels and audio services
remain outside rule execution.

## Code hygiene

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
directly, then emit original text and drawing operations. `DetailPlan` and
`ItemUiEffect` remain compatibility output types. Their address-labelled numeric
receipts are not a memory executor. Attack estimates additionally retain three
definition receipts for existing consumers. Private scratch writes and simulated
Toolbox observations are not part of the desktop rendering contract.

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
  code in the 25% ceiling, including when production code becomes smaller.

## Local acceptance

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
