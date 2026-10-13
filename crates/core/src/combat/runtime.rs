use super::*;

fn spell_identity(id: i16) -> [i16; 3] {
    [id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1]
}

#[derive(PartialEq, Eq)]
enum FireOutcome {
    Failed,
    Physical,
    Spell,
}

fn position(c: &Combat, body: i16) -> (i16, i16) {
    if body < 10 {
        c.squad.positions[body as usize]
    } else {
        c.enemies.positions[(body - 10) as usize]
    }
}
fn hostile(c: &Combat, w: &[Warrior], a: i16, b: i16) -> Result<bool, RuleError> {
    let side = |body: i16| {
        if body < 10 {
            word(&w[body as usize].bytes, 0x10)
        } else {
            word(&c.enemies.records[(body - 10) as usize].bytes, 0xe)
        }
    };
    Ok(side(a)? != side(b)?)
}
fn alive(c: &Combat, w: &[Warrior], body: i16) -> bool {
    if (0..6).contains(&body) {
        w[body as usize].bytes[0x9df] != 0
    } else if body >= 10 {
        c.enemies
            .records
            .get((body - 10) as usize)
            .is_some_and(|m| m.stamina().unwrap_or(0) > 0 && m.bytes[0xb0] != 255)
    } else {
        false
    }
}
fn has_active_warrior(warriors: &[Warrior]) -> bool {
    warriors
        .iter()
        .any(|warrior| warrior.bytes[0x9df] != 0 && word(&warrior.bytes, 0x10).unwrap_or(1) == 0)
}
pub fn neighbors(c: &Combat, w: &[Warrior], body: i16) -> Result<Vec<i16>, RuleError> {
    let p = position(c, body);
    neighbors_at(c, w, body, p)
}
fn neighbors_at(
    c: &Combat,
    w: &[Warrior],
    body: i16,
    p: (i16, i16),
) -> Result<Vec<i16>, RuleError> {
    let mut result = vec![];
    let (minimum_x, minimum_y) = if body >= 10 {
        super::footprint::neighbor_scan(&c.enemies.records[(body - 10) as usize])?
    } else {
        (-1, -1)
    };
    for dy in minimum_y..=1 {
        for dx in minimum_x..=1 {
            let cell = centurions_content::mission::OpeningMission::index(
                i32::from(p.0 + dx + c.map.shift.0 + 5),
                i32::from(p.1 + dy + c.map.shift.1 + 5),
            )?;
            let other = c.map.tiles[cell].wrapping_abs();
            if other < 110
                && other != body
                && alive(c, w, other)
                && hostile(c, w, body, other)?
                && !result.contains(&other)
            {
                result.push(other);
            }
        }
    }
    Ok(result)
}
#[allow(clippy::too_many_arguments)]
fn resolve_attack(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    a: i16,
    b: i16,
    index: usize,
    actor: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    if a >= 10 && !has_active_warrior(w) {
        c.attack_packet = Some(if let Some(attacks) = attacks {
            attack::attack_packet(attacks)?
        } else {
            attack::default_attack_packet()
        });
        return Ok(vec![]);
    }
    let retained = c.attack_packet.clone().unwrap_or_else(|| vec![0; 58]);
    let result = if a < 10 && b >= 10 {
        if word(&w[a as usize].bytes, 0xacc)? < word(&w[a as usize].bytes, 0xaca)? {
            load_natural_work(c, items, buffers)?;
        }
        if let Some(attacks) = attacks {
            let copies_packet =
                word(&w[a as usize].bytes, 0xacc)? < word(&w[a as usize].bytes, 0xaca)?;
            let (result, packet) = attack::unarmed_attack_with_packet(
                &mut w[a as usize],
                &mut c.enemies.records[(b - 10) as usize],
                random,
                attacks,
                &retained,
                c.difficulty,
            )?;
            if copies_packet {
                c.attack_packet = Some(packet);
            }
            result
        } else {
            attack::unarmed_attack(
                &mut w[a as usize],
                &mut c.enemies.records[(b - 10) as usize],
                random,
                c.difficulty,
            )?
        }
    } else if a >= 10 && b < 10 {
        let attacker_x = position(c, a).0;
        let target_x = position(c, b).0;
        let facing = &mut c.enemies.records[(a - 10) as usize].bytes[0x107];
        if attacker_x < target_x && *facing == 0 {
            *facing = 1;
        } else if target_x < attacker_x && *facing == 1 {
            *facing = 0;
        }
        let melee_id = word(&c.enemies.records[(a - 10) as usize].bytes, 0x28)?;
        if matches!(melee_id, 177..=181 | 187 | 192 | 194) {
            let weapon = items.and_then(|items| items.get(melee_id as usize)).ok_or(
                RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "melee weapon content missing",
                },
            )?;
            load_melee_work(c, a, weapon, items, buffers)?;
            let attacker_x = position(c, a).0;
            let target_x = position(c, b).0;
            if melee_id == 187 {
                let attacks = attacks.ok_or(RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "guardian attack table missing",
                })?;
                let (result, packet) = super::enemy_weapons::melee_warrior_with_buffs(
                    c,
                    w,
                    random,
                    (a, b, index, actor),
                    weapon,
                    c.difficulty,
                    attacks,
                    &retained,
                    buffs,
                )?;
                c.attack_packet = Some(packet);
                result
            } else if let Some(attacks) = attacks {
                let (result, packet) = super::enemy_weapons::melee_with_buffs(
                    &mut w[b as usize],
                    &mut c.enemies.records[(a - 10) as usize],
                    weapon,
                    random,
                    c.difficulty,
                    attacker_x,
                    target_x,
                    attacks,
                    &retained,
                    buffs,
                )?;
                c.attack_packet = Some(packet);
                result
            } else {
                super::enemy_weapons::melee(
                    &mut w[b as usize],
                    &mut c.enemies.records[(a - 10) as usize],
                    weapon,
                    random,
                    c.difficulty,
                    attacker_x,
                    target_x,
                )?
            }
        } else {
            load_natural_work(c, items, buffers)?;
            if let Some(attacks) = attacks {
                let (result, packet) = attack::natural_attack_with_buffs(
                    &mut w[b as usize],
                    &mut c.enemies.records[(a - 10) as usize],
                    index,
                    random,
                    attacks,
                    &retained,
                    buffs,
                    c.difficulty,
                )?;
                c.attack_packet = Some(packet);
                result
            } else {
                attack::natural_attack(
                    &mut w[b as usize],
                    &mut c.enemies.records[(a - 10) as usize],
                    index,
                    random,
                    c.difficulty,
                )?
            }
        }
    } else if a >= 10 && b >= 10 {
        let melee_id = word(&c.enemies.records[(a - 10) as usize].bytes, 0x28)?;
        if matches!(melee_id, 177 | 178 | 181 | 187 | 192 | 194) {
            let weapon = items.and_then(|items| items.get(melee_id as usize)).ok_or(
                RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "melee weapon content missing",
                },
            )?;
            load_melee_work(c, a, weapon, items, buffers)?;
            if let Some(attacks) = attacks {
                let (result, packet) = super::enemy_weapons::melee_enemy_with_packet(
                    c,
                    random,
                    (a, b, index, actor),
                    weapon,
                    c.difficulty,
                    attacks,
                    &retained,
                )?;
                c.attack_packet = Some(packet);
                result
            } else {
                super::enemy_weapons::melee_enemy(
                    c,
                    random,
                    (a, b, index, actor),
                    weapon,
                    c.difficulty,
                )?
            }
        } else if let Some(attacks) = attacks {
            load_natural_work(c, items, buffers)?;
            let (result, packet) = super::ally_turns::natural_attack_enemy_with_actor(
                c,
                w,
                random,
                (a, b, index, actor),
                attacks,
                &retained,
            )?;
            c.attack_packet = Some(packet);
            result
        } else {
            load_natural_work(c, items, buffers)?;
            super::ally_turns::natural_attack_enemy(c, w, random, a, b, index)?
        }
    } else {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "combat between bodies of the same record type",
        });
    };
    if result.killed {
        lifecycle::kill_body(c, w, b)?;
    }
    let mut events: Vec<Event> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    events.push(Event::AttackResolved {
        attacker: a,
        target: b,
        damage: result.damage,
        defeated: result.killed,
    });
    if a >= 10 {
        let melee_id = word(&c.enemies.records[(a - 10) as usize].bytes, 0x28)?;
        let impact = if word(&c.enemies.records[(a - 10) as usize].bytes, 0x4e)? != 0
            && matches!(melee_id, 177..=181 | 187 | 192 | 194)
        {
            word(&items.unwrap()[melee_id as usize], 0xe)?
        } else {
            let monster = &c.enemies.records[(a - 10) as usize].bytes;
            let attack = if monster[0x93 + index * 4] == 0 {
                0
            } else {
                index
            };
            i16::from(monster[0x95 + attack * 4] as i8)
        };
        events.push(Event::Sound {
            resource_id: if result.damage == 0 { 650 } else { impact },
        });
    } else if result.damage == 0 {
        events.push(Event::Sound { resource_id: 650 });
    } else if !result.killed {
        events.push(Event::Sound {
            resource_id: word(&w[a as usize].bytes, 0x128)?,
        });
    }
    Ok(events)
}

