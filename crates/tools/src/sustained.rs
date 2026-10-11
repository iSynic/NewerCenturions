use centurions_content::{Content, InventoryRecord, mission::OpeningMission, word};
use centurions_core::{
    Command, Event, MedicalTarget, Phase, Presentation, Session, apply, load, save,
};
use std::{cmp::Reverse, collections::BinaryHeap, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) struct Play<'a> {
    pub(crate) content: &'a Content,
    pub(crate) artifacts: &'a Path,
    pub(crate) state: Session,
    pub(crate) commands: Vec<Command>,
    pub(crate) ordered_events: Vec<Vec<Event>>,
    pub(crate) battles: usize,
    pub(crate) supplies: i32,
    pub(crate) reloads: usize,
    pub(crate) completion: bool,
}
impl<'a> Play<'a> {
    fn gear_command(&self) -> Result<Option<Command>> {
        if !self.completion
            || self.state.phase != Phase::Exploration
            || self.state.combat.is_some()
            || self
                .state
                .exploration
                .as_ref()
                .is_none_or(|p| p.presentation.is_some())
        {
            return Ok(None);
        }
        for warrior in [2, 3] {
            if self
                .equipped_firearm(warrior)?
                .is_some_and(|item| Self::ordinary_empty_45(&item))
                && let Some(command) = self.empty_firearm_reload(warrior)?
            {
                return Ok(Some(command));
            }
        }
        let warrior = 0;
        let member = &self.state.warriors[warrior];
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("equipment mission unavailable")?;
        for id in [420, 38] {
            if member.stamina()? <= 0 {
                break;
            }
            let definition = &self.content.items[id as usize];
            if !centurions_core::equipment::can_use(member, definition, mission)? {
                continue;
            }
            let inventory = member.inventory()?;
            if let Some(slot) = inventory.iter().position(|item| item.id() == id) {
                let item = &inventory[slot];
                if id == 38
                    && item.0[9] != 0
                    && word(&item.0, 2)? == 0
                    && word(&item.0, 4)? == 0
                    && self.state.ammunition.get(3).is_some_and(|stock| *stock > 0)
                {
                    let command = Command::FitAmmunition {
                        warrior,
                        slot,
                        ammunition: 203,
                    };
                    if apply(command.clone(), &mut self.state.clone(), self.content).is_ok() {
                        return Ok(Some(command));
                    }
                }
                if item.0[9] != 0 || word(&member.bytes, 0x3a)? < i16::from(item.0[13] as i8) {
                    continue;
                }
                let command = Command::EquipItem { warrior, slot };
                if apply(command.clone(), &mut self.state.clone(), self.content).is_ok() {
                    return Ok(Some(command));
                }
            } else if let Some(slot) = self
                .state
                .crate_items
                .iter()
                .position(|item| item.id() == id)
            {
                let item = &self.state.crate_items[slot];
                if word(&member.bytes, 0x3a)? < i16::from(item.0[13] as i8) {
                    continue;
                }
                let command = Command::TakeCrate { slot, warrior };
                let mut candidate = self.state.clone();
                if apply(command.clone(), &mut candidate, self.content).is_ok()
                    && let Some(slot) = candidate.warriors[warrior]
                        .inventory()?
                        .iter()
                        .position(|item| item.id() == id)
                    && apply(
                        Command::EquipItem { warrior, slot },
                        &mut candidate,
                        self.content,
                    )
                    .is_ok()
                {
                    return Ok(Some(command));
                }
            }
        }
        self.armor_command(mission)
    }

    fn armor_command(&self, mission: &OpeningMission) -> Result<Option<Command>> {
        for id in [400, 403, 440, 460] {
            let definition = &self.content.items[id as usize];
            let class = u8::try_from(word(definition, 26)?)?;
            let equipment_slot = 0xc6 + 2 * usize::from(class);
            let mut candidates = Vec::new();
            for (warrior, member) in self.state.warriors.iter().enumerate() {
                if member.stamina()? <= 0
                    || word(&member.bytes, equipment_slot)? != 0
                    || !centurions_core::equipment::can_use(member, definition, mission)?
                {
                    continue;
                }
                candidates.push((
                    word(&member.bytes, 0x2c)?,
                    word(&member.bytes, 0x52)?,
                    warrior,
                ));
            }
            candidates.sort_unstable();
            for (_, _, warrior) in &candidates {
                let member = &self.state.warriors[*warrior];
                if let Some(slot) = member
                    .inventory()?
                    .iter()
                    .position(|item| item.id() == id && Self::ordinary_armor(item, class))
                    && centurions_core::equipment::wear(member, self.content, mission, slot, false)?
                        .changed
                {
                    return Ok(Some(Command::EquipItem {
                        warrior: *warrior,
                        slot,
                    }));
                }
            }
            if let Some(slot) = self
                .state
                .crate_items
                .iter()
                .position(|item| item.id() == id && Self::ordinary_armor(item, class))
            {
                let item = &self.state.crate_items[slot];
                for (_, _, warrior) in candidates {
                    let member = &self.state.warriors[warrior];
                    if word(&member.bytes, 0x3a)? >= i16::from(item.0[13] as i8)
                        && centurions_core::equipment::collect_item(
                            member,
                            item,
                            self.content,
                            mission,
                        )?
                        .collected
                    {
                        return Ok(Some(Command::TakeCrate { slot, warrior }));
                    }
                }
            }
        }
        Ok(None)
    }

    fn ordinary_armor(item: &InventoryRecord, class: u8) -> bool {
        item.0[2..8].iter().all(|value| *value == 0)
            && item.0[9] == 0
            && item.0[10] == class
            && item.0[13] == 1
            && item.0[14..22].iter().all(|value| *value == 0)
    }

    fn equipped_firearm(&self, warrior: usize) -> Result<Option<InventoryRecord>> {
        let member = &self.state.warriors[warrior];
        let id = word(&member.bytes, 0xe4)?;
        Ok(member
            .inventory()?
            .into_iter()
            .find(|item| item.id() == id && item.0[9] != 0))
    }

