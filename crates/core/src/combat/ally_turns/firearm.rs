//! A friendly monster fires. Its range test looks at the farthest living
//! opponent, even when another visible opponent is in range; this differs from
//! both the warriors' shooting and a hostile monster's.
use super::{alive, result_audio};
use crate::combat::attack::AttackResult;
use crate::combat::{
    Combat, damage, enemy_weapons, firearm, firearm::weapon_traits::WeaponTraits, lifecycle,
    runtime,
};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{ItemDefinition, SpellInfoRecord},
};
use damage::{DamageFlags, DamagePacket, resolve_damage};

/// A shot's resolution against a warrior record, the firing monster and the draws.
pub(crate) type Duel<'a> = &'a mut dyn FnMut(
    &mut Warrior,
    &mut MonsterRecord,
    &mut RandomState,
) -> Result<AttackResult, RuleError>;

/// What a friendly firearm monster's shot needs of the battle: the monsters and their draws, who
/// can be aimed at and seen, the weapon and ammunition in hand, and the effects of a kill.
pub(crate) trait ScientistWorld {
    /// The battle's random stream.
    fn random(&mut self) -> &mut RandomState;
    /// How many monsters are in the battle.
    fn monster_count(&self) -> Result<usize, RuleError>;
    /// The monster at `index`.
    fn monster(&self, index: usize) -> Result<&MonsterRecord, RuleError>;
    /// The monster at `index`, for changing.
    fn monster_mut(&mut self, index: usize) -> Result<&mut MonsterRecord, RuleError>;
    /// A monster and the random stream together, so a roll can change the monster.
    fn monster_and_random(
        &mut self,
        index: usize,
    ) -> Result<(&mut MonsterRecord, &mut RandomState), RuleError>;
    /// Resolves the shot against a warrior record and the firing monster.
    fn duel(
        &mut self,
        warrior: usize,
        monster: usize,
        resolve: Duel<'_>,
    ) -> Result<AttackResult, RuleError>;
    /// Whether the range scan counts a body: a warrior in battle with stamina
    /// left, or a monster with stamina that is not disabled.
    fn targetable(&self, body: i16) -> Result<bool, RuleError>;
    /// Which side a warrior or monster fights for.
    fn faction(&self, body: i16) -> Result<i16, RuleError>;
    /// Where a body stands on the battlefield.
    fn position(&self, body: i16) -> Result<(i16, i16), RuleError>;
    /// Whether the line between two cells is clear.
    fn visible(
        &self,
        mission: &centurions_content::mission::OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError>;
    /// The body whose turn it is, if any.
    fn active_body(&self) -> Result<Option<i16>, RuleError>;
    /// The battle's difficulty setting.
    fn difficulty(&self) -> i16;
    /// The traits of the weapon and ammunition currently in hand.
    fn weapon_traits(&mut self) -> &mut WeaponTraits;
    /// The loaded weapon and ammunition records.
    fn buffers(&mut self) -> &mut crate::ammunition::LoadedItems;
    /// Makes the shot's attack packet the spell being resolved, with its attack count.
    fn publish(&mut self, packet: Vec<u8>, count: i16);
    /// Reloads the item shown in hand for `body`.
    fn refresh_item(&mut self, items: &[Vec<u8>], body: i16) -> Result<(), RuleError>;
    /// Kills the body; true when losing the party ends the battle.
    fn kill(&mut self, body: i16, events: &mut Vec<Event>) -> Result<bool, RuleError>;
}

/// How a rifle shot ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shot {
    /// The monster held its fire.
    Withheld,
    /// The shot was taken, hit or miss.
    Fired,
    /// The shot killed a warrior and the party was lost, ending the battle.
    Ended,
}

/// The attack packet for the rifle: the base attack for firearm shots with the weapon's and
/// ammunition's modifiers added.
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

/// The live battle, seen as a `ScientistWorld`.
struct Live<'a> {
    c: &'a mut Combat,
    w: &'a mut [Warrior],
    random: &'a mut RandomState,
    buffers: &'a mut crate::ammunition::LoadedItems,
    attack_count: &'a mut i16,
}