fn load_natural_work(
    c: &mut Combat,
    items: Option<&[Vec<u8>]>,
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<(), RuleError> {
    if let Some(items) = items {
        let definition = items
            .first()
            .ok_or_else(|| RuleError::Invalid("natural weapon definition missing".into()))?;
        c.weapon_work.load_pair(definition, definition)?;
        buffers.load_natural(items)?;
    } else {
        *buffers = Default::default();
    }
    Ok(())
}

fn load_melee_work(
    c: &mut Combat,
    attacker: i16,
    weapon: &[u8],
    items: Option<&[Vec<u8>]>,
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<(), RuleError> {
    if word(&c.enemies.records[(attacker - 10) as usize].bytes, 0x4e)? != 0 {
        c.weapon_work.load_weapon(weapon)?;
        if let Some(items) = items {
            buffers.load_weapon(
                items,
                word(&c.enemies.records[(attacker - 10) as usize].bytes, 0x28)?,
            )?;
        } else {
            buffers.weapon = None;
        }
        Ok(())
    } else {
        load_natural_work(c, items, buffers)
    }
}

pub(crate) fn refresh_body_item(
    combat: &Combat,
    warriors: &[Warrior],
    items: &[Vec<u8>],
    body: i16,
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<(), RuleError> {
    let id = if (0..6).contains(&body) {
        let warrior = &warriors[body as usize];
        let secondary = warrior.bytes[0x9dd] != 0;
        let id = word(&warrior.bytes, if secondary { 0xe4 } else { 0xca })?;
        if id == 0 {
            if secondary {
                return Err(RuleError::Unsupported {
                    action: "combat.item_refresh",
                    dependency: "missing selected secondary during native item refresh",
                });
            }
            if combat.active_body() != Some(body) {
                return Ok(());
            }
        }
        id
    } else if body >= 10 {
        let id = word(&combat.enemies.records[(body - 10) as usize].bytes, 0x4e)?;
        if id == 0 {
            return Ok(());
        }
        id
    } else {
        return Err(RuleError::Invalid(
            "item refresh body outside represented actors".into(),
        ));
    };
    buffers.load_item_from(items, id)?;
    Ok(())
}
pub fn check_neighbors(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        None,
        None,
        &mut false,
        &[0; 20],
        &mut Default::default(),
        false,
    )
}

pub fn check_neighbors_with_definitions(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        Some(items),
        None,
        &mut false,
        &[0; 20],
        &mut Default::default(),
        false,
    )
}

pub fn check_neighbors_with_context(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_buffs(c, w, random, body, items, attacks, &[0; 20])
}

pub fn check_neighbors_with_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        Some(items),
        Some(attacks),
        &mut false,
        buffs,
        &mut Default::default(),
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_neighbors_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        Some(items),
        Some(attacks),
        &mut false,
        buffs,
        buffers,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn check_neighbors_with_items(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    stopped: &mut bool,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    for _ in 0..110 {
        if *stopped {
            return Ok(events);
        }
        let list = neighbors(c, w, body)?;
        let mut changed = false;
        for &other in &list {
            let guard = if other < 10 {
                word(&w[other as usize].bytes, 0x16)? != 0
                    && word(&w[other as usize].bytes, 0x60)? == 0
            } else {
                let m = &c.enemies.records[(other - 10) as usize].bytes;
                m[0x104] != 0
                    && m[0xb3] == 0
                    && m[0xb0] != 255
                    && (body >= 10 || !matches!(word(&w[body as usize].bytes, 0x34)?, 4 | 5))
            };
            if guard {
                if display_writers {
                    refresh_body_item(c, w, items.unwrap(), other, buffers)?;
                }
                changed = true;
                if other < 10 {
                    put_word(&mut w[other as usize].bytes, 0x16, 0);
                } else {
                    c.enemies.records[(other - 10) as usize].bytes[0x104] = 0;
                }
                events.extend(resolve_attack(
                    c, w, random, other, body, 0, body, items, attacks, buffs, buffers,
                )?);
                if other >= 10 && !has_active_warrior(w) {
                    *stopped = true;
                }
                if !alive(c, w, body) {
                    return Ok(events);
                }
            }
        }
        if changed && display_writers {
            refresh_body_item(c, w, items.unwrap(), body, buffers)?;
        }
        if list.is_empty() || body < 10 {
            return Ok(events);
        }
        let target = list[(random.bound(list.len() as i16) - 1) as usize];
        let e = (body - 10) as usize;
        put_word(&mut c.enemies.records[e].bytes, 0x10c, target);
        let count = (c.enemies.records[e].bytes[0x8a] as i8) / 2;
        let mut killed = false;
        for index in 0..count.max(0) as usize {
            let m = &mut c.enemies.records[e].bytes;
            m[0x109] = (m[0x109] as i8).wrapping_sub(2) as u8;
            if (m[0x109] as i8) < 0 {
                if m[0x109] == 255 {
                    m[0x104] = 1;
                }
                return Ok(events);
            }
            m[0x106] = (m[0x106] as i8).wrapping_sub(3) as u8;
            if (m[0x109] as i8) < 2 {
                m[0x104] = u8::from((m[0x109] as i8) >= 1);
            }
            events.extend(resolve_attack(
                c, w, random, body, target, index, body, items, attacks, buffs, buffers,
            )?);
            if !has_active_warrior(w) {
                *stopped = true;
            }
            if !alive(c, w, target) {
                killed = true;
                break;
            }
        }
        if !killed {
            return Ok(events);
        }
    }
    Err(RuleError::Invalid(
        "neighbor attack exceeded body capacity".into(),
    ))
}
pub fn warrior_departure(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_with_buffs(c, w, random, body, destination, items, attacks, &[0; 20])
}
#[allow(clippy::too_many_arguments)]
pub fn warrior_departure_with_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_inner(
        c,
        w,
        random,
        body,
        destination,
        items,
        attacks,
        buffs,
        &mut Default::default(),
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn warrior_departure_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_inner(
        c,
        w,
        random,
        body,
        destination,
        items,
        attacks,
        buffs,
        buffers,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn warrior_departure_inner(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
) -> Result<Vec<Event>, RuleError> {
    if word(&w[body as usize].bytes, 0x8e)? != 0 {
        return Ok(vec![]);
    }
    let mut previous = neighbors(c, w, body)?;
    let projected = neighbors_at(c, w, body, destination)?;
    previous.retain(|other| !projected.contains(other));
    previous.sort_unstable();
    let mut events = vec![];
    let mut changed = false;
    for other in previous {
        if other < 10 {
            if word(&w[other as usize].bytes, 0x60)? != 0 {
                continue;
            }
            put_word(&mut w[other as usize].bytes, 0x16, 0);
        } else {
            let m = &mut c.enemies.records[(other - 10) as usize].bytes;
            if m[0xb3] != 0
                || m[0xb0] == 255
                || matches!(word(&w[body as usize].bytes, 0x34)?, 4 | 5)
            {
                continue;
            }
            m[0x104] = 0;
        }
        if display_writers {
            refresh_body_item(c, w, items, other, buffers)?;
        }
        changed = true;
        events.extend(resolve_attack(
            c,
            w,
            random,
            other,
            body,
            0,
            body,
            Some(items),
            Some(attacks),
            buffs,
            buffers,
        )?);
        if !alive(c, w, body) {
            changed = false;
            break;
        }
    }
    if changed && display_writers {
        refresh_body_item(c, w, items, body, buffers)?;
    }
    Ok(events)
}
pub fn enemy_action(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    enemy_action_with_context(
        c,
        w,
        mission,
        random,
        items,
        &mut EnemyWork::default(),
        &[0; 20],
    )
}

pub struct EnemyWork {
    pub item_context: Option<i16>,
    pub attack_count: i16,
    pub packet: Option<Vec<u8>>,
    pub identity: Option<[i16; 3]>,
    pub spell_attempt: i8,
    pub spell_disabled: u8,
    pub buffers: crate::ammunition::WorkBuffers,
    // Legacy helper fixtures omit combatupdate2. Session execution includes its writers.
    pub display_writers: bool,
}
impl Default for EnemyWork {
    fn default() -> Self {
        Self {
            item_context: None,
            attack_count: 0,
            packet: Some(vec![0; 58]),
            identity: Some([0; 3]),
            spell_attempt: 0,
            spell_disabled: 0,
            buffers: Default::default(),
            display_writers: false,
        }
    }
}

pub fn enemy_action_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
) -> Result<Vec<Event>, RuleError> {
    enemy_action_with_work_and_buffs(c, w, mission, random, items, work, &[0; 20])
}

pub fn enemy_action_with_work_and_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    c.attack_packet = work.packet.clone();
    c.attack_identity = work.identity;
    let events = enemy_action_with_context(c, w, mission, random, items, work, buffs)?;
    work.packet = c.attack_packet.clone();
    work.identity = c.attack_identity;
    Ok(events)
}

fn record_attack_context(work: &mut EnemyWork) -> Result<(), RuleError> {
    if let Some(item) = work.buffers.item.as_ref() {
        work.item_context = Some(word(item, 32)?);
    }
    Ok(())
}

fn enemy_action_with_context(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active enemy missing".into()))?;
    if body < 10 {
        return Err(RuleError::Invalid("active body is a warrior".into()));
    }
    let e = (body - 10) as usize;
    if word(&c.enemies.records[e].bytes, 0xe)? == 0 && word(&c.enemies.records[e].bytes, 0x26)? == 1
    {
        return Err(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "work-item context requires a session",
        });
    }
    let mut events = vec![];
    if (c.enemies.records[e].bytes[0x109] as i8) < 2 {
        return Ok(events);
    }
    let mut attempt = 0;
    let mut interrupted = false;
    let mut reactions_stopped = false;
    for _ in 0..32 {
        let m = &mut c.enemies.records[e].bytes;
        if (m[0x109] as i8) < 2 {
            return Ok(events);
        }
        m[0x104] = u8::from((m[0x109] as i8) >= 1);
        if m[0xb2] != 0 {
            return Err(RuleError::Unsupported {
                action: "combat.enemy",
                dependency: "fleeing enemy",
            });
        }
        let mut skip_neighbors = false;
        let weapon = word(m, 0x48)?;
        if random.bound(100) <= weapon {
            match enemy_fire(c, w, mission, random, (items, work), body, &mut events)? {
                FireOutcome::Physical => continue,
                FireOutcome::Spell => {
                    attempt += 1;
                    if word(&c.enemies.records[e].bytes, 0x48)? != 0
                        && (c.enemies.records[e].bytes[0x109] as i8) > 1
                        && attempt < 2
                        && enemy_fire(c, w, mission, random, (items, work), body, &mut events)?
                            == FireOutcome::Physical
                    {
                        continue;
                    }
                    return Ok(events);
                }
                FireOutcome::Failed => {}
            }
            attempt += 1;
        } else {
            let spell = word(m, 0x42)?;
            if random.bound(100) <= spell {
                let blocked = work.spell_disabled != 0
                    || [0xb7, 0xcf, 0xd9, 0xb3, 0x105]
                        .iter()
                        .any(|offset| m[*offset] != 0);
                skip_neighbors = blocked;
                let cast = if blocked {
                    false
                } else {
                    attempt += 1;
                    enemy_spell(c, w, mission, random, (items, work), body, &mut events)?
                };
                if cast {
                    if word(&c.enemies.records[e].bytes, 0x48)? != 0
                        && (c.enemies.records[e].bytes[0x109] as i8) > 1
                        && attempt < 2
                        && enemy_fire(c, w, mission, random, (items, work), body, &mut events)?
                            == FireOutcome::Physical
                    {
                        continue;
                    }
                    return Ok(events);
                }
            }
        }
        let adjacent = if skip_neighbors || c.enemies.records[e].bytes[0xb3] != 0 {
            vec![]
        } else {
            check_neighbors_with_items(
                c,
                w,
                random,
                body,
                Some(items),
                Some(&mission.combat.as_ref().unwrap().attacks),
                &mut reactions_stopped,
                buffs,
                &mut work.buffers,
                work.display_writers,
            )?
        };
        interrupted |= adjacent
            .iter()
            .any(|event| matches!(event, Event::AttackResolved { .. }));
        record_attack_context(work)?;
        events.extend(adjacent);
        let mut movement_failed = false;
        for _ in 0..128 {
            if reactions_stopped {
                break;
            }
            if !alive(c, w, body) || !w.iter().any(|w| w.bytes[0x9df] != 0) {
                break;
            }
            let m = &c.enemies.records[e].bytes;
            if (m[0x106] as i8) <= 0 || (m[0x109] as i8) <= 1 || m[0xb3] != 0 {
                break;
            }
            select_enemy_target(c, w, mission, random, body)?;
            let moved = enemy::move_enemy(c, mission, e, random)?;
            events.extend(
                moved
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            if moved.moved {
                let p = position(c, body);
                events.push(Event::TacticalMoved {
                    body,
                    x: p.0,
                    y: p.1,
                });
                events.extend(collide_fields(
                    c,
                    w,
                    mission,
                    &mut work.attack_count,
                    random,
                    body,
                )?);
            }
            let adjacent = check_neighbors_with_items(
                c,
                w,
                random,
                body,
                Some(items),
                Some(&mission.combat.as_ref().unwrap().attacks),
                &mut reactions_stopped,
                buffs,
                &mut work.buffers,
                work.display_writers,
            )?;
            interrupted |= adjacent
                .iter()
                .any(|event| matches!(event, Event::AttackResolved { .. }));
            record_attack_context(work)?;
            events.extend(adjacent);
            if !moved.moved {
                movement_failed = true;
                break;
            }
        }
        let m = &c.enemies.records[e].bytes;
        if attempt < 2 && word(m, 0x42)? != 0 && !interrupted {
            c.enemies.records[e].bytes[0x106] = 0;
            let m = &c.enemies.records[e].bytes;
            let blocked = work.spell_disabled != 0
                || [0xb3, 0xb7, 0xcf, 0xd9, 0x105]
                    .iter()
                    .any(|offset| m[*offset] != 0);
            if !blocked {
                attempt = if movement_failed { attempt + 2 } else { 2 };
                enemy_spell(c, w, mission, random, (items, work), body, &mut events)?;
                attempt += 1;
            }
        }
        let m = &c.enemies.records[e].bytes;
        if word(m, 0x48)? == 0 || (m[0x109] as i8) < 2 || attempt > 1 {
            return Ok(events);
        }
        attempt += 1;
        if enemy_fire(c, w, mission, random, (items, work), body, &mut events)?
            == FireOutcome::Physical
        {
            continue;
        }
        return Ok(events);
    }
    Err(RuleError::Invalid(
        "enemy action exceeded point budget".into(),
    ))
}

fn increment_stat(bytes: &mut [u8], offset: usize, amount: i32) {
    let value =
        i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()).wrapping_add(amount);
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn enemy_death_sound(monster: &MonsterRecord) -> Result<i16, RuleError> {
    centurions_content::span(&monster.bytes, 0, 288)?;
    if monster.bytes[0xab] != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.death",
            dependency: "alternate enemy defeat feedback",
        });
    }
    Ok(word(&monster.bytes, 0x6a)?)
}