    fn ordinary_empty_45(item: &InventoryRecord) -> bool {
        const EMPTY_FIREARM: [u8; 22] = [
            0, 2, 0, 0, 0, 201, 0, 12, 0, 1, 15, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        item.0 == EMPTY_FIREARM
    }

    fn empty_firearm_reload(&self, warrior: usize) -> Result<Option<Command>> {
        let Some(item) = self.equipped_firearm(warrior)? else {
            return Ok(None);
        };
        let ammunition = word(&item.0, 4)?;
        let reserve = self
            .state
            .ammunition
            .get(ammunition.wrapping_sub(200) as usize)
            .copied()
            .unwrap_or(0);
        let needed = word(&item.0, 6)?.wrapping_sub(word(&item.0, 2)?);
        if word(&item.0, 2)? != 0
            || needed <= 0
            || reserve <= 0
            || (ammunition == 203 && reserve < i32::from(needed))
        {
            return Ok(None);
        }
        if item.id() == 165 && ammunition == 280 && (1..=3).contains(&reserve) {
            let Some(combat) = self.state.combat.as_ref() else {
                return Ok(None);
            };
            if combat.active_body() != Some(warrior as i16) {
                return Ok(None);
            }
            let start = combat.squad.positions[warrior];
            let mut adjacent = centurions_core::combat::runtime::neighbors(
                combat,
                &self.state.warriors,
                warrior as i16,
            )?;
            adjacent.sort_unstable();
            for target in adjacent.into_iter().filter(|body| *body >= 10) {
                let end = combat.enemies.positions[(target - 10) as usize];
                let command = Command::Move {
                    dx: i8::try_from(end.0 - start.0)?,
                    dy: i8::try_from(end.1 - start.1)?,
                };
                let mut candidate = self.state.clone();
                if apply(command.clone(), &mut candidate, self.content).is_ok()
                    && candidate.ammunition[80] == reserve - 1
                    && word(&candidate.warriors[warrior].inventory()?[0].0, 2)? == 1
                {
                    return Ok(Some(command));
                }
            }
            return Ok(None);
        }
        let command = if Self::ordinary_empty_45(&item) {
            let slot = self.state.warriors[warrior]
                .inventory()?
                .iter()
                .position(|record| record.0 == item.0)
                .ok_or("equipped ordinary firearm instance unavailable")?;
            Command::FitAmmunition {
                warrior,
                slot,
                ammunition: 201,
            }
        } else {
            Command::Reload { warrior }
        };
        Ok(
            apply(command.clone(), &mut self.state.clone(), self.content)
                .is_ok()
                .then_some(command),
        )
    }

    fn laboratory_combat(&self) -> bool {
        self.state.mission_stage == 1
            && self
                .state
                .exploration
                .as_ref()
                .is_some_and(|p| (p.map, p.x, p.y) == (1, 8, 3))
            && self
                .state
                .combat
                .as_ref()
                .is_some_and(|combat| matches!(combat.battle, 14 | 15))
    }

    fn can_spend_firearm_round(&self, warrior: usize, target: usize) -> Result<bool> {
        let Some(item) = self.equipped_firearm(warrior)? else {
            return Ok(false);
        };
        if !self.completion || item.id() != 38 || self.state.mission_stage >= 2 {
            return Ok(true);
        }
        let combat = self.state.combat.as_ref().unwrap();
        let available = i32::from(word(&item.0, 2)?) + self.state.ammunition[3];
        Ok(if self.laboratory_combat() {
            available > 0
        } else {
            word(&combat.enemies.records[target].bytes, 0x42)? > 0 && available > 3
        })
    }

    fn learned_spell(&self, body: i16, spell: i16) -> Result<bool> {
        let Some(warrior) = self.state.warriors.get(body as usize) else {
            return Ok(false);
        };
        let school = word(&warrior.bytes, 0x36)?;
        let row = spell % 1000 / 100 - 1;
        let column = spell % 100 - 1;
        Ok(school == spell / 1000
            && (0..7).contains(&row)
            && (0..7).contains(&column)
            && warrior.bytes[0x9f0 + row as usize * 7 + column as usize] != 0)
    }

    fn safe_spell_area(&self, center: (i16, i16), spell: i16) -> Result<bool> {
        let combat = self.state.combat.as_ref().unwrap();
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let packet = centurions_core::combat::spells::load_packet(
            &mission
                .combat
                .as_ref()
                .ok_or("tactical attack table unavailable")?
                .attacks,
            spell,
        )?;
        let shape = usize::from(packet[31]);
        if packet[1] != 3 || shape == 0 || packet[7] != 0 {
            return Ok(false);
        }
        let pattern = centurions_content::span(&mission.areas, (shape - 1) * 49, 49)?;
        let covers = |position: (i16, i16)| {
            let x = position.0 - center.0 + 3;
            let y = position.1 - center.1 + 3;
            (0..7).contains(&x) && (0..7).contains(&y) && pattern[y as usize * 7 + x as usize] != 0
        };
        for y in 0..7 {
            for x in 0..7 {
                if pattern[y * 7 + x] != 0 {
                    let ax = center.0 + combat.map.shift.0 + 2 + x as i16;
                    let ay = center.1 + combat.map.shift.1 + 2 + y as i16;
                    if !(0..90).contains(&ax) || !(0..90).contains(&ay) {
                        return Ok(false);
                    }
                }
            }
        }
        for (index, warrior) in self.state.warriors.iter().enumerate() {
            if warrior.stamina()? > 0 && covers(combat.squad.positions[index]) {
                return Ok(false);
            }
        }
        for (index, enemy) in combat.enemies.records.iter().enumerate() {
            if enemy.stamina()? > 0
                && (word(&enemy.bytes, 0x0e)? == 0 || enemy.bytes[0xb0] == 0xff)
                && covers(combat.enemies.positions[index])
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn hostile_spell_area(&self, center: (i16, i16), spell: i16) -> Result<usize> {
        let combat = self.state.combat.as_ref().unwrap();
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let packet = centurions_core::combat::spells::load_packet(
            &mission.combat.as_ref().unwrap().attacks,
            spell,
        )?;
        let pattern =
            centurions_content::span(&mission.areas, (usize::from(packet[31]) - 1) * 49, 49)?;
        let mut count = 0;
        for (index, enemy) in combat.enemies.records.iter().enumerate() {
            let position = combat.enemies.positions[index];
            let x = position.0 - center.0 + 3;
            let y = position.1 - center.1 + 3;
            if enemy.stamina()? > 0
                && word(&enemy.bytes, 0x0e)? != 0
                && enemy.bytes[0xb0] != 0xff
                && (0..7).contains(&x)
                && (0..7).contains(&y)
                && pattern[y as usize * 7 + x as usize] != 0
            {
                count += 1;
            }
        }
        Ok(count)
    }

    fn medical_command(&self, body: i16) -> Result<Option<Command>> {
        if !self.completion || self.state.mission_stage != 2 || !(0..6).contains(&body) {
            return Ok(None);
        }
        let arena = self.state.combat.as_ref().unwrap();
        let actor = &self.state.warriors[body as usize];
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("medical mission unavailable")?;
        let organic = |warrior: &centurions_content::Warrior| {
            let race = word(&warrior.bytes, 0x32).unwrap_or(0);
            (1..=16).contains(&race)
                && (0x2000_u16 & (0x8000_u16 >> (race - 1))) == 0
                && word(&warrior.bytes, 0x9a).unwrap_or(1) == 0
        };
        let mut revive = self
            .state
            .warriors
            .iter()
            .enumerate()
            .filter_map(|(index, warrior)| {
                let hp = warrior.stamina().ok()?;
                ((-9..=0).contains(&hp) && organic(warrior)).then_some((hp, index))
            })
            .collect::<Vec<_>>();
        revive.sort_unstable();
        let learned_heal = self.learned_spell(body, 1101)?
            && word(&actor.bytes, 0x58)? >= 6
            && centurions_core::combat::player_psi::can_cast(
                &self.state.warriors,
                body,
                self.state.psi_disabled,
            )?;
        let mut living = vec![];
        if !learned_heal && word(&actor.bytes, 0x40)? >= 2 && word(&actor.bytes, 0x3c)? >= 3 {
            for (index, warrior) in self.state.warriors.iter().enumerate() {
                let hp = warrior.stamina()?;
                let maximum = word(&warrior.bytes, 0x52)?;
                let missing = maximum.wrapping_sub(hp);
                let target = arena.squad.positions[index];
                if hp <= 0
                    || maximum <= 0
                    || missing < 8
                    || warrior.bytes[0x9df] == 0
                    || !organic(warrior)
                    || !centurions_core::combat::player_psi::in_range(
                        arena.squad.positions[body as usize],
                        target,
                        4,
                    )
                    || !centurions_core::combat::lifecycle::can_see(
                        arena,
                        mission,
                        arena.squad.positions[body as usize],
                        target,
                        0,
                    )?
                {
                    continue;
                }
                let cell = OpeningMission::index(
                    i32::from(target.0) + i32::from(arena.map.shift.0) + 5,
                    i32::from(target.1) + i32::from(arena.map.shift.1) + 5,
                )?;
                if arena.map.tiles[cell].wrapping_abs() == index as i16 {
                    living.push((index, hp, maximum, missing, target));
                }
            }
            living.sort_by(|a, b| {
                (i32::from(a.1) * i32::from(b.2))
                    .cmp(&(i32::from(b.1) * i32::from(a.2)))
                    .then_with(|| b.3.cmp(&a.3))
                    .then_with(|| a.0.cmp(&b.0))
            });
        }
        let selection = revive
            .first()
            .map(|(_, index)| MedicalTarget::Portrait { warrior: *index })
            .or_else(|| {
                living
                    .first()
                    .map(|(_, _, _, _, (x, y))| MedicalTarget::LivingPoint { x: *x, y: *y })
            });
        if self.state.medical_continuation.is_some() {
            return selection
                .map(|selection| Some(Command::SelectMedicalTarget { selection }))
                .ok_or_else(|| "prepared medical selection lost its unchanged target".into());
        }
        if selection.is_some()
            && let Some(slot) = actor
                .inventory()?
                .iter()
                .position(|item| item.id() == 700 && word(&item.0, 2).is_ok_and(|qty| qty > 0))
            && centurions_core::combat::medical::begin(
                &self.state,
                self.content,
                centurions_core::combat::medical::BeginRequest {
                    warrior: body as usize,
                    slot,
                },
            )
            .is_ok()
        {
            return Ok(Some(Command::BeginMedicalItem {
                warrior: body as usize,
                slot,
            }));
        }
        if !revive.is_empty() && learned_heal {
            let power = (actor.psi_points()? / 6).min(4);
            if centurions_core::combat::medical::begin_learned_heal(
                &self.state,
                self.content,
                power,
            )
            .is_ok()
            {
                return Ok(Some(Command::BeginLearnedHeal { power }));
            }
        }
        if self.state.bandage_ready.is_some_and(|ready| ready != 0) {
            let mut bleeding = self
                .state
                .warriors
                .iter()
                .enumerate()
                .filter_map(|(index, warrior)| {
                    let hp = warrior.stamina().ok()?;
                    ((-9..=0).contains(&hp) && warrior.bytes[0x9de] != 0).then_some((
                        word(&warrior.bytes, 0x32).ok()? != 3,
                        hp,
                        index,
                    ))
                })
                .collect::<Vec<_>>();
            bleeding.sort_unstable();
            if let Some((_, _, index)) = bleeding.first() {
                return Ok(Some(Command::StopBleeding {
                    warrior: Some(*index),
                }));
            }
        }
        Ok(None)
    }

    fn support_command(&self, body: i16) -> Result<Option<Command>> {
        if !self.completion
            || !(0..6).contains(&body)
            || word(&self.state.warriors[body as usize].bytes, 0x40)? <= 0
            || !centurions_core::combat::player_psi::can_cast(
                &self.state.warriors,
                body,
                self.state.psi_disabled,
            )?
        {
            return Ok(None);
        }
        let combat = self.state.combat.as_ref().unwrap();
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let tables = &mission.combat.as_ref().unwrap().attacks;
        let start = combat.squad.positions[body as usize];
        let points = word(&self.state.warriors[body as usize].bytes, 0x58)?;
        let earlier_healing = self.state.mission_stage == 2 && matches!(combat.battle, 50 | 51);
        let admitted = |spell, power, target| {
            let command = Command::CastPsi {
                spell,
                power,
                target: Some(target),
            };
            centurions_core::combat::player_psi::validate(
                combat,
                &self.state.warriors,
                mission,
                self.state.psi_disabled,
                centurions_core::combat::player_psi::CastRequest {
                    spell,
                    power,
                    target: Some(target),
                },
            )
            .is_ok()
                && apply(command, &mut self.state.clone(), self.content).is_ok()
        };
        let mut urgent = vec![];
        if self.learned_spell(body, 1101)? {
            let packet = centurions_core::combat::spells::load_packet(tables, 1101)?;
            let cost = centurions_core::combat::player_psi::cost(&packet, 1)?;
            if points >= cost {
                for (index, warrior) in self.state.warriors.iter().enumerate() {
                    let hp = warrior.stamina()?;
                    let maximum = word(&warrior.bytes, 0x52)?;
                    let missing = maximum.wrapping_sub(hp);
                    let race = word(&warrior.bytes, 0x32)?;
                    if hp <= 0
                        || maximum <= 0
                        || (!earlier_healing && i32::from(hp) * 2 > i32::from(maximum))
                        || missing < 8
                        || warrior.bytes[0x9df] == 0
                        || word(&warrior.bytes, 0x9a)? != 0
                        || !(1..=16).contains(&race)
                        || (word(&packet, 50)? as u16 & (0x8000_u16 >> (race - 1))) != 0
                    {
                        continue;
                    }
                    let target = combat.squad.positions[index];
                    let cell = OpeningMission::index(
                        i32::from(target.0) + i32::from(combat.map.shift.0) + 5,
                        i32::from(target.1) + i32::from(combat.map.shift.1) + 5,
                    )?;
                    if combat.map.tiles[cell].wrapping_abs() != index as i16 {
                        continue;
                    }
                    let power = if missing >= 16 && points >= cost.wrapping_mul(2) {
                        2
                    } else {
                        1
                    };
                    urgent.push((index, hp, maximum, missing, power, target));
                }
                urgent.sort_by(|a, b| {
                    (i32::from(a.1) * i32::from(b.2))
                        .cmp(&(i32::from(b.1) * i32::from(a.2)))
                        .then_with(|| b.3.cmp(&a.3))
                        .then_with(|| a.0.cmp(&b.0))
                });
                for &(_, _, _, _, power, target) in &urgent {
                    if admitted(1101, power, target) {
                        return Ok(Some(Command::CastPsi {
                            spell: 1101,
                            power,
                            target: Some(target),
                        }));
                    }
                }
            }
        }
        let sample_melee = self.state.mission_stage == 1
            && combat.battle == 22
            && self
                .state
                .exploration
                .as_ref()
                .is_some_and(|p| (p.map, p.x, p.y) == (1, 66, 85));
        if !self.laboratory_combat() && !sample_melee && self.learned_spell(body, 1305)? {
            let packet = centurions_core::combat::spells::load_packet(tables, 1305)?;
            let pattern =
                centurions_content::span(&mission.areas, (usize::from(packet[31]) - 1) * 49, 49)?;
            let mut unwarded = 0;
            for (index, warrior) in self.state.warriors.iter().enumerate() {
                let position = combat.squad.positions[index];
                let x = position.0 - start.0 + 3;
                let y = position.1 - start.1 + 3;
                if warrior.stamina()? > 0
                    && warrior.bytes[0x9df] != 0
                    && word(&warrior.bytes, 0x76)? == 0
                    && (0..7).contains(&x)
                    && (0..7).contains(&y)
                    && pattern[y as usize * 7 + x as usize] != 0
                {
                    unwarded += 1;
                }
            }
            if unwarded >= 2 && admitted(1305, 1, start) {
                return Ok(Some(Command::CastPsi {
                    spell: 1305,
                    power: 1,
                    target: Some(start),
                }));
            }
        }
        if let Some(&(_, _, _, _, power, target)) = urgent.first() {
            let packet = centurions_core::combat::spells::load_packet(tables, 1101)?;
            let range = centurions_core::combat::player_psi::range(&packet, power)?;
            if !centurions_core::combat::player_psi::in_range(start, target, range) {
                return self.approach_support(body, target, range);
            }
        }
        Ok(None)
    }

    fn approach_support(
        &self,
        body: i16,
        target: (i16, i16),
        range: i16,
    ) -> Result<Option<Command>> {
        let member = &self.state.warriors[body as usize];
        let budget = i32::from(word(&member.bytes, 0x3c)?) - 3;
        if budget <= 0 || word(&member.bytes, 0x40)? < 2 {
            return Ok(None);
        }
        let combat = self.state.combat.as_ref().unwrap();
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let start = combat.squad.positions[body as usize];
        let origin = (combat.map.shift.0 + 5, combat.map.shift.1 + 5);
        let cell = (start.0 + origin.0, start.1 + origin.1);
        let start_index = OpeningMission::index(i32::from(cell.0), i32::from(cell.1))?;
        let mut costs = vec![i32::MAX; 8100];
        let mut first_steps = vec![None; 8100];
        let mut queue = BinaryHeap::new();
        costs[start_index] = 0;
        queue.push(Reverse((0_i32, 0_usize, cell.0, cell.1)));
        let mut sequence = 0;
        while let Some(Reverse((cost, _, x, y))) = queue.pop() {
            let index = OpeningMission::index(i32::from(x), i32::from(y))?;
            if cost != costs[index] || cost > budget {
                continue;
            }
            let position = (x - origin.0, y - origin.1);
            if centurions_core::combat::player_psi::in_range(position, target, range) {
                if index == start_index {
                    return Ok(None);
                }
                let (dx, dy) = first_steps[index].ok_or("support path has no first step")?;
                let command = Command::Move { dx, dy };
                let mut candidate = self.state.clone();
                if apply(command.clone(), &mut candidate, self.content).is_ok()
                    && candidate.combat.as_ref().is_some_and(|next| {
                        next.active_body() == Some(body)
                            && next.squad.positions[body as usize] != start
                    })
                    && word(&candidate.warriors[body as usize].bytes, 0x3c)? >= 3
                    && word(&candidate.warriors[body as usize].bytes, 0x40)? >= 2
                    && centurions_core::combat::player_psi::can_cast(
                        &candidate.warriors,
                        body,
                        candidate.psi_disabled,
                    )?
                {
                    return Ok(Some(command));
                }
                return Ok(None);
            }
            for (dx, dy) in [
                (1_i8, 0_i8),
                (0, 1),
                (-1, 0),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ] {
                let next = (x + i16::from(dx), y + i16::from(dy));
                let Ok(next_index) = OpeningMission::index(i32::from(next.0), i32::from(next.1))
                else {
                    continue;
                };
                let tile = combat.map.tiles[next_index];
                if tile < 1000 || mission.tile_word(tile - 1000, 4)? != 0 {
                    continue;
                }
                let next_cost = cost
                    + i32::from(
                        centurions_core::rules::movement_cost(
                            mission.tile_word(tile - 1000, 2)?,
                            dx,
                            dy,
                        )
                        .max(1),
                    );
                if next_cost <= budget && next_cost < costs[next_index] {
                    costs[next_index] = next_cost;
                    first_steps[next_index] = if index == start_index {
                        Some((dx, dy))
                    } else {
                        first_steps[index]
                    };
                    sequence += 1;
                    queue.push(Reverse((next_cost, sequence, next.0, next.1)));
                }
            }
        }
        Ok(None)
    }

    fn affordable_area_spell(&self, body: i16) -> Result<Option<i16>> {
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let attacks = &mission
            .combat
            .as_ref()
            .ok_or("tactical attack table unavailable")?
            .attacks;
        for spell in [2204, 2101] {
            if self.learned_spell(body, spell)? {
                let packet = centurions_core::combat::spells::load_packet(attacks, spell)?;
                if self.state.warriors[body as usize].psi_points()?
                    >= centurions_core::combat::player_psi::cost(&packet, 1)?
                {
                    return Ok(Some(spell));
                }
            }
        }
        Ok(None)
    }

    fn psi_command(&self, body: i16) -> Result<Option<Command>> {
        if !self.completion
            || word(&self.state.warriors[body as usize].bytes, 0x40)? <= 0
            || !centurions_core::combat::player_psi::can_cast(
                &self.state.warriors,
                body,
                self.state.psi_disabled,
            )?
        {
            return Ok(None);
        }
        let combat = self.state.combat.as_ref().unwrap();
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let admitted = |spell, target| {
            centurions_core::combat::player_psi::validate(
                combat,
                &self.state.warriors,
                mission,
                self.state.psi_disabled,
                centurions_core::combat::player_psi::CastRequest {
                    spell,
                    power: 1,
                    target,
                },
            )
            .is_ok()
        };
        let start = combat.squad.positions[body as usize];
        let area_spell = self.affordable_area_spell(body)?;
        let mut casters = vec![];
        let mut hostiles = 0;
        for (index, enemy) in combat.enemies.records.iter().enumerate() {
            if enemy.stamina()? > 0 && word(&enemy.bytes, 0x0e)? != 0 && enemy.bytes[0xb0] != 0xff {
                hostiles += 1;
                if word(&enemy.bytes, 0x42)? > 0 {
                    casters.push(index);
                }
            }
        }
        casters.sort_by_key(|index| {
            let end = combat.enemies.positions[*index];
            (
                word(&combat.enemies.records[*index].bytes, 0x110).unwrap_or(i16::MAX),
                (start.0 - end.0).abs() + (start.1 - end.1).abs(),
                *index,
            )
        });
        let laboratory = self.laboratory_combat();
        if (self.state.mission_stage >= 2 || laboratory)
            && let Some(spell) = area_spell
        {
            let mut centers = std::collections::BTreeSet::new();
            for (index, enemy) in combat.enemies.records.iter().enumerate() {
                if enemy.stamina()? <= 0
                    || word(&enemy.bytes, 0x0e)? == 0
                    || enemy.bytes[0xb0] == 0xff
                {
                    continue;
                }
                let position = combat.enemies.positions[index];
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        let center = (position.0 + dx, position.1 + dy);
                        if self.safe_spell_area(center, spell)? {
                            let count = self.hostile_spell_area(center, spell)?;
                            if count >= if laboratory { 1 } else { 2 } {
                                centers.insert((
                                    Reverse(count),
                                    (start.0 - center.0).abs() + (start.1 - center.1).abs(),
                                    center,
                                    index,
                                ));
                            }
                        }
                    }
                }
            }
            for &(_, _, target, _) in &centers {
                let command = Command::CastPsi {
                    spell,
                    power: 1,
                    target: Some(target),
                };
                if admitted(spell, Some(target))
                    && apply(command.clone(), &mut self.state.clone(), self.content).is_ok()
                {
                    return Ok(Some(command));
                }
            }
            if let Some(&(_, _, center, index)) = centers.first() {
                return self.approach_target(body, index, Some((spell, center)));
            }
        }
        let mut safe_caster = false;
        if let Some(spell) = area_spell {
            for index in casters {
                let target = combat.enemies.positions[index];
                if !self.safe_spell_area(target, spell)? {
                    continue;
                }
                safe_caster = true;
                let command = Command::CastPsi {
                    spell,
                    power: 1,
                    target: Some(target),
                };
                if admitted(spell, Some(target))
                    && apply(command.clone(), &mut self.state.clone(), self.content).is_ok()
                {
                    return Ok(Some(command));
                }
            }
        }
        if !safe_caster
            && hostiles >= 2
            && word(&self.state.warriors[body as usize].bytes, 0x58)? >= 25
            && self.learned_spell(body, 2302)?
        {
            let command = Command::CastPsi {
                spell: 2302,
                power: 1,
                target: None,
            };
            if admitted(2302, None)
                && apply(command.clone(), &mut self.state.clone(), self.content).is_ok()
            {
                return Ok(Some(command));
            }
        }
        Ok(None)
    }

    fn approach_target(
        &self,
        body: i16,
        target: usize,
        psi_target: Option<(i16, (i16, i16))>,
    ) -> Result<Option<Command>> {
        let combat = self.state.combat.as_ref().unwrap();
        let start = combat.squad.positions[body as usize];
        let end = psi_target.map_or(combat.enemies.positions[target], |(_, center)| center);
        if !self.completion {
            let distance = |point: (i16, i16)| (point.0 - end.0).abs() + (point.1 - end.1).abs();
            let mut directions = [
                (1_i8, 0_i8),
                (0, 1),
                (-1, 0),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ];
            directions.sort_by_key(|(dx, dy)| {
                distance((start.0 + i16::from(*dx), start.1 + i16::from(*dy)))
            });
            for (dx, dy) in directions {
                if distance((start.0 + i16::from(dx), start.1 + i16::from(dy))) >= distance(start) {
                    continue;
                }
                let command = Command::Move { dx, dy };
                let mut candidate = self.state.clone();
                if apply(command.clone(), &mut candidate, self.content).is_ok()
                    && candidate
                        .combat
                        .as_ref()
                        .is_some_and(|c| c.squad.positions[body as usize] != start)
                {
                    return Ok(Some(command));
                }
            }
        }
        if !self.completion || word(&self.state.warriors[body as usize].bytes, 0x3c)? <= 0 {
            return Ok(None);
        }
        let mission = self
            .content
            .map(self.state.exploration.as_ref().unwrap().map)
            .ok_or("tactical terrain unavailable")?;
        let origin = (combat.map.shift.0 + 5, combat.map.shift.1 + 5);
        let start_cell = (start.0 + origin.0, start.1 + origin.1);
        let start_index = OpeningMission::index(i32::from(start_cell.0), i32::from(start_cell.1))?;
        let mut costs = vec![i32::MAX; 8100];
        let mut first_steps = vec![None; 8100];
        let mut queue = BinaryHeap::new();
        costs[start_index] = 0;
        queue.push(Reverse((0_i32, 0_usize, start_cell.0, start_cell.1)));
        let mut sequence = 0;
        let selected_spell = if let Some((spell, _)) = psi_target {
            Some(spell)
        } else if word(&combat.enemies.records[target].bytes, 0x42)? > 0 {
            self.affordable_area_spell(body)?
        } else {
            None
        };
        let psi_spell = if let Some(spell) = selected_spell
            && self.learned_spell(body, spell)?
            && centurions_core::combat::player_psi::can_cast(
                &self.state.warriors,
                body,
                self.state.psi_disabled,
            )?
            && self.safe_spell_area(end, spell)?
        {
            let packet = centurions_core::combat::spells::load_packet(
                &mission
                    .combat
                    .as_ref()
                    .ok_or("tactical attack table unavailable")?
                    .attacks,
                spell,
            )?;
            let cost = centurions_core::combat::player_psi::cost(&packet, 1)?;
            (self.state.warriors[body as usize].psi_points()? >= cost)
                .then_some((spell, packet, cost))
        } else {
            None
        };
        let budget = if psi_spell.is_some() {
            if word(&self.state.warriors[body as usize].bytes, 0x40)? < 2 {
                return Ok(None);
            }
            let budget = i32::from(word(&self.state.warriors[body as usize].bytes, 0x3c)?) - 3;
            if budget <= 0 {
                return Ok(None);
            }
            budget
        } else {
            i32::MAX
        };
        let range = if let Some((_, packet, _)) = &psi_spell {
            i32::from(centurions_core::combat::player_psi::range(packet, 1)?)
        } else if let Some(item) = self.equipped_firearm(body as usize)?
            && word(&item.0, 2)? > 0
            && self.can_spend_firearm_round(body as usize, target)?
        {
            let prepared = centurions_core::combat::firearm::prepare(
                &self.state.warriors[body as usize],
                &self.content.items[item.id() as usize],
                &self.content.items[word(&item.0, 4)? as usize],
                &mission
                    .combat
                    .as_ref()
                    .ok_or("tactical attack table unavailable")?
                    .attacks,
            )?;
            i32::from(centurions_core::combat::player_psi::range(
                &prepared.packet,
                prepared.shots,
            )?)
        } else {
            1
        };
        while let Some(Reverse((cost, _, x, y))) = queue.pop() {
            let index = OpeningMission::index(i32::from(x), i32::from(y))?;
            if cost != costs[index] || cost > budget {
                continue;
            }
            let position = (x - origin.0, y - origin.1);
            let dx = i32::from(position.0 - end.0);
            let dy = i32::from(position.1 - end.1);
            if dx * dx + dy * dy < (range + 1) * (range + 1)
                && centurions_core::combat::lifecycle::can_see(combat, mission, position, end, 0)?
            {
                if index == start_index {
                    return Ok(None);
                }
                let (dx, dy) = first_steps[index].ok_or("tactical path has no first step")?;
                let command = Command::Move { dx, dy };
                let mut candidate = self.state.clone();
                if apply(command.clone(), &mut candidate, self.content).is_ok()
                    && candidate.combat.as_ref().is_some_and(|c| {
                        c.squad.positions[body as usize] != start
                            && (psi_spell.is_none() || c.active_body() == Some(body))
                    })
                {
                    if let Some((spell, _, cost)) = &psi_spell
                        && (word(&candidate.warriors[body as usize].bytes, 0x3c)? < 3
                            || word(&candidate.warriors[body as usize].bytes, 0x40)? < 2
                            || candidate.warriors[body as usize].psi_points()? < *cost
                            || !self.learned_spell(body, *spell)?
                            || !centurions_core::combat::player_psi::can_cast(
                                &candidate.warriors,
                                body,
                                candidate.psi_disabled,
                            )?)
                    {
                        return Ok(None);
                    }
                    return Ok(Some(command));
                }
                return Ok(None);
            }
            for (dx, dy) in [
                (1_i8, 0_i8),
                (0, 1),
                (-1, 0),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ] {
                let next = (x + i16::from(dx), y + i16::from(dy));
                let Ok(next_index) = OpeningMission::index(i32::from(next.0), i32::from(next.1))
                else {
                    continue;
                };
                let tile = combat.map.tiles[next_index];
                if tile < 1000 || mission.tile_word(tile - 1000, 4)? != 0 {
                    continue;
                }
                let next_cost = cost
                    + i32::from(
                        centurions_core::rules::movement_cost(
                            mission.tile_word(tile - 1000, 2)?,
                            dx,
                            dy,
                        )
                        .max(1),
                    );
                if next_cost <= budget && next_cost < costs[next_index] {
                    costs[next_index] = next_cost;
                    first_steps[next_index] = if index == start_index {
                        Some((dx, dy))
                    } else {
                        first_steps[index]
                    };
                    sequence += 1;
                    queue.push(Reverse((next_cost, sequence, next.0, next.1)));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn command(&mut self, command: Command) -> Result<()> {
        let events = apply(command.clone(), &mut self.state, self.content).map_err(|e| {
            let _ = save(
                &self.artifacts.join("route-failure.json"),
                &self.state,
                self.content,
            );
            format!(
                "command {} {command:?} at {:?}: {e}",
                self.commands.len(),
                self.state
                    .exploration
                    .as_ref()
                    .map(|p| (p.x, p.y, p.day, p.hour, p.minute))
            )
        })?;
        for event in &events {
            match event {
                Event::BattleEnded { victory: true } => self.battles += 1,
                Event::AmmunitionAdded {
                    ammunition: 200,
                    quantity,
                    ..
                } => self.supplies += i32::from(*quantity),
                _ => {}
            }
        }
        self.commands.push(command);
        self.ordered_events.push(events);
        Ok(())
    }

    pub(crate) fn settle(&mut self) -> Result<()> {
        for _ in 0..2000 {
            if self.state.exploration.as_ref().is_some_and(|p| {
                matches!(
                    p.presentation,
                    Some(Presentation::Messages { .. } | Presentation::Survivors)
                )
            }) {
                self.command(Command::Continue)?;
                continue;
            }
            if self
                .state
                .combat_story
                .as_ref()
                .is_some_and(|story| !story.waiting_child || self.state.combat.is_none())
                && !self.state.exploration.as_ref().is_some_and(|position| {
                    matches!(position.presentation, Some(Presentation::Loot))
                })
            {
                self.command(Command::Continue)?;
                continue;
            }
            if self.state.phase == Phase::Combat {
                let combat = self.state.combat.as_ref().unwrap();
                let body = combat.active_body().ok_or("battle has no active body")?;
                let command = if body >= 10 {
                    Command::RunEnemyTurn
                } else if let Some(command) = self.medical_command(body)? {
                    command
                } else if let Some(command) = self.support_command(body)? {
                    command
                } else if let Some(command) = self.psi_command(body)? {
                    command
                } else if self.completion
                    && let Some(command) = self.empty_firearm_reload(body as usize)?
                {
                    self.reloads += 1;
                    command
                } else {
                    let mut selected = None;
                    let mut targets: Vec<_> =
                        (10..10 + combat.enemies.records.len() as i16).collect();
                    let start = combat.squad.positions[body as usize];
                    let priority = |index: usize| {
                        let record = &combat.enemies.records[index].bytes;
                        let end = combat.enemies.positions[index];
                        let weapon = if self.completion && self.state.mission_stage >= 2 {
                            self.content
                                .items
                                .get(word(record, 0x26).unwrap_or(0).unsigned_abs() as usize)
                        } else {
                            None
                        };
                        (
                            word(record, 0x42).unwrap_or(0) <= 0,
                            Reverse(weapon.and_then(|item| word(item, 60).ok()).unwrap_or(0)),
                            Reverse(weapon.and_then(|item| word(item, 62).ok()).unwrap_or(0)),
                            word(record, 0x60).unwrap_or(0) == 0,
                            word(record, 0x110).unwrap_or(i16::MAX),
                            (start.0 - end.0).abs() + (start.1 - end.1).abs(),
                        )
                    };
                    targets.sort_by_key(|target| priority((*target - 10) as usize));
                    for target in targets {
                        if !self.can_spend_firearm_round(body as usize, (target - 10) as usize)? {
                            continue;
                        }
                        let mut candidate = self.state.clone();
                        let result = apply(
                            Command::ShootEnemy { body: target },
                            &mut candidate,
                            self.content,
                        );
                        if result.is_ok() {
                            selected = Some(Command::ShootEnemy { body: target });
                            break;
                        }
                    }
                    let ordinary_target = selected.as_ref().is_some_and(|command| {
                        if let Command::ShootEnemy { body: target } = command {
                            let record = &combat.enemies.records[(*target - 10) as usize].bytes;
                            word(record, 0x60).unwrap_or(0) == 0
                                && word(record, 0x42).unwrap_or(0) <= 0
                        } else {
                            false
                        }
                    });
                    if (selected.is_none() || ordinary_target)
                        && let Some((target, _)) = combat
                            .enemies
                            .records
                            .iter()
                            .enumerate()
                            .filter(|(_, record)| {
                                let firearm = self.completion
                                    && self.state.mission_stage == 2
                                    && matches!(combat.battle, 50 | 51)
                                    && self
                                        .content
                                        .items
                                        .get(word(&record.bytes, 0x26).unwrap_or(0).unsigned_abs()
                                            as usize)
                                        .is_some_and(|weapon| {
                                            word(weapon, 26).unwrap_or(0) == 15
                                                && word(weapon, 60).unwrap_or(0) > 0
                                        });
                                record.stamina().unwrap_or(0) > 0
                                    && word(&record.bytes, 0x0e).unwrap_or(0) != 0
                                    && (word(&record.bytes, 0x60).unwrap_or(0) != 0
                                        || word(&record.bytes, 0x42).unwrap_or(0) > 0
                                        || firearm)
                            })
                            .min_by_key(|(index, _)| priority(*index))
                        && let Some(command) = self.approach_target(body, target, None)?
                    {
                        let mut candidate = self.state.clone();
                        if apply(command.clone(), &mut candidate, self.content).is_ok()
                            && candidate
                                .combat
                                .as_ref()
                                .is_some_and(|c| c.squad.positions[body as usize] != start)
                        {
                            selected = Some(command);
                        }
                    }
                    let mut adjacent = centurions_core::combat::runtime::neighbors(
                        combat,
                        &self.state.warriors,
                        body,
                    )?;
                    adjacent.sort_by_key(|target| priority((*target - 10) as usize));
                    if let Some(target) = adjacent.first()
                        && word(&combat.enemies.records[(*target - 10) as usize].bytes, 0x42)? > 0
                        && !matches!(selected, Some(Command::ShootEnemy { body }) if body == *target)
                    {
                        let command = Command::AttackEnemy { body: *target };
                        if apply(command.clone(), &mut self.state.clone(), self.content).is_ok() {
                            selected = Some(command);
                        }
                    }
                    if selected.is_none()
                        && let Some(command) = self.empty_firearm_reload(body as usize)?
                    {
                        self.reloads += 1;
                        command
                    } else if let Some(command) = selected {
                        command
                    } else {
                        let mut adjacent = centurions_core::combat::runtime::neighbors(
                            combat,
                            &self.state.warriors,
                            body,
                        )?;
                        adjacent.sort_by_key(|target| priority((*target - 10) as usize));
                        let fallback =
                            if word(&self.state.warriors[body as usize].bytes, 0x40)? >= 2 {
                                Command::Guard
                            } else {
                                Command::EndTurn
                            };
                        adjacent
                            .first()
                            .map_or(fallback, |target| Command::AttackEnemy { body: *target })
                    }
                };
                self.command(command)?;
            } else if let Some(presentation) = self
                .state
                .exploration
                .as_ref()
                .and_then(|p| p.presentation.as_ref())
            {
                if matches!(
                    presentation,
                    Presentation::Question { .. } | Presentation::StageChange { .. }
                ) {
                    save(
                        &self.artifacts.join("camp-question.json"),
                        &self.state,
                        self.content,
                    )?;
                    let answer = !self.completion
                        || !matches!(
                            presentation,
                            Presentation::Question {
                                extra: 82 | 478,
                                ..
                            }
                        );
                    self.command(Command::Choose { answer })?;
                } else if matches!(presentation, Presentation::WarriorChoice { .. }) {
                    let index = self
                        .state
                        .warriors
                        .iter()
                        .position(|warrior| warrior.stamina().is_ok_and(|hp| hp > 0))
                        .ok_or("the story requires a living warrior")?;
                    self.command(Command::ChooseWarrior { index })?;
                } else if let Presentation::Typed { id, .. } = presentation {
                    self.command(Command::SubmitText {
                        text: if *id == 8 { "rescue 911" } else { "wildriver7" }.into(),
                    })?;
                } else if matches!(presentation, Presentation::Menu { .. }) {
                    save(
                        &self.artifacts.join("camp-menu.json"),
                        &self.state,
                        self.content,
                    )?;
                    if self.completion {
                        self.command(Command::SelectMenuEntry { index: 2 })?;
                    } else {
                        self.command(Command::CancelMenu)?;
                    }
                } else if matches!(presentation, Presentation::Loot) {
                    if let Some(slot) = self
                        .state
                        .booty_items
                        .iter()
                        .position(|item| item.id() != 0)
                    {
                        if self.completion
                            && self.state.mission_stage == 2
                            && self.state.booty_items[slot].id() == 700
                            && word(&self.state.booty_items[slot].0, 2)? > 0
                        {
                            let before = self
                                .state
                                .booty_items
                                .iter()
                                .filter(|item| item.id() == 700)
                                .count();
                            self.command(Command::TakeLoot { slot, warrior: 4 })?;
                            if self
                                .state
                                .booty_items
                                .iter()
                                .filter(|item| item.id() == 700)
                                .count()
                                != before - 1
                            {
                                return Err(
                                    "earned medical loot collection did not clear its source"
                                        .into(),
                                );
                            }
                        } else if self.completion
                            && matches!(self.state.booty_items[slot].id(), 804..=809 | 832..=841 | 848)
                            && centurions_core::story_inventory::query_party_item(
                                &self.state.warriors,
                                self.state.booty_items[slot].id(),
                            )?
                            .is_none()
                        {
                            self.command(Command::TakeLoot { slot, warrior: 0 })?;
                        } else {
                            self.command(Command::StoreLoot { slot })?;
                        }
                    } else {
                        self.command(if centurions_core::needs_spell_confirmation(&self.state) {
                            Command::KeepCurrentSpells
                        } else {
                            Command::Continue
                        })?;
                    }
                } else {
                    self.command(Command::Continue)?;
                }
            } else if let Some(command) = self.gear_command()? {
                self.command(command)?;
            } else {
                return Ok(());
            }
        }
        Err("settling the route exceeded its command budget".into())
    }

    pub(crate) fn prepare_owned_firearms(&mut self) -> Result<()> {
        const EMPTY_FIREARM: [u8; 22] = [
            0, 2, 0, 0, 0, 0, 0, 12, 0, 0, 15, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        if self.state.phase != Phase::Exploration
            || self.state.mission_stage != 1
            || self.state.combat.is_some()
            || self.state.exploration.as_ref().is_none_or(|position| {
                position.map != 1
                    || (position.x, position.y) != (6, 1)
                    || position.presentation.is_some()
            })
        {
            return Err("ordinary firearm preparation requires the first-stage checkpoint".into());
        }
        for warrior in [2, 3] {
            let Some(slot) = self
                .state
                .crate_items
                .iter()
                .position(|item| item.0 == EMPTY_FIREARM)
            else {
                break;
            };
            if self.state.ammunition.get(1).is_none_or(|stock| *stock <= 0) {
                return Err("ordinary firearm preparation requires ammunition 201".into());
            }
            self.command(Command::TakeCrate { slot, warrior })?;
            let slot = self.state.warriors[warrior]
                .inventory()?
                .iter()
                .position(|item| item.0 == EMPTY_FIREARM)
                .ok_or("collected ordinary firearm instance unavailable")?;
            self.command(Command::EquipItem { warrior, slot })?;
            let slot = self.state.warriors[warrior]
                .inventory()?
                .iter()
                .position(|item| {
                    let mut record = item.0;
                    let equipped = record[9] != 0;
                    record[9] = 0;
                    equipped && record == EMPTY_FIREARM
                })
                .ok_or("equipped ordinary firearm instance unavailable")?;
            self.command(Command::FitAmmunition {
                warrior,
                slot,
                ammunition: 201,
            })?;
        }
        Ok(())
    }

    pub(crate) fn enter_laboratory(&mut self) -> Result<()> {
        self.walk_to((9, 4))?;
        self.settle()?;
        let preparation_day = self.state.exploration.as_ref().unwrap().day;
        self.laboratory_preparation_turn(preparation_day)?;
        if self.laboratory_needs_psi()? {
            self.walk_to((9, 3))?;
            self.settle()?;
            let move_budget = self.laboratory_recovery_move_budget()?;
            let mut moves = 0;
            while self.laboratory_needs_psi()? {
                self.laboratory_preparation_turn(preparation_day)?;
                let position = self.state.exploration.as_ref().unwrap();
                if moves == move_budget || (position.hour == 23 && position.minute >= 50) {
                    return Err("laboratory psi recovery exceeded its daylight move budget".into());
                }
                let dx = if (position.x, position.y) == (9, 3) {
                    1
                } else if (position.x, position.y) == (10, 3) {
                    -1
                } else {
                    return Err("laboratory psi recovery left its authored corridor".into());
                };
                self.command(Command::Move { dx, dy: 0 })?;
                self.settle()?;
                self.laboratory_preparation_turn(preparation_day)?;
                moves += 1;
            }
            self.walk_to((9, 4))?;
            self.settle()?;
        }
        self.laboratory_preparation_turn(preparation_day)?;
        let position = self.state.exploration.as_ref().unwrap();
        if (position.x, position.y) != (9, 4) || self.laboratory_needs_psi()? {
            return Err("laboratory preparation returned without full caster psi".into());
        }
        self.command(Command::CastPartyPsi {
            warrior: 5,
            spell: 1103,
            power: 2,
        })?;
        self.walk_to((8, 3))?;
        self.settle()
    }

    fn laboratory_needs_psi(&self) -> Result<bool> {
        let mut missing = false;
        for warrior in &self.state.warriors {
            let maximum = word(&warrior.bytes, 0x5a)?;
            if warrior.stamina()? <= 0 || maximum <= 0 || word(&warrior.bytes, 0x36)? == 0 {
                continue;
            }
            if self.state.psi_disabled != 0 || word(&warrior.bytes, 0x90)? != 0 {
                return Err("laboratory psi recovery requires eligible living casters".into());
            }
            missing |= word(&warrior.bytes, 0x58)? < maximum;
        }
        Ok(missing)
    }

    fn laboratory_recovery_move_budget(&self) -> Result<i32> {
        let mut hours = 0;
        for warrior in &self.state.warriors {
            let maximum = word(&warrior.bytes, 0x5a)?;
            if warrior.stamina()? <= 0 || maximum <= 0 || word(&warrior.bytes, 0x36)? == 0 {
                continue;
            }
            let missing = (i32::from(maximum) - i32::from(word(&warrior.bytes, 0x58)?)).max(0);
            let gain = i32::from(((word(&warrior.bytes, 0x3a)? / 2) as i8).max(1));
            hours = hours.max((missing + gain - 1) / gain);
        }
        let position = self.state.exploration.as_ref().unwrap();
        let map = self
            .content
            .map(position.map)
            .ok_or("laboratory map unavailable")?;
        let mut shortest_move = i32::MAX;
        for (x, y) in [(9, 3), (10, 3)] {
            let index = OpeningMission::index(x, y)?;
            if position.events[index] > 0 {
                return Err("laboratory recovery corridor contains an event".into());
            }
            let tile = position.floor[index];
            let property = if tile.unsigned_abs() > 200 || tile < 0 {
                map.default_tile
            } else {
                tile
            };
            let ticks = if tile < 0 {
                3
            } else {
                map.tile_word(property, 2)?
            };
            if ticks < 0 {
                return Err("laboratory recovery corridor has negative time".into());
            }
            let minutes = i32::from(ticks as u8) * i32::from(map.minutes_per_tick);
            if minutes <= 0 {
                return Err("laboratory recovery corridor does not advance time".into());
            }
            shortest_move = shortest_move.min(minutes);
        }
        Ok((hours * 60 + shortest_move - 1) / shortest_move)
    }

    fn laboratory_preparation_turn(&self, day: i16) -> Result<()> {
        if self.state.phase != Phase::Exploration
            || self.state.mission_stage != 1
            || self.state.combat.is_some()
            || self.state.combat_story.is_some()
            || self.state.movement_continuation.is_some()
            || self.state.exploration.as_ref().is_none_or(|p| {
                p.map != 1
                    || ![(9, 4), (9, 3), (10, 3)].contains(&(p.x, p.y))
                    || p.day != day
                    || p.presentation.is_some()
                    || p.script.is_some()
            })
        {
            return Err("laboratory preparation requires a free authored corridor turn".into());
        }
        Ok(())
    }

    pub(crate) fn walk_to(&mut self, target: (i32, i32)) -> Result<()> {
        self.walk_to_with_sample_preparation(target, false)
    }

    pub(crate) fn walk_to_sample_battle(&mut self) -> Result<()> {
        self.walk_to_with_sample_preparation((66, 85), true)
    }

    fn walk_to_with_sample_preparation(
        &mut self,
        target: (i32, i32),
        mut prepare: bool,
    ) -> Result<()> {
        let p = self.state.exploration.as_ref().unwrap();
        let start_map = p.map;
        let map = self.content.map(p.map).unwrap();
        let start = (p.x, p.y);
        let regions = &p.region_state;
        let mut parent = vec![None; 8100];
        let mut costs = vec![usize::MAX; 8100];
        let mut sequence = 0_usize;
        let mut queue = BinaryHeap::from([Reverse((0_usize, sequence, start.0, start.1))]);
        parent[OpeningMission::index(start.0, start.1)?] = Some(start);
        costs[OpeningMission::index(start.0, start.1)?] = 0;
        while let Some(Reverse((cost, _, x, y))) = queue.pop() {
            if cost != costs[OpeningMission::index(x, y)?] {
                continue;
            }
            if (x, y) == target {
                break;
            }
            for (dx, dy) in [
                (1, 0),
                (0, 1),
                (-1, 0),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ] {
                let next = (x + dx, y + dy);
                if !(0..90).contains(&next.0) || !(0..90).contains(&next.1) {
                    continue;
                }
                let index = OpeningMission::index(next.0, next.1)?;
                // Stay in the swamp opening, using the corridor west of the forest.

                if p.events[index] > 0
                    && next != target
                    && (0..100).any(|slot| {
                        centurions_content::mission::long(&p.event_headers, slot * 42).ok()
                            == Some(next.0 + next.1 * 100 + p.map * 10000)
                            && p.event_headers[slot * 42 + 7] as i8 > 0
                    })
                    && !(p.map == 0 && [(3, 3), (32, 82), (34, 83), (36, 84)].contains(&next))
                    && !(p.map == 1
                        && [(6, 2), (67, 84), (67, 85), (68, 84), (68, 82), (68, 83)]
                            .contains(&next))
                {
                    continue;
                }
                let tile = p.floor[index];
                let property = if !(0..=200).contains(&tile) {
                    map.default_tile
                } else {
                    tile
                };
                if p.events[index] <= 0
                    && (map.tile_word(property, 4)? != 0 || map.tile_word(property, 8)? != 0)
                {
                    continue;
                }
                let secret = (next.0 - 1).max(0)..=(next.0 + 1).min(89);
                if !self.completion
                    && secret.clone().any(|nx| {
                        ((next.1 - 1).max(0)..=(next.1 + 1).min(89))
                            .any(|ny| p.floor[(nx * 90 + ny) as usize].unsigned_abs() > 2999)
                    })
                {
                    continue;
                }
                // The numbered regions with mandatory event scripts are outside this route.
                if !self.completion
                    && (0..20).any(|region| {
                        let o = region * 8;
                        word(regions, o).unwrap_or(90) <= next.1 as i16
                            && next.1 as i16 <= word(regions, o + 4).unwrap_or(-1)
                            && word(regions, o + 2).unwrap_or(90) <= next.0 as i16
                            && next.0 as i16 <= word(regions, o + 6).unwrap_or(-1)
                            && word(regions, 160 + region * 2).unwrap_or(0) > 0
                            && (0..3).any(|event| {
                                word(regions, 400 + region * 6 + event * 2).unwrap_or(0) > 0
                                    && ![6, 366, 367].contains(
                                        &word(regions, 280 + region * 6 + event * 2).unwrap_or(-1),
                                    )
                            })
                    })
                {
                    continue;
                }
                let forest = self.completion
                    && p.map == 1
                    && (0..2).any(|region| {
                        let o = region * 8;
                        word(regions, o).unwrap_or(90) <= next.1 as i16
                            && next.1 as i16 <= word(regions, o + 4).unwrap_or(-1)
                            && word(regions, o + 2).unwrap_or(90) <= next.0 as i16
                            && next.0 as i16 <= word(regions, o + 6).unwrap_or(-1)
                            && word(regions, 160 + region * 2).unwrap_or(0) > 0
                    });
                let next_cost = cost + if forest { 21 } else { 1 };
                if next_cost < costs[index] {
                    costs[index] = next_cost;
                    parent[index] = Some((x, y));
                    sequence += 1;
                    queue.push(Reverse((next_cost, sequence, next.0, next.1)));
                }
            }
        }
        let mut path = vec![];
        let mut cursor = target;
        while cursor != start {
            path.push(cursor);
            cursor=parent[OpeningMission::index(cursor.0,cursor.1)?].ok_or_else(||format!("no supported walking path to supply; {} reachable cells; x extent {} y extent {}",parent.iter().filter(|p|p.is_some()).count(),parent.iter().enumerate().filter(|(_,p)|p.is_some()).map(|(i,_)|i/90).max().unwrap_or(0),parent.iter().enumerate().filter(|(_,p)|p.is_some()).map(|(i,_)|i%90).max().unwrap_or(0)))?;
        }
        path.reverse();
        for next in path {
            let p = self.state.exploration.as_ref().unwrap();
            let direction = (next.0 - p.x, next.1 - p.y);
            if prepare && next == target {
                if !self.completion
                    || self.state.phase != Phase::Exploration
                    || self.state.mission_stage != 1
                    || self.state.combat.is_some()
                    || self.state.combat_story.is_some()
                    || self.state.movement_continuation.is_some()
                    || (p.map, p.x, p.y) != (1, 67, 84)
                    || p.presentation.is_some()
                    || p.script.is_some()
                {
                    return Err("sample battle preparation requires its free approach cell".into());
                }
                self.command(Command::CastPartyPsi {
                    warrior: 5,
                    spell: 1103,
                    power: 1,
                })?;
                prepare = false;
            }
            self.command(Command::Move {
                dx: direction.0 as i8,
                dy: direction.1 as i8,
            })?;
            self.settle()
                .map_err(|e| format!("at {},{}: {e}", next.0, next.1))?;
            let p = self.state.exploration.as_ref().unwrap();
            if p.map != start_map {
                return Ok(());
            }
            if (p.x, p.y) != next {
                if next == target {
                    return Ok(());
                }
                return self.walk_to_with_sample_preparation(target, prepare);
            }
        }
        let p = self.state.exploration.as_ref().unwrap();
        if (p.x, p.y) != target {
            return Err("walking route did not reach its destination".into());
        }
        Ok(())
    }
}

pub fn verify(root: &Path, content: &Content) -> Result<()> {
    verify_route(root, content, true)
}
pub fn verify_day_two(root: &Path, content: &Content) -> Result<()> {
    verify_route(root, content, false)
}
fn verify_route(root: &Path, content: &Content, supply: bool) -> Result<()> {
    let directory = root.join("analysis/test-saves");
    std::fs::create_dir_all(&directory)?;
    for seed in [12, 1] {
        let initial = Session::new(content);
        let mut play = Play {
            content,
            artifacts: &directory,
            state: initial.clone(),
            commands: vec![],
            ordered_events: vec![],
            battles: 0,
            supplies: 0,
            reloads: 0,
            completion: false,
        };
        play.state.random.seed = seed;
        let initial = play.state.clone();
        play.command(Command::ImportStarters)?;
        play.command(Command::EnterMission {
            name: "The Thunder of Guns".into(),
        })?;
        play.command(Command::Move { dx: 1, dy: 1 })?;
        play.settle()?;
        if supply {
            if let Err(error) = play.walk_to((58, 77)) {
                save(
                    &directory.join(format!("supply-failure-{seed}.json")),
                    &play.state,
                    content,
                )?;
                return Err(error);
            }
            if play.supplies != 256 {
                return Err(format!(
                    "authored cache added {} starter rounds, expected 256",
                    play.supplies
                )
                .into());
            }
            save(
                &directory.join(format!("supply-{seed}.json")),
                &play.state,
                content,
            )?;
            let before_reload = play.state.ammunition[0];
            for warrior in 0..6 {
                play.command(Command::Reload { warrior })?;
                play.reloads += 1;
            }
            if play.state.ammunition[0] >= before_reload {
                return Err("supplied ammunition did not reload the starter weapons".into());
            }
        }
        let required_battles = if supply { play.battles + 1 } else { 2 };
        play.walk_to(if supply {
            (15, 10)
        } else if seed == 12 {
            (6, 3)
        } else {
            (8, 5)
        })?;
        for step in 0..2000 {
            if play.battles >= required_battles && play.state.exploration.as_ref().unwrap().day >= 2
            {
                break;
            }
            play.command(Command::Move {
                dx: if step % 2 == 0 { -1 } else { 1 },
                dy: 0,
            })?;
            play.settle()?;
        }
        if play.battles < required_battles || play.state.exploration.as_ref().unwrap().day < 2 {
            return Err("route did not cross midnight and win two encounters".into());
        }
        let mut uninterrupted = initial;
        let mut resumed = None;
        for (index, command) in play.commands.iter().enumerate() {
            if index == play.commands.len() / 2 {
                let path = directory.join(format!("sustained-checkpoint-{seed}.json"));
                save(&path, &uninterrupted, content)?;
                resumed = Some(load(&path, content)?);
            }
            let events = apply(command.clone(), &mut uninterrupted, content)?;
            if let Some(restored) = resumed.as_mut() {
                let actual = apply(command.clone(), restored, content)?;
                if actual != events || *restored != uninterrupted {
                    return Err(format!("resumed route {seed} differs at command {index}").into());
                }
            }
        }
        if uninterrupted != play.state {
            return Err("uninterrupted route differs from planned execution".into());
        }
        save(
            &directory.join(format!(
                "{}-{seed}.json",
                if supply { "sustained" } else { "day-two" }
            )),
            &play.state,
            content,
        )?;
        std::fs::write(
            directory.join(format!(
                "{}-replay-{seed}.json",
                if supply { "sustained" } else { "day-two" }
            )),
            serde_json::to_vec_pretty(&play.commands)?,
        )?;
        println!(
            "Seed {seed}: {} supplied rounds; {} reload commands; {} victories; day {}; {} commands; complete resumed state and ordered events match",
            play.supplies,
            play.reloads,
            play.battles,
            play.state.exploration.as_ref().unwrap().day,
            play.commands.len()
        );
    }
    Ok(())
}
