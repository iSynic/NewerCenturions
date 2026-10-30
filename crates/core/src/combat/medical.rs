use crate::combat::spells::{spell_identity, spell_level};
use crate::combat::{Combat, firearm::weapon_traits, lifecycle, player_psi, spells};
use crate::{Event, Phase, RandomState, RuleError, Session};
use crate::{MedicalContinuation, MedicalOrigin, MedicalTarget, equipment, items};
use centurions_content::{Content, mission::OpeningMission};
use centurions_content::{
    Warrior,
    records::{
        CharacterStatisticsMut, Condition, ItemDefinition, SpellInfoRecord, Statistic,
        WarriorRecord, WarriorRecordMut,
    },
};

pub(in crate::combat) mod preparation;
use preparation::{prepared_packet, validate_packet};

/// The warrior and inventory slot of the medicine item to use.
pub struct BeginRequest {
    pub warrior: usize,
    pub slot: usize,
}

/// What using a medical item changes before its target is chosen: the warrior with the item spent,
/// the weapon state, the attack packet, and the spell identity and power it casts with.
pub struct BeginResult {
    pub warrior: Warrior,
    pub weapon_traits: weapon_traits::WeaponTraits,
    pub loaded_item_field32: i16,
    pub packet: Vec<u8>,
    pub identity: [i16; 3],
    pub count: i16,
    pub events: Vec<Event>,
}

/// What the turn does after healing: finish the warrior's action, end the turn, or nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUp {
    FinishPlayerAction,
    EndTurn,
    None,
}

/// What healing did, and what the turn does next.
pub struct MedicalResolution {
    pub events: Vec<Event>,
    pub follow_up: FollowUp,
}

/// Reports a healing case this build cannot yet run, without changing anything.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.medical",
        dependency,
    }
}

/// Starts a warrior using a medicine item: the warrior must hold the turn with an attack left, be
/// able to use the item, and have a charge (unless the item supplies unlimited quantity). One
/// charge is spent; the last charge of an item that is discarded when empty removes it, with a
/// sound.
///
/// Original: combatchoice
pub fn begin(
    state: &Session,
    content: &Content,
    request: BeginRequest,
) -> Result<BeginResult, RuleError> {
    if state.phase != Phase::Combat
        || state.combat_story.is_some()
        || state.medical_continuation.is_some()
    {
        return Err(RuleError::Invalid(
            "medical item requires an unobstructed combat turn".into(),
        ));
    }
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("medical combat missing".into()))?;
    if combat.active_body() != Some(request.warrior as i16) || request.warrior >= 6 {
        return Err(RuleError::Invalid(
            "medical item requires the active warrior".into(),
        ));
    }
    let warrior = state
        .warriors
        .get(request.warrior)
        .ok_or_else(|| RuleError::Invalid("medical warrior missing".into()))?;
    if warrior.record().action_points()? < 1 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let item = warrior
        .inventory()?
        .get(request.slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("medical inventory slot missing".into()))?;
    let quantity = item.charge()?;
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("medical mission missing".into()))?;
    let definition = content
        .items
        .get(item.id() as usize)
        .ok_or_else(|| RuleError::Invalid("needle definition missing".into()))?;
    if !equipment::can_use(warrior, definition, mission)? {
        return Err(RuleError::Invalid("warrior cannot use medical item".into()));
    }
    if quantity <= 0 && ItemDefinition(definition).supplied_quantity()? >= 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let packet = prepared_packet(content, mission, item.id(), warrior)?;
    if !player_psi::mode_allowed(&packet, true)? {
        return Err(RuleError::Invalid(
            "medical item unavailable in combat".into(),
        ));
    }
    let mut result = warrior.clone();
    if quantity > 0 {
        result
            .record_mut()
            .set_inventory_quantity(request.slot, quantity.wrapping_sub(1));
        let weight = result
            .record()
            .load()?
            .wrapping_sub(ItemDefinition(definition).charge_weight()?);
        result.record_mut().set_load(weight);
        if quantity == 1 && ItemDefinition(definition).discarded_when_empty()? != 0 {
            items::remove_inventory_slot(&mut result, request.slot)?;
        }
    }
    let mut traits = combat.weapon_traits;
    traits.prepare(definition, &content.items[0])?;
    Ok(BeginResult {
        warrior: result,
        weapon_traits: traits,
        loaded_item_field32: ItemDefinition(definition).ammunition_caliber()?,
        packet,
        identity: {
            let spell = ItemDefinition(definition).spell()?;
            spell_identity(spell)
        },
        // In battle the weapon's burst count replaces the item's power.
        count: ItemDefinition(definition).attack_count()?,
        events: if quantity == 1 && ItemDefinition(definition).discarded_when_empty()? != 0 {
            vec![Event::Sound { resource_id: 655 }]
        } else {
            Vec::new()
        },
    })
}

