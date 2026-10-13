use super::*;
use crate::{MedicalContinuation, MedicalOrigin, MedicalTarget, equipment, items};
use centurions_content::{Content, mission::OpeningMission};

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

fn prepared_packet(content: &Content, mission: &OpeningMission) -> Result<Vec<u8>, RuleError> {
    let weapon = content
        .items
        .get(700)
        .ok_or_else(|| RuleError::Invalid("needle definition missing".into()))?;
    let ammunition = content
        .items
        .first()
        .ok_or_else(|| RuleError::Invalid("empty ammunition definition missing".into()))?;
    if weapon.len() != 150
        || ammunition.len() != 150
        || word(weapon, 26)? != 28
        || word(weapon, 42)? != -1
        || word(weapon, 44)? != 1401
        || word(weapon, 76)? != 1
        || word(weapon, 36)? != 5
        || word(weapon, 38)? != 1
    {
        return Err(unsupported("other medical item definition"));
    }
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("medical attack content missing".into()))?
        .attacks;
    let mut packet = spells::load_packet(attacks, 1401)?;
    packet[0] = weapon[77];
    packet[17] = weapon[55];
    packet[18] = weapon[53];
    packet[2] = packet[2].wrapping_add(word(weapon, 60)?.wrapping_add(word(ammunition, 60)?) as u8);
    packet[5] = packet[5]
        .wrapping_add(weapon[81])
        .wrapping_add(ammunition[81]);
    put_word(
        &mut packet,
        34,
        word(weapon, 62)?.wrapping_add(word(ammunition, 62)?),
    );
    if word(&packet, 34)? != 0 {
        put_word(&mut packet, 32, 1);
    }
    for (destination, source) in [
        (23, 140),
        (24, 64),
        (25, 66),
        (26, 68),
        (27, 70),
        (28, 72),
        (29, 74),
    ] {
        packet[destination] = word(weapon, source)?.wrapping_add(word(ammunition, source)?) as u8;
    }
    for (destination, source) in [(36, 12), (38, 14)] {
        let value = if word(ammunition, source)? != 0 {
            word(ammunition, source)?
        } else {
            word(weapon, source)?
        };
        put_word(&mut packet, destination, value);
    }
    packet[1] = word(weapon, 56)?.wrapping_add(word(ammunition, 56)?) as u8;
    if word(ammunition, 56)? != 0 {
        packet[1] = word(ammunition, 56)? as u8;
    }
    packet[31] = weapon[59];
    if word(ammunition, 58)? != 0 {
        packet[31] = word(ammunition, 58)? as u8;
    }
    validate_packet(&packet)?;
    Ok(packet)
}

