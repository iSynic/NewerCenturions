"""Refresh the bounded action inventory from checked-in instruction fixtures."""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

MISSION_ROWS = [
    ('encounter-choice', 'story.encounter_choice', 'VerifyEncounterChoice', 'story', ['story.dialogue'], True),
    ('story-flags', 'story.flag_logic', 'VerifyStoryFlags', 'story', ['story.replace_script'], True),
    ('story-inventory', 'story.inventory_persistent', 'VerifyStoryInventory', 'story_inventory', ['inventory.initialize_plain', 'inventory.store'], True),
    ('map-actions', 'story.mutable_map', 'VerifyMapActions', 'travel', ['story.replace_script', 'story.flag_logic'], True),
    ('travel', 'travel.map_state', 'VerifyTravel', 'lib', ['foundation.initialize'], True),
    ('stage-travel', 'mission.stage_change', 'VerifyStageTravel', 'mission', ['travel.map_state', 'story.prologue'], True),
    ('typed-encounter', 'story.typed_encounter', 'VerifyTypedEncounter', 'typed_encounter', ['story.encounter_choice', 'story.replace_script'], False),
    ('manual-encounter', 'story.manual_encounter', 'VerifyManualEncounter', 'story_manual_encounter', ['story.typed_encounter', 'story.replace_script', 'travel.map_state'], False),
    ('map-picture', 'story.map_picture', 'VerifyMapPicture', 'story_map_picture', ['travel.map_state', 'story.replace_script'], False),
    ('story-delay', 'story.delay', 'VerifyStoryDelay', 'story_delay', ['story.replace_script'], False),
    ('schedule-edit', 'story.schedule_edit', 'VerifyScheduleEdit', 'story_schedule', ['time.calendar', 'story.replace_script'], False),
    ('story-variables', 'story.variables', 'VerifyStoryVariables', 'story_variables', ['story.flag_logic', 'story.inventory_persistent'], False),
    ('loot-collection', 'story.item_collection', 'VerifyLootCollection', 'story_collection', ['inventory.initialize_plain', 'combat.movement_capacity'], True),
    ('story-allies', 'story.add_ally', 'VerifyStoryAllies', 'story_allies', ['foundation.random'], False),
    ('ally-combat', 'combat.allies', 'VerifyAllyCombat', 'combat/allies', ['story.add_ally', 'combat.map', 'combat.enemy_placement', 'combat.victory'], False),
    ('header-destination', 'travel.door_destination', 'VerifyHeaderDestination', 'story_destination', ['travel.map_state', 'story.complete_once'], False),
    ('story-skill', 'story.skill_choice', 'VerifyStorySkill', 'story_skill', ['story.replace_script'], False),
    ('genetic-enemies', 'combat.genetic_enemies', 'VerifyGeneticEnemies', 'combat', ['combat.enemy_placement'], False),
    ('genetic-victory', 'combat.genetic_rewards', 'VerifyGeneticVictory', 'combat/runtime', ['combat.genetic_enemies', 'combat.victory', 'story.item_collection'], False),
    ('encounter-battle', 'story.battle_selection', 'VerifyEncounterBattle', 'combat', ['foundation.random', 'story.sound', 'story.dialogue'], False),
    ('battle-entry', 'combat.scenario_entry', 'VerifyBattleEntry', 'combat', ['story.battle_selection', 'combat.map', 'combat.squad_formation', 'combat.genetic_enemies', 'combat.initiative'], False),
    ('ally-turns', 'combat.scientist_turns', 'VerifyAllyTurnsAudio', 'combat/ally_turns', ['combat.allies', 'combat.attack.natural', 'combat.damage'], False),
    ('ally-victory', 'combat.ally_rewards', 'VerifyAllyVictory', 'combat/runtime', ['combat.allies', 'combat.victory'], False),
    ('natural-slow', 'combat.natural_slow', 'VerifyNaturalSlow', 'combat/attack', ['combat.attack.natural', 'combat.damage'], False),
    ('story-party', 'story.party_selection', 'VerifyStoryParty', 'story_party', ['story.battle_selection', 'combat.initiative'], False),
    ('item-use', 'inventory.use_scenario_item', 'VerifyStoryItemUse', 'story_item_use', ['story.replace_script', 'inventory.store'], False),
    ('story-health', 'story.outdoor_health', 'VerifyStoryHealth', 'story_health', ['foundation.random', 'story.sound'], False),
    ('story-reduce', 'survival.outdoor_reduce', 'VerifyStoryReduce', 'story_reduce', ['story.outdoor_health'], False),
    ('treasure-rewards', 'story.treasure_rewards', 'VerifyTreasureRewards', 'story_treasure', ['inventory.initialize_plain', 'story.inventory_persistent'], False),
    ('move-away', 'travel.move_away', 'VerifyMoveAway', 'story_move_away', ['exploration.hour'], False),
    ('registration-caller', 'story.registration_gate', 'VerifyRegistrationCaller', 'story_registration', ['travel.move_away'], False),
    ('large-placement', 'combat.large_placement', 'VerifyLargePlacement', 'combat/footprint', ['combat.enemy_placement'], False),
    ('large-body-move', 'combat.large_movement', 'VerifyLargeBodyMove', 'combat/footprint', ['combat.enemy_move', 'combat.large_placement'], False),
    ('required-natural', 'combat.natural_disease', 'VerifyRequiredNatural', 'combat/natural_conditions', ['combat.attack.natural', 'combat.damage'], False),
    ('disease-tick', 'combat.disease_tick', 'VerifyDiseaseTick', 'combat/natural_conditions', ['combat.natural_disease'], False),
    ('disease-round', 'combat.disease_round', 'VerifyDiseaseRound', 'combat/runtime', ['combat.disease_tick', 'combat.next_round'], False),
    ('outdoor-disease', 'survival.outdoor_disease', 'VerifyOutdoorDisease', 'story_reduce', ['survival.outdoor_reduce', 'combat.natural_disease'], False),
    ('inactive-initiative', 'combat.inactive_initiative', 'VerifyInactiveInitiative', 'combat/entry_party', ['combat.initiative'], False),
    ('inactive-battle-entry', 'combat.inactive_entry', 'VerifyInactiveBattleEntry', 'combat', ['combat.inactive_initiative', 'combat.scenario_entry'], False),
    ('inactive-battle-return', 'combat.inactive_return', 'VerifyInactiveBattleReturn', 'combat/runtime', ['combat.inactive_entry', 'combat.victory'], False),
    ('required-battle-entry', 'combat.required_large_entry', 'VerifyRequiredBattleEntry', 'combat', ['combat.large_placement', 'combat.inactive_entry'], False),
    ('death-scripts', 'combat.queued_death_stories', 'VerifyDeathScripts', 'combat/death_scripts', ['combat.death_cleanup'], False),
    ('nested-victory', 'combat.nested_rewards', 'VerifyNestedVictory', 'combat/runtime', ['combat.queued_death_stories', 'combat.victory'], False),
    ('nested-caller', 'combat.nested_caller', 'VerifyNestedCaller', 'combat/runtime', ['combat.nested_rewards', 'combat.queued_death_stories'], False),
    ('npc-battle-caller', 'story.npc_battle', 'VerifyNpcBattleCaller', 'story_npc_battle', ['combat.scenario_entry', 'combat.victory'], False),
    ('terminal-stage', 'mission.terminal_transition', 'VerifyTerminalStage', 'mission', ['mission.stage_change', 'travel.map_state'], False),
    ('battle-rewards', 'combat.scenario_rewards', 'VerifyBattleRewards', 'combat/rewards', ['combat.victory', 'story.item_collection'], False),
    ('booty-availability', 'combat.reward_availability', 'VerifyBootyAvailability', 'combat/rewards', ['combat.scenario_rewards', 'story.item_collection'], False),
    ('level-promotion', 'party.level_promotion', 'VerifyLevelPromotion', 'story_promotion', ['combat.scenario_rewards'], False),
    ('required-enemy-weapons', 'combat.required_enemy_weapons', 'VerifyRequiredEnemyWeapons', 'combat/enemy_weapons', ['combat.damage'], False),
    ('required-melee', 'combat.required_melee', 'VerifyRequiredMelee', 'combat/enemy_weapons', ['combat.required_enemy_weapons'], False),
    ('required-weapon-fallback', 'combat.weapon_fallback', 'VerifyRequiredWeaponFallback', 'combat/attack', ['combat.attack.natural'], False),
    ('enemy-spell-selection', 'combat.enemy_spell_selection', 'VerifyEnemySpells', 'combat/spells', ['foundation.random'], False),
    ('enemy-spell-damage', 'combat.enemy_spell_damage', 'VerifyEnemySpells', 'combat/spells', ['combat.damage'], False),
    ('spell-targets', 'combat.spell_targets', 'VerifySpellTargets', 'combat/spells', ['combat.enemy_spell_selection'], False),
    ('firearm-targets', 'combat.firearm_targets', 'VerifyFirearmTargets', 'combat/spells', ['combat.spell_targets'], False),
    ('spell-bodies', 'combat.spell_bodies', 'VerifySpellBodies', 'combat/spells', ['combat.enemy_spell_damage'], False),
    ('enemy-firearm-bodies', 'combat.enemy_firearm_bodies', 'VerifyEnemyFirearmBodies', 'combat/spells', ['combat.required_enemy_weapons'], False),
    ('spell-conditions', 'combat.spell_condition36', 'VerifySpellConditions', 'combat/spell_conditions', ['combat.spell_targets'], False),
    ('additional-spells', 'combat.additional_spells', 'VerifyAdditionalSpells', 'combat/spells', ['combat.enemy_spell_damage', 'combat.spell_targets'], False),
    ('condition-targets', 'combat.condition_targets', 'VerifyConditionTargets', 'combat/spells', ['combat.spell_condition36'], False),
    ('spell-fields', 'combat.spell_fields', 'VerifySpellFields', 'combat/spells', ['combat.spell_targets', 'combat.next_round'], False),
    ('natural-poison', 'combat.natural_poison', 'VerifyNaturalPoison', 'combat/natural_conditions', ['combat.attack.natural', 'combat.damage'], False),
    ('outdoor-poison', 'survival.outdoor_poison', 'VerifyOutdoorPoison', 'story_reduce', ['combat.natural_poison', 'survival.outdoor_reduce'], False),
    ('poison-round', 'combat.poison_round', 'VerifyPoisonRound', 'combat/runtime', ['combat.natural_poison', 'combat.next_round'], False),
    ('final-enemy-weapons', 'combat.final_enemy_weapons', 'VerifyFinalEnemyWeapons', 'combat/enemy_weapons', ['combat.required_enemy_weapons'], False),
    ('enemy-fire-caller', 'combat.enemy_fire_caller', 'VerifyEnemyFireCallerAudio', 'combat/runtime', ['combat.final_enemy_weapons', 'combat.firearm_targets', 'combat.spell_fields'], False),
    ('final-turn-stories', 'combat.final_turn_stories', 'VerifyFinalTurnStories', 'combat/final_turn_stories', ['combat.enemy_placement', 'combat.queued_death_stories'], False),
    ('genetic-melee', 'combat.genetic_melee', 'VerifyGeneticMelee', 'combat/enemy_weapons', ['combat.required_melee', 'combat.damage'], False),
    ('warrior-opportunity', 'combat.warrior_opportunity', 'VerifyWarriorOpportunity', 'combat', ['combat.walk', 'combat.genetic_melee', 'combat.attack_packets'], False),
    ('map-seven-weapons', 'combat.map_seven_weapons', 'VerifyMapSevenWeapons', 'combat/enemy_weapons', ['combat.final_enemy_weapons', 'combat.damage'], False),
    ('map-seven-fallback', 'combat.map_seven_fallback', 'VerifyMapSevenFallback', 'combat/attack', ['combat.weapon_fallback'], False),
    ('map-seven-fire-caller', 'combat.map_seven_fire_caller', 'VerifyMapSevenFireCallerAudio', 'combat/runtime', ['combat.map_seven_weapons', 'combat.firearm_targets', 'combat.attack_packets'], False),
    ('remove-allies', 'story.remove_allies', 'VerifyRemoveAllies', 'story_remove_allies', ['combat.allies'], False),
    ('round-stories', 'combat.round_stories', 'VerifyRoundStories', 'combat/round_stories', ['combat.next_round', 'combat.final_turn_stories', 'combat.enemy_placement'], False),
    ('mission-deaths', 'combat.mission_deaths', 'VerifyMissionDeaths', 'combat/mission_deaths', ['combat.queued_death_stories', 'combat.death_cleanup'], False),
    ('attack-packets', 'combat.attack_packets', 'VerifyAttackPackets', 'combat/attack', ['combat.attack.unarmed', 'combat.attack.natural', 'combat.required_melee', 'combat.scientist_turns'], False),
    ('random-items', 'inventory.initialize_random', 'VerifyRandomItems', 'items', ['inventory.initialize_plain', 'foundation.random'], False),
    ('enemy-spell-caller', 'combat.enemy_spell_caller', 'VerifyEnemySpellCaller', 'combat/runtime', ['combat.enemy_spell_selection', 'combat.spell_targets', 'combat.spell_bodies', 'combat.spell_fields', 'combat.attack_packets', 'combat.monster_melee'], False),
    ('enemy-spell-caller-low-action', 'combat.enemy_spell_low_action', 'VerifyEnemySpellCallerLowAction', 'combat/runtime', ['combat.enemy_spell_caller'], False),
    ('enemy-weapon-zero-caller', 'combat.enemy_weapon_zero', 'VerifyEnemyWeaponZeroCaller', 'combat/runtime', ['combat.attack_packets', 'combat.enemy_spell_caller'], False),
    ('parts-weapons', 'combat.parts_weapons', 'VerifyPartsWeapons', 'combat/enemy_weapons', ['combat.map_seven_weapons', 'combat.large_placement', 'combat.damage'], False),
    ('parts-natural', 'combat.parts_natural', 'VerifyPartsNatural', 'combat/attack', ['combat.attack.natural', 'combat.weapon_fallback'], False),
    ('parts-fire-caller', 'combat.parts_fire_caller', 'VerifyPartsFireCallerAudio', 'combat/runtime', ['combat.parts_weapons', 'combat.firearm_targets', 'combat.attack_packets'], False),
    ('monster-melee', 'combat.monster_melee', 'VerifyMonsterMelee', 'combat/enemy_weapons', ['combat.required_melee', 'combat.scientist_turns', 'combat.damage', 'combat.attack_packets'], False),
    ('guardian-melee', 'combat.guardian_melee', 'VerifyGuardianMelee', 'combat/enemy_weapons', ['combat.monster_melee', 'combat.scientist_turns', 'combat.natural_poison', 'combat.attack_packets', 'combat.enemy_turn'], False),
    ('player-psi', 'combat.player_psi', 'VerifyPlayerPsi', 'combat/player_psi', ['combat.spell_targets', 'combat.spell_bodies', 'combat.spell_fields', 'combat.attack_packets', 'session.save_reload'], False),
    ('endgame-weapons', 'combat.endgame_weapons', 'VerifyEndgameWeapons', 'combat/enemy_weapons', ['combat.parts_weapons', 'combat.endgame_melee_bodies', 'combat.firearm_targets', 'combat.attack_packets', 'combat.monster_elemental'], False),
    ('endgame-melee-bodies', 'combat.endgame_melee_bodies', 'VerifyEndgameMeleeBodies', 'combat/enemy_weapons', ['combat.monster_melee', 'combat.required_melee', 'combat.damage', 'combat.attack_packets'], False),
    ('monster-elemental', 'combat.monster_elemental', 'VerifyMonsterElemental', 'combat/ally_turns', ['combat.scientist_turns', 'combat.attack.natural', 'combat.damage'], False),
    ('monster-disease', 'combat.monster_disease', 'VerifyMonsterDisease', 'combat/ally_turns', ['combat.scientist_turns', 'combat.natural_disease', 'combat.damage'], False),
]