fn append_spell_outcomes(
    c: &mut Combat,
    w: &mut [Warrior],
    targets: Vec<(usize, super::attack::AttackResult)>,
    race_excluded: Vec<usize>,
    identity: (Option<i16>, i16, &[u8], &[i16]),
    enemy_defeat_feedback: bool,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let mut excluded = race_excluded.into_iter().peekable();
    for (target, outcome) in targets {
        while excluded.peek().is_some_and(|body| *body < target) {
            excluded.next();
            events.push(Event::Sound { resource_id: 651 });
        }
        events.extend(
            outcome
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        // Native result audio follows surviving party hits at 151bca/1587b8.
        if identity.2[19] == 8
            && target < w.len()
            && identity.3[target] >= 0
            && !outcome.killed
            && outcome.damage != 0
        {
            let caster = identity.0.unwrap_or(c.order[c.cursor]);
            let sound = if caster >= 10 {
                let enemy = &c.enemies.records[(caster - 10) as usize].bytes;
                if word(enemy, 0x4e)? != 0 {
                    c.weapon_work.sound()?
                } else {
                    return Err(RuleError::Unsupported {
                        action: "combat.spell",
                        dependency: "noncaster category-eight result audio",
                    });
                }
            } else {
                word(identity.2, 38)?
            };
            events.push(Event::Sound { resource_id: sound });
        }
        if outcome.killed {
            let sound = if target >= 10 && enemy_defeat_feedback {
                Some(enemy_death_sound(&c.enemies.records[target - 10])?)
            } else {
                None
            };
            lifecycle::kill_body(c, w, target as i16)?;
            if let Some(resource_id) = sound {
                events.push(Event::Sound { resource_id });
            }
        }
        events.push(Event::SpellResolved {
            caster: identity.0,
            spell: identity.1,
            target: target as i16,
            damage: outcome.damage,
            defeated: outcome.killed,
        });
    }
    events.extend(excluded.map(|_| Event::Sound { resource_id: 651 }));
    Ok(())
}

fn resolve_spell_mask(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    spell: (i16, &[u8], i16),
    mask: &[i16; 110],
    caster: i16,
    memory: super::spells::RestrictionMemoryView<'_>,
) -> Result<Vec<Event>, RuleError> {
    let (id, packet, count) = spell;
    if id == 1703 {
        let result = super::spell_conditions::resolve_1703(
            w,
            &c.enemies.records,
            packet,
            count,
            mask,
            random,
        )?;
        w.clone_from_slice(&result.warriors);
        c.enemies.records = result.enemies;
        if mask.iter().any(|flag| *flag > 0) {
            c.field_used[0] = 1;
        }
        let mut events: Vec<_> = result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        events.extend(
            mask.iter()
                .enumerate()
                .filter(|(_, flag)| **flag > 0)
                .map(|(target, _)| Event::SpellResolved {
                    caster: Some(caster),
                    spell: id,
                    target: target as i16,
                    damage: 0,
                    defeated: false,
                }),
        );
        return Ok(events);
    }
    let previous_hp = w
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    let result = super::spells::resolve_bodies_with_memory(
        w,
        &c.enemies.records,
        mask,
        &super::spells::DirectSpell {
            packet,
            level: id % 1000 / 100 - 1,
            count,
            armor_filter: c.weapon_work.armor_filter()?,
            weapon_sound: c.weapon_work.sound()?,
        },
        memory,
        random,
    )?;
    w.clone_from_slice(&result.warriors);
    c.enemies.records = result.enemies;
    if !result.targets.is_empty() {
        c.field_used[0] = 1;
    }
    let mut events: Vec<_> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    append_spell_outcomes(
        c,
        w,
        result.targets,
        result.race_excluded,
        (Some(caster), id, packet, &previous_hp),
        false,
        &mut events,
    )?;
    Ok(events)
}

fn refresh_enemy_item_alias(
    c: &Combat,
    enemy: usize,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
) -> Result<Option<i16>, RuleError> {
    let selected = word(&c.enemies.records[enemy].bytes, 0x4e)?;
    if selected == 0 {
        return Ok(None);
    }
    work.item_context = Some(work.buffers.load_item_from(items, selected)?);
    let index = usize::try_from(selected.wrapping_abs())
        .map_err(|_| RuleError::Invalid("invalid NPC selected item".into()))?;
    let item = items
        .get(index)
        .ok_or_else(|| RuleError::Invalid("NPC selected item missing".into()))?;
    Ok(Some(word(item, 0x32)?))
}

fn enemy_spell(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    resources: (&[Vec<u8>], &mut EnemyWork),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    resources.1.spell_attempt = 0;
    enemy_spell_continue(c, w, mission, random, resources, body, events)
}

fn enemy_spell_continue(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    resources: (&[Vec<u8>], &mut EnemyWork),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let (items, work) = resources;
    let e = (body - 10) as usize;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("spell content missing".into()))?;
    let attempts = c.enemies.records[e].bytes[0x7e] as i8;
    let mut cast = false;
    while work.spell_attempt < attempts {
        let Some((id, packet)) =
            super::spells::select_packet(&c.enemies.records[e], &tables.attacks, random)?
        else {
            break;
        };
        c.attack_packet = Some(packet.clone());
        c.attack_identity = Some(spell_identity(id));
        if matches!(packet[1], 9 | 10 | 12) {
            let result = super::spells::global_targets(c, w, e, &packet)?;
            c.enemies.records[e] = result.caster;
            let alias = refresh_enemy_item_alias(c, e, items, work)?;
            let memory =
                super::spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, alias);

            events.extend(resolve_spell_mask(
                c,
                w,
                random,
                (id, &packet, work.attack_count),
                &result.mask,
                body,
                memory,
            )?);
            cast = true;
            work.spell_attempt = work.spell_attempt.wrapping_add(1);
            continue;
        }
        let selection =
            super::spells::select_targets(c, w, mission, &mission.areas, e, &packet, random)?;
        work.attack_count = selection.count;
        if selection.targets.is_empty() {
            break;
        }
        c.enemies.records[e] = selection.caster;
        let alias = refresh_enemy_item_alias(c, e, items, work)?;
        let memory =
            super::spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, alias);

        if packet[7] != 0 {
            random.bound(4);
        }
        work.attack_count = work.attack_count.max(1);
        let mut origin = selection.targets[0].clone();
        for target in selection.targets {
            let context = super::spells::SpellContext {
                packet: &packet,
                id,
                count: work.attack_count,
                caster: body,
                initiative: c.cursor as i16,
                collision: false,
            };
            events.push(Event::Sound {
                resource_id: word(&packet, 36)?,
            });
            let mask = super::spells::target_mask_with_origin(
                c,
                w,
                &mission.areas,
                (&target, &origin),
                &context,
                &c.spell_fields,
                random,
            )?;
            origin.first = -1;
            origin.second = -1;
            c.spell_fields = mask.fields;
            events.extend(resolve_spell_mask(
                c,
                w,
                random,
                (id, &packet, work.attack_count),
                &mask.mask,
                body,
                memory,
            )?);
        }
        cast = true;
        work.spell_attempt = work.spell_attempt.wrapping_add(1);
    }
    Ok(cast)
}

