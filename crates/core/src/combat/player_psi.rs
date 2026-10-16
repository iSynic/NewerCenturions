use super::*;
use centurions_content::mission::OpeningMission;

pub const COMBAT_SPELLS: &[i16] = &[
    1101, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1301, 1302, 1303, 1304, 1305, 1306, 1307, 1401,
    1402, 1404, 1405, 1407, 1503, 1504, 1505, 1507, 1601, 1602, 1604, 1701, 1702, 1703, 1704, 1706,
    1707, 2101, 2102, 2104, 2105, 2106, 2107, 2202, 2203, 2204, 2205, 2206, 2207, 2301, 2302, 2303,
    2304, 2306, 2307, 2402, 2403, 2404, 2406, 2407, 2501, 2502, 2503, 2504, 2507, 2601, 2602, 2603,
    2605, 2606, 2702, 2703, 2704, 2705, 2706, 3102, 3106, 3107, 3201, 3202, 3203, 3204, 3205, 3206,
    3301, 3302, 3303, 3304, 3305, 3306, 3402, 3403, 3404, 3405, 3406, 3407, 3501, 3502, 3504, 3505,
    3507, 3601, 3602, 3603, 3605, 3701, 3702, 3703, 3704, 3706, 3707,
];

pub const OUTDOOR_PARTY_SPELLS: &[i16] = &[
    1105, 1301, 1302, 1303, 1304, 1305, 1306, 1307, 1505, 1706, 3401,
];
pub const OUTDOOR_TARGET_SPELLS: &[i16] = &[
    1101, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1401, 1405, 1503, 1602, 1604, 1703, 1704, 1705,
    3201, 3202, 3203, 3204, 3301, 3303, 3304, 3305, 3505, 3603, 3605, 3703, 3704, 3705, 3707,
];

pub const PARTY_SPELLS: &[i16] = &[1102, 1103, 1407, 1501, 1507, 1605, 3101, 3103, 3406, 3606];

#[derive(Clone, Copy, Debug)]
pub struct CastRequest {
    pub rotation: u8,
    pub spell: i16,
    pub power: i16,
    pub target: Option<(i16, i16)>,
}

#[derive(Clone, Copy, Debug)]
pub struct PartyCastRequest {
    pub warrior: usize,
    pub spell: i16,
    pub power: i16,
}

pub struct PartyCastResult {
    pub warriors: Vec<Warrior>,
    pub memory: Vec<u8>,
    pub buffs: Vec<u8>,
    pub packet: Vec<u8>,
    pub identity: [i16; 3],
    pub count: i16,
    pub scenario_duration: Option<i16>,
    pub events: Vec<Event>,
}

pub fn cast_party(
    warriors: &[Warrior],
    memory: &[u8],
    buffs: &[u8],
    mission: &OpeningMission,
    random: &mut RandomState,
    disabled: u8,
    request: PartyCastRequest,
) -> Result<PartyCastResult, RuleError> {
    if memory.len() != 480 || buffs.len() != 20 {
        return Err(RuleError::Invalid("outdoor psi input size".into()));
    }
    let packet = validate_party(warriors, mission, disabled, request)?;
    let actor = &warriors[request.warrior];
    let charge = cost(&packet, request.power)?;
    let identity = [
        request.spell / 1000 - 1,
        request.spell % 1000 / 100 - 1,
        request.spell % 100 - 1,
    ];
    let mut result = PartyCastResult {
        warriors: warriors.to_vec(),
        memory: memory.to_vec(),
        buffs: buffs.to_vec(),
        packet: packet.clone(),
        identity,
        count: request.power,
        scenario_duration: None,
        events: vec![Event::Sound {
            resource_id: word(&packet, 36)?.wrapping_neg(),
        }],
    };
    let remaining = actor.psi_points()?.wrapping_sub(charge);
    put_word(&mut result.warriors[request.warrior].bytes, 0x58, remaining);
    for (index, value) in identity.into_iter().chain([request.power]).enumerate() {
        put_word(
            &mut result.memory,
            request.warrior * 80 + 72 + index * 2,
            value,
        );
    }
    if packet[1] != 7 {
        return Ok(result);
    }
    let mut next_random = random.clone();
    let mut duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..request.power {
            duration = duration.wrapping_add(
                next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
            );
        }
    }
    let effect = word(&packet, 40)?;
    if effect == 110 {
        result.scenario_duration = Some(duration);
    } else {
        super::party_buffs::extend(&mut result.buffs, effect, duration)?;
    }
    *random = next_random;
    Ok(result)
}