impl ScientistWorld for Live<'_> {
    /// The battle's random stream.
    fn random(&mut self) -> &mut RandomState {
        self.random
    }
    /// The battle's monster count.
    fn monster_count(&self) -> Result<usize, RuleError> {
        Ok(self.c.roster.records.len())
    }
    /// The monster at `index`.
    fn monster(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        Ok(&self.c.roster.records[index])
    }
    /// The monster at `index`, for changing.
    fn monster_mut(&mut self, index: usize) -> Result<&mut MonsterRecord, RuleError> {
        Ok(&mut self.c.roster.records[index])
    }
    /// A monster and the battle's random stream together.
    fn monster_and_random(
        &mut self,
        index: usize,
    ) -> Result<(&mut MonsterRecord, &mut RandomState), RuleError> {
        Ok((&mut self.c.roster.records[index], self.random))
    }
    /// Resolves the shot against a warrior with the live records and draws.
    fn duel(
        &mut self,
        warrior: usize,
        monster: usize,
        resolve: Duel<'_>,
    ) -> Result<AttackResult, RuleError> {
        resolve(
            &mut self.w[warrior],
            &mut self.c.roster.records[monster],
            self.random,
        )
    }
    /// A warrior in battle with stamina left, or a monster that is standing and not disabled.
    fn targetable(&self, body: i16) -> Result<bool, RuleError> {
        if (0..6).contains(&body) {
            return match self.w.get(body as usize) {
                Some(warrior) => {
                    Ok(warrior.record().in_battle() != 0 && warrior.record().stamina()? > 0)
                }
                None => Ok(false),
            };
        }
        Ok(alive(self.c, self.w, body))
    }
    /// The side the body fights for.
    fn faction(&self, body: i16) -> Result<i16, RuleError> {
        self.c.body_faction(self.w, body)
    }
    /// Where the body stands.
    fn position(&self, body: i16) -> Result<(i16, i16), RuleError> {
        Ok(self.c.body_position(body))
    }
    /// Whether the line between the cells is clear on the battlefield.
    fn visible(
        &self,
        mission: &centurions_content::mission::OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        lifecycle::can_see(self.c, mission, from, to, 0)
    }
    /// The body whose turn it is.
    fn active_body(&self) -> Result<Option<i16>, RuleError> {
        Ok(self.c.active_body())
    }
    /// The battle's difficulty.
    fn difficulty(&self) -> i16 {
        self.c.difficulty
    }
    /// The battle's weapon traits.
    fn weapon_traits(&mut self) -> &mut WeaponTraits {
        &mut self.c.weapon_traits
    }
    /// The weapon and ammunition records in hand.
    fn buffers(&mut self) -> &mut crate::ammunition::LoadedItems {
        self.buffers
    }
    /// Makes the packet the battle's spell, sets the attack count and gives the cast the identity
    /// of a firearm shot.
    fn publish(&mut self, packet: Vec<u8>, count: i16) {
        *self.attack_count = count;
        self.c.spell_info = packet;
        self.c.cast_identity = [3, 0, 0];
    }
    /// Reloads the item shown in hand for `body`.
    fn refresh_item(&mut self, items: &[Vec<u8>], body: i16) -> Result<(), RuleError> {
        runtime::refresh_body_item(self.c, self.w, items, body, self.buffers)
    }
    /// Kills the body; the battle goes on.
    fn kill(&mut self, body: i16, _events: &mut Vec<Event>) -> Result<bool, RuleError> {
        lifecycle::kill_body(self.c, self.w, body)?;
        Ok(false)
    }
}

/// A friendly rifle monster shoots in the live battle; true when it fired. See `fire_in`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fire(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    events: &mut Vec<Event>,
    actor: (i16, i16),
    buffers: &mut crate::ammunition::LoadedItems,
    display_writers: bool,
    attack_count: &mut i16,
) -> Result<bool, RuleError> {
    let mut world = Live {
        c: combat,
        w: warriors,
        random,
        buffers,
        attack_count,
    };
    Ok(fire_in(&mut world, mission, items, events, actor, display_writers)? == Shot::Fired)
}