fn enemy_fire(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items_context: (&[Vec<u8>], &mut EnemyWork),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<FireOutcome, RuleError> {
    let (items, work) = items_context;
    let e = (body - 10) as usize;
    let weapon_id = word(&c.enemies.records[e].bytes, 0x26)?;
    let weapon = items
        .get(weapon_id as usize)
        .ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "weapon content missing",
        })?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    let item_context = match work.item_context {
        Some(context) => context,
        None if word(&c.enemies.records[e].bytes, 0x4c)? == 0 => 0,
        None => {
            return Err(RuleError::Unsupported {
                action: "combat.enemy_firearm",
                dependency: "unreconstructable legacy work-item context",
            });
        }
    };
    let ammo_id = super::enemy_weapons::ammo_id(&c.enemies.records[e], item_context)?;
    let ammo = items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("enemy ammunition outside content".into()))?;
    work.buffers.prepare_pair(weapon, ammo)?;
    let mut prepared = if weapon_id == 0 {
        let packet = c.attack_packet.clone().ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "unreconstructable retained attack packet",
        })?;
        c.weapon_work.load_pair(weapon, ammo)?;
        put_word(&mut c.enemies.records[e].bytes, 0x4e, 0);
        firearm::PreparedFirearm {
            sound: word(&packet, 38)?,
            packet,
            slot: 0,
            shots: i16::from(weapon[77] as i8),
            rounds: word(weapon, 22)?,
            weight: 0,
            armor_filter: word(weapon, 136)? != 0,
        }
    } else if matches!(
        weapon_id,
        2 | 3 | 5 | 9 | 12 | 26 | 28 | 29 | 37 | 38 | 50 | 100 | 165 | 680 | 683
    ) {
        super::enemy_weapons::prepare(&mut c.enemies.records[e], weapon, ammo, attacks)?
    } else {
        firearm::prepare_enemy(&mut c.enemies.records[e], weapon, attacks)?
    };
    if weapon_id != 0 {
        c.weapon_work.prepare(weapon, ammo)?;
        c.attack_packet = Some(prepared.packet.clone());
        c.attack_identity = Some(spell_identity(word(weapon, 44)?));
        work.attack_count = prepared.shots;
    }
    prepared.armor_filter = c.weapon_work.armor_filter()?;
    prepared.sound = c.weapon_work.sound()?;
    let selection = if weapon_id == 0 {
        super::spells::select_retained_targets(c, w, mission, &mission.areas, e, &prepared, random)?
    } else {
        super::spells::select_firearm_targets(c, w, mission, &mission.areas, e, &prepared, random)?
    };
    work.attack_count = selection.count;
    if !selection.cast {
        return Ok(FireOutcome::Failed);
    }
    c.enemies.records[e] = selection.caster;
    if work.display_writers {
        refresh_enemy_item_alias(c, e, items, work)?;
    }
    let physical = prepared.packet[19] == 9;
    work.attack_count = if physical {
        1
    } else {
        work.attack_count.max(1)
    };
    prepared.shots = work.attack_count;
    let id = if weapon_id == 0 {
        let identity = c.attack_identity.ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "unreconstructable retained spell identity",
        })?;
        (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1
    } else {
        word(weapon, 44)?
    };
    if prepared.packet[7] != 0 {
        random.bound(4);
    }
    let mut origin = selection.targets.first().cloned();
    for target in selection.targets {
        if prepared.packet[1] < 2 {
            let target_body = target.first;
            if !alive(c, w, target_body) {
                continue;
            }
        }
        events.push(Event::Sound {
            resource_id: word(&prepared.packet, 36)?,
        });
        let context = super::spells::SpellContext {
            packet: &prepared.packet,
            id,
            count: work.attack_count,
            caster: body,
            initiative: c.cursor as i16,
            collision: false,
        };
        let mask = super::spells::target_mask_with_origin(
            c,
            w,
            &mission.areas,
            (&target, origin.as_ref().unwrap()),
            &context,
            &c.spell_fields,
            random,
        )?;
        if let Some(origin) = origin.as_mut() {
            origin.first = -1;
            origin.second = -1;
        }
        c.spell_fields = mask.fields;
        if !physical {
            events.extend(resolve_spell_mask(
                c,
                w,
                random,
                (id, &prepared.packet, work.attack_count),
                &mask.mask,
                body,
                super::spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, None),
            )?);
            continue;
        }
        random.range(
            i16::from(prepared.packet[13] as i8),
            i16::from(prepared.packet[14] as i8),
        );
        for (target, flag) in mask.mask.iter().enumerate() {
            if *flag <= 0 {
                continue;
            }
            let result = if target < 6 {
                firearm::resolve_enemy_after_duration(
                    &mut w[target],
                    &mut c.enemies.records[e],
                    &prepared,
                    random,
                    c.difficulty,
                )?
            } else if target >= 10 {
                let caster = c.enemies.records[e].clone();
                super::spells::resolve_enemy_firearm_body_after_duration(
                    &mut c.enemies.records[target - 10],
                    &caster,
                    &prepared,
                    c.difficulty,
                    random,
                )?
            } else {
                return Err(RuleError::Invalid("firearm selected empty body".into()));
            };
            if result.killed {
                lifecycle::kill_body(c, w, target as i16)?;
            }
            events.extend(
                result
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            if result.damage != 0 && !result.killed && prepared.packet[19] == 9 {
                events.push(Event::Sound {
                    resource_id: super::ally_turns::result_audio(
                        &c.enemies.records[e],
                        0,
                        result.damage,
                        word(&prepared.packet, 38)?,
                    )?,
                });
            }
            events.push(Event::AttackResolved {
                attacker: body,
                target: target as i16,
                damage: result.damage,
                defeated: result.killed,
            });
        }
    }
    if physical {
        Ok(FireOutcome::Physical)
    } else {
        work.spell_attempt = work.spell_attempt.wrapping_add(1);
        enemy_spell_continue(c, w, mission, random, (items, work), body, events)?;
        Ok(FireOutcome::Spell)
    }
}
pub fn select_enemy_target(
    c: &mut Combat,
    w: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    body: i16,
) -> Result<i16, RuleError> {
    if body < 10 || body >= 10 + c.enemies.records.len() as i16 {
        return Err(RuleError::Invalid("invalid targeting enemy".into()));
    }
    let e = (body - 10) as usize;
    let mut target = word(&c.enemies.records[e].bytes, 0x10c)?;
    if !alive(c, w, target) || !hostile(c, w, body, target)? {
        let mut selected = None;
        for _ in 0..10000 {
            let t = random.bound(c.enemies.records.len() as i16 + 10) - 1;
            if alive(c, w, t) && hostile(c, w, body, t)? {
                selected = Some(t);
                break;
            }
        }
        target = selected.ok_or_else(|| RuleError::Invalid("no eligible enemy target".into()))?;
    }
    let mut scan = 0;
    let selected_target = target;
    loop {
        put_word(&mut c.enemies.records[e].bytes, 0x10c, target);
        if lifecycle::can_see(c, mission, position(c, body), position(c, target), 0)? {
            break;
        }
        if scan >= 110 {
            target = selected_target;
            put_word(&mut c.enemies.records[e].bytes, 0x10c, target);
            break;
        }
        target = scan;
        scan += 1;
        while !alive(c, w, target) || !hostile(c, w, body, target)? {
            if scan >= 110 {
                target = selected_target;
                break;
            }
            target = scan;
            scan += 1;
        }
    }
    Ok(target)
}

pub fn player_guard(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Combat
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| !story.waiting_child)
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "guard requires an unobstructed player combat turn".into(),
        ));
    }
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let warrior = state
        .warriors
        .get_mut(body as usize)
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    if warrior.bytes[0x9df] == 0 || word(&warrior.bytes, 0x50)? <= 0 {
        return Err(RuleError::Invalid(
            "guard requires a living participant".into(),
        ));
    }
    if word(&warrior.bytes, 0x40)? < 2 {
        return Err(RuleError::Invalid(
            "guard requires a current player input turn".into(),
        ));
    }
    put_word(&mut warrior.bytes, 0x16, 1);
    let mut events = vec![Event::Sound { resource_id: 10121 }];
    events.extend(end_turn(state, content)?);
    Ok(events)
}

pub fn player_attack(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    if !(0..6).contains(&body) || !neighbors(c, &state.warriors, body)?.contains(&target) {
        return Err(RuleError::Invalid(
            "target is not adjacent to active warrior".into(),
        ));
    }
    let w = &mut state.warriors[body as usize].bytes;
    if word(w, 0x40)? < 2 {
        return Ok(vec![]);
    }
    let movement = word(w, 0x3c)?.wrapping_sub(3).max(0);
    let actions = word(w, 0x40)?.wrapping_sub(2);
    put_word(w, 0x3c, movement);
    put_word(w, 0x40, actions);
    let mut events = resolve_attack(
        c,
        &mut state.warriors,
        &mut state.random,
        body,
        target,
        0,
        body,
        Some(&content.items),
        Some(
            &content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap()
                .combat
                .as_ref()
                .unwrap()
                .attacks,
        ),
        &state.party_buffs,
        &mut state.ammunition_work,
    )?;
    events.extend(finish_player_action(state, content, body)?);
    Ok(events)
}
pub fn enemy_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let mut party_hp = state
        .warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let actor = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active enemy missing".into()))?;
    let e = usize::try_from(actor - 10)
        .map_err(|_| RuleError::Invalid("active body is not an enemy".into()))?;
    let mut events = if word(&c.enemies.records[e].bytes, 0xe)? == 0
        && word(&c.enemies.records[e].bytes, 0x26)? == 1
    {
        let context = state.loaded_item_field32.ok_or(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "unreconstructable legacy work-item context",
        })?;
        super::ally_turns::scientist_turn_with_work(
            c,
            &mut state.warriors,
            mission,
            &mut state.random,
            &content.items,
            context,
            &state.party_buffs,
            &mut state.ammunition_work,
        )?
    } else {
        let mut work = EnemyWork {
            item_context: state.loaded_item_field32,
            attack_count: state.attack_count,
            packet: c.attack_packet.clone(),
            identity: c.attack_identity,
            spell_attempt: 0,
            spell_disabled: state.enemy_spell_disabled,
            buffers: state.ammunition_work.clone(),
            display_writers: true,
        };
        let events = enemy_action_with_work_and_buffs(
            c,
            &mut state.warriors,
            mission,
            &mut state.random,
            &content.items,
            &mut work,
            &state.party_buffs,
        )?;
        state.loaded_item_field32 = work.item_context;
        state.attack_count = work.attack_count;
        state.ammunition_work = work.buffers;
        events
    };
    if let Some(health) = state.health_context.as_mut() {
        for event in &events {
            match event {
                Event::SpellResolved {
                    target,
                    damage,
                    defeated,
                    ..
                } => {
                    if let Some(hp) = usize::try_from(*target)
                        .ok()
                        .and_then(|body| party_hp.get_mut(body))
                        && *damage > 0
                        && *hp >= 0
                    {
                        health.affected[*target as usize] = -1;
                        *hp = hp.wrapping_sub(*damage);
                    }
                    if *defeated {
                        health.last_body = *target;
                    }
                }
                Event::AttackResolved {
                    target,
                    damage,
                    defeated,
                    ..
                } => {
                    if let Some(hp) = usize::try_from(*target)
                        .ok()
                        .and_then(|body| party_hp.get_mut(body))
                        && *damage > 0
                    {
                        *hp = hp.wrapping_sub(*damage);
                    }
                    if *defeated {
                        health.last_body = *target;
                    }
                }
                _ => {}
            }
        }
    }
    events.extend(finish_player_action(state, content, actor)?);
    refresh_active_item_context(state, content)?;
    Ok(events)
}

pub fn player_psi(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
    target: Option<(i16, i16)>,
) -> Result<Vec<Event>, RuleError> {
    let actor = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("psi casting requires an active warrior".into()))?;
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    if state.psi_memory.is_none() {
        state.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
    }
    let mut events = super::player_psi::cast(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        state.psi_memory.as_mut().unwrap(),
        state.psi_disabled,
        super::player_psi::CastRequest {
            spell,
            power,
            target,
        },
    )?;
    state.attack_count = power;
    state.psi_menu_choices[actor as usize] = Some(crate::PsiMenuChoice {
        row: (spell % 1000 / 100 - 1) as u8,
        column: (spell % 100 - 1) as u8,
        power,
    });
    events.extend(finish_player_action(state, content, actor)?);
    Ok(events)
}

fn medical_turn(state: &Session) -> Result<usize, RuleError> {
    if state.phase != Phase::Combat
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| !story.waiting_child)
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "medical aid requires an unobstructed combat turn".into(),
        ));
    }
    state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .map(|body| body as usize)
        .ok_or_else(|| RuleError::Invalid("medical aid requires an active warrior".into()))
}

fn refresh_active_item_context(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    let Some(body) = state.combat.as_ref().and_then(Combat::active_body) else {
        return Ok(());
    };
    if body >= 10 {
        let combat = state.combat.as_ref().unwrap();
        let item = word(&combat.enemies.records[(body - 10) as usize].bytes, 0x4e)?;
        if item != 0 {
            state.load_item_work(content, item)?;
        }
        return Ok(());
    }
    if !(0..6).contains(&body) {
        return Err(RuleError::Invalid(
            "item refresh body outside represented actors".into(),
        ));
    }
    let warrior = &state.warriors[body as usize];
    let item = word(
        &warrior.bytes,
        if warrior.bytes[0x9dd] == 0 {
            0xca
        } else {
            0xe4
        },
    )?;
    if warrior.bytes[0x9dd] != 0
        && !warrior
            .inventory()?
            .iter()
            .any(|record| record.id() == item && record.0[9] == 1)
    {
        return Err(RuleError::Unsupported {
            action: "combat.medical_aid",
            dependency: "missing selected secondary during native item refresh",
        });
    }
    state.load_item_work(content, item)?;
    Ok(())
}

pub fn stop_bleeding(
    state: &mut Session,
    content: &Content,
    warrior: Option<usize>,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    match state.bandage_ready {
        None => {
            return Err(RuleError::Unsupported {
                action: "combat.stop_bleeding",
                dependency: "unreconstructable original interaction marker",
            });
        }
        Some(0) => {
            return Err(RuleError::Invalid(
                "Stop Bleeding is unavailable after the previous action".into(),
            ));
        }
        Some(_) => {}
    }
    if warrior.is_some_and(|member| member >= state.warriors.len()) {
        return Err(RuleError::Invalid("bandaging target outside squad".into()));
    }
    state.party_mask.fill(0);
    let mut events = vec![Event::Sound { resource_id: 10105 }];
    if let Some(member) = warrior {
        state.party_mask[member] = 1;
        state.warriors[member].bytes[0x9de] = 0;
        events.push(Event::Sound { resource_id: 141 });
    }
    events.extend(end_turn(state, content)?);
    refresh_active_item_context(state, content)?;
    Ok(events)
}