fn validate_packet(packet: &[u8]) -> Result<(), RuleError> {
    const PREPARED: [u8; 58] = [
        1, 1, 4, 0, 0, 0, 0, 0, 0, 3, 0, 8, 16, 0, 0, 0, 0, 15, 6, 8, 8, 1, 1, 0, 0, 0, 0, 0, 0, 0,
        10, 1, 0, 0, 0, 0, 1, 175, 1, 174, 0, 57, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0,
    ];
    if packet != PREPARED {
        return Err(unsupported("other prepared medical attack"));
    }
    Ok(())
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
    if word(&warrior.bytes, 0x40)? < 1 {
        return Err(RuleError::Invalid(
            "medical item requires an action point".into(),
        ));
    }
    let item = warrior
        .inventory()?
        .get(request.slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("medical inventory slot missing".into()))?;
    if item.id() != 700
        || word(&item.0, 4)? != 0
        || word(&item.0, 6)? != 1
        || item.0[8] != 0
        || item.0[9] != 0
        || !matches!(item.0[10], 24 | 28)
        || item.0[11..13].iter().any(|value| *value != 0)
        || item.0[13] != 1
        || item.0[14..].iter().any(|value| *value != 0)
    {
        return Err(unsupported("other owned medical item"));
    }
    let quantity = word(&item.0, 2)?;
    if quantity <= 0 {
        return Err(RuleError::Invalid("medical item is exhausted".into()));
    }
    if !matches!((item.0[10], quantity), (24, 1 | 2) | (28, 1)) {
        return Err(unsupported("other owned medical item quantity"));
    }
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("medical mission missing".into()))?;
    let definition = content
        .items
        .get(700)
        .ok_or_else(|| RuleError::Invalid("needle definition missing".into()))?;
    if !equipment::can_use(warrior, definition, mission)? {
        return Err(RuleError::Invalid("warrior cannot use medical item".into()));
    }
    let packet = prepared_packet(content, mission)?;
    if !player_psi::mode_allowed(&packet, true)? {
        return Err(RuleError::Invalid(
            "medical item unavailable in combat".into(),
        ));
    }
    let mut result = warrior.clone();
    put_word(
        &mut result.bytes,
        0x132 + request.slot * 22 + 2,
        quantity.wrapping_sub(1),
    );
    let weight = word(&result.bytes, 0x9d6)?.wrapping_sub(word(definition, 36)?);
    put_word(&mut result.bytes, 0x9d6, weight);
    if quantity == 1 {
        items::remove_inventory_slot(&mut result, request.slot)?;
    }
    let mut work = combat.weapon_work;
    work.prepare(definition, &content.items[0])?;
    Ok(BeginResult {
        warrior: result,
        weapon_work: work,
        loaded_item_field32: word(definition, 32)?,
        packet,
        identity: [0, 3, 0],
        count: 1,
        events: if quantity == 1 {
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
    if !matches!(power, 1..=4) {
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
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("learned Heal mission missing".into()))?;
    let packet = learned_packet(mission)?;
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
    if combat.weapon_work.armor_filter()? {
        return Err(unsupported("retained armor-filter healing"));
    }
    let mut result = warrior.clone();
    let w = &mut result.bytes;
    let remaining = word(w, 0x58)?.wrapping_sub(cost);
    put_word(w, 0x58, remaining);
    let casts = i32::from_be_bytes(w[0xa66..0xa6a].try_into().unwrap()).wrapping_add(1);
    w[0xa66..0xa6a].copy_from_slice(&casts.to_be_bytes());
    let counter = word(w, 0xac8)?.wrapping_add(1);
    put_word(w, 0xac8, counter);
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
        loaded_item_field32: state
            .loaded_item_field32
            .ok_or_else(|| unsupported("unreconstructable independent item context"))?,
        packet,
        identity: [0, 0, 0],
        count: power,
        events,
    })
}