/// Starts a warrior casting the learned Heal spell at power 1 to 7. The warrior must be able to
/// cast, know Heal and have the psi; the psi is spent and the cast counted.
pub fn begin_learned_heal(
    state: &Session,
    content: &Content,
    power: i16,
) -> Result<BeginResult, RuleError> {
    if state.phase != Phase::Combat
        || state.combat_story.is_some()
        || state.medical_continuation.is_some()
    {
        return Err(RuleError::Invalid(
            "learned Heal requires an unobstructed combat turn".into(),
        ));
    }
    if !matches!(power, 1..=7) {
        return Err(unsupported("other learned portrait Heal power"));
    }
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("learned Heal combat missing".into()))?;
    let actor = combat
        .active_body()
        .filter(|actor| (0..6).contains(actor))
        .ok_or_else(|| RuleError::Invalid("learned Heal requires an active warrior".into()))?;
    if !player_psi::can_cast(&state.warriors, actor, state.psi_disabled)? {
        return Err(RuleError::Invalid(
            "warrior cannot cast learned Heal".into(),
        ));
    }
    let warrior = &state.warriors[actor as usize];
    if !warrior.knows_spell(1101)? {
        return Err(RuleError::Invalid("warrior has not learned Heal".into()));
    }
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("learned Heal mission missing".into()))?;
    let packet = learned_packet(mission, 1101)?;
    if !player_psi::mode_allowed(&packet, true)? {
        return Err(RuleError::Invalid(
            "learned Heal unavailable in combat".into(),
        ));
    }
    let cost = player_psi::cost(&packet, power)?;
    if warrior.psi_points()? < cost {
        return Err(RuleError::Invalid(
            "insufficient psi for learned Heal".into(),
        ));
    }
    combat.weapon_traits.armor_filter();
    let mut result = warrior.clone();
    let remaining = result.record().psi_points()?.wrapping_sub(cost);
    result.record_mut().set_psi_points(remaining);
    CharacterStatisticsMut(&mut result.bytes).add(Statistic::ScansCast, 1);
    let counter = result.record().spells_so_far()?.wrapping_add(1);
    result.record_mut().set_spells_so_far(counter);
    let mut events = vec![
        Event::Sound { resource_id: 141 },
        Event::Sound { resource_id: 20002 },
        Event::Sound { resource_id: 141 },
    ];
    if power != 1 {
        events.push(Event::Sound { resource_id: 137 });
    }
    Ok(BeginResult {
        warrior: result,
        weapon_traits: combat.weapon_traits,
        loaded_item_field32: state.globals.loaded_item_field32,
        packet,
        identity: [0, 0, 0],
        count: power,
        events,
    })
}

/// The attack packet of a learned healing spell, which must target one warrior and have effect 57.
fn learned_packet(mission: &OpeningMission, spell: i16) -> Result<Vec<u8>, RuleError> {
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or("learned healing content missing")?
        .attacks;
    let packet = spells::load_packet(attacks, spell)?;
    if SpellInfoRecord(&packet).target_type() != 1 || SpellInfoRecord(&packet).effect()? != 57 {
        return Err(unsupported("scan has no original portrait healing handler"));
    }
    Ok(packet)
}