pub fn validate_party(
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    request: PartyCastRequest,
) -> Result<Vec<u8>, RuleError> {
    if (!PARTY_SPELLS.contains(&request.spell)
        && !OUTDOOR_PARTY_SPELLS.contains(&request.spell)
        && !OUTDOOR_TARGET_SPELLS.contains(&request.spell))
        || !(1..=7).contains(&request.power)
    {
        return Err(RuleError::Unsupported {
            action: "psi.party",
            dependency: "other outdoor learned spell or power",
        });
    }
    if !(1..=6).contains(&warriors.len()) {
        return Err(RuleError::Invalid("outdoor psi input size".into()));
    }
    let actor = warriors
        .get(request.warrior)
        .ok_or_else(|| RuleError::Invalid("outdoor psi warrior".into()))?;
    for warrior in warriors {
        centurions_content::span(&warrior.bytes, 0, 3138)?;
    }
    if disabled != 0
        || [0x60, 0x68, 0x90, 0x98, 0xac]
            .into_iter()
            .any(|offset| word(&actor.bytes, offset).unwrap_or(1) != 0)
        || word(&actor.bytes, 0x50)? <= 0
        || word(&actor.bytes, 0x58)? <= 0
    {
        return Err(RuleError::Invalid("warrior cannot cast outdoor psi".into()));
    }
    if !actor.knows_spell(request.spell)? {
        return Err(RuleError::Invalid(
            "warrior has not learned this psi spell".into(),
        ));
    }
    let packet = packet(mission, request.spell)?;
    if !mode_allowed(&packet, false)? {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Warning { index: 22 }],
        });
    }
    if !((packet[1] == 7 && matches!(word(&packet, 40)?, 0..=9 | 110))
        || (matches!(packet[1], 0 | 1 | 3 | 4 | 9)
            && (matches!(word(&packet, 40)?, 1..=40 | 57 | 64))))
    {
        return Err(RuleError::Invalid("psi unavailable outdoors".into()));
    }
    let charge = cost(&packet, request.power)?;
    if actor.psi_points()? < charge {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    Ok(packet)
}

pub struct CastAdmission {
    pub allowed: bool,
    pub sounds: Vec<i16>,
}

pub fn admission(
    warriors: &[Warrior],
    member: i16,
    disabled: u8,
) -> Result<CastAdmission, RuleError> {
    let mut result = CastAdmission {
        allowed: false,
        sounds: vec![],
    };
    if !(0..6).contains(&member) {
        return Ok(result);
    }
    if disabled != 0 {
        result.sounds.push(113);
        return Ok(result);
    }
    let w = &warriors
        .get(member as usize)
        .ok_or_else(|| RuleError::Invalid("psi warrior missing".into()))?
        .bytes;
    centurions_content::span(w, 0, 3138)?;
    result.allowed = [0x98, 0xac, 0x60, 0x68, 0x90, 0x14]
        .into_iter()
        .all(|offset| word(w, offset).is_ok_and(|value| value == 0))
        && word(w, 0x50)? > 0
        && word(w, 0x58)? > 0
        && word(w, 0xac8)? < word(w, 0xac6)?;
    Ok(result)
}

pub fn can_cast(warriors: &[Warrior], member: i16, disabled: u8) -> Result<bool, RuleError> {
    Ok(admission(warriors, member, disabled)?.allowed)
}

pub fn cost(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(i16::from(packet[30]).wrapping_mul(power).wrapping_abs())
}

pub fn mode_allowed(packet: &[u8], combat: bool) -> Result<bool, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(packet[if combat { 21 } else { 22 }] != 0)
}

pub fn range(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(i16::from(packet[2] as i8)
        .wrapping_add(i16::from(packet[3] as i8).wrapping_mul(power))
        .wrapping_abs())
}

pub fn in_range(from: (i16, i16), to: (i16, i16), range: i16) -> bool {
    let dx = f64::from(to.0.wrapping_sub(from.0));
    let dy = f64::from(to.1.wrapping_sub(from.1));
    (dx * dx + dy * dy).sqrt().trunc() <= f64::from(range)
}

fn packet(mission: &OpeningMission, spell: i16) -> Result<Vec<u8>, RuleError> {
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("psi content missing".into()))?;
    spells::load_packet(&tables.attacks, spell)
}

