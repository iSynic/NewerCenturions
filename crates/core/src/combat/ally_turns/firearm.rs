//! Friendly enemy-record firearm caller. Its range gate considers the furthest
//! living opponent, even when another visible opponent is in range. Keep this
//! distinct from player shooting and the hostile enemy caller.
use super::{alive, faction, position, result_audio};
use crate::combat::{
    Combat, damage, enemy_weapons, firearm, lifecycle, runtime, weapon_work::WeaponWork,
};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{AttackPacket, ItemDefinition},
};
use damage::{DamageFlags, DamagePacket, resolve_damage};

/// The records and work this caller reads and writes, live or retained after teardown.
/// The duel's resolution over a warrior record, the firing NPC and the draws.
pub(crate) type Duel<'a> =
    &'a mut dyn FnMut(
        &mut Warrior,
        &mut MonsterRecord,
        &mut RandomState,
    ) -> Result<crate::combat::attack::AttackResult, RuleError>;

pub(crate) trait ScientistWorld {
    fn random(&mut self) -> &mut RandomState;
    fn npc_count(&self) -> Result<usize, RuleError>;
    fn npc(&self, index: usize) -> Result<&MonsterRecord, RuleError>;
    fn npc_mut(&mut self, index: usize) -> Result<&mut MonsterRecord, RuleError>;
    fn npc_and_random(
        &mut self,
        index: usize,
    ) -> Result<(&mut MonsterRecord, &mut RandomState), RuleError>;
    /// Resolves the duration roll against a warrior table record and the firing NPC.
    fn duel(
        &mut self,
        warrior: usize,
        npc: usize,
        resolve: Duel<'_>,
    ) -> Result<crate::combat::attack::AttackResult, RuleError>;
    fn alive(&self, body: i16) -> Result<bool, RuleError>;
    fn faction(&self, body: i16) -> Result<i16, RuleError>;
    fn position(&self, body: i16) -> Result<(i16, i16), RuleError>;
    fn visible(
        &self,
        mission: &centurions_content::mission::OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError>;
    fn active_body(&self) -> Result<Option<i16>, RuleError>;
    fn difficulty(&self) -> i16;
    fn weapon_work(&mut self) -> &mut WeaponWork;
    fn buffers(&mut self) -> &mut crate::ammunition::WorkBuffers;
    fn publish(&mut self, packet: Vec<u8>, count: i16);
    fn refresh_item(&mut self, items: &[Vec<u8>], body: i16) -> Result<(), RuleError>;
    /// killbody; true when party loss leaves the caller.
    fn kill(&mut self, body: i16, events: &mut Vec<Event>) -> Result<bool, RuleError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shot {
    Withheld,
    Fired,
    Ended,
}

fn packet(weapon: &[u8], ammo: &[u8], attacks: &[u8]) -> Result<Vec<u8>, RuleError> {
    use centurions_content::records::{WeaponAttackModifiers, WeaponAttackParameters};
    let mut packet = centurions_content::span(attacks, 3 * 2842, 58)?.to_vec();
    let mut parameters = WeaponAttackParameters::read(&packet)?;
    firearm::compose_weapon_attack(
        &mut parameters,
        &WeaponAttackModifiers::read(weapon)?,
        &WeaponAttackModifiers::read(ammo)?,
        0,
    );
    parameters.write(&mut packet);
    Ok(packet)
}

struct Live<'a> {
    c: &'a mut Combat,
    w: &'a mut [Warrior],
    random: &'a mut RandomState,
    buffers: &'a mut crate::ammunition::WorkBuffers,
    attack_count: &'a mut i16,
}

impl ScientistWorld for Live<'_> {
    fn random(&mut self) -> &mut RandomState {
        self.random
    }
    fn npc_count(&self) -> Result<usize, RuleError> {
        Ok(self.c.enemies.records.len())
    }
    fn npc(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        Ok(&self.c.enemies.records[index])
    }
    fn npc_mut(&mut self, index: usize) -> Result<&mut MonsterRecord, RuleError> {
        Ok(&mut self.c.enemies.records[index])
    }
    fn npc_and_random(
        &mut self,
        index: usize,
    ) -> Result<(&mut MonsterRecord, &mut RandomState), RuleError> {
        Ok((&mut self.c.enemies.records[index], self.random))
    }
    fn duel(
        &mut self,
        warrior: usize,
        npc: usize,
        resolve: Duel<'_>,
    ) -> Result<crate::combat::attack::AttackResult, RuleError> {
        resolve(
            &mut self.w[warrior],
            &mut self.c.enemies.records[npc],
            self.random,
        )
    }
    fn alive(&self, body: i16) -> Result<bool, RuleError> {
        Ok(alive(self.c, self.w, body))
    }
    fn faction(&self, body: i16) -> Result<i16, RuleError> {
        faction(self.c, self.w, body)
    }
    fn position(&self, body: i16) -> Result<(i16, i16), RuleError> {
        Ok(position(self.c, body))
    }
    fn visible(
        &self,
        mission: &centurions_content::mission::OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        lifecycle::can_see(self.c, mission, from, to, 0)
    }
    fn active_body(&self) -> Result<Option<i16>, RuleError> {
        Ok(self.c.active_body())
    }
    fn difficulty(&self) -> i16 {
        self.c.difficulty
    }
    fn weapon_work(&mut self) -> &mut WeaponWork {
        &mut self.c.weapon_work
    }
    fn buffers(&mut self) -> &mut crate::ammunition::WorkBuffers {
        self.buffers
    }
    fn publish(&mut self, packet: Vec<u8>, count: i16) {
        *self.attack_count = count;
        self.c.attack_packet = packet;
        self.c.attack_identity = [3, 0, 0];
    }
    fn refresh_item(&mut self, items: &[Vec<u8>], body: i16) -> Result<(), RuleError> {
        runtime::refresh_body_item(self.c, self.w, items, body, self.buffers)
    }
    fn kill(&mut self, body: i16, _events: &mut Vec<Event>) -> Result<bool, RuleError> {
        lifecycle::kill_body(self.c, self.w, body)?;
        Ok(false)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn fire(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    events: &mut Vec<Event>,
    actor: (i16, i16),
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
    attack_count: &mut i16,
) -> Result<bool, RuleError> {
    let mut world = Live {
        c,
        w,
        random,
        buffers,
        attack_count,
    };
    Ok(fire_in(&mut world, mission, items, events, actor, display_writers)? == Shot::Fired)
}

/// The caller over either world; a retained world's killbody may end the party.
pub(crate) fn fire_in(
    world: &mut impl ScientistWorld,
    mission: &centurions_content::mission::OpeningMission,
    items: &[Vec<u8>],
    events: &mut Vec<Event>,
    actor: (i16, i16),
    display_writers: bool,
) -> Result<Shot, RuleError> {
    let (item_context, body) = actor;
    let a = (body - 10) as usize;
    let source = world.npc(a)?.clone();
    let weapon_id = source.record().primary_weapon()?;
    if weapon_id != 1 {
        return Err(RuleError::Unsupported {
            action: "combat.scientist_firearm",
            dependency: "ally rifle caller requires firearm one",
        });
    }
    let weapon = items
        .get(weapon_id as usize)
        .ok_or_else(|| RuleError::Invalid("ally weapon missing".into()))?;
    let ammo_id = enemy_weapons::ammo_id(&source, item_context)?;
    let ammo = items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("ally ammunition missing".into()))?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    world.npc_mut(a)?.record_mut().set_selected_item(1);
    let p = packet(weapon, ammo, attacks)?;
    world.weapon_work().prepare(weapon, ammo)?;
    world.buffers().prepare_pair(weapon, ammo)?;
    world.publish(p.clone(), AttackPacket(&p).attack_count());
    world.random().bound(7);
    let shots = i16::from(ItemDefinition(weapon).selected_attack_count());
    let range = AttackPacket(&p)
        .attack_count()
        .wrapping_mul(shots)
        .wrapping_add(AttackPacket(&p).range_modifier())
        .wrapping_abs();
    let start = world.position(body)?;
    let mut distances = [127_i16; 110];
    let mut count = 0;
    let mut maximum = 0;
    let bound = world.npc_count()? as i16 + 10;
    for target in 0..bound {
        if !world.alive(target)? || world.faction(body)? == world.faction(target)? {
            continue;
        }
        let end = world.position(target)?;
        let dx = f32::from(start.0 - end.0);
        let dy = f32::from(start.1 - end.1);
        let distance = (dx * dx + dy * dy).sqrt().trunc() as i16;
        maximum = maximum.max(distance);
        if distance <= range && world.visible(mission, start, end)? {
            distances[target as usize] = distance;
            count += 1;
        }
    }
    if count == 0 || range < maximum {
        return Ok(Shot::Withheld);
    }
    let mut selected = None;
    for _ in 0..100 {
        let target = (world.random().bound(bound) - 1) as usize;
        if distances[target] <= range {
            selected = Some(target);
            break;
        }
    }
    let Some(target) = selected else {
        return Ok(Shot::Withheld);
    };
    let actor = world.npc_mut(a)?;
    let action_points = actor.record().action_points().wrapping_sub(2);
    let movement_points = actor.record().movement_points().wrapping_sub(3);
    actor.record_mut().set_action_points(action_points);
    actor.record_mut().set_movement_points(movement_points);
    actor.record_mut().set_reaction_ready(action_points >= 1);
    if display_writers {
        world.refresh_item(items, body)?;
    }
    events.push(Event::Sound {
        resource_id: AttackPacket(&p).launch_sound()?,
    });
    let (minimum_duration, maximum_duration) = AttackPacket(&p).duration_bounds();
    world.random().range(minimum_duration, maximum_duration);
    if target < 6 {
        let prepared = firearm::PreparedFirearm {
            sound: AttackPacket(&p).impact_sound()?,
            packet: p,
            slot: 0,
            shots: 1,
            rounds: ItemDefinition(weapon).rounds_per_shot()?,
            weight: 0,
            armor_filter: world.weapon_work().armor_filter(),
        };
        let difficulty = world.difficulty();
        let result = world.duel(target, a, &mut |warrior, shooter, random| {
            firearm::resolve_enemy_after_duration(warrior, shooter, &prepared, random, difficulty)
        })?;
        if result.killed && world.kill(target as i16, events)? {
            return Ok(Shot::Ended);
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if result.damage != 0 {
            if !result.killed {
                events.push(Event::Sound {
                    resource_id: result_audio(world.npc(a)?, 0, result.damage, prepared.sound)?,
                });
            }
            events.push(Event::AttackResolved {
                attacker: body,
                target: target as i16,
                damage: result.damage,
                defeated: result.killed,
                icon: crate::event::PresentationIcon(result.icon),
                flight: crate::event::SpellArea::single(target as i16, &prepared.packet),
            });
        }
        return Ok(Shot::Fired);
    }
    let t = target - 10;
    let mut hit = 33_i16;
    if world.active_body()?.is_some_and(|actor| actor > 9) {
        hit = hit
            .wrapping_add(source.record().level()?.wrapping_mul(5))
            .wrapping_add(source.record().missile_accuracy_bonus()?);
    }
    let difficulty = world.difficulty();
    let (victim, random) = world.npc_and_random(t)?;
    let victim = victim.record();
    if victim.turn_skip_duration() == 0 {
        hit = hit
            .wrapping_sub(victim.missile_dodge_bonus()?)
            .wrapping_sub(random.bound(victim.speed()? / 3))
            .wrapping_sub(victim.level()?.wrapping_mul(3))
            .wrapping_sub(i16::from(victim.guarding()).wrapping_mul(victim.level()?));
        hit = hit.wrapping_add(match victim.size()?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        if random.bound(100)
            > hit.clamp(
                15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
                90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            )
        {
            return Ok(Shot::Fired);
        }
    }
    let (victim, random) = world.npc_and_random(t)?;
    let result = resolve_damage(
        &mut victim.bytes,
        true,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    if result.shield_sound {
        events.push(Event::Sound { resource_id: 206 });
    }
    if result.total != 0 {
        let hp = world.npc(t)?.record().stamina()?.wrapping_sub(result.total);
        world.npc_mut(t)?.record_mut().set_stamina(hp);
        if hp < 1 {
            if world.kill(target as i16, events)? {
                return Ok(Shot::Ended);
            }
        } else {
            events.push(Event::Sound {
                resource_id: AttackPacket(&p).impact_sound()?,
            });
            events.push(Event::Sound {
                resource_id: result_audio(
                    world.npc(a)?,
                    0,
                    result.total,
                    AttackPacket(&p).impact_sound()?,
                )?,
            });
        }
        events.push(Event::AttackResolved {
            attacker: body,
            target: target as i16,
            damage: result.total,
            defeated: hp < 1,
            icon: crate::event::PresentationIcon(result.sound),
            flight: crate::event::SpellArea::single(target as i16, &p),
        });
    }
    Ok(Shot::Fired)
}