MISSION_CONTRACTS = {
    'player-psi': dict(
        mapped_entry_addresses=['0x13beea', '0x10cc02', '0x10d2b4', '0x11af0a', '0x11b164', '0x11937a', '0x151022', '0x1510e8'],
        preconditions='Active supplied warrior; learned2204 or2302; power1/2; sufficient word58 psi; native condition/hit/cast-limit/disabled gates; point range12 or global hostile mask',
        reads='Full warrior/enemy records, DataS packet, DataAD area, map/positions,480-byte saved psi memory,disabled byte and RNG',
        writes='Psi58,cast countersa66/ac8,movement3c/AP40 before resolution; complete combatants/map/fields/usage,packet/identity,memory descriptor and RNG; caster damage/kills counters',
        random_and_ordered_effects='Native admission has no AP minimum. Costs25 perpower, movementminus3 clamp0, APminus2unclamped. Storm targeting/duration precedes original resistance/damage; global2302 uses original active nonzero-faction mask. Full146 cases cover96 controlled casts,16 gates,32 affordability/range boundaries and2 real campaign inputs. Command tests compare full Session/events across save/reload and atomic unsupported/invalid choices',
        stage='integrated',
        integration='Fresh verify-stage-one seeds12/1; seed1 verify-samples5962commands6victories and verify-laboratory406commands5victories; seed12 verified1908-command sample prefix ending at unsupported2207; complete resumed state and ordered events'),
    'guardian-melee': dict(
        mapped_entry_addresses=['0x101eb6', '0x110e96', '0x130d38', '0x136398', '0x102012', '0x102038', '0x102ba2', '0x103580', '0x158a26'],
        preconditions='Actual MD29 Hive guardian bodies16/21 default base DataID187 class2, warrior0 or MD32 body22; loaded/unloaded index0/3, explicit retainedactor and target boundary variants; sixteen complete reaction callers',
        reads='All thirteen monster and six warrior records, base weapon187 work bytes, selected natural condition5, signed poison/save/race fields, retained actor, map/positions/order, persistent58-byte packet and RNG',
        writes='Complete party/enemy records, packet, retained actor current weapon, caller action/movement budgets, guard/facing/target flags and RNG; caller maps/positions/order remain fixture-identical',
        random_and_ordered_effects='Loaded187 writes retained actor before hit/damage and suppresses poison. Unloaded condition5 draws strength then save1; warrior signed-word duration wraps and writes only below30, race3 strength0; monster signed-byte duration wraps without cap. Failed save with nonnegative prior condition emits667 before result audio even when warrior counter remains unchanged. Full caller first guardian writes active22 current0-to187; MD32 primary0/default0 reloads class0 and clears187-to0 before natural packet. Public RunEnemyTurn controlled primary2 counterattack rejects after verified guardian mutation, preserving complete Session including RNG'),
    'schedule-edit': dict(
        mapped_entry_addresses=['0x140336', '0x145c82', '0x145d74', '0x1383e6', '0x138458', '0x14673c'],
        preconditions='Opcode54 exact five-word extra and loaded40-byte scheduled table; actual scripts168/169/305 extra330/331/574 plus controlled records0/1/11 and signed day variants',
        reads='Full480-byte authored TD3 table, extra fields, signed current day; native virtual EDCD and CTD3 services preserve ordered file effects',
        writes='Selected record words4/2/0 only, preserving all other table bytes; shared exploration.schedule is already persisted',
        random_and_ordered_effects='Unsigned-u16 extra0 selects record via MULU40. Nonnegative signed extra1 writes chanceword4, extra2 intervalword2; nonzero extra3 resets dayword0 then nonnegative extra4 adds with i16 wrapping. No RNG or world/party mutation. Four out-of-table probes stop at original seek before unsupported reads/writes and reject unchanged in Rust'),
    'story-delay': dict(
        mapped_entry_addresses=['0x140336', '0x1205f8', '0x120632', '0x14673c'],
        preconditions='Opcode73 signed parameter; seven native dispatch callers and seven controlled TickCount loops. Production accepts positive parameters only',
        reads='Signed parameter, script slot, retained default word439568 and wrapping unsigned TickCount for native-loop cases',
        writes='Next script slot and EventDelay ticks for positive parameters; no rule-record or gameplay RNG change',
        random_and_ordered_effects='Native caller passes parameter before advancing. Native loop returns only when unsigned wrapped elapsed is strictly greater than sign-extended duration; positive elapsed_ticks emits parameter+1. Parameter0 retained default and negative unsigned durations reject atomically in production; bounded parameter-1 loop observation does not prove termination'),
    'warrior-opportunity': dict(
        mapped_entry_addresses=['0x11bfc0', '0x11ce76', '0x110e96', '0x101eb6', '0x11c982'],
        preconditions='Active warrior walking to fixture-supported terrain with empty spell fields; MD13 loaded178 or friendly scientist65; nineteen layouts at seeds12/1 and three forced attack-return cases',
        reads='Six complete warrior records, three complete enemy records, old/new adjacency, factions, guard/eligibility bytes, AP, tactical map/positions, packet and RNG',
        writes='Full combatant records, guard bytes, facing, AP, tactical map/position/under tile, attack packet and RNG; forced return cases prove caller abort only',
        random_and_ordered_effects='Preguard mode0 ignores return; mode1 records old adjacency before facing/cost checks. Mode2 attacks ascending eligible IDs only when old adjacency is nonzero and new zero; clear guard104 before attack2index0. Return1 aborts/getup before AP/map writes. Successful AP subtraction precedes map/position, tile audio, empty collision and final mode0; class4/5 and activeword8e suppress reactions'),
    'endgame-weapons': dict(
        mapped_entry_addresses=['0x1146ee', '0x151022', '0x101eb6', '0x110e96', '0x1145c8', '0x115776'],
        preconditions='Actual MD115/119/120/121; base DataID guns28/29/680 and default194; authored faction/chance, explicit packet/identity/count/context and hostile body for side0 MD121',
        reads='Complete party/enemy records, base item bands, selected natural condition, packet/identity, target lists/masks, tactical state and RNG',
        writes='Full party/enemy records, packet/identity, map/positions/order, action budget, count/context and RNG; native normal hit/miss and survivor impact audio',
        random_and_ordered_effects='Attack2 copies DataS4103 then checks active human-faction warriors; absent humans return before weapon writes/RNG/audio and set native deadcount to total. Caller stops movement but retains terminal firearm retry. Prepared guns, native casts, natural/default194 calls and condition14 preserve ordered effects; MD121 keeps side0/chance0'),
    'endgame-melee-bodies': dict(
        mapped_entry_addresses=['0x101eb6'],
        preconditions='Actual MD119 default DataID194 against scientist65; current0/194/authored29, attackindex0/1, retainedactor0/2 and active human party',
        reads='Three complete enemy records, weapon194 class2 definition, target defense/shield, retained actor and RNG',
        writes='All three enemy records, all58 attack packet bytes and RNG; unchanged warrior party; caller owns death cleanup',
        random_and_ordered_effects='Loaded default194 writes retained actor before hit/damage/saves; unloaded packet keeps original zero-maximum arithmetic; five defense variants retain native impact/miss audio and RNG order'),
    'monster-elemental': dict(
        mapped_entry_addresses=['0x101eb6', '0x110e96', '0x158be4', '0x158bf6'],
        preconditions='Authored MD120 natural14 against scientist65, selected slot0/1, retained actor0/2 and explicit save/protection/HP inputs',
        reads='Complete party/three enemy records, selected signed maximum byte94, save4, protectionbytebe, retained packet/actor and RNG',
        writes='Target HP subtracts ordinary total then extra signed word; full attacker/actor records, packet, tactical caller state and RNG',
        random_and_ordered_effects='Retain unused level strength draw before Rand(selected maximum); save4 then any nonzero protectionbe each halve towardzero; nonzero extra emits native344 before ordinary impact; misses emit650'),
    'manual-encounter': dict(
        mapped_entry_addresses=['0x10a478', '0x10a5dc', '0x1400e4'],
        preconditions='Manual Encounter control, signed outdoor coordinates and exact644-byte region record; direct mode1 script0/250 typed cancellation cases',
        reads='Descending region bounds/chances, three ordered script slots, signed coordinates and RNG; exact ED3/ED2 for direct cases',
        writes='Cleared positive region chances, saved region bytes, zero last movement, manual flag and selected script caller arguments',
        random_and_ordered_effects='Scan descending regions with exclusive right/bottom bounds; last successful of three ordered slot draws wins; save before mode1 dispatch; script0 loads typed0 rather than acting as a no-op'),
    'map-picture': dict(
        mapped_entry_addresses=['0x140336', '0x14036a', '0x141af0', '0x10f186', '0x14673c'],
        preconditions='Opcode113 with signed word parameter; controlled previous values and authored script85slot0/script86slot4',
        reads='Current map metadata word, signed parameter, complete party/world bytes',
        writes='Signed map metadata word40857a followed by one centerpict redraw request',
        random_and_ordered_effects='Store the signed parameter before the redraw boundary and ordinary continuation; no RNG and no party/world changes; visible artwork mapping is not recovered'),
    'booty-availability': dict(
        mapped_entry_addresses=['0x105748', '0x1057e4', '0x105978', '0x10598e', '0x107e52'],
        preconditions='Six original warrior inputs, zero enemy, no awarded items or pending level gain; explicit signed HP/condition cases',
        reads='Complete party/enemy records, HPword50, conditionword90, rewards, loot state and RNG',
        writes='Normalized party/enemy records, signed unavailable byte442739, prepared rewards and loot Close state',
        random_and_ordered_effects='Clear incoming availability before recount; each warrior counts once for HPbelow1 or negative condition; positive conditions clear without counting; original reward/Close RNG and sounds retained'),
    'monster-disease': dict(
        mapped_entry_addresses=['0x101eb6', '0x110e96'],
        preconditions='MD67 selected natural condition9 against scientist65 with explicit duration/save/shield inputs; supported adjacent caller',
        reads='Complete party/enemy records, selected condition, signed duration byte0xce, save7, tactical caller state and RNG',
        writes='Full target/attacker records, signed duration byte, packet, caller state and RNG',
        random_and_ordered_effects='Positive damage precedes strength draw and save7; failed save adds with signed-byte wrapping unless duration is negative; applied104 precedes ordinary result audio102 or650'),
    'genetic-melee': dict(
        mapped_entry_addresses=['0x101eb6', '0x110e96'],
        preconditions='Loaded melee178 for MD13/36 or the controlled level3 route enemy; living warrior target without reflection',
        reads='Full warrior/attacker records, weapon definition, attack index, facing, action budget and RNG',
        writes='Full warrior/attacker records, attack packet, ordered caller attacks and RNG',
        random_and_ordered_effects='Original hit, component damage and save draws; adjacent caller retains attack indices and point costs; native impact/miss sound slice executes'),
    'map-seven-weapons': dict(
        mapped_entry_addresses=['0x1146ee', '0x151022', '0x101eb6', '0x110e96'],
        preconditions='Authored map7 actors MD69/70/71, guns9/38/3 or default180; explicit item context0/1 and controlled target masks',
        reads='Weapon/ammo definitions, complete combatant records, attack packet, target mask and RNG',
        writes='Prepared packet/current weapon, complete party or scientist target records, adjacent caller state and RNG',
        random_and_ordered_effects='Preparation precedes native resist/damage/saves; melee hit/miss audio and repeated adjacent attacks preserve original order'),
    'map-seven-fallback': dict(
        mapped_entry_addresses=['0x101eb6'],
        preconditions='MD69/70/71 current gun9/38/3, controlled default melee0 and facing1; supported warrior target',
        reads='Current/default weapon words, natural attack index, complete party/attacker and RNG',
        writes='Cleared current weapon, natural attack packet, complete party/attacker records and RNG',
        random_and_ordered_effects='Reject class15 for melee before natural hit/damage draws; both attack slots and eight seeds retain draw order'),
    'map-seven-fire-caller': dict(
        mapped_entry_addresses=['0x1145c8', '0x10d4fe', '0x15cb0e', '0x151022'],
        preconditions='MD69/70/71 authored guns and factions; fixture-supported target positions, item context and actor points',
        reads='Complete party/enemy records, tactical map, positions/order, item context, attack count and RNG',
        writes='Cast lists/masks, complete combatants/map/order, item context/count, movement and RNG',
        random_and_ordered_effects='Native preparation, target choice, cast resolution, living-target checks, death and movement execute in caller order; natural prelude observes item-context reset'),
    'remove-allies': dict(
        mapped_entry_addresses=['0x142146', '0x1046e0'],
        preconditions='Opcode88 signed class parameter with0..20 stored ally records',
        reads='Signed class byte, signed parameter, ally count and all twenty records',
        writes='Ally count, complete twenty-record buffer including stale tail and rebuilt menu',
        random_and_ordered_effects='No RNG; below slot5 repeatedly copy only the next record, then advance outer scan and preserve skipped adjacent matches; later slots only decrement count'),
    'round-stories': dict(
        mapped_entry_addresses=['0x125d34', '0x1400e4', '0x109210', '0x149e0e'],
        preconditions='Initiative wrap with BD146 five round-script slots and fixture-supported map/capacity',
        reads='Signed-byte round, BD scripts, opcode126 predicates, MD definitions, factions, placement map and RNG',
        writes='Wrapped round byte, persisted round-script continuation, spawned records/positions/map and RNG',
        random_and_ordered_effects='Increment round before scanning five slots; authored363/364/338/340 and opcode124 build/placement retain order; capture ends before hourly reduction'),
    'mission-deaths': dict(
        mapped_entry_addresses=['0x12e4e4', '0x121524', '0x1400e4'],
        preconditions='MD119/121/117 authored death336/337, direct or queued caller with explicit retained actor',
        reads='All100 enemy records, signed classes/factions, unavailable byte, queue/actor, party/map/order and ED scripts',
        writes='Complete enemy/party/map/order snapshots, unavailable counter, story effects and caller cleanup',
        random_and_ordered_effects='No RNG; native kill or emptyque dispatch runs story119/123 before remaining caller cleanup; signed counters and retained actor differ between direct and queued calls'),
    'attack-packets': dict(
        mapped_entry_addresses=['0x100af0', '0x101eb6'],
        preconditions='Fixture-supported unarmed, natural0/4/5/9/14, loaded178 or scientist-target natural attack; explicit retained packet/count/identity',
        reads='All58 retained packet bytes, count/identity, warrior budget, natural/weapon definition, target defenses and RNG',
        writes='All58 packet bytes, full attacker/target records and RNG; descriptor identity/count remain unchanged',
        random_and_ordered_effects='Unarmed exhausted budget returns before copying4103; attack2 copies4103 before hit; successful damage fills authored min/max/elements or natural impact word'),
    'random-items': dict(
        mapped_entry_addresses=['0x12d558'],
        preconditions='Fresh22-byte item, one of38 independently exercised signed IDs, enhancement-1/0/1/3 and difficulty0',
        reads='Signed item/enhancement words, exact base/scenario definitions and RNG',
        writes='Complete22-byte inventory record and RNG',
        random_and_ordered_effects='Original quantity initialization and enhancement traversal retain cumulative upgrade arithmetic and every draw; includes mandatory178/37/192'),
    'enemy-spell-caller': dict(
        mapped_entry_addresses=['0x1145c8', '0x10d4fe', '0x15cb0e', '0x151022'],
        preconditions='MD13/28/94 authored spell tables and fixture-supported positions/targets, item context and count0/1/7',
        reads='Complete combatants, tactical map/order, spell tables, count/identity/packet, fields/used slots and RNG',
        writes='Ordered casts/resolution snapshots, applied masks, all combatant/field/map/order/work-register state and RNG',
        random_and_ordered_effects='Native two/three attempts, cast targeting, reflect/resist/damage/death and movement retain order; zero-area unknown-heap cases provide diagnostic evidence and must reject without committing partial state'),
    'enemy-spell-caller-low-action': dict(
        mapped_entry_addresses=['0x1145c8', '0x10d4fe', '0x15cb0e', '0x151022'],
        preconditions='MD13/28/94 forced spell chance with initial AP2 and authored two/three attempts',
        reads='Signed-byte AP, attempt count, retained spell state, full combatants/map/fields and RNG',
        writes='Signed-byte AP including negative values, casts/masks, combatants/fields/work registers and RNG',
        random_and_ordered_effects='Attempt order and subtraction retain negative AP; diagnostic zero-area paths reject the entire candidate rather than applying partial original snapshots'),
    'enemy-weapon-zero-caller': dict(
        mapped_entry_addresses=['0x1145c8', '0x125a76', '0x10d4fe', '0x151022'],
        preconditions='MD28 weapon0 with explicit retained blank/1713/4101 packet, type byte19 intact or cleared, count0/1/7 and controlled positions',
        reads='Retained58-byte packet, descriptor identity/count, item work context, targets/map/order and RNG',
        writes='Complete actor-turn combatants/map/order/fields, packet/identity/count, cast masks and RNG',
        random_and_ordered_effects='Weapon0 skips prepattack and consumes the retained packet; getup clears only byte19; actual targeting/resolution/movement retains caller order'),
    'parts-weapons': dict(
        mapped_entry_addresses=['0x1146ee', '0x151022', '0x101eb6', '0x110e96'],
        preconditions='Required BD125/126/127 actors MD33/56/68/72, gun37 or melee192/default180; explicit masks/item context and supported target',
        reads='Authored definitions, complete combatants, packet, size3 footprint cells, caller points and RNG',
        writes='Prepared packet/current weapon, full party or scientist records, adjacent map/order state and RNG',
        random_and_ordered_effects='Original preparation/resist/damage/saves and melee audio retain order; MD56 caller uses all four native footprint cells and AP1/2/3/4/6'),
    'parts-natural': dict(
        mapped_entry_addresses=['0x101eb6'],
        preconditions='MD33/56 natural attacks or MD72 gun37 with controlled default melee0; supported warrior target',
        reads='Complete party/attacker, signed natural min/max bytes, attack index and RNG',
        writes='Current weapon, complete party/attacker, all58 packet bytes and RNG',
        random_and_ordered_effects='Gun37 rejection clears current weapon before natural attack; all three slots against six warriors over eight seeds preserve original draw order'),
    'parts-fire-caller': dict(
        mapped_entry_addresses=['0x1145c8', '0x10d4fe', '0x15cb0e', '0x151022'],
        preconditions='MD72 authored gun37/faction with fixture-supported positions, actor points and item context',
        reads='Complete party/enemy records, map/positions/order, packet/count/item context and RNG',
        writes='Ordered casts/masks, combatants/map/order, terminal work context and RNG',
        random_and_ordered_effects='Native targeting, cast resolution, death/movement and optional natural context-reset prelude retain caller order'),
    'monster-melee': dict(
        mapped_entry_addresses=['0x101eb6'],
        preconditions='Default177/178/181/192 against scientist65, loaded or unloaded; explicit retained actor; reflection/immunity and effective condition10 remain unsupported',
        reads='Full three enemy records, selected natural condition, weapon definition, positions, actor and RNG',
        writes='Complete three enemy records, all58 packet bytes and RNG; party unchanged; caller owns death cleanup',
        random_and_ordered_effects='Default-weapon write uses retained actor; hit marks target105 before zero-damage resolution; unloaded177/178 condition10 is allowed only when zero bonus makes damage impossible'),
}


