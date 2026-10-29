use crate::combat::{Combat, lifecycle, player_psi, spells, weapon_work};
use crate::{Event, Phase, RandomState, RuleError, Session};
use crate::{MedicalContinuation, MedicalOrigin, MedicalTarget, equipment, items};
use centurions_content::{Content, mission::OpeningMission};
use centurions_content::{
    Warrior,
    records::{
        AttackPacket, CharacterStatisticsMut, Condition, ItemDefinition, Statistic, WarriorRecord,
        WarriorRecordMut,
    },
};

pub(in crate::combat) mod preparation;
use preparation::{prepared_packet, validate_packet};

pub struct BeginRequest {
    pub warrior: usize,
    pub slot: usize,
}

pub struct BeginResult {
    pub warrior: Warrior,
    pub weapon_work: weapon_work::WeaponWork,
    pub loaded_item_field32: i16,
    pub packet: Vec<u8>,
    pub identity: [i16; 3],
    pub count: i16,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUp {
    FinishPlayerAction,
    EndTurn,
    None,
}

pub struct MedicalResolution {
    pub events: Vec<Event>,
    pub follow_up: FollowUp,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.medical",
        dependency,
    }
}

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
    let quantity = item.quantity()?;
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
            .carried_weight()?
            .wrapping_sub(ItemDefinition(definition).unit_weight()?);
        result.record_mut().set_carried_weight(weight);
        if quantity == 1 && ItemDefinition(definition).discarded_when_empty()? != 0 {
            items::remove_inventory_slot(&mut result, request.slot)?;
        }
    }
    let mut work = combat.weapon_work;
    work.prepare(definition, &content.items[0])?;
    Ok(BeginResult {
        warrior: result,
        weapon_work: work,
        loaded_item_field32: ItemDefinition(definition).retained_field32()?,
        packet,
        identity: {
            let spell = ItemDefinition(definition).spell()?;
            [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1]
        },
        // Combatchoice1190b8 replaces the outdoor power with the weapon burst count.
        count: ItemDefinition(definition).attack_count()?,
        events: if quantity == 1 && ItemDefinition(definition).discarded_when_empty()? != 0 {
            vec![Event::Sound { resource_id: 655 }]
        } else {
            Vec::new()
        },
    })
}

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
    combat.weapon_work.armor_filter();
    let mut result = warrior.clone();
    let remaining = result.record().psi_points()?.wrapping_sub(cost);
    result.record_mut().set_psi_points(remaining);
    CharacterStatisticsMut(&mut result.bytes).add(Statistic::ScansCast, 1);
    let counter = result.record().scans_used()?.wrapping_add(1);
    result.record_mut().set_scans_used(counter);
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
        weapon_work: combat.weapon_work,
        loaded_item_field32: state.native_work.loaded_item_field32,
        packet,
        identity: [0, 0, 0],
        count: power,
        events,
    })
}