pub fn begin_medical_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    let actor = state
        .warriors
        .get_mut(warrior)
        .ok_or_else(|| RuleError::Invalid("medical warrior outside squad".into()))?;
    let selected = actor
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("medical inventory slot missing".into()))?;
    crate::items::compact(actor)?;
    if actor.inventory()?.get(slot) != Some(&selected) {
        return Err(RuleError::Unsupported {
            action: "combat.medical_aid",
            dependency: "inventory selection moved during item pane initialization",
        });
    }
    crate::equipment::calculate_weight(actor, content)?;
    for member in &mut state.warriors {
        put_word(&mut member.bytes, 0x60, 0);
    }
    let result = super::medical::begin(
        state,
        content,
        super::medical::BeginRequest { warrior, slot },
    )?;
    state.warriors[warrior] = result.warrior;
    state.selected_warrior = warrior;
    state.loaded_item_field32 = Some(result.loaded_item_field32);
    state.attack_count = result.count;
    state.ammunition_work.weapon = Some(crate::ammunition::definition(content, 700)?);
    state.ammunition_work.ammo = Some(crate::ammunition::definition(content, 0)?);
    state.party_mask.fill(0);
    let mut health =
        crate::story_health::context(&state.warriors, warrior, state.health_context.as_ref());
    health.marks_affected = false;
    health.effect = result.packet[18] as i8;
    state.health_context = Some(health);
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(result.packet);
    combat.attack_identity = Some(result.identity);
    combat.weapon_work = result.weapon_work;
    state.medical_continuation = Some(crate::MedicalContinuation {
        origin: crate::MedicalOrigin::OwnedNeedle,
        actor: warrior,
        item: 700,
        spell: 1401,
        power: result.count,
    });
    refresh_active_item_context(state, content)?;
    Ok(result.events)
}

pub fn begin_learned_heal(
    state: &mut Session,
    content: &Content,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let actor = medical_turn(state)?;
    if state
        .health_context
        .as_ref()
        .is_some_and(|health| health.marks_affected)
    {
        return Err(RuleError::Unsupported {
            action: "combat.learned_portrait_heal",
            dependency: "affected-target marking during prepared Heal",
        });
    }
    if state.psi_menu_choices[actor]
        .as_ref()
        .is_some_and(|choice| {
            choice.row > 6 || choice.column > 6 || !(1..=4).contains(&choice.power)
        })
    {
        return Err(RuleError::Unsupported {
            action: "combat.learned_portrait_heal",
            dependency: "other remembered psi menu selection",
        });
    }
    let mut result = super::medical::begin_learned_heal(state, content, power)?;
    if state.psi_menu_choices[actor]
        .as_ref()
        .is_some_and(|choice| choice.row != 0)
    {
        result.events[2] = Event::Sound { resource_id: 10129 };
    }
    let previous_power = state.psi_menu_choices[actor]
        .as_ref()
        .map_or(1, |choice| choice.power);
    result
        .events
        .retain(|event| !matches!(event, Event::Sound { resource_id: 137 }));
    if power != previous_power {
        result.events.push(Event::Sound { resource_id: 137 });
    }
    state.warriors[actor] = result.warrior;
    state.selected_warrior = actor;
    state.psi_menu_choices[actor] = Some(crate::PsiMenuChoice {
        row: 0,
        column: 0,
        power,
    });
    state.attack_count = result.count;
    state.party_mask.fill(0);
    let mut health =
        crate::story_health::context(&state.warriors, actor, state.health_context.as_ref());
    health.affected[actor] = 1;
    health.effect = result.packet[18] as i8;
    state.health_context = Some(health);
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(result.packet);
    combat.attack_identity = Some(result.identity);
    combat.weapon_work = result.weapon_work;
    state.medical_continuation = Some(crate::MedicalContinuation {
        origin: crate::MedicalOrigin::LearnedHeal,
        actor,
        item: 0,
        spell: 1101,
        power,
    });
    refresh_active_item_context(state, content)?;
    Ok(result.events)
}

pub fn select_medical_target(
    state: &mut Session,
    content: &Content,
    selection: crate::MedicalTarget,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    let pending = state
        .medical_continuation
        .clone()
        .ok_or_else(|| RuleError::Invalid("no medical target selection is pending".into()))?;
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .ok_or_else(|| RuleError::Invalid("medical map missing".into()))?;
    let resolution = super::medical::resolve(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &pending,
        &selection,
    )?;
    let packet = state
        .combat
        .as_ref()
        .unwrap()
        .attack_packet
        .as_ref()
        .unwrap();
    if let Some(health) = state.health_context.as_mut() {
        health.effect = packet[18] as i8;
    }
    if pending.origin == crate::MedicalOrigin::LearnedHeal
        && matches!(selection, crate::MedicalTarget::Portrait { .. })
    {
        if state.psi_memory.is_none() {
            state.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
        }
        let memory = state.psi_memory.as_mut().unwrap();
        for (index, value) in [0, 0, 0, pending.power].into_iter().enumerate() {
            put_word(memory, pending.actor * 80 + 72 + index * 2, value);
        }
    }
    state.medical_continuation = None;
    state.party_mask.fill(0);
    let mut events = resolution.events;
    match resolution.follow_up {
        super::medical::FollowUp::FinishPlayerAction => {
            events.extend(finish_player_action(state, content, pending.actor as i16)?);
        }
        super::medical::FollowUp::EndTurn => events.extend(end_turn(state, content)?),
        super::medical::FollowUp::None => {}
    }
    refresh_active_item_context(state, content)?;
    Ok(events)
}

pub fn party_psi(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state.combat.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some() || position.script.is_some())
    {
        return Err(RuleError::Invalid(
            "outdoor psi requires an unobstructed exploration turn".into(),
        ));
    }
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    if state.psi_memory.is_none() {
        state.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
    }
    let result = super::player_psi::cast_party(
        &state.warriors,
        state.psi_memory.as_ref().unwrap(),
        &state.party_buffs,
        mission,
        &mut state.random,
        state.psi_disabled,
        super::player_psi::PartyCastRequest {
            warrior,
            spell,
            power,
        },
    )?;
    state.warriors = result.warriors;
    state.psi_memory = Some(result.memory);
    state.party_buffs = result.buffs;
    state.attack_packet = Some(result.packet);
    state.attack_identity = Some(result.identity);
    state.attack_count = result.count;
    state.psi_menu_choices[warrior] = Some(crate::PsiMenuChoice {
        row: result.identity[1] as u8,
        column: result.identity[2] as u8,
        power,
    });
    Ok(result.events)
}

pub fn fit_ammunition(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
    ammunition: i16,
) -> Result<Vec<Event>, RuleError> {
    if warrior >= state.warriors.len()
        || !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || state.exploration.as_ref().is_none_or(|p| {
            p.presentation.is_some() || (state.phase == Phase::Exploration && p.script.is_some())
        })
        || (state.phase == Phase::Combat
            && (state.combat.as_ref().and_then(Combat::active_body) != Some(warrior as i16)
                || word(&state.warriors[warrior].bytes, 0x40)? < 1))
    {
        return Err(RuleError::Invalid(
            "ammunition selection requires an unobstructed active mission turn".into(),
        ));
    }
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("ammunition fitting slot missing".into()))?;
    if matches!(item.id(), 1 | 2 | 38 | 165) {
        let reserve_index = usize::try_from(ammunition.wrapping_sub(200))
            .map_err(|_| RuleError::Invalid("ammunition outside reserve table".into()))?;
        let quality = state
            .loot_quality
            .as_deref()
            .map(|quality| quality[..4].try_into().unwrap());
        let in_combat = state.phase == Phase::Combat;
        let result = crate::ammunition::fit(
            &state.warriors[warrior],
            &state.ammunition,
            content,
            crate::ammunition::FitRequest {
                slot,
                reserve_index,
                in_combat,
                loot_quality: quality,
            },
            &state.ammunition_work,
            &mut state.random,
        )?;
        let mut events = if in_combat {
            vec![Event::Sound { resource_id: 141 }]
        } else {
            vec![]
        };
        events.extend(crate::ammunition::commit_result(state, warrior, result)?);
        if in_combat {
            events.extend(finish_player_action(state, content, warrior as i16)?);
        }
        return Ok(events);
    }
    let reserve = match (item.id(), ammunition) {
        (2, 201) => 1,
        (38, 203) => 3,
        (637, 307) if state.phase == Phase::Exploration => 107,
        _ => {
            return Err(RuleError::Unsupported {
                action: "equipment.fit_ammunition",
                dependency: "other firearm and selected ammunition pair",
            });
        }
    };
    if state.ammunition.get(reserve).copied().unwrap_or(0) <= 0 {
        return Err(RuleError::Invalid(
            "selected ammunition is unavailable".into(),
        ));
    }
    let in_combat = state.phase == Phase::Combat;
    let result = super::firearm::fit_ammunition(
        &state.warriors[warrior],
        slot,
        content,
        &state.ammunition,
        in_combat,
        &mut state.random,
    )?;
    state.warriors[warrior] = result.warrior;
    state.ammunition = result.ammunition;
    state.loaded_item_field32 = Some(result.loaded_item_field32);
    state.ammunition_work.load_item(content, item.id())?;
    if result.loaded {
        state.ammunition_work.ammo = Some(crate::ammunition::definition(content, ammunition)?);
        if let Some(combat) = state.combat.as_mut() {
            combat
                .weapon_work
                .load_ammunition(&content.items[ammunition as usize])?;
        } else {
            state
                .weapon_work
                .load_ammunition(&content.items[ammunition as usize])?;
        }
    }
    let mut events: Vec<_> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    if in_combat {
        events.extend(finish_player_action(state, content, warrior as i16)?);
    }
    Ok(events)
}

pub fn player_shoot(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let refresh_health = word(&state.warriors[body as usize].bytes, 0xe4)? == 165;
    let mut events = player_shoot_action(state, content, target)?;
    events.insert(0, Event::Sound { resource_id: 145 });
    events.extend(finish_player_action(state, content, body)?);
    if refresh_health && state.combat.as_ref().and_then(Combat::active_body) != Some(body) {
        refresh_active_item_context(state, content)?;
    }
    Ok(events)
}