/// Applies the chosen healing target. The result is all or nothing: on an error the battle, the
/// warriors and the random stream are left as they were.
pub fn resolve(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    pending: &MedicalContinuation,
    target: &MedicalTarget,
) -> Result<MedicalResolution, RuleError> {
    if pending.actor >= warriors.len()
        || !(1..=6).contains(&warriors.len())
        || pending.origin != MedicalOrigin::OwnedItem
            && combat.active_body() != Some(pending.actor as i16)
    {
        return Err(unsupported("other medical continuation"));
    }
    let packet = &combat.spell_info;
    match pending.origin {
        MedicalOrigin::OwnedNeedle => {
            if !(700..=703).contains(&pending.item)
                || pending.spell != 1401
                || pending.power != 1
                || combat.cast_identity != [0, 3, 0]
            {
                return Err(unsupported("other owned medical continuation"));
            }
            validate_packet(packet)?;
            if combat.weapon_traits.sound() != 430 {
                return Err(unsupported("other retained medical weapon sound"));
            }
        }
        MedicalOrigin::OwnedItem => {
            if pending.item <= 0
                || pending.spell <= 1100
                || combat.cast_identity != spell_identity(pending.spell)
                || SpellInfoRecord(packet).target_type() != 1
                || SpellInfoRecord(packet).effect()? != 57
            {
                return Err("invalid owned item healing continuation".into());
            }
            validate_packet(packet)?;
        }
        MedicalOrigin::LearnedHeal => {
            if pending.item != 0
                || !matches!(pending.power, 1..=7)
                || combat.cast_identity != spell_identity(pending.spell)
            {
                return Err(unsupported("other learned medical continuation"));
            }
            if packet != &learned_packet(mission, pending.spell)? {
                return Err(unsupported("other prepared learned Heal packet"));
            }
            if matches!(target, MedicalTarget::LivingPoint { .. }) {
                return Err(unsupported(
                    "learned Heal point targeting through portrait continuation",
                ));
            }
        }
    }
    let mut next_combat = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let result = resolve_inner(
        &mut next_combat,
        &mut next_warriors,
        mission,
        &mut next_random,
        pending,
        target,
    )?;
    *combat = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok(result)
}