def extend_mission_actions(document):
    for action in document['actions']:
        action['state_dependencies'] = action.get('dependencies', '')
        action['dependencies'] = []
        action['stage'] = 'integrated'
        action['integration'] = 'v0.2.0 production opening, fight, day-two and sustained regressions'
    for fixture, ident, probe, module, dependencies, integrated in MISSION_ROWS:
        path = ROOT / f'evidence/fixtures/{fixture}.json'
        if not path.exists():
            continue
        data = json.loads(path.read_text())
        script = f'tools/ghidra/{probe}.java'
        source = (ROOT / script).read_text()
        anchors = list(dict.fromkeys(re.findall(r'(?:execute|jump)\((0x[0-9a-fA-F]+)', source)))
        if fixture == 'ally-turns':
            anchors = ['0x1145c8', '0x101eb6']
        tests = {
            'map-actions': 'binary_travel', 'travel': 'binary_stage_travel',
            'stage-travel': 'binary_stage_travel', 'genetic-enemies': 'binary_genetic_rewards',
            'genetic-victory': 'binary_genetic_rewards', 'loot-collection': 'binary_story_collection',
            'item-use': 'binary_story_item_use',
            'large-body-move': 'binary_large_body_move',
            'required-natural': 'binary_required_natural',
            'inactive-battle-return': 'binary_inactive_battle_entry',
            'nested-caller': 'live_story_transactions',
            'npc-battle-caller': 'binary_npc_battle',
            'terminal-stage': 'binary_terminal_stage',
            'registration-caller': 'binary_registration',
            'enemy-spell-selection': 'binary_enemy_spells',
            'enemy-spell-damage': 'binary_enemy_spells',
            'spell-targets': 'binary_enemy_spells',
            'firearm-targets': 'binary_enemy_spells',
            'enemy-firearm-bodies': 'binary_enemy_spells',
            'additional-spells': 'binary_spell_bodies',
            'condition-targets': 'binary_enemy_spells',
            'enemy-spell-caller-low-action': 'binary_enemy_spell_caller',
        }.get(fixture, 'binary_' + fixture.replace('-', '_'))
        identity = data.get('cache_identity', data.get('identity', {key: value for key, value in data.items()
                                              if key.endswith('sha256') or key == 'ghidra'})
                            )
        count = sum(len(value) for key, value in data.items()
                    if isinstance(value, list) and (key == 'cases' or key.endswith('_cases')))
        if not count and fixture == 'terminal-stage':
            count = 1
        action = dict(
            id=ident, resource='Centurions 1.3.2 CODE 1 and supplied scenario records',
            mapped_entry_addresses=anchors, authority=data['authority'], fixture=str(path.relative_to(ROOT)),
            probe=script, cache_identity=identity, case_count=count,
            implementation=f'crates/core/src/{module}.rs', tests=f'crates/core/tests/native/{tests}.rs',
            dependencies=dependencies, stage='integrated' if integrated else 'implemented',
            integration='verify-stage-one, seeds 12 and 1, complete resumed state and ordered events' if integrated else None,
            failure='Invalid or unsupported command rejects the complete candidate, including RNG',
            reference_observation='Bounded original instructions; interception and numeric models are specified by fixture authority. Live route proof is separate.',
        )
        if fixture in MISSION_CONTRACTS:
            action.update(MISSION_CONTRACTS[fixture])
            action['numeric_widths'] = 'Authored signed16-bit words and signed/unsigned bytes; signed-byte enemy AP/conditions; i32 warrior counters; u32 RNG seed'
            if fixture == 'story-delay':
                action['numeric_widths'] = 'Signed16-bit parameter/default/script slot; sign-extended duration compared against wrapping unsigned32-bit elapsed TickCount; u32 event ticks'
            elif fixture == 'schedule-edit':
                action['numeric_widths'] = 'Unsigned16-bit record index multiplied to32-bit byte offset; signed16-bit fields/current day; wrapped signed16-bit day addition'
            diagnostics = sum(bool(case.get('diagnostic')) for key, value in data.items()
                              if isinstance(value, list) and (key == 'cases' or key.endswith('_cases'))
                              for case in value if isinstance(case, dict))
            if fixture == 'schedule-edit':
                diagnostics += sum(bool(case.get('boundary_only')) for case in data['cases'])
            action['oracle_verified_case_count'] = count - diagnostics
            action['diagnostic_case_count'] = diagnostics
        document['actions'].append(action)
    document['schema_version'] = 2
    document['stage_order'] = ['discovered', 'contracted', 'oracle-verified', 'implemented', 'integrated']
    document['route'] = dict(
        scenario='The Thunder of Guns', seeds=[12, 1], specification='evidence/mission-route.md',
        replay='crates/tools/src/completion.rs',
        checkpoints=[
            dict(id='mission.first_stage', stage='integrated', command='verify-stage-one', dependencies=[
                'story.encounter_choice', 'story.flag_logic', 'story.inventory_persistent', 'story.mutable_map',
                'travel.map_state', 'mission.stage_change', 'story.item_collection']),
            dict(id='mission.samples', stage='discovered', command='verify-samples', dependencies=[
                'mission.first_stage', 'travel.door_destination', 'combat.scenario_entry', 'combat.genetic_rewards',
                'story.add_ally', 'combat.allies', 'story.item_collection']),
            dict(id='mission.password_exit', stage='discovered', dependencies=['mission.samples', 'story.typed_encounter', 'story.variables', 'mission.stage_change']),
            dict(id='mission.installations', stage='discovered', dependencies=['mission.password_exit', 'travel.door_destination', 'mission.stage_change']),
            dict(id='mission.final_battles', stage='discovered', dependencies=['mission.installations', 'story.skill_choice', 'combat.scenario_entry']),
            dict(id='mission.terminal', stage='discovered', dependencies=['mission.final_battles', 'mission.stage_change', 'session.save_reload']),
        ],
        blockers=['normal production replay through the remaining mission checkpoints',
                  'unsupported branches encountered by required story, combat or survival commands'],
    )