fn learned_packet(mission: &OpeningMission, spell: i16) -> Result<Vec<u8>, RuleError> {
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or("learned healing content missing")?
        .attacks;
    let packet = spells::load_packet(attacks, spell)?;
    if AttackPacket(&packet).targeting_mode() != 1 || AttackPacket(&packet).effect()? != 57 {
        return Err(unsupported("scan has no original portrait healing handler"));
    }
    Ok(packet)
}

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
    let packet = &combat.attack_packet;
    match pending.origin {
        MedicalOrigin::OwnedNeedle => {
            if !(700..=703).contains(&pending.item)
                || pending.spell != 1401
                || pending.power != 1
                || combat.attack_identity != [0, 3, 0]
            {
                return Err(unsupported("other owned medical continuation"));
            }
            validate_packet(packet)?;
            if combat.weapon_work.sound() != 430 {
                return Err(unsupported("other retained medical weapon sound"));
            }
        }
        MedicalOrigin::OwnedItem => {
            if pending.item <= 0
                || pending.spell <= 1100
                || combat.attack_identity
                    != [
                        pending.spell / 1000 - 1,
                        pending.spell % 1000 / 100 - 1,
                        pending.spell % 100 - 1,
                    ]
                || AttackPacket(packet).targeting_mode() != 1
                || AttackPacket(packet).effect()? != 57
            {
                return Err("invalid owned item healing continuation".into());
            }
            validate_packet(packet)?;
        }
        MedicalOrigin::LearnedHeal => {
            if pending.item != 0
                || !matches!(pending.power, 1..=7)
                || combat.attack_identity
                    != [
                        pending.spell / 1000 - 1,
                        pending.spell % 1000 / 100 - 1,
                        pending.spell % 100 - 1,
                    ]
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
            combat.ready = 0;
            let w = &mut warriors[pending.actor].bytes;
            let movement = WarriorRecord::from_bytes(w)
                .movement_points()?
                .wrapping_sub(3)
                .max(0);
            WarriorRecordMut::from_bytes(w).set_movement_points(movement);
            if pending.origin == MedicalOrigin::LearnedHeal {
                let refund = pending.power.wrapping_mul(4).wrapping_sub(1);
                let points = WarriorRecord::from_bytes(w)
                    .psi_points()?
                    .wrapping_add(refund);
                let counter = WarriorRecord::from_bytes(w).scans_used()?.wrapping_sub(1);
                WarriorRecordMut::from_bytes(w).set_psi_points(points);
                WarriorRecordMut::from_bytes(w).set_scans_used(counter);
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
                    feedback: vec![crate::feedback::NativeFeedback::Sound { resource_id: 143 }],
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
            let from = combat.squad.positions[pending.actor];
            let packet = &combat.attack_packet;
            if !player_psi::in_range(from, (x, y), player_psi::range(packet, pending.power)?) {
                return Err(RuleError::Refused {
                    feedback: vec![crate::feedback::NativeFeedback::Combat {
                        kind: crate::feedback::CombatRefusal::OutOfRange,
                    }],
                });
            }
            if !lifecycle::can_see(combat, mission, from, (x, y), 0)? {
                return Err(RuleError::Refused {
                    feedback: vec![crate::feedback::NativeFeedback::Combat {
                        kind: crate::feedback::CombatRefusal::Blocked,
                    }],
                });
            }
            let cell = OpeningMission::index(
                i32::from(x) + i32::from(combat.map.shift.0) + 5,
                i32::from(y) + i32::from(combat.map.shift.1) + 5,
            )?;
            let body = combat.map.tiles[cell].wrapping_abs();
            if !(0..warriors.len() as i16).contains(&body)
                || warriors[body as usize].record().stamina()? <= 0
            {
                return Err(RuleError::Invalid(
                    "medical point requires a living warrior".into(),
                ));
            }
            let w = &mut warriors[pending.actor].bytes;
            let movement = WarriorRecord::from_bytes(w)
                .movement_points()?
                .wrapping_sub(3)
                .max(0);
            let ap = WarriorRecord::from_bytes(w)
                .action_points()?
                .wrapping_sub(2);
            WarriorRecordMut::from_bytes(w).set_movement_points(movement);
            WarriorRecordMut::from_bytes(w).set_action_points(ap);
            // combatchoice 0x11af6c calls cast with the clicked warrior's triple:
            // its launch sound, then the flight's draws before the body resolves.
            let launch = Event::Sound {
                resource_id: AttackPacket(&combat.attack_packet).launch_sound()?,
            };
            let jitter = spells::flight_jitter(&combat.attack_packet, random);
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
                    &combat.attack_packet,
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
    combat.inactive_rounds = 0;
    mask[member] = 1;
    combat.ready = i16::from(!portrait);
    let result = spells::resolve_support_with_context(
        warriors,
        &combat.enemies.records,
        &mask,
        pending.spell,
        &spells::DirectSpell {
            packet: &combat.attack_packet,
            level: pending.spell % 1000 / 100 - 1,
            count: pending.power,
            armor_filter: combat.weapon_work.armor_filter(),
            weapon_sound: 0,
        },
        random,
    )?;
    combat.ready = 0;
    warriors.clone_from_slice(&result.warriors);
    combat.victory_work.add_total(result.total_delta);
    combat.enemies.records = result.enemies;
    combat.unavailable = combat.unavailable.wrapping_sub(result.recovered as i8);
    if result.healed {
        combat.attack_packet[18] = 4;
    }
    if !portrait && !result.targets.is_empty() {
        combat.field_used[0] = 1;
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

fn restore_portrait(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    member: usize,
) -> Result<(), RuleError> {
    let q = (member as i16 / 3).wrapping_abs();
    let r = (member as i16).wrapping_sub(q.wrapping_mul(3));
    let origin = (
        combat.map.shift.0.wrapping_add(5),
        combat.map.shift.1.wrapping_add(5),
    );
    let mut placement = None;
    // The original widens the square without limit; past this radius every
    // in-window cell has been tried, so finding none means it never ends.
    let reach = |center: i32| (center - 7).max(83 - center).max(1);
    let last = reach(i32::from(q.wrapping_add(r)) + 5 + i32::from(origin.0))
        .max(reach(i32::from(q) + 5 + i32::from(origin.1)));
    for radius in 1..=last.min(i32::from(i16::MAX)) as i16 {
        for dy in -radius..radius {
            for dx in -radius..radius {
                let position = (
                    q.wrapping_add(r).wrapping_add(dx).wrapping_add(5),
                    q.wrapping_add(dy).wrapping_add(5),
                );
                let map = (
                    i32::from(position.0) + i32::from(origin.0),
                    i32::from(position.1) + i32::from(origin.1),
                );
                if !(7..83).contains(&map.0) || !(7..83).contains(&map.1) {
                    continue;
                }
                let cell = OpeningMission::index(map.0, map.1)?;
                let terrain = combat.map.tiles[cell];
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
        .find(|index| combat.order[*index] == -1)
        .ok_or_else(|| RuleError::Invalid("initiative order has no empty slot".into()))?;
    combat.squad.positions[member] = position;
    combat.squad.under_tiles[member] = terrain;
    combat.map.tiles[cell] = member as i16;
    combat.order[initiative] = member as i16;
    let w = &mut warriors[member].bytes;
    if WarriorRecord::from_bytes(w).condition(Condition::MovingSlowly)? >= 0 {
        WarriorRecordMut::from_bytes(w).set_condition(Condition::MovingSlowly, 3);
    }
    WarriorRecordMut::from_bytes(w).set_bleeding_flag(0);
    WarriorRecordMut::from_bytes(w).set_faction(0);
    WarriorRecordMut::from_bytes(w).set_target_body(-1);
    WarriorRecordMut::from_bytes(w).set_active_flag(1);
    WarriorRecordMut::from_bytes(w).set_damage_marker(1);
    if WarriorRecord::from_bytes(w).condition(Condition::ScanBlocked)? >= 0 {
        WarriorRecordMut::from_bytes(w).set_condition(Condition::ScanBlocked, 2);
    }
    WarriorRecordMut::from_bytes(w).set_defeated_marker(initiative as i16);
    Ok(())
}