/// Applies the chosen target. Cancelling costs the warrior three movement and refunds a learned
/// Heal's psi. A fallen warrior's portrait, or a living warrior in range and in sight, is healed; a
/// revived warrior returns to the map. Healing through a portrait ends the turn.
fn resolve_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    pending: &MedicalContinuation,
    target: &MedicalTarget,
) -> Result<MedicalResolution, RuleError> {
    let mut flights = vec![];
    let (member, portrait, mut events) = match *target {
        MedicalTarget::ConfirmedCancel => {
            combat.in_spell = 0;
            let w = &mut warriors[pending.actor].bytes;
            let movement = WarriorRecord::from_bytes(w)
                .movement()?
                .wrapping_sub(3)
                .max(0);
            WarriorRecordMut::from_bytes(w).set_movement(movement);
            if pending.origin == MedicalOrigin::LearnedHeal {
                let refund = pending.power.wrapping_mul(4).wrapping_sub(1);
                let points = WarriorRecord::from_bytes(w)
                    .psi_points()?
                    .wrapping_add(refund);
                let counter = WarriorRecord::from_bytes(w)
                    .spells_so_far()?
                    .wrapping_sub(1);
                WarriorRecordMut::from_bytes(w).set_psi_points(points);
                WarriorRecordMut::from_bytes(w).set_spells_so_far(counter);
            }
            return Ok(MedicalResolution {
                events: vec![
                    Event::Sound { resource_id: 6000 },
                    Event::Sound { resource_id: 20005 },
                ],
                follow_up: FollowUp::None,
            });
        }
        MedicalTarget::Portrait { warrior } => {
            let w = &warriors
                .get(warrior)
                .ok_or_else(|| RuleError::Invalid("medical portrait outside party".into()))?
                .bytes;
            if !(-9..=0).contains(&WarriorRecord::from_bytes(w).stamina()?) {
                return Err(RuleError::Refused {
                    feedback: vec![crate::feedback::GameFeedback::Sound { resource_id: 143 }],
                });
            }
            (warrior, true, vec![Event::Sound { resource_id: -141 }])
        }
        MedicalTarget::LivingPoint { x, y } => {
            if warriors[pending.actor].record().action_points()? < 1 {
                return Err(RuleError::Invalid(
                    "medical continuation has no action point".into(),
                ));
            }
            let from = combat.squad.pos[pending.actor];
            let packet = &combat.spell_info;
            if !player_psi::in_range(from, (x, y), player_psi::range(packet, pending.power)?) {
                return Err(RuleError::Refused {
                    feedback: vec![crate::feedback::GameFeedback::Combat {
                        kind: crate::feedback::CombatRefusal::OutOfRange,
                    }],
                });
            }
            if !lifecycle::can_see(combat, mission, from, (x, y), 0)? {
                return Err(RuleError::Refused {
                    feedback: vec![crate::feedback::GameFeedback::Combat {
                        kind: crate::feedback::CombatRefusal::Blocked,
                    }],
                });
            }
            let cell = crate::combat::records::grid_index((x, y), combat.field.map_shift)?;
            let body = combat.field.tiles[cell].wrapping_abs();
            if !(0..warriors.len() as i16).contains(&body)
                || warriors[body as usize].record().stamina()? <= 0
            {
                return Err(RuleError::Invalid(
                    "medical point requires a living warrior".into(),
                ));
            }
            crate::combat::records::spend_attack_action(&mut warriors[pending.actor])?;
            // The launch sound plays, then the flight's draws come before the
            // warrior is treated.
            let launch = Event::Sound {
                resource_id: SpellInfoRecord(&combat.spell_info).launch_sound()?,
            };
            let jitter = spells::flight_jitter(&combat.spell_info, random);
            let triple = spells::SpellTarget {
                first: body,
                second: 0,
                kind: 1,
            };
            flights.push((
                1,
                spells::flight(
                    combat,
                    pending.actor as i16,
                    &triple,
                    &combat.spell_info,
                    1,
                    jitter,
                ),
            ));
            (
                body as usize,
                false,
                vec![Event::Sound { resource_id: 145 }, launch],
            )
        }
    };
    let mut mask = [0; 110];
    combat.blank_round = 0;
    mask[member] = 1;
    combat.in_spell = i16::from(!portrait);
    let result = spells::resolve_support_with_context(
        warriors,
        &combat.roster.records,
        &mask,
        pending.spell,
        &spells::DirectSpell {
            packet: &combat.spell_info,
            level: spell_level(pending.spell),
            count: pending.power,
            armor_filter: combat.weapon_traits.armor_filter(),
            weapon_sound: 0,
        },
        random,
    )?;
    combat.in_spell = 0;
    warriors.clone_from_slice(&result.warriors);
    combat.kill_counts.add_num_enemy(result.total_delta);
    combat.roster.records = result.enemies;
    combat.kill_party = combat.kill_party.wrapping_sub(result.recovered as i8);
    if result.healed {
        combat.spell_info[18] = 4;
    }
    if !portrait && !result.targets.is_empty() {
        combat.collide_check[0] = 1;
    }
    events.extend(
        result
            .sounds
            .into_iter()
            .chain(result.race_excluded.into_iter().map(|_| 651))
            .map(|resource_id| Event::Sound { resource_id }),
    );
    for (body, outcome) in result.targets {
        events.extend(
            outcome
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        events.push(Event::SpellResolved {
            caster: Some(pending.actor as i16),
            spell: pending.spell,
            target: body as i16,
            damage: outcome.damage,
            defeated: false,
            icon: crate::event::PresentationIcon(outcome.icon),
            flight: Default::default(),
        });
    }
    crate::event::Flight::attach(&mut events, &flights);
    if portrait && warriors[member].record().stamina()? > 0 {
        restore_portrait(combat, warriors, mission, member)?;
        events.push(Event::Sound { resource_id: 631 });
    }
    Ok(MedicalResolution {
        events,
        follow_up: if portrait {
            FollowUp::EndTurn
        } else {
            FollowUp::FinishPlayerAction
        },
    })
}

/// Puts a revived warrior back on the map, searching outward from the warrior's place in the
/// starting formation for an open cell.
fn restore_portrait(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    member: usize,
) -> Result<(), RuleError> {
    let row = (member as i16 / 3).wrapping_abs();
    let column = (member as i16).wrapping_sub(row.wrapping_mul(3));
    let origin = (
        combat.field.map_shift.0.wrapping_add(5),
        combat.field.map_shift.1.wrapping_add(5),
    );
    let mut placement = None;
    // The search widens without limit; past this radius every cell has been
    // tried, so finding none means the game never returns.
    let reach = |center: i32| (center - 7).max(83 - center).max(1);
    let last = reach(i32::from(row.wrapping_add(column)) + 5 + i32::from(origin.0))
        .max(reach(i32::from(row) + 5 + i32::from(origin.1)));
    for radius in 1..=last.min(i32::from(i16::MAX)) as i16 {
        for dy in -radius..radius {
            for dx in -radius..radius {
                let position = (
                    row.wrapping_add(column).wrapping_add(dx).wrapping_add(5),
                    row.wrapping_add(dy).wrapping_add(5),
                );
                let map = (
                    i32::from(position.0) + i32::from(origin.0),
                    i32::from(position.1) + i32::from(origin.1),
                );
                if !(7..83).contains(&map.0) || !(7..83).contains(&map.1) {
                    continue;
                }
                let cell = OpeningMission::index(map.0, map.1)?;
                let terrain = combat.field.tiles[cell];
                if terrain > 999 && mission.tile_word(terrain.wrapping_sub(1000), 4)? == 0 {
                    placement = Some((position, cell, terrain));
                    break;
                }
            }
            if placement.is_some() {
                break;
            }
        }
        if placement.is_some() {
            break;
        }
    }
    let (position, cell, terrain) = placement.ok_or_else(|| {
        unsupported("portrait revival has no passable cell; the original search never ends")
    })?;
    // Slots 1..109 outnumber the 106 possible bodies, so one is always empty.
    let initiative = (1..110)
        .rev()
        .find(|index| combat.battle_order[*index] == -1)
        .ok_or_else(|| RuleError::Invalid("initiative order has no empty slot".into()))?;
    combat.squad.pos[member] = position;
    combat.squad.char_under[member] = terrain;
    combat.field.tiles[cell] = member as i16;
    combat.battle_order[initiative] = member as i16;
    let w = &mut warriors[member].bytes;
    if WarriorRecord::from_bytes(w).condition(Condition::MovingSlowly)? >= 0 {
        WarriorRecordMut::from_bytes(w).set_condition(Condition::MovingSlowly, 3);
    }
    WarriorRecordMut::from_bytes(w).set_bleeding(0);
    WarriorRecordMut::from_bytes(w).set_faction(0);
    WarriorRecordMut::from_bytes(w).set_target(-1);
    WarriorRecordMut::from_bytes(w).set_in_battle(1);
    WarriorRecordMut::from_bytes(w).set_been_attacked(1);
    if WarriorRecord::from_bytes(w).condition(Condition::ScanBlocked)? >= 0 {
        WarriorRecordMut::from_bytes(w).set_condition(Condition::ScanBlocked, 2);
    }
    WarriorRecordMut::from_bytes(w).set_position(initiative as i16);
    Ok(())
}
