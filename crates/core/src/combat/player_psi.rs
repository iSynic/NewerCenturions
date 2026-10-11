use super::*;
use centurions_content::mission::OpeningMission;

#[derive(Clone, Copy, Debug)]
pub struct CastRequest {
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
    let identity = [0, 0, 2];
    let mut result = PartyCastResult {
        warriors: warriors.to_vec(),
        memory: memory.to_vec(),
        buffs: buffs.to_vec(),
        packet: packet.clone(),
        identity,
        count: request.power,
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
    let mut next_random = random.clone();
    let mut duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    for _ in 0..request.power {
        duration = duration.wrapping_add(
            next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
        );
    }
    super::party_buffs::extend_inertial(&mut result.buffs, duration)?;
    *random = next_random;
    Ok(result)
}

pub fn validate_party(
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    request: PartyCastRequest,
) -> Result<Vec<u8>, RuleError> {
    if request.spell != 1103 || !matches!(request.power, 1 | 2) {
        return Err(RuleError::Unsupported {
            action: "psi.party",
            dependency: "other outdoor learned spell or power",
        });
    }
    if warriors.len() != 6 {
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
    if !matches!(request.spell, 1101 | 1305 | 2101 | 2204 | 2302) || !matches!(request.power, 1 | 2)
    {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "other learned spell or power",
        });
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
        return Err(RuleError::Invalid("psi unavailable in combat".into()));
    }
    if warriors[actor as usize].psi_points()? < cost(&packet, request.power)? {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    if request.spell == 2302 {
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
        return Err(RuleError::Invalid("psi target outside range".into()));
    }
    let center = (
        i32::from(target.0) + i32::from(combat.map.shift.0) + 5,
        i32::from(target.1) + i32::from(combat.map.shift.1) + 5,
    );
    if request.spell == 1101 {
        support_body(combat, warriors, target)?;
        return Ok(());
    }
    if matches!(request.spell, 1305 | 2101)
        && !lifecycle::can_see(combat, mission, from, target, 0)?
    {
        return Err(RuleError::Invalid("psi target outside sight".into()));
    }
    let shape = usize::from(packet[31]);
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
                if (0..110).contains(&body)
                    && (body >= 6 || word(&warriors[body as usize].bytes, 0x9a)? != 0)
                {
                    return Err(RuleError::Unsupported {
                        action: "combat.player_psi",
                        dependency: "other support target or reflection",
                    });
                }
            }
        }
    }
    Ok(())
}

fn support_body(
    combat: &Combat,
    warriors: &[Warrior],
    target: (i16, i16),
) -> Result<i16, RuleError> {
    let cell = OpeningMission::index(
        i32::from(target.0) + i32::from(combat.map.shift.0) + 5,
        i32::from(target.1) + i32::from(combat.map.shift.1) + 5,
    )?;
    let body = combat.map.tiles[cell].wrapping_abs();
    if !(0..6).contains(&body) || body as usize >= warriors.len() {
        return Err(RuleError::Invalid(
            "support psi requires an occupied warrior body".into(),
        ));
    }
    if word(&warriors[body as usize].bytes, 0x9a)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "support spell reflection",
        });
    }
    Ok(body)
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
    validate(combat, warriors, mission, disabled, request)?;
    if memory.len() != 480 {
        return Err(RuleError::Invalid("psi memory size".into()));
    }
    let mut next_combat = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut next_memory = memory.clone();
    let events = cast_inner(
        &mut next_combat,
        &mut next_warriors,
        mission,
        &mut next_random,
        &mut next_memory,
        request,
    )?;
    *combat = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    *memory = next_memory;
    Ok(events)
}

fn cast_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut [u8],
    request: CastRequest,
) -> Result<Vec<Event>, RuleError> {
    let actor = combat.active_body().unwrap() as usize;
    let packet = packet(mission, request.spell)?;
    let descriptor = [
        request.spell / 1000 - 1,
        request.spell % 1000 / 100 - 1,
        request.spell % 100 - 1,
    ];
    combat.attack_packet = Some(packet.clone());
    combat.attack_identity = Some(descriptor);
    let w = &mut warriors[actor].bytes;
    let remaining = word(w, 0x58)?.wrapping_sub(cost(&packet, request.power)?);
    put_word(w, 0x58, remaining);
    let casts = i32::from_be_bytes(w[0xa66..0xa6a].try_into().unwrap()).wrapping_add(1);
    w[0xa66..0xa6a].copy_from_slice(&casts.to_be_bytes());
    for (offset, value) in [
        (0xac8, word(w, 0xac8)?.wrapping_add(1)),
        (0x3c, word(w, 0x3c)?.wrapping_sub(3).max(0)),
        (0x40, word(w, 0x40)?.wrapping_sub(2)),
    ] {
        put_word(w, offset, value);
    }
    let mut events = vec![];
    let mask = if request.spell == 2302 {
        let mut mask = [0; 110];
        for (body, warrior) in warriors.iter().enumerate() {
            if warrior.bytes[0x9df] != 0 && word(&warrior.bytes, 0x10)? != 0 {
                mask[body] = 1;
            }
        }
        for (index, enemy) in combat.enemies.records.iter().enumerate() {
            if word(&enemy.bytes, 0x110)? > 0 && word(&enemy.bytes, 0xe)? != 0 {
                mask[index + 10] = 1;
            }
        }
        mask
    } else {
        let target = request.target.unwrap();
        let target = if request.spell == 1101 {
            spells::SpellTarget {
                first: support_body(combat, warriors, target)?,
                second: 0,
                kind: 1,
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
        let mask = spells::target_mask(
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
        )?;
        combat.spell_fields = mask.fields;
        mask.mask
    };
    for (index, value) in descriptor.into_iter().chain([request.power]).enumerate() {
        put_word(memory, actor * 80 + 72 + index * 2, value);
    }
    if matches!(request.spell, 1101 | 1305) {
        if request.spell == 1101 && combat.weapon_work.armor_filter()? {
            return Err(RuleError::Unsupported {
                action: "combat.player_psi",
                dependency: "retained armor-filter healing",
            });
        }
        let result = spells::resolve_player_support(
            warriors,
            &combat.enemies.records,
            &mask,
            &packet,
            request.spell,
            request.power,
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
    let result = spells::resolve_bodies(
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
    for (target, outcome) in result.targets {
        if target >= 10 && outcome.damage != 0 {
            let w = &mut warriors[actor].bytes;
            let total = i32::from_be_bytes(w[0xa46..0xa4a].try_into().unwrap())
                .wrapping_add(i32::from(outcome.damage));
            w[0xa46..0xa4a].copy_from_slice(&total.to_be_bytes());
            if outcome.killed {
                let kills = i32::from_be_bytes(w[0xa5a..0xa5e].try_into().unwrap()).wrapping_add(1);
                w[0xa5a..0xa5e].copy_from_slice(&kills.to_be_bytes());
            }
        }
        events.extend(
            outcome
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if outcome.killed {
            lifecycle::kill_body(combat, warriors, target as i16)?;
        }
        events.push(Event::SpellResolved {
            caster: Some(actor as i16),
            spell: request.spell,
            target: target as i16,
            damage: outcome.damage,
            defeated: outcome.killed,
        });
    }
    Ok(events)
}