pub fn player_shoot_action(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    if !(0..6).contains(&body)
        || target < 10
        || !alive(c, &state.warriors, target)
        || !hostile(c, &state.warriors, body, target)?
    {
        return Err(RuleError::Invalid("select a living hostile enemy".into()));
    }
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    let warrior = &state.warriors[body as usize];
    let weapon_id = word(&warrior.bytes, 0xe4)?;
    let ammunition_id = warrior
        .inventory()?
        .iter()
        .find(|item| item.id() == weapon_id && item.0[9] == 1)
        .map(|item| word(&item.0, 4))
        .transpose()?
        .ok_or_else(|| RuleError::Invalid("equipped firearm absent from inventory".into()))?;
    let weapon = content
        .items
        .get(weapon_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("firearm definition missing".into()))?;
    let ammunition = content
        .items
        .get(ammunition_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("ammunition definition missing".into()))?;
    let prepared = firearm::prepare(warrior, weapon, ammunition, attacks)?;
    c.weapon_work.prepare(weapon, ammunition)?;
    state.ammunition_work.prepare_pair(weapon, ammunition)?;
    state.ammunition_work.item = Some(crate::ammunition::definition(content, weapon_id)?);
    state.loaded_item_field32 = Some(word(weapon, 0x20)?);
    state.attack_count = prepared.shots;
    c.attack_packet = Some(prepared.packet.clone());
    c.attack_identity = Some(spell_identity(word(weapon, 44)?));
    if prepared.shots != 1 {
        return Err(RuleError::Unsupported {
            action: "combat.shoot",
            dependency: "multiple-shot targeting",
        });
    }
    let start = position(c, body);
    let end = position(c, target);
    let dx = f32::from(start.0 - end.0);
    let dy = f32::from(start.1 - end.1);
    let distance = (dx * dx + dy * dy).sqrt().trunc() as i16;
    let range = super::player_psi::range(&prepared.packet, prepared.shots)?;
    if distance > range || !lifecycle::can_see(c, mission, start, end, 0)? {
        return Err(RuleError::Invalid(
            "enemy is outside firearm range or line of sight".into(),
        ));
    }
    let w = &mut state.warriors[body as usize].bytes;
    let slot = 0x132 + 22 * prepared.slot;
    if word(w, 0x40)? < 2 {
        return Err(RuleError::Invalid(
            "firing requires two action points".into(),
        ));
    }
    if word(w, slot + 2)? < prepared.rounds {
        return Err(RuleError::Invalid(
            "firearm empty; reload or select fists".into(),
        ));
    }
    let loaded = word(w, slot + 2)?.wrapping_sub(prepared.rounds);
    let weight = word(w, 0x9d6)?.wrapping_sub(prepared.rounds.wrapping_mul(prepared.weight));
    let actions = word(w, 0x40)?.wrapping_sub(2);
    let movement = word(w, 0x3c)?.wrapping_sub(3).max(0);
    put_word(w, slot + 2, loaded);
    put_word(w, 0x9d6, weight);
    put_word(w, 0x40, actions);
    put_word(w, 0x3c, movement);
    let result = firearm::resolve(
        &mut state.warriors[body as usize],
        &mut c.enemies.records[(target - 10) as usize],
        &prepared,
        &mut state.random,
        c.difficulty,
    )?;
    if result.killed {
        lifecycle::kill_body(c, &mut state.warriors, target)?;
        if weapon_id == 165
            && let Some(health) = state.health_context.as_mut()
        {
            health.last_body = target;
        }
    }
    let mut events = vec![Event::Sound {
        resource_id: word(&prepared.packet, 36)?,
    }];
    events.extend(
        result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id }),
    );
    if result.killed && weapon_id == 165 {
        let sound = word(&c.enemies.records[(target - 10) as usize].bytes, 0x6a)?;
        if sound != 0 {
            events.push(Event::Sound { resource_id: sound });
        }
    }
    if result.damage != 0 && !result.killed && prepared.packet[19] == 9 {
        events.push(Event::Sound {
            resource_id: word(&prepared.packet, 38)?,
        });
    }
    events.push(Event::AttackResolved {
        attacker: body,
        target,
        damage: result.damage,
        defeated: result.killed,
    });
    Ok(events)
}
pub fn initialize_active_turn(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
) -> Result<(Vec<Event>, bool), RuleError> {
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active combatant missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    if body >= 10
        && matches!(
            word(&c.enemies.records[(body - 10) as usize].bytes, 0x5e)?,
            276 | 277 | 345
        )
    {
        let turn = super::final_turn_stories::initialize(
            c,
            w,
            mission,
            &tables.monsters,
            &tables.encounters,
            random,
        )?;
        let mut events = vec![];
        for effect in turn.effects {
            use super::final_turn_stories::TurnEffect;
            match effect {
                TurnEffect::Sound(resource_id) => events.push(Event::Sound { resource_id }),
                TurnEffect::Text(id) => events.push(Event::Message {
                    id,
                    text: mission.dialogue(id)?,
                }),
                TurnEffect::Spawn { body, monster } => events.push(Event::CombatantAdded {
                    body,
                    definition: monster,
                }),
                TurnEffect::SelfRemoval { body } => events.push(Event::CombatantRemoved { body }),
            }
        }
        return Ok((events, turn.skipped_turn));
    }
    initialize_turn(body, w, &mut c.enemies.records, &tables.race_moves, random)?;
    Ok((vec![], false))
}

pub fn end_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    state.party_mask.fill(0);
    state.bandage_ready = Some(1);
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    if let Some(packet) = c.attack_packet.as_mut() {
        packet[19] = 0;
    }
    if !state.warriors.iter().any(|w| w.bytes[0x9df] != 0) {
        return Err(RuleError::Unsupported {
            action: "combat.exit",
            dependency: "defeat rewards and exit cleanup",
        });
    }
    if let Some(body) = c.active_body()
        && body < 10
    {
        let w = &mut state.warriors[body as usize].bytes;
        if word(w, 0x90)? > 0 {
            put_word(w, 0x90, 0);
        }
    }
    for _ in 0..220 {
        c.cursor += 1;
        c.spell_fields = super::spells::tick_fields(&c.spell_fields, c.cursor as i16)?;
        c.field_used = super::spells::reset_used(&c.field_used)?;
        if c.cursor == 110 {
            c.cursor = 0;
            c.round = c.round.wrapping_add(1);
            let mission = content
                .map(state.exploration.as_ref().unwrap().map)
                .ok_or_else(|| RuleError::Invalid("combat map missing".into()))?;
            let tables = mission.combat.as_ref().unwrap();
            let mut scripts = c.round_scripts.clone();
            let story = super::round_stories::advance(
                c,
                &mut scripts,
                mission,
                &tables.monsters,
                &tables.encounters,
                0,
                &mut state.random,
            )?;
            c.round_scripts = scripts;
            for effect in story.effects {
                use super::round_stories::RoundEffect;
                events.push(match effect {
                    RoundEffect::Sound(resource_id) => Event::Sound { resource_id },
                    RoundEffect::Text(id) => Event::Message {
                        id,
                        text: mission.dialogue(id)?,
                    },
                    RoundEffect::Spawn { body, monster } => Event::CombatantAdded {
                        body,
                        definition: monster,
                    },
                });
            }
            super::party_buffs::reduce(&mut state.party_buffs)?;
            let outcome = reduce_round_with_fields(
                c,
                &mut state.warriors,
                &mut state.random,
                Some(mission),
                &mut state.attack_count,
                &mut state.party_mask,
                RoundContext {
                    audio: true,
                    selected_warrior: state.selected_warrior,
                },
            )?;
            if let Some(body) = outcome.last_defeated_body
                && let Some(health) = state.health_context.as_mut()
            {
                health.last_body = body;
            }
            events.extend(outcome.events);
            if state
                .warriors
                .iter()
                .any(|warrior| warrior.bytes[0x9de] != 0)
            {
                events.push(Event::Sound { resource_id: 10122 });
            }
            events.extend([
                Event::Sound { resource_id: -139 },
                Event::Sound { resource_id: -139 },
                Event::Sound { resource_id: -139 },
            ]);
        }
        let Some(body) = c.active_body() else {
            continue;
        };
        if !alive(c, &state.warriors, body) {
            continue;
        }
        if body < 10 && word(&state.warriors[body as usize].bytes, 0x60)? != 0
            || body >= 10 && c.enemies.records[(body - 10) as usize].bytes[0xb3] != 0
        {
            continue;
        }
        let mission = content
            .map(
                state
                    .exploration
                    .as_ref()
                    .map_or(0, |position| position.map),
            )
            .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
        let (turn, skipped) =
            initialize_active_turn(c, &mut state.warriors, mission, &mut state.random)?;
        events.extend(turn);
        if skipped {
            continue;
        }
        if body < 10 && word(&state.warriors[body as usize].bytes, 0x90)? == 0 {
            events.push(Event::Sound { resource_id: 138 });
        }
        if body < 6 {
            state.selected_warrior = body as usize;
        }
        events.push(Event::TurnStarted {
            body,
            round: c.round,
        });
        return Ok(events);
    }
    Err(RuleError::Invalid("no active combatant".into()))
}
pub fn reduce_round(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
) -> Result<Vec<Event>, RuleError> {
    reduce_round_with_fields(
        c,
        w,
        random,
        None,
        &mut 0,
        &mut [0; 6],
        RoundContext {
            audio: false,
            selected_warrior: 0,
        },
    )
    .map(|outcome| outcome.events)
}

struct RoundContext {
    audio: bool,
    selected_warrior: usize,
}

