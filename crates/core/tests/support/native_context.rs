use centurions_core::Session;

/// Historical native snapshots do not record WORD4435d4. All their existing
/// fields remain exact; added work has separate original/source command controls.
/// Encounter headers, labels, Ready WORD438760 and the two processing bytes are
/// also absent from these historical whole-Session captures, as are retained initiative
/// and the neighbor presence/list work. Reconstruction turn-notification rounds
/// and retained copies of round globals are outside those original record snapshots.
/// They also omit character-work WORD447d96, covered by its separate native cases.
/// Retained shop work has the same separate native coverage.
/// Walking's selector and saved retry locals have separate native cases as well.
/// The early-booty byte is also absent from these historical snapshots.
/// NPC attempt/inhibit work and its transient attack journal are unrecorded as well.
/// Boat status, facing and shore-attempt work are absent from these captures.
pub fn captured_session(state: &Session) -> Session {
    let mut captured = state.clone();
    captured.native_work.boat = 0;
    captured.native_work.boat_facing = 0;
    captured.native_work.shore_attempts = 0;
    captured.native_work.npc_work = Default::default();
    captured.native_work.early_booty = 0;
    captured.native_work.walking_confusion = 0;
    captured.native_work.spell_target_tail = vec![0; 104];
    captured.native_work.target_work = Default::default();
    captured.native_work.monster_work = Default::default();
    captured.native_work.bestiary = Default::default();
    let mut current = captured.combat_story.as_mut();
    while let Some(story) = current {
        if let Some(caller) = story.return_to.as_mut() {
            let mut document = serde_json::to_value(&**caller).unwrap();
            let row = document["WarriorMove"].as_object_mut().unwrap();
            row.remove("locals");
            **caller = serde_json::from_value(document).unwrap();
        }
        current = story.parent.as_deref_mut();
    }
    captured.native_work.character_bare_hand_bonus = 0;
    captured.native_work.shop_work = Default::default();
    captured.native_work.turn_round = None;
    captured.native_work.round_work = Default::default();
    captured.ordinary_battle_rewards = None;
    captured.recovery_return = None;
    captured.native_work.battle_rewards_suppressed = 0;
    captured.native_work.recovery_work = Default::default();
    captured.native_work.npc_backing.active_count = 0;
    captured.native_work.treasure_record = vec![0; 88];
    captured.native_work.extra_record = vec![0; 10];
    captured.native_work.tactical_work = centurions_core::combat::tactical_work::TacticalWork {
        map: centurions_core::combat::TacticalMap {
            tiles: vec![0; 8100],
            edge_word: 0,
            shift: (0, 0),
        },
        saved_land: None,
        positions: [(0, 0); 6],
        under_tiles: [0; 6],
        spell_fields: vec![0; 1200],
        field_used: vec![0; 128],
        battle: None,
    };
    captured.native_work.victory_work = Default::default();
    captured.native_work.neighbor_work = Default::default();
    captured.native_work.weapon_work.natural_attack_index = 0;
    captured.native_work.encounter_work = Default::default();
    captured.native_work.ready = 0;
    captured.native_work.death_processing = Default::default();
    captured.native_work.initiative_work = centurions_core::combat::retained::InitiativeWork {
        order: vec![0; 110],
        cursor: 0,
    };
    if let Some(combat) = &mut captured.combat {
        combat.npc_inhibit_write = None;
        combat.target_tail = vec![0; 104];
        combat.target_work = Default::default();
        combat.retained_npcs.active_count = 0;
        combat.victory_work = Default::default();
        combat.neighbor_work = Default::default();
        combat.ready = 0;
        combat.weapon_work.natural_attack_index = 0;
    }
    captured
}

/// A source-composed caller around a record-only receipt needs a declared
/// history for the new counter pair. This control constructs one possible
/// combatsetup/killbody history: each nonremoved nonzero-faction body entered
/// the battle once, and each already defeated such body was killed once.
/// It does not decode or qualify unrecorded native history. Call only while
/// assembling an input, never after an action or in a capture projection.
pub fn source_victory_context(state: &mut Session) {
    source_victory_context_recorded(state, None, None);
}

/// As `source_victory_context`, keeping any counter the capture recorded.
pub fn source_victory_context_recorded(
    state: &mut Session,
    recorded_total: Option<i16>,
    recorded_defeated: Option<i16>,
) {
    let combat = state.combat.as_mut().unwrap();
    let mut total = 0_i16;
    let mut defeated = 0_i16;
    for record in &combat.enemies.records {
        if record.record().faction().unwrap() != 0 && record.record().mechanical_state() != 255 {
            total += 1;
            defeated += i16::from(record.stamina().unwrap() <= 0);
        }
    }
    // The constructed party's nonzero factions are control acquisitions.
    for record in &state.warriors {
        if record.record().faction().unwrap() != 0 {
            total += 1;
            defeated += i16::from(record.stamina().unwrap() <= 0);
        }
    }
    let dead = recorded_defeated.unwrap_or(defeated);
    combat.victory_work.defeated = dead;
    // A recorded defeated prefix can include bodies outside this helper slice.
    combat.victory_work.total =
        recorded_total.unwrap_or(total.wrapping_sub(defeated).wrapping_add(dead));
    state.native_work.victory_work = combat.victory_work;
}

/// Deserialize a record foundation with the declared source caller history.
pub fn source_frame(value: serde_json::Value) -> Session {
    let mut state = serde_json::from_value(value).unwrap();
    source_victory_context(&mut state);
    state
}