/// A friendly rifle monster takes a shot: it holds fire when no opponent is both in range and in
/// sight, or when any opponent is out of range. Otherwise it picks a random target in range, spends
/// 2 action points and 3 movement, and resolves the hit against a warrior or a monster; killing a
/// warrior may end the battle.
///
/// Original: combat
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
    let source = world.monster(a)?.clone();
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
    world.monster_mut(a)?.record_mut().set_weapon(1);
    let p = packet(weapon, ammo, attacks)?;
    world.weapon_traits().prepare(weapon, ammo)?;
    world.buffers().prepare_pair(weapon, ammo)?;
    world.publish(p.clone(), SpellInfoRecord(&p).attack_count());
    world.random().bound(7);
    let shots = i16::from(ItemDefinition(weapon).selected_attack_count());
    let range = SpellInfoRecord(&p)
        .attack_count()
        .wrapping_mul(shots)
        .wrapping_add(SpellInfoRecord(&p).range_modifier())
        .wrapping_abs();
    let start = world.position(body)?;
    let mut distances = [127_i16; 110];
    let mut count = 0;
    let mut maximum = 0;
    let bound = world.monster_count()? as i16 + 10;
    for target in 0..bound {
        // Packets with the cannot byte at 4 aim at the shooter's own side; all others at the
        // other side.
        if !world.targetable(target)?
            || (world.faction(body)? == world.faction(target)?)
                != (SpellInfoRecord(&p).cannot() == 4)
        {
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
    let actor = world.monster_mut(a)?;
    let action_points = actor.record().action_points().wrapping_sub(2);
    let movement_points = actor.record().movement().wrapping_sub(3);
    actor.record_mut().set_action_points(action_points);
    actor.record_mut().set_movement(movement_points);
    actor.record_mut().set_reaction_ready(action_points >= 1);
    if display_writers {
        world.refresh_item(items, body)?;
    }
    events.push(Event::Sound {
        resource_id: SpellInfoRecord(&p).launch_sound()?,
    });
    let (minimum_duration, maximum_duration) = SpellInfoRecord(&p).duration_range();
    world.random().range(minimum_duration, maximum_duration);
    if target < 6 {
        let prepared = firearm::PreparedFirearm {
            sound: SpellInfoRecord(&p).impact_sound()?,
            packet: p,
            slot: 0,
            shots: 1,
            rounds: ItemDefinition(weapon).rounds_per_shot()?,
            weight: 0,
            armor_filter: world.weapon_traits().armor_filter(),
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
                    resource_id: result_audio(world.monster(a)?, 0, result.damage, prepared.sound)?,
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
    let target_index = target - 10;
    let mut hit = 33_i16;
    if world.active_body()?.is_some_and(|actor| actor > 9) {
        hit = hit
            .wrapping_add(source.record().hit_dice()?.wrapping_mul(5))
            .wrapping_add(source.record().missile_accuracy_bonus()?);
    }
    let difficulty = world.difficulty();
    let (victim, random) = world.monster_and_random(target_index)?;
    let victim = victim.record();
    if victim.turn_skip_duration() == 0 {
        hit = hit
            .wrapping_sub(victim.missile_dodge_bonus()?)
            .wrapping_sub(random.bound(victim.agility()? / 3))
            .wrapping_sub(victim.hit_dice()?.wrapping_mul(3))
            .wrapping_sub(i16::from(victim.guarding()).wrapping_mul(victim.hit_dice()?));
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
    let (victim, random) = world.monster_and_random(target_index)?;
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
        let hp = world
            .monster(target_index)?
            .record()
            .stamina()?
            .wrapping_sub(result.total);
        world
            .monster_mut(target_index)?
            .record_mut()
            .set_stamina(hp);
        if hp < 1 {
            if world.kill(target as i16, events)? {
                return Ok(Shot::Ended);
            }
        } else {
            events.push(Event::Sound {
                resource_id: SpellInfoRecord(&p).impact_sound()?,
            });
            events.push(Event::Sound {
                resource_id: result_audio(
                    world.monster(a)?,
                    0,
                    result.total,
                    SpellInfoRecord(&p).impact_sound()?,
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