fn reduce_round_with_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    context: RoundContext,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = vec![];
    let mut last_defeated_body = None;
    for member in 0..w.len() {
        let b = &mut w[member].bytes;
        if [0x8a, 0x8c, 0xa2, 0xa0]
            .iter()
            .any(|o| word(b, *o).unwrap_or(1) != 0)
        {
            return Err(RuleError::Unsupported {
                action: "combat.round",
                dependency: "ongoing regeneration, damage or bleeding",
            });
        }
        let health = word(b, 0x50)?;
        if word(b, 0x66)? != 0 && health > 0 && health < word(b, 0x52)? && word(b, 0x90)? == 0 {
            let healed = health
                .wrapping_add(word(b, 0x66)?.wrapping_abs())
                .min(word(b, 0x52)?);
            put_word(b, 0x50, healed);
            if context.audio && healed != health && b[0x9df] != 0 {
                events.push(Event::Sound { resource_id: 624 });
            }
        }
        if let Some(tick) = super::natural_conditions::disease_tick(&w[member].bytes)? {
            put_word(&mut w[member].bytes, 0x50, tick.health);
            if tick.killed {
                lifecycle::kill_body(c, w, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        if let Some(tick) = super::natural_conditions::poison_tick(&w[member].bytes)? {
            put_word(&mut w[member].bytes, 0x50, tick.health);
            if tick.killed {
                lifecycle::kill_body(c, w, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        let b = &mut w[member].bytes;
        for offset in (0x5e..0xae).step_by(2) {
            let value = word(b, offset)?;
            if value > 0 {
                put_word(b, offset, value - 1);
            }
        }
    }
    for member in 0..w.len() {
        c.field_used = super::spells::reset_used(&c.field_used)?;
        if word(&w[member].bytes, 0x50)? > 0 {
            if let Some(mission) = mission {
                let collision_events = collide_fields_impl(
                    c,
                    w,
                    mission,
                    count,
                    random,
                    FieldActor {
                        body: member as i16,
                        selected_warrior: Some(context.selected_warrior),
                    },
                    party_mask,
                )?;
                record_spell_deaths(&collision_events, &mut last_defeated_body);
                events.extend(collision_events);
            } else if c
                .spell_fields
                .as_chunks::<20>()
                .0
                .iter()
                .any(|field| field[15] != 0)
            {
                return Err(RuleError::Unsupported {
                    action: "combat.round",
                    dependency: "spell field content missing",
                });
            }
        }
        let b = &mut w[member].bytes;
        put_word(b, 0x14, 0);
        if b[0x9de] != 0 && word(b, 0x50)? > -10 {
            let health = word(b, 0x50)?.wrapping_sub(1);
            put_word(b, 0x50, health);
            if health < -9 {
                b[0x9de] = 0;
                for (offset, delta) in [(0xa62, -1_i32), (0xa5e, 1)] {
                    let value = i32::from_be_bytes(b[offset..offset + 4].try_into().unwrap())
                        .wrapping_add(delta);
                    b[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
                }
                events.push(Event::Sound {
                    resource_id: random.bound(2).wrapping_add(506),
                });
            }
        }
    }
    let enemy = reduce_enemy_round_impl(
        c,
        w,
        random,
        mission,
        count,
        party_mask,
        context.selected_warrior,
    )?;
    events.extend(enemy.events);
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body: enemy.last_defeated_body.or(last_defeated_body),
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct EnemyRoundOutcome {
    pub events: Vec<Event>,
    pub last_defeated_body: Option<i16>,
}

fn record_spell_deaths(events: &[Event], last_body: &mut Option<i16>) {
    for event in events {
        if let Event::SpellResolved {
            target,
            defeated: true,
            ..
        } = event
        {
            *last_body = Some(*target);
        }
    }
}

pub fn reduce_enemy_round_with_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    selected_warrior: usize,
) -> Result<EnemyRoundOutcome, RuleError> {
    if c.enemies.records.len() > 100
        || c.enemies.positions.len() != c.enemies.records.len()
        || c.enemies
            .records
            .iter()
            .any(|record| record.bytes.len() != 288)
        || !(1..=6).contains(&w.len())
        || w.iter().any(|warrior| warrior.bytes.len() != 3138)
        || selected_warrior >= w.len()
    {
        return Err(RuleError::Invalid("invalid enemy round input".into()));
    }
    c.validate(w)?;
    let mut next = c.clone();
    let mut next_warriors = w.to_vec();
    let mut next_random = random.clone();
    let mut next_count = *count;
    let mut next_mask = *party_mask;
    let events = reduce_enemy_round_impl(
        &mut next,
        &mut next_warriors,
        &mut next_random,
        mission,
        &mut next_count,
        &mut next_mask,
        selected_warrior,
    )?;
    *c = next;
    w.clone_from_slice(&next_warriors);
    *random = next_random;
    *count = next_count;
    *party_mask = next_mask;
    Ok(events)
}

fn reduce_enemy_round_impl(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    selected_warrior: usize,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = Vec::new();
    let mut last_defeated_body = None;
    for index in 0..c.enemies.records.len() {
        let m = &mut c.enemies.records[index];
        if [0xcc, 0xd4, 0xd3].iter().any(|o| m.bytes[*o] != 0) {
            return Err(RuleError::Unsupported {
                action: "combat.round",
                dependency: "ongoing enemy health or psi effect",
            });
        }
        for o in 0xb2..0xda {
            if (m.bytes[o] as i8) > 0 {
                m.bytes[o] -= 1;
            }
        }
        c.enemies.records[index].bytes[0x105] = 0;
        if let Some(heal) = super::natural_conditions::enemy_poison_heal(&c.enemies.records[index])?
        {
            put_word(&mut c.enemies.records[index].bytes, 0x110, heal.health);
            c.enemies.records[index].bytes[0x105] = 1;
            events.push(Event::Sound { resource_id: 624 });
        }
        if let Some(tick) =
            super::natural_conditions::enemy_drain_tick(&c.enemies.records[index].bytes)?
        {
            put_word(&mut c.enemies.records[index].bytes, 0x110, tick.health);
            events.push(Event::Sound { resource_id: 107 });
            if tick.killed {
                let monster = &c.enemies.records[index].bytes;
                if word(monster, 0x60)? != 0 || (word(monster, 0x0e)? != 0 && monster[0xb0] != 0xff)
                {
                    return Err(RuleError::Unsupported {
                        action: "combat.enemy_health_round",
                        dependency: "health-drain death script or qualifying death counter",
                    });
                }
                let sound = enemy_death_sound(&c.enemies.records[index])?;
                lifecycle::kill_body(c, w, index as i16 + 10)?;
                last_defeated_body = Some(index as i16 + 10);
                events.push(Event::Sound { resource_id: sound });
            }
        }
        if let Some(tick) = super::natural_conditions::enemy_poison_tick(&c.enemies.records[index])?
        {
            put_word(&mut c.enemies.records[index].bytes, 0x110, tick.health);
            c.enemies.records[index].bytes[0x105] = 1;
            events.push(Event::Sound { resource_id: 104 });
            if tick.killed {
                let monster = &c.enemies.records[index].bytes;
                if word(monster, 0x60)? != 0 || (word(monster, 0x0e)? != 0 && monster[0xb0] != 0xff)
                {
                    return Err(RuleError::Unsupported {
                        action: "combat.enemy_health_round",
                        dependency: "poison death script or qualifying death counter",
                    });
                }
                let sound = enemy_death_sound(&c.enemies.records[index])?;
                lifecycle::kill_body(c, w, index as i16 + 10)?;
                last_defeated_body = Some(index as i16 + 10);
                events.push(Event::Sound { resource_id: sound });
            }
        }
        c.field_used = super::spells::reset_used(&c.field_used)?;
        if word(&c.enemies.records[index].bytes, 0x110)? > 0 {
            if let Some(mission) = mission {
                let collision_events = collide_fields_impl(
                    c,
                    w,
                    mission,
                    count,
                    random,
                    FieldActor {
                        body: index as i16 + 10,
                        selected_warrior: Some(selected_warrior),
                    },
                    party_mask,
                )?;
                record_spell_deaths(&collision_events, &mut last_defeated_body);
                events.extend(collision_events);
            } else if c
                .spell_fields
                .as_chunks::<20>()
                .0
                .iter()
                .any(|field| field[15] != 0)
            {
                return Err(RuleError::Unsupported {
                    action: "combat.round",
                    dependency: "spell field content missing",
                });
            }
        }
        c.enemies.records[index].bytes[0x105] = 0;
    }
    // The native loop also visits the known zero records after the represented bodies.
    if c.enemies.records.len() < 100 {
        c.field_used = super::spells::reset_used(&c.field_used)?;
    }
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body,
    })
}

pub fn collide_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    collide_fields_with_mask(c, w, mission, count, random, body, &mut [0; 6])
}

pub fn collide_fields_with_mask(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    body: i16,
    party_mask: &mut [i16; 6],
) -> Result<Vec<Event>, RuleError> {
    let mut next = c.clone();
    let mut next_warriors = w.to_vec();
    let mut next_count = *count;
    let mut next_random = random.clone();
    let mut next_mask = *party_mask;
    let events = collide_fields_impl(
        &mut next,
        &mut next_warriors,
        mission,
        &mut next_count,
        &mut next_random,
        FieldActor {
            body,
            selected_warrior: None,
        },
        &mut next_mask,
    )?;
    *c = next;
    w.clone_from_slice(&next_warriors);
    *count = next_count;
    *random = next_random;
    *party_mask = next_mask;
    Ok(events)
}

struct FieldActor {
    body: i16,
    selected_warrior: Option<usize>,
}

fn collide_fields_impl(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    actor: FieldActor,
    party_mask: &mut [i16; 6],
) -> Result<Vec<Event>, RuleError> {
    let body = actor.body;
    if !c
        .spell_fields
        .as_chunks::<20>()
        .0
        .iter()
        .any(|field| field[15] != 0)
    {
        return Ok(vec![]);
    }
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("spell collision tables missing".into()))?;
    let mut events = vec![];
    let mut usage = super::spells::FieldUsage {
        slot: 0,
        used: c.field_used.clone(),
    };
    for slot in 0..60 {
        usage.slot = slot;
        let Some(collision) = super::spells::collision_field(
            c,
            w,
            &c.spell_fields,
            &usage,
            body as usize,
            &tables.attacks,
        )?
        else {
            continue;
        };
        *count = collision.count;
        c.attack_packet = Some(collision.packet.clone());
        let mut identity = spell_identity(collision.id);
        identity[1] = collision.level;
        c.attack_identity = Some(identity);
        let context = super::spells::SpellContext {
            packet: &collision.packet,
            id: collision.id,
            count: *count,
            caster: body,
            initiative: c.cursor as i16,
            collision: true,
        };
        let mask = super::spells::target_mask(
            c,
            w,
            &mission.areas,
            &collision.target,
            &context,
            &c.spell_fields,
            random,
        )?;
        party_mask.copy_from_slice(&mask.mask[..6]);
        let previous_hp = w
            .iter()
            .map(Warrior::stamina)
            .collect::<Result<Vec<_>, _>>()?;
        let result = super::spells::resolve_collision_with_item_cache(
            w,
            &c.enemies.records,
            &mask.mask,
            &super::spells::DirectSpell {
                packet: &collision.packet,
                level: collision.level,
                count: *count,
                armor_filter: c.weapon_work.armor_filter()?,
                weapon_sound: c.weapon_work.sound()?,
            },
            &mut usage,
            &mission.item_cache,
            random,
        )?;
        w.clone_from_slice(&result.warriors);
        c.enemies.records = result.enemies;
        if let Some(member) = actor.selected_warrior
            && c.active_body()
                .is_some_and(|active| (0..6).contains(&active))
        {
            let warrior = w
                .get_mut(member)
                .ok_or_else(|| RuleError::Invalid("field credit warrior missing".into()))?;
            for (target, outcome) in &result.targets {
                if *target >= 10 && outcome.damage > 0 {
                    increment_stat(&mut warrior.bytes, 0xa46, i32::from(outcome.damage));
                    if collision.packet[19] == 9 {
                        increment_stat(&mut warrior.bytes, 0xa4a, 1);
                    }
                    if outcome.killed {
                        increment_stat(&mut warrior.bytes, 0xa5a, 1);
                    }
                }
            }
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        append_spell_outcomes(
            c,
            w,
            result.targets,
            result.race_excluded,
            (None, collision.id, &collision.packet, &previous_hp),
            actor.selected_warrior.is_some(),
            &mut events,
        )?;
    }
    c.field_used = usage.used;
    Ok(events)
}

pub fn victory_records(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    random: &mut RandomState,
) -> Result<i32, RuleError> {
    Ok(victory_loot(warriors, enemies, random, None)?.0)
}

pub fn victory_loot(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    random: &mut RandomState,
    content: Option<&Content>,
) -> Result<(i32, Vec<InventoryRecord>), RuleError> {
    let mut loot_ids = vec![];
    for w in warriors.iter_mut() {
        let b = &mut w.bytes;
        put_word(b, 0x10, 0);
        b[0x9de] = 0;
        put_word(b, 0x60, 0);
        for o in (0x5e..0x66).step_by(2) {
            if word(b, o)? > 0 {
                put_word(b, o, 0);
            }
        }
        if word(b, 0x90)? > 0 {
            put_word(b, 0x90, 0);
        }
    }
    let mut experience = 0_i32;
    for m in enemies.iter_mut() {
        let b = &mut m.bytes;
        let level = word(b, 0)?;
        if !matches!(level, 0..=3 | 7) || word(b, 0x10e)? != 0 || word(b, 0x3a)? > 0 {
            return Err(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "other monster rewards",
            });
        }
        random.range(0, word(b, 0x3a)?);
        let flying = if b[0x7f] != 0 {
            30 * i32::from(level)
        } else {
            0
        };
        let (base, multiplier) = if level == 7 {
            (200, 24)
        } else {
            [(15, 3), (30, 6), (45, 9), (65, 12)][level as usize]
        };
        experience = experience.wrapping_add(
            base + flying + multiplier * i32::from(word(b, 0x112)?) + i32::from(word(b, 0x54)?),
        );
        for offset in (0x26..0x32).step_by(2) {
            let id = word(b, offset)?;
            if id == 0 {
                continue;
            }
            let content = content.ok_or(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "monster loot content",
            })?;
            let definition = content
                .items
                .get(id.unsigned_abs() as usize)
                .ok_or_else(|| RuleError::Invalid("monster loot outside item bands".into()))?;
            if random.bound(150) < 70 || matches!(word(definition, 0x1a)?, 23..=25) {
                loot_ids.push(id);
            }
        }
        for offset in [0xb2, 0xb3, 0xb4, 0xb7, 0xcf, 0xd8, 0xd4, 0x105] {
            b[offset] = 0;
        }
        put_word(b, 0xe, 1);
    }
    let eligible = warriors
        .iter()
        .filter(|w| {
            w.stamina().unwrap_or(0) > 0
                && word(&w.bytes, 0x90).unwrap_or(-1) >= 0
                && w.bytes[0x9df] != 0
        })
        .count() as i32;
    let share = if eligible != 0 {
        (experience as f32 as i32 / eligible).abs()
    } else {
        0
    };
    for w in warriors {
        if w.stamina()? > 0 && word(&w.bytes, 0x90)? >= 0 && w.bytes[0x9df] != 0 {
            for (offset, sign) in [(0x9d2, 1_i32), (0x9ce, -1)] {
                let current = i32::from_be_bytes(w.bytes[offset..offset + 4].try_into().unwrap());
                let next = current.wrapping_add(share.wrapping_mul(sign));
                w.bytes[offset..offset + 4].copy_from_slice(&next.to_be_bytes());
            }
        }
    }
    let mut loot = vec![];
    if let Some(content) = content {
        for id in loot_ids {
            loot.push(items::initialize_at_difficulty(id, 0, 0, content, random)?);
        }
        loot.sort_by_key(|item| std::cmp::Reverse(item.0[10] as i8));
    }
    Ok((share, loot))
}
fn finish_victory(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    if combat.enemies.records.iter().any(|record| {
        word(&record.bytes, 0x110).unwrap_or(0) > 0
            && word(&record.bytes, 0xe).unwrap_or(1) == 0
            && word(&record.bytes, 0x5a).unwrap_or(0) != 0
    }) {
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Survivors);
        return Ok(vec![Event::StoryPresented]);
    }
    confirm_survivors(state, content)
}