ROWS = [
    ('calendar', 'time.calendar', 'VerifyCalendar', 'crates/core/src/time.rs', 'crates/core/src/time.rs', 'Selected opening conditions and bounded schedule prerequisites', 'Six warriors, food definition, clock and mutable Data TD3', 'Complete warriors, clock, day, fatigue, schedule and RNG', 'Reduce before rollover; schedule scan before midnight healing; eligible dispatch repeats scan and healing; signed regeneration retained'),
    ('ammunition-init', 'inventory.initialize_ammunition', 'VerifyAmmunitionInit', 'crates/core/src/items.rs', 'crates/core/tests/native/binary_loot.rs', 'Original starter ammunition 200', '150-byte item definition and RNG', '22-byte pack and RNG', 'Quantity draw precedes fixed quantity override and enhancement draw'),
    ('ammunition-collection', 'inventory.collect_ammunition', 'VerifyAmmoCollection', 'crates/core/src/items.rs', 'crates/core/src/items.rs', 'Positive compatible pack 200, kind 26', 'Item, definition, shared reserve and RNG', 'Reserve, loot ID and RNG', 'Collection consumes the pack directly; default Rand(2) sound then slot ID clear'),
    ('ammunition-drop', 'inventory.consume_carried_pack', 'VerifyAmmoDrop', 'crates/core/src/items.rs', 'crates/core/src/items.rs', 'Unequipped zero-weight compatible carried pack; unobstructed exploration', 'Warrior inventory, pack, reserve and definitions', 'Reserve and compacted complete warrior', 'Reserve conversion then original drop/compact semantics; invalid quantities/equipment/phase reject atomically'),
    ('supply-init', 'inventory.initialize_supply_cache', 'VerifySupplyInit', 'crates/core/src/items.rs', 'crates/core/tests/native/binary_loot.rs', 'Authored treasure 30 item IDs and supported enhancement fields', 'Item/ammunition definitions and RNG', 'Complete initialized items and RNG', 'Original per-item random quantity and enhancement traversal'),
    ('opening-stories', 'story.opening_army_and_camp', 'VerifyOpeningStories', 'crates/core/src/exploration.rs', 'crates/core/tests/native/binary_opening_stories.rs', 'Death scripts 3/4 or camp 6 with peaceful Yes and menu Cancel', 'Full enemy records, dialogue, EDCD, scripts, region table and menu 0', 'Enemy removal, script continuation, region 2, same-map teleport and RNG', 'Nearest-body ties/radius retain source behavior; question and menu sound 20005; region/teleport before cargo menu'),
    ('enemy-firearm-preparation', 'combat.prepare_army_firearm', 'VerifyEnemyFirearmPreparation', 'crates/core/src/combat/firearm.rs', 'crates/core/tests/native/binary_firearm.rs', 'Authored enemy weapons 680/681', 'Enemy, item/ammo definitions and attack table', 'Attack packet and enemy weapon fields', 'Original loadweapon/loadammo/prepattack/loadspell2 preparation'),
    ('enemy-gun-resolve', 'combat.army_firearm', 'VerifyEnemyGunResolve', 'crates/core/src/combat/firearm.rs', 'crates/core/tests/native/binary_firearm.rs', 'Recovered chemical weapons 680/681 against starter profiles', 'Attack packet, saves, conditions and RNG', 'Complete warrior/enemy records and RNG', 'Original resist, damage and save order; target selection is a separate boundary'),
    ('enemy-ranged-turn', 'combat.army_enemy_turn', 'VerifyEnemyRangedTurn', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_enemy_turn.rs', 'Fixture-supported armed army decisions', 'Combatants, positions, initiative, map and RNG', 'Combatants, movement, conditions, next turn and RNG', 'Original decision, LOS/range, cast resolution and terminal turn end'),
    ('sparse-turn', 'combat.sparse_next_turn', 'VerifySparseTurn', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_round.rs', 'Supported empty initiative slots and empty spell queue', 'Initiative and warrior equipment/points', 'Cursor and complete records', 'Retain original empty-slot selection and compactitems/movecalc order'),
    ('bleeding-round', 'combat.bleeding_round', 'VerifyBleeding', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_round.rs', 'Fixture-supported unconscious and bleeding starter injuries', 'Full warriors, inactive flags, counters and RNG', 'Full warriors, round state and RNG', 'Preserve getup/reduce/heal/movecalc and the death-sound draw'),
    ('army-victory', 'combat.army_victory', 'VerifyArmyVictory', 'crates/core/src/combat/lifecycle.rs', 'crates/core/tests/native/binary_victory.rs', 'Defeated authored army classes 16/17/22/23 and supported loot fields', 'Warriors, enemies, reward/profile/item tables and RNG', 'Full warriors/enemies, sorted complete loot and RNG', 'Normal battle reward flag zero; original random item initialization and sorting'),
    ('startup', 'foundation.initialize', 'VerifyStartup', 'tools/decode_globals.py', 'tools/ghidra/VerifyStartup.java', 'Original initialized data and relocations', 'CODE/global initialization', 'Initialized data, relocation targets', 'No gameplay RNG'),
    ('rules', 'foundation.random', 'VerifyRules', 'crates/core/src/lib.rs', 'crates/core/tests/native/binary_rules.rs', 'Signed bound/range inputs and saved seed', 'RNG seed', 'RNG seed and draw counter', 'One QuickDraw draw per bound/range, including zero-width ranges'),
    ('storage', 'inventory.store', 'VerifyStorage', 'crates/core/src/lib.rs', 'crates/core/tests/native/binary_storage.rs', '1000 inventory slots and a 22-byte item', 'Item and destination storage', 'First empty slot', 'No RNG; item event before storage sound; full storage returns no mutation'),
    ('plain-loot', 'inventory.initialize_plain', 'VerifyPlainLoot', 'crates/core/src/items.rs', 'crates/core/tests/native/binary_loot.rs', 'Plain kind-23 quest item without enhancements', '150-byte item definition', '22-byte inventory record', 'Quantity draw only when authored base is positive'),
    ('time', 'exploration.hour', 'VerifyTime', 'crates/core/src/time.rs', 'crates/core/src/time.rs', 'First-map supported hour and conditions', 'Clock and six warriors', 'Clock, conditions and healed warrior fields', 'Original reduce/heal draw order; unsupported time branches reject'),
    ('regional', 'exploration.regional_encounter', 'VerifyRegional', 'crates/core/src/exploration.rs', 'crates/core/src/exploration.rs', 'Supported region on first map', 'Region table and RNG', 'Selected encounter and RNG', 'Original regional draw order; stop boundary precedes combat'),
    ('combat-map', 'combat.map', 'VerifyCombatMap', 'crates/core/src/combat.rs', 'crates/core/tests/native/binary_combat_map.rs', 'Opening terrain/property table', 'Exploration cells and properties', '90 by 90 tactical map and shift', 'Decoration draws in original cell traversal order'),
    ('direction', 'combat.direction', 'VerifyDirection', 'crates/core/src/combat.rs', 'crates/core/tests/native/binary_direction.rs', 'Supported zero authored distance', 'Battle direction and supplied RNG', 'Direction and origin', 'Bounded SANE/trigonometric model; supplied Rand outcomes'),
    ('formation', 'combat.squad_formation', 'VerifyFormation', 'crates/core/src/combat.rs', 'crates/core/tests/native/binary_formation.rs', 'Six starters and supported empty placement map', 'Warriors, item table, tactical cells', 'Warrior positions, under tiles and map', 'Placement retries retain original ordering'),
    ('enemies', 'combat.enemy_placement', 'VerifyEnemies', 'crates/core/src/combat.rs', 'crates/core/tests/native/binary_enemies.rs', 'Selected authored battle and monster definitions', 'Battle, 288-byte definitions, map', 'Enemy records, positions, map and RNG', 'Original enemy adjustment and placement draw order'),
    ('initiative', 'combat.initiative', 'VerifyInitiative', 'crates/core/src/combat.rs', 'crates/core/tests/native/binary_initiative.rs', 'Supported surprise mode and living combatants', 'Speeds and RNG', 'Initiative order and RNG', 'Preserves original adjacent comparison order'),
    ('combat-movement', 'combat.movement_capacity', 'VerifyCombatMovement', 'crates/core/src/combat.rs', 'crates/core/tests/native/binary_combat_movement.rs', 'Supported starter profiles/conditions', 'Warrior, inventory, race/caste data', 'Movement capacity', 'No RNG; explicit single-precision conversions'),
    ('walk', 'combat.walk', 'VerifyWalk', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_walk.rs', 'Active warrior, adjacent step, enough points; no active spell field', 'Map/properties, warrior, nearby bodies', 'Position, map, movement/action points', 'Collision before committed move; sounds retain source order'),
    ('unarmed', 'combat.attack.unarmed', 'VerifyUnarmed', 'crates/core/src/combat/attack.rs', 'crates/core/tests/native/binary_unarmed.rs', 'Unarmed starter against living monster 49', 'Warrior, enemy and RNG', 'Damage, flags, counters and RNG', 'Original hit/critical/damage draw order; direct sounds compared'),
    ('firearm-preparation', 'combat.attack.prepare_firearm', 'VerifyFirearm', 'crates/core/src/combat/firearm.rs', 'crates/core/tests/native/binary_firearm.rs', 'Starter secondary 1, ammo 200, descriptor 4101, no enhancements', 'Equipped inventory, weapon/ammo, Data S', '58-byte attack packet', 'No RNG; SANE conversion modeled'),
    ('gun-resolve', 'combat.attack.firearm', 'VerifyGunResolve', 'crates/core/src/combat/firearm.rs', 'crates/core/tests/native/binary_firearm.rs', 'Prepared physical starter firearm and living monster 49', 'Packet, warrior, enemy and RNG', 'Health, attack counters, flags, RNG', 'Duration draw precedes hit/resist and damage; caller composes ammo/AP costs'),
    ('reload', 'combat.reload', 'VerifyReload', 'crates/core/src/combat/firearm.rs', 'crates/core/tests/native/binary_firearm.rs', 'Active starter weapon and compatible nonnegative reserve', 'Loaded/capacity words, reserve, silent flag', 'Loaded ammunition, reserve, RNG', 'Audible successful refill draws Rand(2) then emits 404/405; partial branch preserves reserve'),
    ('damage', 'combat.damage', 'VerifyDamage', 'crates/core/src/combat/damage.rs', 'crates/core/tests/native/binary_damage.rs', 'Fixture-supported packet categories and damage flags', 'Packet, target armor/shield/saves and RNG', 'Damage totals, target flags/shield, RNG', 'Shield checks and per-component draws preserve order'),
    ('natural', 'combat.attack.natural', 'VerifyNatural', 'crates/core/src/combat/attack.rs', 'crates/core/tests/native/binary_natural.rs', 'Monster 49 against starter; supported normal conditions', 'Enemy, warrior, item definitions, RNG', 'Warrior health/flags, enemy counters, RNG', 'Original attack2 calls through damage; presentation intercepted'),
    ('enemy-move', 'combat.enemy_move', 'VerifyEnemyMove', 'crates/core/src/combat/enemy.rs', 'crates/core/tests/native/binary_enemy_move.rs', 'Size-zero monster 49 and supported map/collision', 'Enemy position, map and properties', 'Enemy, position, map and RNG', 'Original movement/placement traversal and direct sounds'),
    ('enemy-turn', 'combat.enemy_turn', 'VerifyEnemyTurn', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_enemy_turn.rs', 'Active monster 49; supported normal AI branches', 'All combatants, map, initiative, RNG', 'Movement/attacks, combatants, next turn, RNG', 'Selection, LOS, attack/movement and turn end compose in original order'),
    ('turn', 'combat.next_turn', 'VerifyTurn', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_turn.rs', 'Supported order, empty spell queue and starter profiles', 'Initiative cursor, combatants', 'Cursor, mode, compacted equipment, points', 'Original compactitems/movecalc before turn presentation'),
    ('round', 'combat.next_round', 'VerifyRound', 'crates/core/src/combat/runtime.rs', 'crates/core/tests/native/binary_round.rs', 'Supported starter injuries; empty spell queue', 'Combatants, profiles, conditions and RNG', 'Round, regenerated points, conditions, RNG', 'Original reduce/heal/compactitems/movecalc sequence'),
    ('lifecycle', 'combat.death_cleanup', 'VerifyLifecycle', 'crates/core/src/combat/lifecycle.rs', 'crates/core/tests/native/binary_lifecycle.rs', 'Defeated starter or size-zero enemy', 'Body record, position, under tile, map', 'Body flags and map occupancy', 'No gameplay RNG; presentation intercepted'),
    ('victory', 'combat.victory', 'VerifyVictory', 'crates/core/src/combat/lifecycle.rs', 'crates/core/tests/native/binary_victory.rs', 'All selected enemies defeated; supported empty battle loot', 'Warriors, enemies and battle reward data', 'Warriors, experience, exploration phase, RNG', 'Original cleanup/rewards through recorded stop; profile loading modeled'),
]


def main():
    actions = []
    for fixture, ident, probe, implementation, tests, pre, reads, writes, ordering in ROWS:
        data = json.loads((ROOT / f'evidence/fixtures/{fixture}.json').read_text())
        script = f'tools/ghidra/{probe}.java'
        source = (ROOT / script).read_text()
        pcs = list(dict.fromkeys(re.findall(r'writeRegister\("PC",\s*(0x[0-9a-fA-F]+)', source)))
        pcs.extend(x for x in re.findall(r'execute\((0x[0-9a-fA-F]+)', source) if x not in pcs)
        if fixture == 'storage':
            pcs = ['0x100a38', '0x100a9a']
        actions.append(dict(id=ident, resource='Centurions 1.3.2 CODE 1', mapped_entry_addresses=pcs,
                            preconditions=pre, reads=reads, writes=writes,
                            numeric_widths='signed 16-bit words, signed/unsigned authored bytes; RNG seed u32; counters/coordinates i32 where authored',
                            failure='Production command rejects unsupported/invalid dependencies atomically, including RNG',
                            random_and_ordered_effects=ordering, authority=data.get('authority'),
                            fixture=f'evidence/fixtures/{fixture}.json', case_count=len(data.get('cases', [])),
                            probe=script, implementation=implementation, tests=tests, dependencies=reads,
                            reference_observation='bounded original instruction execution; see fixture authority for intercepted boundaries'))
    document = dict(schema_version=1, authority='Supplied Centurions 1.3.2', actions=actions,
                    composed_actions=[dict(id='exploration.opening_route', implementation='crates/core/src/exploration.rs',
                                           verification='centurions-tools verify-opening; original first-step guest capture compared by verify-movement'),
                                      dict(id='session.save_reload', implementation='crates/core/src/lib.rs',
                                           verification='centurions-tools verify-save and verify-fight resumed whole-state/event replay'),
                                      dict(id='combat.target_bounds_fix', implementation='crates/core/src/combat/runtime.rs',
                                           verification='crates/core/tests/native/enemy_target_bounds.rs; fixture opening-enemy-prefix.json; intentional original bounds deviation')],
                    deferred=['whole live-original battle parity', 'player weapons and ammunition beyond registered contracts', 'maps and story opcodes beyond registered contracts', 'interactive active schedules and campaign prerequisites', 'spell fields and targeting beyond registered contracts', 'complete normal production campaign replay', 'original campaign saves'])
    document['native_capture_provenance'] = 'evidence/native-capture-provenance.json'
    static_rows = [
        ('foundation.import_starters', '0x1538ba', 'crates/core/src/lib.rs', 'Six hash-verified original warrior files', '3138-byte content records', 'Six runtime warrior records', 'No RNG; StartersImported'),
        ('exploration.enter_mission', '0x1538ba', 'crates/core/src/lib.rs', 'Six imported starters and opening scenario', 'Data CS/DD/RD/LD and warrior bytes', 'Mission, exploration, initialized warrior fields', 'No RNG; MissionEntered; setup checked against guest snapshot'),
        ('exploration.move', '0x1400e4', 'crates/core/src/exploration.rs', 'First map, adjacent nonzero direction, no pending page', 'Map/properties, position, clock, regions and door headers', 'Position, facing, clock, warriors, event continuation, RNG', 'Tile sounds, time update, regional dispatch, then local script/search; composed route and guest first-step comparison'),
        ('story.dialogue', '0x1400e4', 'crates/core/src/exploration.rs', 'Opcode 1 with supported Data SD2 record', 'Script cursor, dialogue, living warrior names', 'Cursor and pending dialogue', 'No RNG; name substitution then StoryPresented'),
        ('story.sound', '0x1400e4', 'crates/core/src/exploration.rs', 'Opcode 9', 'Script parameter', 'Script cursor', 'No RNG; Sound event before subsequent opcode'),
        ('story.replace_script', '0x1400e4', 'crates/core/src/exploration.rs', 'Opcode 39 and valid Data ED3 record', 'Replacement record', 'Current script operation/parameter bytes and cursor', 'No RNG; restart cursor at zero within bounded dispatch loop'),
        ('story.prologue', '0x13ebde', 'crates/core/src/exploration.rs', 'Opcode 62, TEXT 1000', 'Script and original TEXT', 'Cursor and pending prologue', 'No RNG; StoryPresented; desktop presentation adapts original instructions'),
        ('story.grant_loot', '0x1400e4', 'crates/core/src/exploration.rs', 'Opcode 10, plain quest treasure without enhancement/reward fields', 'Data TD, item definitions, warriors, RNG', 'Booty slots, supported condition cleanup, cursor and pending loot', 'Initialize each authored item in order then StoryPresented'),
        ('story.begin_battle', '0x1400e4', 'crates/core/src/exploration.rs', 'Opcode 2 and supported authored battle', 'Script parameter and battle content', 'Combat phase, map, combatants and RNG', 'Compose recovered map/formation/enemies/initiative actions'),
        ('story.complete_once', '0x1400e4', 'crates/core/src/exploration.rs', 'Eight script slots complete', 'Current door-header slot', 'Chance byte becomes 0xff; script cleared', 'No RNG; once-only event state is persisted'),
        ('inventory.take_quest_loot', '0x105748', 'crates/core/src/exploration.rs', 'Pending loot, live item slot, recipient below 100 items; kind 23 and zero weight', 'Booty item, recipient inventory/count, item definition', 'Recipient inventory/count and cleared booty slot', 'No RNG; item sound; TakeLoot composed route verified'),
        ('combat.select_weapon_mode', '0x11b43c', 'crates/core/src/lib.rs', 'Active warrior in supported battle', 'Active body and current mode', 'Warrior mode byte 0x9dd', 'No RNG; mode selection has no point cost'),
        ('combat.player_shoot', '0x11b43c', 'crates/core/src/combat/runtime.rs', 'Active starter, living target in range/LOS, two AP and loaded round', 'Mode, inventory, packet, map, target and RNG', 'Loaded rounds, weight, MP/AP, target, counters and possibly battle phase', 'Prepare, consume ammunition/costs, launch sound, resolve, death cleanup, automatic turn/victory; caller composition source-backed'),
    ]
    for ident, anchor, implementation, pre, reads, writes, effects in static_rows:
        document['actions'].append(dict(id=ident, resource='Centurions 1.3.2 CODE 1', mapped_entry_addresses=[anchor],
                                       implementation=implementation, preconditions=pre, reads=reads, writes=writes,
                                       numeric_widths='Authored signed 16-bit parameters/fields; byte cursors/flags; i32 map positions/time containers',
                                       failure='Invalid or unsupported branch rejects the entire command candidate, including RNG',
                                       random_and_ordered_effects=effects, dependencies=reads,
                                       reference_observation='Static original caller/source reconstruction and composed production command verification; live observations only where explicitly stated in coverage.md',
                                       tests='centurions-tools verify-opening/verify-fight; no independent whole-action instruction fixture claimed'))
    for fixture, ident, tests in [('live-calendar', 'reference.live_calendar', 'crates/core/tests/native/live_calendar.rs'), ('live-ammunition', 'reference.live_ammunition_collection', 'crates/core/tests/native/live_ammunition.rs')]:
        data = json.loads((ROOT / f'evidence/fixtures/{fixture}.json').read_text())
        document['actions'].append(dict(id=ident, resource='Centurions 1.3.2 CODE 1', authority=data['authority'],
            fixture=f'evidence/fixtures/{fixture}.json', case_count=len(data['cases']), tests=tests,
            reference_observation='Live application before/after captures; controlled inputs and presentation instrumentation explicitly retained'))
    document['composed_actions'].append(dict(id='exploration.sustained_opening', implementation='crates/tools/src/sustained.rs',
        verification='verify-day-two and verify-sustained, seeds 12 and 1; whole resumed state and ordered events compared after every subsequent command',
        acquisition='Authored cache converts packs directly to reserves; typed carried-pack use is verified separately'))
    extend_mission_actions(document)
    document['policy_deviations'] = [dict(
        id='registration.assumed_satisfied',
        implementation='crates/core/src/exploration.rs',
        behavior='Opcode98 continues with no registration check; production promotion always uses registered behavior; no external registration receipt is loaded or required.',
        tests='crates/core/tests/native/live_story_transactions.rs',
        historical_evidence='Registration and unregistered-promotion fixtures preserve original behavior for archaeology only; their rejected branches are not production gates.')]
    (ROOT / 'evidence/actions.json').write_text(json.dumps(document, indent=2) + '\n')
    print(f'Wrote {len(document["actions"])} action entries with explicit evidence levels')


if __name__ == '__main__':
    main()