pub fn validate(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    request: CastRequest,
) -> Result<(), RuleError> {
    if !COMBAT_SPELLS.contains(&request.spell) || !(1..=7).contains(&request.power) {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "other learned spell or power",
        });
    }
    if request.rotation > 3 {
        return Err(RuleError::Invalid("psi rotation".into()));
    }
    let actor = combat
        .active_body()
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("psi casting requires an active warrior".into()))?;
    if !can_cast(warriors, actor, disabled)? {
        return Err(RuleError::Invalid("warrior cannot cast psi".into()));
    }
    if !warriors[actor as usize].knows_spell(request.spell)? {
        return Err(RuleError::Invalid(
            "warrior has not learned this psi spell".into(),
        ));
    }
    let packet = packet(mission, request.spell)?;
    if !mode_allowed(&packet, true)? {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Warning { index: 23 }],
        });
    }
    if warriors[actor as usize].psi_points()? < cost(&packet, request.power)? {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    if packet[1] == 7 {
        if range(&packet, request.power)? != 0
            || !(0..10).contains(&word(&packet, 40)?)
            || request.target.is_some()
        {
            return Err(RuleError::Invalid(
                "party spell uses automatic caster targeting".into(),
            ));
        }
        return Ok(());
    }
    if matches!(packet[1], 9 | 10 | 12) {
        if request.target.is_some() {
            return Err(RuleError::Invalid(
                "global psi spell does not take a point target".into(),
            ));
        }
        return Ok(());
    }
    let target = request
        .target
        .ok_or_else(|| RuleError::Invalid("psi point target missing".into()))?;
    let from = *combat
        .squad
        .positions
        .get(actor as usize)
        .ok_or_else(|| RuleError::Invalid("psi caster position missing".into()))?;
    if !in_range(from, target, range(&packet, request.power)?) {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
                kind: crate::feedback::CombatRefusal::OutOfRange,
            }],
        });
    }
    let center = (
        i32::from(target.0) + i32::from(combat.map.shift.0) + 5,
        i32::from(target.1) + i32::from(combat.map.shift.1) + 5,
    );
    if i16::from(packet[2] as i8) + i16::from(packet[3] as i8) > 0
        && !lifecycle::can_see(combat, mission, from, target, 0)?
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
                kind: crate::feedback::CombatRefusal::Blocked,
            }],
        });
    }
    if packet[1] < 2 {
        selected_body(combat, warriors, target)?;
        return Ok(());
    }
    let shape = match packet[1] {
        3 => {
            usize::from(packet[31])
                + if packet[7] != 0 {
                    usize::from(request.rotation)
                } else {
                    0
                }
        }
        4 => request.power as usize,
        _ => 1,
    };
    if shape == 0 {
        return Err(RuleError::Invalid("psi area missing".into()));
    }
    let area = centurions_content::span(&mission.areas, (shape - 1) * 49, 49)?;
    for y in 0..7 {
        for x in 0..7 {
            if area[y * 7 + x] != 0
                && (!(0..90).contains(&(center.0 - 3 + x as i32))
                    || !(0..90).contains(&(center.1 - 3 + y as i32)))
            {
                return Err(RuleError::Unsupported {
                    action: "combat.player_psi",
                    dependency: "spell area outside tactical map",
                });
            }
            if request.spell == 1305 && area[y * 7 + x] != 0 {
                let body = combat.map.tiles
                    [(center.0 - 3 + x as i32) as usize * 90 + (center.1 - 3 + y as i32) as usize]
                    .wrapping_abs();
                if (6..10).contains(&body) {
                    return Err(RuleError::Invalid("empty support body".into()));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn selected_body(
    combat: &Combat,
    warriors: &[Warrior],
    target: (i16, i16),
) -> Result<i16, RuleError> {
    let cell = OpeningMission::index(
        i32::from(target.0 + combat.map.shift.0 + 5),
        i32::from(target.1 + combat.map.shift.1 + 5),
    )?;
    let body = combat.map.tiles[cell].wrapping_abs();
    if (0..warriors.len() as i16).contains(&body) && warriors[body as usize].bytes[0x9df] != 0
        || (10..10 + combat.enemies.records.len() as i16).contains(&body)
            && word(&combat.enemies.records[(body - 10) as usize].bytes, 0x110)? > 0
    {
        Ok(body)
    } else {
        Err(RuleError::Refused { feedback: vec![] })
    }
}

pub fn cast(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut Vec<u8>,
    disabled: u8,
    request: CastRequest,
) -> Result<Vec<Event>, RuleError> {
    if packet(mission, request.spell)?[1] == 7 {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "party duration state required",
        });
    }
    cast_with_buffs(
        combat,
        warriors,
        mission,
        random,
        memory,
        &mut vec![0; 20],
        disabled,
        request,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cast_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut Vec<u8>,
    buffs: &mut Vec<u8>,
    disabled: u8,
    request: CastRequest,
) -> Result<Vec<Event>, RuleError> {
    validate(combat, warriors, mission, disabled, request)?;
    if memory.len() != 480 {
        return Err(RuleError::Invalid("psi memory size".into()));
    }
    let mut next_combat = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut next_memory = memory.clone();
    let mut next_buffs = buffs.clone();
    let events = cast_inner(
        &mut next_combat,
        &mut next_warriors,
        mission,
        &mut next_random,
        &mut next_memory,
        &mut next_buffs,
        request,
        CastPayment::Charge,
    )?;
    *combat = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    *memory = next_memory;
    *buffs = next_buffs;
    Ok(events)
}

enum CastPayment {
    Charge,
    Prepared,
    AdditionalTarget,
}

pub(crate) fn cast_selection(
    state: &mut Session,
    mission: &OpeningMission,
    selection: &super::psi_selection::Selection,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let combat = state.combat.as_mut().ok_or("psi combat missing")?;
    let memory = state.psi_memory.as_mut().ok_or("psi memory missing")?;
    for (index, body) in selection.targets.iter().copied().enumerate() {
        let target = if body < 10 {
            combat.squad.positions[body as usize]
        } else {
            combat.enemies.positions[(body - 10) as usize]
        };
        events.extend(cast_inner(
            combat,
            &mut state.warriors,
            mission,
            &mut state.random,
            memory,
            &mut state.party_buffs,
            CastRequest {
                spell: selection.spell,
                power: selection.power,
                target: Some(target),
                rotation: 0,
            },
            if index == 0 {
                CastPayment::Prepared
            } else {
                CastPayment::AdditionalTarget
            },
        )?);
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
fn cast_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut [u8],
    buffs: &mut [u8],
    request: CastRequest,
    payment: CastPayment,
) -> Result<Vec<Event>, RuleError> {
    let actor = combat.active_body().unwrap() as usize;
    let packet = packet(mission, request.spell)?;
    combat.inactive_rounds = 0;
    let descriptor = [
        request.spell / 1000 - 1,
        request.spell % 1000 / 100 - 1,
        request.spell % 100 - 1,
    ];
    combat.attack_packet = Some(packet.clone());
    combat.attack_identity = Some(descriptor);
    let w = &mut warriors[actor].bytes;
    if matches!(payment, CastPayment::Charge) {
        let remaining = word(w, 0x58)?.wrapping_sub(cost(&packet, request.power)?);
        put_word(w, 0x58, remaining);
        let casts = i32::from_be_bytes(w[0xa66..0xa6a].try_into().unwrap()).wrapping_add(1);
        w[0xa66..0xa6a].copy_from_slice(&casts.to_be_bytes());
        let count = word(w, 0xac8)?.wrapping_add(1);
        put_word(w, 0xac8, count);
    }
    if !matches!(payment, CastPayment::AdditionalTarget) {
        let movement = word(w, 0x3c)?.wrapping_sub(3).max(0);
        let points = word(w, 0x40)?.wrapping_sub(2);
        put_word(w, 0x3c, movement);
        put_word(w, 0x40, points);
    }
    let mut events = vec![];
    if matches!(packet[1], 9 | 10) {
        events.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
            address: if packet[1] == 9 { 0x401840 } else { 0x401820 },
            x: 325,
            y: 322,
            wait: 60,
            sound: 0,
        }));
    }
    let mask = if matches!(packet[1], 9 | 10 | 12) {
        let mut mask = [0; 110];
        for (body, warrior) in warriors.iter().enumerate() {
            if warrior.bytes[0x9df] != 0
                && (packet[1] == 12 || (word(&warrior.bytes, 0x10)? == 0) == (packet[1] == 9))
            {
                mask[body] = 1;
            }
        }
        for (index, enemy) in combat.enemies.records.iter().enumerate() {
            if word(&enemy.bytes, 0x110)? > 0
                && (packet[1] == 12 || (word(&enemy.bytes, 0xe)? == 0) == (packet[1] == 9))
            {
                mask[index + 10] = 1;
            }
        }
        mask
    } else {
        let target = request.target.unwrap_or(combat.squad.positions[actor]);
        let target = if packet[1] < 2 {
            spells::SpellTarget {
                first: selected_body(combat, warriors, target)?,
                second: 0,
                kind: if selected_body(combat, warriors, target)? < 10 {
                    1
                } else {
                    2
                },
            }
        } else {
            spells::SpellTarget {
                first: target.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                second: target.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
                kind: 3,
            }
        };
        events.push(Event::Sound {
            resource_id: word(&packet, 36)?,
        });
        let mask = spells::target_mask_rotated(
            combat,
            warriors,
            &mission.areas,
            &target,
            &spells::SpellContext {
                packet: &packet,
                id: request.spell,
                count: request.power,
                caster: actor as i16,
                initiative: combat.cursor as i16,
                collision: false,
            },
            &combat.spell_fields,
            random,
            request.rotation,
        )?;
        combat.spell_fields = mask.fields;
        mask.mask
    };
    for (index, value) in descriptor.into_iter().chain([request.power]).enumerate() {
        put_word(memory, actor * 80 + 72 + index * 2, value);
    }
    if packet[1] == 7 {
        let mut duration = random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
        if packet[15] != 0 {
            for _ in 0..request.power {
                duration = duration.wrapping_add(
                    random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
                );
            }
        }
        super::party_buffs::extend(buffs, word(&packet, 40)?, duration)?;
        return Ok(events);
    }
    if packet[9] <= 3
        && matches!(packet[19], 7 | 8 | 10)
        && matches!(word(&packet, 40)?, 1..=40 | 57)
        && (word(&packet, 40)? == 57
            || (packet[12] == 0
                && word(&packet, 34)? == 0
                && packet[23..30].iter().all(|v| *v == 0)))
    {
        let result = spells::resolve_support_for_caster(
            warriors,
            &combat.enemies.records,
            &mask,
            request.spell,
            &spells::DirectSpell {
                packet: &packet,
                level: descriptor[1],
                count: request.power,
                armor_filter: combat.weapon_work.armor_filter()?,
                weapon_sound: combat.weapon_work.sound()?,
            },
            Some(actor as i16),
            random,
        )?;
        warriors.clone_from_slice(&result.warriors);
        combat.enemies.records = result.enemies;
        combat.unavailable = combat.unavailable.wrapping_sub(result.recovered as i8);
        if result.healed {
            combat.attack_packet.as_mut().unwrap()[18] = 4;
        }
        if !result.targets.is_empty() {
            combat.field_used[0] = 1;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        for (target, outcome) in result.targets {
            if target >= 10 && outcome.damage != 0 {
                let w = &mut warriors[actor].bytes;
                let total = i32::from_be_bytes(w[0xa46..0xa4a].try_into().unwrap())
                    .wrapping_add(i32::from(outcome.damage));
                w[0xa46..0xa4a].copy_from_slice(&total.to_be_bytes());
            }
            events.extend(
                outcome
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            events.push(Event::SpellResolved {
                caster: Some(actor as i16),
                spell: request.spell,
                target: target as i16,
                damage: outcome.damage,
                defeated: false,
            });
        }
        return Ok(events);
    }
    let previous_hp = warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    let result = spells::resolve_bodies_with_memory(
        warriors,
        &combat.enemies.records,
        &mask,
        &spells::DirectSpell {
            packet: &packet,
            level: descriptor[1],
            count: request.power,
            armor_filter: combat.weapon_work.armor_filter()?,
            weapon_sound: combat.weapon_work.sound()?,
        },
        spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, None),
        random,
    )?;
    warriors.clone_from_slice(&result.warriors);
    combat.enemies.records = result.enemies;
    if !result.targets.is_empty() {
        combat.field_used[0] = 1;
    }
    events.extend(
        result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id }),
    );
    for (target, outcome) in &result.targets {
        if *target >= 10 && outcome.damage != 0 {
            let w = &mut warriors[actor].bytes;
            let total = i32::from_be_bytes(w[0xa46..0xa4a].try_into().unwrap())
                .wrapping_add(i32::from(outcome.damage));
            w[0xa46..0xa4a].copy_from_slice(&total.to_be_bytes());
            if outcome.killed {
                let kills = i32::from_be_bytes(w[0xa5a..0xa5e].try_into().unwrap()).wrapping_add(1);
                w[0xa5a..0xa5e].copy_from_slice(&kills.to_be_bytes());
            }
        }
    }
    runtime::append_spell_outcomes(
        combat,
        warriors,
        result.targets,
        result.race_excluded,
        (Some(actor as i16), request.spell, &packet, &previous_hp),
        false,
        &mut events,
    )?;
    Ok(events)
}