pub fn confirm_survivors(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let synchronization = super::allies::synchronize(
        &state.allies,
        state.ally_count,
        &mut combat.enemies.records,
        &mut state.random,
    )?;
    state.allies = synchronization.menu;
    state.ally_count = synchronization.count;
    let mut events: Vec<_> = synchronization
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    state.exploration.as_mut().unwrap().presentation = None;
    if let Some(start) = state.battle_npc.as_ref() {
        let script = state
            .exploration
            .as_mut()
            .unwrap()
            .script
            .as_mut()
            .ok_or_else(|| RuleError::Invalid("NPC battle continuation missing".into()))?;
        script.cursor = start.restore(&mut script.record, false)?.unwrap() as u8;
    }
    let rewards = super::rewards::prepare(
        &state.warriors,
        &combat.enemies.records,
        content,
        state.difficulty,
        0,
        state.party_currency,
        &mut state.random,
    )?;
    combat.unavailable = rewards.unavailable;
    if let Some(id) = rewards.last_item_id {
        state.loaded_item_field32 = Some(state.ammunition_work.load_item(content, id)?);
    }
    state.warriors = rewards.warriors;
    combat.enemies.records = rewards.enemies;
    state.party_currency = rewards.currency;
    let needs_promotion = state.warriors.iter().any(|warrior| {
        warrior.stamina().unwrap_or(0) > 0
            && i32::from_be_bytes(warrior.bytes[0x9ce..0x9d2].try_into().unwrap()) < 0
    });
    for item in &mut state.booty_items {
        item.0.fill(0);
    }
    for (index, item) in rewards.items.into_iter().enumerate() {
        if index >= state.booty_items.len() {
            return Err(RuleError::Invalid(
                "battle loot exceeds booty capacity".into(),
            ));
        }
        state.booty_items[index] = item;
    }
    if needs_promotion || state.booty_items.iter().any(|i| i.id() != 0) {
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    }
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    state.weapon_work = combat.weapon_work;
    let mut health = crate::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = combat.unavailable;
    if let Some(packet) = combat.attack_packet.as_ref() {
        health.effect = packet[18] as i8;
    }
    state.health_context = Some(health);
    state.combat = None;
    if !state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        state.combat_story = None;
    } else {
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    }
    state.handled_deaths.clear();
    state.phase = Phase::Exploration;
    events.extend(finish_battle_wrapper(state, content)?);
    Ok(events)
}

fn finish_battle_wrapper(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if state.battle_npc.take().is_some() {
        crate::story_npc_battle::finish(
            crate::story_npc_battle::NpcBattleReturn::Reward,
            &mut state.warriors,
            [0; 2],
        )?;
    }
    let failure_script = state.battle_group_script.take().unwrap_or(0);
    state.warriors =
        crate::story_party::group_return(&state.warriors, false, failure_script)?.warriors;
    events.push(Event::BattleEnded { victory: true });
    if state.battle_after_message != 0 {
        let map = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        events.push(Event::Message {
            id: state.battle_after_message,
            text: map.dialogue(state.battle_after_message)?,
        });
    }
    state.battle_after_message = 0;
    if state.battle_treasure != 0 {
        let map = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        let treasure = centurions_content::span(
            &map.treasure,
            state.battle_treasure.unsigned_abs() as usize * 88,
            88,
        )?;
        for item in crate::story_inventory::grant_story_treasure(
            treasure,
            content,
            &mut state.random,
            state.difficulty,
        )? {
            state.load_item_work(content, item.id())?;
            let slot = state
                .booty_items
                .iter()
                .position(|item| item.id() == 0)
                .ok_or_else(|| {
                    RuleError::Invalid("battle treasure exceeds booty capacity".into())
                })?;
            state.booty_items[slot] = item;
        }
        state.battle_treasure = 0;
        if state.booty_items.iter().any(|item| item.id() != 0) {
            state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        }
    }
    if state.combat_story.is_none()
        && state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_none() && position.script.is_some())
        && !events
            .iter()
            .any(|event| matches!(event, Event::Message { .. }))
    {
        events.extend(crate::exploration::run_script(state, content)?);
    }
    Ok(events)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathStory {
    pub body: usize,
    pub script: i16,
    pub cursor: usize,
    pub actor: i16,
    #[serde(default)]
    pub entry: super::death_scripts::DeathEntry,
    #[serde(default)]
    pub waiting_child: bool,
    #[serde(default)]
    pub outer_after_message: i16,
    #[serde(default)]
    pub outer_treasure: i16,
    #[serde(default)]
    pub outer_group: Option<i16>,
    #[serde(default)]
    pub outer_npc: Option<crate::story_npc_battle::NpcBattleStart>,
}

fn finish_player_action(
    state: &mut Session,
    content: &Content,
    actor: i16,
) -> Result<Vec<Event>, RuleError> {
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    if !c.death_queue.processing {
        c.death_queue.begin();
    }
    if let Some((_, entry)) = c.death_queue.current() {
        if state.combat_story.is_some() {
            return Err(RuleError::Unsupported {
                action: "combat.death_story",
                dependency: "recursive child battle death scripts",
            });
        }
        state.combat_story = Some(DeathStory {
            body: 0,
            script: entry.script,
            cursor: 0,
            actor,
            entry,
            waiting_child: false,
            outer_after_message: state.battle_after_message,
            outer_treasure: state.battle_treasure,
            outer_group: state.battle_group_script,
            outer_npc: state.battle_npc.clone(),
        });
        return continue_death(state, content);
    }
    c.death_queue.finish();
    if c.enemies
        .records
        .iter()
        .all(|m| m.stamina().unwrap_or(0) <= 0 || word(&m.bytes, 0xe).unwrap_or(1) == 0)
    {
        return finish_victory(state, content);
    }
    if actor >= 10 || word(&state.warriors[actor as usize].bytes, 0x40)? < 2 {
        return end_turn(state, content);
    }
    Ok(vec![])
}

fn finish_death_story(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let story = state
        .combat_story
        .take()
        .ok_or_else(|| RuleError::Invalid("death continuation missing".into()))?;
    if let Some(combat) = state.combat.as_mut() {
        combat.death_queue.acknowledge()?;
        super::death_scripts::clear_body_references(combat, &mut state.warriors, 0)?;
        finish_player_action(state, content, story.actor)
    } else {
        // Child combat replaces the global roster. The outer reward sees count zero.
        for item in &mut state.booty_items {
            item.0.fill(0);
        }
        state.battle_after_message = story.outer_after_message;
        state.battle_treasure = story.outer_treasure;
        state.battle_group_script = story.outer_group;
        state.battle_npc = story.outer_npc;
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        finish_battle_wrapper(state, content)
    }
}

pub fn continue_death(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        if state.combat.is_some() {
            return Err(RuleError::Invalid(
                "finish the child battle before continuing".into(),
            ));
        }
        if state.booty_items.iter().any(|item| item.id() != 0) {
            return Err(RuleError::Invalid(
                "collect or store the equipment before continuing".into(),
            ));
        }
        state.combat_story.as_mut().unwrap().waiting_child = false;
    }
    state.exploration.as_mut().unwrap().presentation = None;
    let map = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .unwrap();
    for _ in 0..8 {
        let story = state.combat_story.as_mut().unwrap();
        if story.cursor >= 8 {
            events.extend(finish_death_story(state, content)?);
            return Ok(events);
        }
        let record = map.script(story.script)?;
        let opcode = word(&record, 8 + story.cursor * 2)?;
        let parameter = word(&record, 24 + story.cursor * 2)?;
        story.cursor += 1;
        match opcode {
            0 => {}
            1 => {
                state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue {
                    id: parameter,
                    text: map.dialogue(parameter)?,
                });
                events.push(Event::StoryPresented);
                return Ok(events);
            }
            9 => events.push(Event::Sound {
                resource_id: parameter,
            }),
            47 => state.flags = crate::story::set_mission_flag(&state.flags, parameter)?,
            2 => {
                let mut continuation = state.combat_story.take().unwrap();
                continuation.waiting_child = true;
                events.extend(super::begin(state, content, parameter)?);
                state.combat_story = Some(continuation);
                return Ok(events);
            }
            125 => {
                let extra = centurions_content::span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let class = word(extra, 0)?;
                let count = word(extra, 2)?;
                let radius = match word(extra, 4)? {
                    0 => 1000,
                    v => v,
                };
                let combat = state.combat.as_mut().unwrap();
                let origin = (
                    story.entry.x - i32::from(combat.map.shift.0) - 5,
                    story.entry.y - i32::from(combat.map.shift.1) - 5,
                );
                for _ in 0..count {
                    let mut nearest = None;
                    let mut distance = radius;
                    for (index, m) in combat.enemies.records.iter().enumerate() {
                        if m.stamina()? <= 0 || i16::from(m.bytes[0x103] as i8) != class {
                            continue;
                        }
                        let p = combat.enemies.positions[index];
                        let dx = (origin.0 - i32::from(p.0)) as f32;
                        let dy = (origin.1 - i32::from(p.1)) as f32;
                        let current = (dx * dx + dy * dy).sqrt().trunc() as i16;
                        if current < distance {
                            distance = current;
                            nearest = Some(index);
                        }
                    }
                    let Some(index) = nearest else {
                        break;
                    };
                    lifecycle::destroy_enemy(combat, index)?;
                }
            }
            119 | 123 => {
                let combat = state.combat.as_mut().unwrap();
                let mut unavailable = combat.unavailable;
                let result = super::mission_deaths::apply(
                    combat,
                    &mut state.warriors,
                    map,
                    &map.combat.as_ref().unwrap().encounters,
                    super::mission_deaths::DeathContext {
                        script: story.script,
                        actor: 0,
                        deferred_faction: Some(story.entry.faction),
                    },
                    &mut unavailable,
                )?;
                combat.unavailable = unavailable;
                let _ = result;
            }
            _ => {
                return Err(RuleError::Unsupported {
                    action: "combat.death_story",
                    dependency: "other death story instructions",
                });
            }
        }
    }
    events.extend(finish_death_story(state, content)?);
    Ok(events)
}