fn learned_packet(mission: &OpeningMission) -> Result<Vec<u8>, RuleError> {
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("learned Heal attack content missing".into()))?
        .attacks;
    let packet = spells::load_packet(attacks, 1101)?;
    const LEARNED: [u8; 58] = [
        0, 1, 255, 0, 0, 0, 0, 0, 0, 3, 0, 1, 8, 0, 0, 0, 0, 8, 13, 8, 8, 1, 1, 0, 0, 0, 0, 0, 0,
        0, 6, 0, 0, 0, 0, 0, 2, 132, 2, 112, 0, 57, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0,
        0,
    ];
    if packet != LEARNED {
        return Err(unsupported("other learned portrait Heal packet"));
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
        || combat.active_body() != Some(pending.actor as i16)
    {
        return Err(unsupported("other medical continuation"));
    }
    let packet = combat
        .attack_packet
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("medical packet missing".into()))?;
    match pending.origin {
        MedicalOrigin::OwnedNeedle => {
            if pending.item != 700
                || pending.spell != 1401
                || pending.power != 1
                || combat.attack_identity != Some([0, 3, 0])
            {
                return Err(unsupported("other owned medical continuation"));
            }
            validate_packet(packet)?;
            if combat.weapon_work.sound()? != 430 {
                return Err(unsupported("other retained medical weapon sound"));
            }
        }
        MedicalOrigin::LearnedHeal => {
            if pending.item != 0
                || pending.spell != 1101
                || !matches!(pending.power, 1..=4)
                || combat.attack_identity != Some([0, 0, 0])
            {
                return Err(unsupported("other learned medical continuation"));
            }
            if packet != &learned_packet(mission)? {
                return Err(unsupported("other prepared learned Heal packet"));
            }
            if matches!(target, MedicalTarget::LivingPoint { .. }) {
                return Err(unsupported(
                    "learned Heal point targeting through portrait continuation",
                ));
            }
        }
    }
    if combat.weapon_work.armor_filter()? {
        return Err(unsupported("other retained medical weapon work"));
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
    let (member, portrait, mut events) = match *target {
        MedicalTarget::ConfirmedCancel => {
            let w = &mut warriors[pending.actor].bytes;
            let movement = word(w, 0x3c)?.wrapping_sub(3).max(0);
            put_word(w, 0x3c, movement);
            if pending.origin == MedicalOrigin::LearnedHeal {
                let refund = match pending.power {
                    1 => 3,
                    2 => 7,
                    3 => 11,
                    4 => 15,
                    _ => return Err(unsupported("other learned portrait Heal refund")),
                };
                let points = word(w, 0x58)?.wrapping_add(refund);
                let counter = word(w, 0xac8)?.wrapping_sub(1);
                put_word(w, 0x58, points);
                put_word(w, 0xac8, counter);
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
            if !(-9..=0).contains(&word(w, 0x50)?) {
                return Err(RuleError::Invalid(
                    "medical portrait must be unconscious".into(),
                ));
            }
            (warrior, true, vec![Event::Sound { resource_id: -141 }])
        }
        MedicalTarget::LivingPoint { x, y } => {
            if word(&warriors[pending.actor].bytes, 0x40)? < 2 {
                return Err(unsupported(
                    "medical point targeting with less than two action points",
                ));
            }
            let from = combat.squad.positions[pending.actor];
            let packet = combat.attack_packet.as_ref().unwrap();
            if !player_psi::in_range(from, (x, y), player_psi::range(packet, pending.power)?) {
                return Err(RuleError::Invalid("medical target outside range".into()));
            }
            if !lifecycle::can_see(combat, mission, from, (x, y), 0)? {
                return Err(RuleError::Invalid("medical target outside sight".into()));
            }
            let cell = OpeningMission::index(
                i32::from(x) + i32::from(combat.map.shift.0) + 5,
                i32::from(y) + i32::from(combat.map.shift.1) + 5,
            )?;
            let body = combat.map.tiles[cell].wrapping_abs();
            if !(0..warriors.len() as i16).contains(&body)
                || word(&warriors[body as usize].bytes, 0x50)? <= 0
            {
                return Err(RuleError::Invalid(
                    "medical point requires a living warrior".into(),
                ));
            }
            let w = &mut warriors[pending.actor].bytes;
            let movement = word(w, 0x3c)?.wrapping_sub(3).max(0);
            let ap = word(w, 0x40)?.wrapping_sub(2);
            put_word(w, 0x3c, movement);
            put_word(w, 0x40, ap);
            (
                body as usize,
                false,
                vec![
                    Event::Sound { resource_id: 145 },
                    Event::Sound { resource_id: 431 },
                ],
            )
        }
    };
    let mut mask = [0; 110];
    mask[member] = 1;
    let result = spells::resolve_player_support(
        warriors,
        &combat.enemies.records,
        &mask,
        combat.attack_packet.as_ref().unwrap(),
        pending.spell,
        pending.power,
        random,
    )?;
    warriors.clone_from_slice(&result.warriors);
    combat.enemies.records = result.enemies;
    combat.unavailable = combat.unavailable.wrapping_sub(result.recovered as i8);
    if result.healed {
        combat.attack_packet.as_mut().unwrap()[18] = 4;
    }
    if !portrait && !result.targets.is_empty() {
        combat.field_used[0] = 1;
    }
    events.extend(
        result
            .sounds
            .into_iter()
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
        });
    }
    if portrait && word(&warriors[member].bytes, 0x50)? > 0 {
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
    for radius in 1_i16..=90 {
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
    let (position, cell, terrain) =
        placement.ok_or_else(|| unsupported("portrait revival has no verified placement"))?;
    let initiative = (1..110)
        .rev()
        .find(|index| combat.order[*index] == -1)
        .ok_or_else(|| unsupported("portrait revival has no empty initiative slot"))?;
    combat.squad.positions[member] = position;
    combat.squad.under_tiles[member] = terrain;
    combat.map.tiles[cell] = member as i16;
    combat.order[initiative] = member as i16;
    let w = &mut warriors[member].bytes;
    if word(w, 0x6a)? >= 0 {
        put_word(w, 0x6a, 3);
    }
    w[0x9de] = 0;
    put_word(w, 0x10, 0);
    put_word(w, 0x18, -1);
    w[0x9df] = 1;
    put_word(w, 0x14, 1);
    if word(w, 0x68)? >= 0 {
        put_word(w, 0x68, 2);
    }
    put_word(w, 0x20, initiative as i16);
    Ok(())
}
