//! Warrior creation dialogs and record initialization from CODE 1.

use crate::RuleError;
use crate::recovered_audio::{self as audio, AudioContext};
use crate::recovered_character_panels::{self as character, CharacterPanelServices};
use crate::recovered_character_ui::{
    self as character_ui, CharacterUiEffect, CharacterUiServices, CharacterUiState,
};
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingEffect, DrawingServices, PictureObservation,
};
use crate::recovered_equipment::read;
use crate::recovered_feedback::{
    self as feedback, FeedbackArg, FeedbackEffect, FeedbackRequest, FeedbackServices,
};
use crate::recovered_files::FileContext;
use crate::recovered_geometry::BodyRect;
use crate::recovered_inventory_edit::{
    self as inventory_edit, InventoryEditEffect, InventoryEditLimits, InventoryEditServices,
};
use crate::recovered_load_control::{self as load, LoadFlow};
use crate::recovered_prompt_control::{
    self as prompt, LibraryRequest, PromptControlEffect, PromptControlServices, PromptControlState,
    PromptLimits, PromptRegisterRequest, PromptReturn,
};
use crate::recovered_records::RecordWrite;
#[cfg(test)]
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_runtime::{self as runtime, RuntimeResult};
use crate::recovered_storage::{self as storage, StorageEffect, StorageLimits, StorageServices};
use crate::recovered_text::PascalString;
use crate::recovered_timing::{self as timing, TimingServices};
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
use centurions_content::Warrior;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterCreationQualification {
    SourcePortWithObservedBoundariesAndProjectedChildren,
}
pub trait CharacterCreationServices:
    PromptControlServices
    + CharacterUiServices
    + InventoryEditServices
    + StorageServices
    + crate::recovered_status_panels::StatusPanelServices
    + crate::recovered_inventory_selection::InventorySelectionServices
{
}
impl<
    T: PromptControlServices
        + CharacterUiServices
        + InventoryEditServices
        + StorageServices
        + crate::recovered_status_panels::StatusPanelServices
        + crate::recovered_inventory_selection::InventorySelectionServices,
> CharacterCreationServices for T
{
}
#[derive(Clone, Debug, PartialEq)]
pub enum CharacterCreationEffect {
    Prompt(PromptControlEffect),
    CharacterUi(CharacterUiEffect),
    InventoryEdit(InventoryEditEffect),
    Storage(StorageEffect),
    Status(crate::recovered_status_panels::StatusPanelEffect),
    Selection(crate::recovered_inventory_selection::InventorySelectionEffect),
    Experience(crate::recovered_economy::ExperienceOutput),
    Movement(crate::recovered_navigation::MovementResult),
}
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterCreationPlan {
    pub state: PromptControlState,
    pub effects: Vec<CharacterCreationEffect>,
    pub native_return: PromptReturn,
    pub flow: LoadFlow,
    pub qualification: CharacterCreationQualification,
}
struct Execution<'a, S> {
    plan: CharacterCreationPlan,
    s: &'a mut S,
    limits: PromptLimits,
    remaining: u32,
    d0: Option<u32>,
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn range(&mut self, lower: i16, upper: i16) -> Result<i16, RuleError> {
        let value = self
            .rand(upper.wrapping_sub(lower).wrapping_add(1))?
            .wrapping_sub(1)
            .wrapping_add(lower);
        self.low(value as u16);
        Ok(value)
    }
    fn range_words(&mut self, frame: u32, lower: i16, upper: i16) -> Result<i16, RuleError> {
        self.word(frame.wrapping_add(10), upper as u16)?;
        self.word(frame.wrapping_add(8), lower as u16)?;
        self.word(
            frame.wrapping_sub(10),
            upper.wrapping_sub(lower).wrapping_add(1) as u16,
        )?;
        self.range(lower, upper)
    }
    fn range_packed(&mut self, frame: u32) -> Result<i16, RuleError> {
        self.long(frame.wrapping_add(8), 7)?;
        self.word(frame.wrapping_sub(10), 8)?;
        self.range(0, 7)
    }
    fn experience(&mut self, levels: i16) -> Result<(), RuleError> {
        use crate::recovered_economy::{
            self as economy, ExperienceCounters, ExperienceField, ExperienceInput, ExperienceState,
        };
        let state = ExperienceState {
            warriors: vec![],
            global: ExperienceCounters {
                remaining: self.l(0x447bea)? as i32,
                total: self.l(0x447bee)? as i32,
            },
        };
        let child = economy::getexp(
            &state,
            ExperienceInput {
                level_count: levels,
                recipient: -1,
                update_total: 1,
            },
        )?;
        for write in &child.writes {
            self.long(
                match write.field {
                    ExperienceField::Remaining => 0x447bea,
                    ExperienceField::Total => 0x447bee,
                },
                write.after as u32,
            )?;
        }
        self.plan
            .effects
            .push(CharacterCreationEffect::Experience(child));
        self.full(0);
        Ok(())
    }
    fn move_calc(&mut self) -> Result<(), RuleError> {
        let mut memory = crate::recovered_install::InstallMemory {
            base_address: 0x44721c,
            bytes: read(&self.plan.state.memory, 0x44721c, 3138)?,
        };
        let race = self.sw(0x44724e)?.wrapping_sub(1);
        let at = index(0x435664, race, 1);
        let b = self.b(at)?;
        // Supply the reached race byte along with the complete record to the actual arithmetic child.
        let base = at.min(memory.base_address);
        let end = memory.base_address + memory.bytes.len() as u32;
        let mut bytes = vec![0; end.wrapping_sub(base) as usize];
        bytes[(memory.base_address - base) as usize..].copy_from_slice(&memory.bytes);
        bytes[(at - base) as usize] = b;
        memory.base_address = base;
        memory.bytes = bytes;
        let child = crate::recovered_navigation::movecalc(&memory, u16::MAX)?;
        for write in &child.writes {
            self.write(write.address, &write.bytes)?;
        }
        self.full(child.native_return as u32);
        self.plan
            .effects
            .push(CharacterCreationEffect::Movement(child));
        Ok(())
    }
    fn calc_stats(&mut self, frame: u32) -> Result<(), RuleError> {
        let local = frame.wrapping_sub(3138);
        self.copy(0x44721c, local, 3138)?;
        let mut warrior = Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes: read(&self.plan.state.memory, local, 3138)?,
        };
        crate::character::calculate_statistics(&mut warrior)?;
        let mut fields = vec![
            0xb48, 0xb4a, 0xb4c, 0xb4e, 0xb50, 0xb52, 0xb54, 0xb56, 0xb58, 0xb5a, 0xb5c, 0xb5e,
            0xb7a, 0xb46,
        ];
        if self.w(local + 0x36)? != 0 {
            fields.push(0xb44);
        } else {
            fields.extend([0xb40, 0xb44]);
        }
        fields.push(0xb60);
        for (skill, destination) in [
            (0xf4, 0xb62),
            (0xf6, 0xb64),
            (0xf8, 0xb66),
            (0xfa, 0xb68),
            (0xfc, 0xb6a),
            (0xfe, 0xb6c),
            (0x100, 0xb6e),
            (0x102, 0xb70),
            (0x104, 0xb72),
        ] {
            if self.w(local + skill)? != 0 {
                fields.push(destination);
            }
        }
        for offset in fields {
            let o = offset as usize;
            self.word(
                local + offset,
                u16::from_be_bytes([warrior.bytes[o], warrior.bytes[o + 1]]),
            )?;
        }
        self.copy(local, 0x44721c, 3138)?;
        self.full(0);
        Ok(())
    }
    fn new_character(&mut self, frame: u32) -> Result<(), RuleError> {
        let sp = frame.wrapping_sub(22);
        self.profile(sp.wrapping_sub(12), self.w(0x44724e)?, self.w(0x447250)?)?;
        for row in 0..7i16 {
            self.word(index(0x44725e, row, 2), 0)?;
            self.word(0x4434ac, 0)?;
            while self.sw(0x4434ac)? < 7 {
                self.step()?;
                self.byte(
                    index(
                        0x447c0c_u32.wrapping_add(u32::from(row as u16) * 7),
                        self.sw(0x4434ac)?,
                        1,
                    ),
                    0,
                )?;
                self.add(0x4434ac, 1)?;
            }
        }
        for i in 0..23i16 {
            self.word(index(0x4472e2, i, 2), 0)?;
        }
        for i in 0..40i16 {
            self.word(index(0x44727a, i, 2), self.w(index(0x41f8ca, i, 2))?)?;
        }
        for i in 0..40i16 {
            if self.w(index(0x41fa7c, i, 2))? == 1 {
                self.word(index(0x44727a, i, 2), u16::MAX)?;
            }
        }
        for i in 0..100u32 {
            self.copy(0x41fc32, 0x44734e + i * 22, 22)?;
        }
        self.word(0x447344, self.w(0x447254)?.wrapping_shl(3).wrapping_add(30))?;
        if self.w(0x447254)? == 3 {
            self.word(0x447344, 38)?;
        }
        let square = self
            .sw(frame.wrapping_add(8))?
            .wrapping_mul(self.sw(frame.wrapping_add(8))?)
            .wrapping_mul(10);
        self.long(0x447c8e, square as i32 as u32)?;
        let dialog = self.trap(0xa97c, vec![W(128), L(0), L(u32::MAX)])?;
        self.long(0x4394bc, dialog)?;
        self.trap(0xa915, vec![L(self.l(0x4394bc)?)])?;
        self.long(0x4394b8, self.l(0x4394bc)?)?;
        self.port(self.l(0x4394bc)?)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        for op in [0xa981, 0xa922, 0xa923] {
            self.trap(op, vec![L(self.l(0x4394bc)?)])?;
        }
        self.gitem(self.l(0x4394bc)?, 68, 0x4434ba)?;
        self.portrait_plot(
            sp.wrapping_sub(24),
            self.w(0x447270)?,
            0x4434ba,
            self.w(0x447250)?,
        )?;
        self.gitem(self.l(0x4394bc)?, 80, 0x4434ba)?;
        self.icon_plot(sp.wrapping_sub(34), self.w(0x447272)?, 0x4434ba)?;
        'reroll: loop {
            self.step()?;
            for i in 0..15i16 {
                self.word(index(0x447d7e, i, 2), 0)?;
                self.word(index(0x447310, i, 2), 0)?;
            }
            for i in 0..12i16 {
                self.word(index(0x4472ca, i, 2), 0)?;
            }
            for i in 0..8i16 {
                self.word(
                    index(0x44732e, i, 2),
                    self.w(index(0x41f886, i, 2))?
                        .wrapping_add(self.w(index(0x41fa2a, i, 2))?),
                )?;
            }
            for i in 0..8i16 {
                self.word(index(0x4472ca, i, 2), self.w(index(0x41f85a, i, 2))?)?;
            }
            for at in [
                0x44725e, 0x447236, 0x447278, 0x447220, 0x44723e, 0x44724c, 0x447246, 0x447244,
                0x447242, 0x447240, 0x447248, 0x447348, 0x44734a, 0x44734c, 0x447342,
            ] {
                self.word(at, 0)?;
            }
            self.word(0x447256, 1)?;
            self.byte(0x447bfb, 1)?;
            for at in [
                0x447c72, 0x447c6e, 0x447c6a, 0x447c66, 0x447c62, 0x447c5e, 0x447c8a, 0x447c86,
                0x447c82, 0x447c7e, 0x447c7a, 0x447c76,
            ] {
                self.long(at, 0)?;
            }
            let power = self
                .w(0x41f926)?
                .wrapping_add(2)
                .wrapping_add(self.w(0x41fafc)?);
            self.word(0x44725c, power)?;
            self.word(0x44722e, power)?;
            self.long(0x447bee, 0)?;
            self.long(0x447bea, 0)?;
            self.long(sp.wrapping_sub(4), 0xffff0001)?;
            self.word(
                sp.wrapping_sub(6),
                self.w(frame.wrapping_add(8))?.wrapping_add(1),
            )?;
            self.experience(self.sw(sp.wrapping_sub(6))?)?;
            self.long(0x447bee, self.l(0x447bea)?)?;
            self.word(0x447d96, 0)?;
            self.word(0x447228, self.w(0x41fae4)?)?;
            self.word(0x447ce2, self.w(0x41fbb6)?)?;
            self.word(0x447ce6, self.w(0x41fafe)?)?;
            self.first(sp.wrapping_sub(18), 134, 134)?;
            if !self.active() {
                return Ok(());
            }
            for at in [0x447bf2, 0x447bf6, 0x44722a] {
                self.word(at, 0)?;
            }
            for i in 0..7i16 {
                self.step()?;
                let row = if i == 6 { i - 1 } else { i };
                let lower = self.sw(index(0x41fa60, row * 2, 2))?.max(self.sw(index(
                    0x41f8a4,
                    row * 2,
                    2,
                ))?);
                let upper = self.sw(index(0x41fa60, row * 2 + 1, 2))?.min(self.sw(index(
                    0x41f8a4,
                    row * 2 + 1,
                    2,
                ))?);
                let value = self
                    .range_words(sp.wrapping_sub(12), lower, upper)?
                    .wrapping_add(
                        self.sw(index(0x41f896, row, 2))?
                            .wrapping_add(self.sw(index(0x41fa3a, row, 2))?),
                    );
                self.word(index(0x447bfe, i, 2), value as u16)?;
            }
            match self.w(0x447254)? {
                1 => {
                    self.add(0x447bfe, 5)?;
                    self.add(0x447c04, 5)?;
                    self.add(0x447c02, -2)?;
                }
                2 => {
                    self.add(0x447bfe, -3)?;
                    self.add(0x447c04, -7)?;
                    self.add(0x447c06, 5)?;
                    self.add(0x447c02, 5)?;
                }
                _ => {
                    for (at, d) in [
                        (0x447bfe, -5),
                        (0x447c06, 5),
                        (0x447c02, 5),
                        (0x447c08, -5),
                        (0x447c0a, 5),
                    ] {
                        self.add(at, d)?;
                    }
                }
            }
            for i in 0..7i16 {
                let at = index(0x447bfe, i, 2);
                let argument = sp.wrapping_sub((i as u32 + 1) * 6);
                self.long(argument.wrapping_add(2), 0x00030063)?;
                self.word(argument, self.w(at)?)?;
                let value = crate::rules::pin(self.sw(argument)?, 3, 99);
                self.word(at, value as u16)?;
            }
            for (iteration, threshold) in [80, 90, 95].into_iter().enumerate() {
                self.word(sp.wrapping_sub(if iteration == 0 { 44 } else { 2 }), 100)?;
                if self.rand(100)? > threshold {
                    let chosen = self.range_packed(sp.wrapping_sub(12))?;
                    self.add(index(0x4472ca, chosen, 2), 5)?;
                }
            }
            self.word(0x44721c, 1)?;
            self.word(frame.wrapping_sub(2), self.w(0x41fad0)?)?;
            self.word(sp.wrapping_sub(2), self.w(frame.wrapping_sub(2))?)?;
            let stamina = self.rand(self.sw(frame.wrapping_sub(2))?)?;
            self.word(0x447d5e, stamina as u16)?;
            self.word(0x447220, self.w(0x41fae0)?.wrapping_add(self.w(0x41f924)?))?;
            self.word(0x447222, self.w(0x41fadc)?.wrapping_add(self.w(0x41f922)?))?;
            let strength = self.sw(0x447bfe)?;
            self.word(sp.wrapping_sub(4), strength as u16)?;
            let values = crate::character::strength_adjustments(strength);
            self.word(0x4434b8, 0)?;
            self.word(0x4434b6, 0)?;
            if !(7..=15).contains(&strength) {
                self.word(0x4434b6, values[0] as u16)?;
                if strength == 4 || strength >= 16 {
                    self.word(0x4434b8, values[1] as u16)?;
                }
            }
            self.add(0x447228, self.sw(0x4434b8)?)?;
            self.add(0x447222, self.sw(0x4434b6)?)?;
            self.word(0x447226, self.w(0x41faec)?)?;
            self.word(0x447224, self.w(0x41fae8)?)?;
            self.add(
                0x44723e,
                self.sw(0x41faf6)?.wrapping_add(self.sw(0x41f920)?),
            )?;
            let address = index(0x41f948, self.sw(0x41faf2)?.wrapping_sub(1), 4);
            let days = self.range_words(
                sp.wrapping_sub(16),
                self.sw(address)?,
                self.sw(address + 2)?,
            )?;
            self.long(0x447be6, days as i32 as u32)?;
            for i in 0..8i16 {
                let at = index(0x44732e, i, 2);
                if self.sw(at)? < -99 {
                    self.word(at, (-99i16) as u16)?;
                }
                if self.sw(at)? > 300 {
                    self.word(at, 300)?;
                }
            }
            self.long(0x447be6, self.l(0x447be6)?.wrapping_mul(365))?;
            for at in [0x447d5c, 0x447276, 0x447274, 0x447252] {
                self.word(at, 0)?;
            }
            let armor = self.sw(0x447c02)?.wrapping_add(self.sw(0x447c00)?) / 6;
            let perception = self.sw(0x447c0a)? / 3;
            self.word(0x447d5c, armor.wrapping_add(perception) as u16)?;
            for (flag, value) in [(0x41fa4a, 1), (0x41fa50, 2), (0x41fa56, 3)] {
                if self.w(flag)? != 0 {
                    self.word(0x447252, value)?;
                }
            }
            self.word(0x44724c, self.w(0x41faf4)?)?;
            self.word(sp.wrapping_sub(2), u16::MAX)?;
            self.move_calc()?;
            self.character_ui(sp.wrapping_sub(12), 1, false)?;
            self.word(sp.wrapping_sub(6), u16::MAX)?;
            self.calc_stats(sp.wrapping_sub(14))?;
            self.word(sp.wrapping_sub(8), u16::MAX)?;
            let warrior = Warrior {
                name: String::new(),
                original_sha256: String::new(),
                bytes: read(&self.plan.state.memory, 0x44721c, 3138)?,
            };
            let agility = crate::character::agility_adjustment(&warrior)?;
            self.word(0x447248, agility as u16)?;
            let stamina = self.w(0x447d5e)?.wrapping_add(self.w(0x447d62)?);
            self.word(0x44726e, stamina)?;
            self.word(0x44726c, stamina)?;
            let psi = self.w(0x447d5c)?.wrapping_add(self.w(0x447d60)?);
            self.word(0x447276, psi)?;
            self.word(0x447274, psi)?;
            self.copy(0x44721c, 0x448aa0, 3138)?;
            let mut level = 0i16;
            while level < self.sw(frame.wrapping_add(8))? {
                self.step()?;
                let child = sp.wrapping_sub(12);
                self.long(child.wrapping_add(8), 1)?;
                self.level_up(child)?;
                if !self.active() {
                    return Ok(());
                }
                level = level.wrapping_add(1);
            }
            self.copy(0x448aa0, 0x44721c, 3138)?;
            loop {
                self.step()?;
                self.character_ui(sp.wrapping_sub(10), 0, true)?;
                self.modal()?;
                self.gitem(self.l(0x4394bc)?, self.w(0x4434b2)?, 0x44292c)?;
                self.sound(130)?;
                if self.w(0x4434b2)? == 66 || self.w(0x4434b2)? == 1 {
                    self.full(0);
                    return Ok(());
                }
                if self.w(0x4434b2)? == 4 {
                    self.edge_button(sp.wrapping_sub(10), false)?;
                    let child = crate::recovered_status_panels::show_special(
                        &self.plan.state.memory,
                        &self.plan.state.random,
                        sp.wrapping_sub(10),
                        self.remaining as usize,
                        self.s,
                    )?;
                    self.plan.state.memory = child.memory;
                    self.plan.state.random = child.random;
                    self.plan.effects.extend(
                        child
                            .effects
                            .into_iter()
                            .map(CharacterCreationEffect::Status),
                    );
                    if let Some(d) = child.native_return.known_long() {
                        self.full(d);
                    } else {
                        self.d0 = None;
                    }
                    self.port(self.l(0x4394bc)?)?;
                    self.long(0x4394b8, self.l(0x4394bc)?)?;
                    continue;
                }
                for (item, mode) in [(2, 0u16), (3, 1), (5, 2)] {
                    if self.w(0x4434b2)? == item {
                        self.edge_button(sp.wrapping_sub(10), false)?;
                        let child = sp.wrapping_sub(20);
                        let actor = u16::from(self.b(0x44274d)?);
                        self.word(child.wrapping_add(16), actor)?;
                        self.long(child.wrapping_add(12), u32::from(mode) << 16)?;
                        self.word(child.wrapping_add(10), u16::from(self.b(0x44274d)?))?;
                        self.word(child.wrapping_add(8), u16::from(self.b(0x44274d)?))?;
                        let limits =
                            crate::recovered_inventory_selection::InventorySelectionLimits {
                                maximum_steps: self.remaining,
                                prompt: self.limits,
                                equipment: crate::recovered_equipment::EquipmentLimits {
                                    maximum_steps: self.remaining as usize,
                                    maximum_polls: self.limits.maximum_polls,
                                    filename_capacity: self.limits.filename_capacity,
                                },
                                maximum_random_draws: u64::from(self.remaining),
                            };
                        let result = crate::recovered_inventory_selection::show_condition(
                            &self.plan.state,
                            child,
                            limits,
                            self.s,
                        )?;
                        self.plan.state = result.state;
                        self.plan.flow = result.flow;
                        self.plan.effects.extend(
                            result
                                .effects
                                .into_iter()
                                .map(CharacterCreationEffect::Selection),
                        );
                        self.returned(result.native_return);
                        if !self.active() {
                            return Ok(());
                        }
                        self.edge_button(sp.wrapping_sub(22), true)?;
                    }
                }
                if self.w(0x4434b2)? == 65 {
                    continue 'reroll;
                }
            }
        }
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn icon_picture(&mut self, frame: u32, incoming: i16) -> Result<(), RuleError> {
        let mode = self.w(frame.wrapping_add(8))?;
        let sp = frame.wrapping_sub(30);
        let mut page = 0i16;
        let mut selected = incoming as u16;
        let mut race_offset = 0i16;
        self.runtime(0x13e13c)?;
        self.word(frame.wrapping_sub(6), self.w(0x447272)?)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(4))])?;
        if mode != 2 {
            self.trap(
                0xa98b,
                vec![P(0x4034e2), P(0x4034e1), P(0x4034e1), P(0x4034e1)],
            )?;
        }
        let dialog = self.new_dialog(170)?;
        self.long(0x4394b8, dialog)?;
        self.standard_port(dialog, 12)?;
        if mode == 2 {
            self.di_str(sp.wrapping_sub(14), 72, 0x4034e1)?;
        }
        self.trap(0xa915, vec![L(dialog)])?;
        self.trap(0xa981, vec![L(dialog)])?;
        'redraw: loop {
            self.step()?;
            self.trap(0xa862, vec![L(69)])?;
            self.word(0x4434aa, 95)?;
            while self.sw(0x4434aa)? < 100 {
                self.step()?;
                self.gitem(self.l(0x4394b8)?, self.w(0x4434aa)?, 0x44292c)?;
                self.edge_button(sp.wrapping_sub(10), false)?;
                self.add(0x4434aa, 1)?;
            }
            if mode != 2 {
                race_offset = self.sw(0x44724e)?.wrapping_mul(6).wrapping_sub(6);
                let gender = match self.w(0x447254)? {
                    1 => 0x4034dc,
                    2 => 0x4034d4,
                    _ => 0x4034ce,
                };
                self.di_str(sp.wrapping_sub(14), 74, gender)?;
                self.indexed(129, self.w(0x44724e)?)?;
                self.di_str(sp.wrapping_sub(14), 75, 0x479ed8)?;
                self.indexed(131, self.w(0x447250)?)?;
                self.di_str(sp.wrapping_sub(20), 76, 0x479ed8)?;
                self.word(0x4434aa, 3)?;
                while self.sw(0x4434aa)? < 9 {
                    self.step()?;
                    self.gitem(self.l(0x4394b8)?, self.w(0x4434aa)?, 0x44292c)?;
                    self.glyph(
                        self.w(0x4434aa)?
                            .wrapping_add(9254)
                            .wrapping_add(race_offset as u16),
                        true,
                        Some(sp.wrapping_sub(10)),
                    )?;
                    self.add(0x4434aa, 1)?;
                }
            }
            self.word(0x438754, 0)?;
            if mode == 0 {
                self.word(0x4434aa, 0)?;
                while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
                    self.step()?;
                    self.gitem(
                        self.l(0x4394b8)?,
                        self.w(0x4434aa)?.wrapping_add(51),
                        0x4434ba,
                    )?;
                    let body = record(0x448aa0, self.sw(0x4434aa)?, 3138);
                    self.portrait_plot(
                        sp.wrapping_sub(24),
                        self.w(body + 0x54)?,
                        0x4434ba,
                        self.w(body + 0x34)?,
                    )?;
                    self.gitem(
                        self.l(0x4394b8)?,
                        self.w(0x4434aa)?.wrapping_add(45),
                        0x4434ba,
                    )?;
                    let body = record(0x448aa0, self.sw(0x4434aa)?, 3138);
                    self.icon_plot(sp.wrapping_sub(34), self.w(body + 0x56)?, 0x4434ba)?;
                    if self.sb(0x44273f)? == self.sw(0x4434aa)? {
                        self.gitem(
                            self.l(0x4394b8)?,
                            self.w(0x4434aa)?.wrapping_add(33),
                            0x4434ba,
                        )?;
                        self.press(sp.wrapping_sub(22), 1, 2, 0, 0x4434ba)?;
                    }
                    self.selector_party_name(sp)?;
                    self.gitem(
                        self.l(0x4394b8)?,
                        self.w(0x4434aa)?.wrapping_add(57),
                        0x44292c,
                    )?;
                    self.add(0x4434aa, 1)?;
                }
            } else if mode != 2 {
                self.gitem(self.l(0x4394b8)?, 33, 0x4434ba)?;
                self.pict(sp.wrapping_sub(18), 0, 0x4434ba)?;
                self.gitem(self.l(0x4394b8)?, 51, 0x4434ba)?;
                self.portrait_plot(
                    sp.wrapping_sub(34),
                    self.w(0x447270)?,
                    0x4434ba,
                    self.w(0x447250)?,
                )?;
                self.gitem(self.l(0x4394b8)?, 45, 0x4434ba)?;
                self.icon_plot(sp.wrapping_sub(44), self.w(0x447272)?, 0x4434ba)?;
                self.selector_name(sp.wrapping_sub(36), 0x447c3d, 65)?;
                self.gitem(self.l(0x4394b8)?, 57, 0x44292c)?;
                self.edge_button(sp.wrapping_sub(60), false)?;
            } else {
                self.gitem(self.l(0x4394b8)?, 45, 0x4434ba)?;
                self.icon_plot(
                    sp.wrapping_sub(18),
                    self.w(0x43957a)?.wrapping_add(9257),
                    0x4434ba,
                )?;
            }
            'pages: loop {
                self.step()?;
                let group = page / 24;
                self.word(0x4434ac, 77)?;
                while self.sw(0x4434ac)? < 81 {
                    self.step()?;
                    self.indexed(
                        129,
                        self.w(0x4434ac)?
                            .wrapping_sub(76)
                            .wrapping_add((group as u16).wrapping_shl(2)),
                    )?;
                    self.di_str(sp.wrapping_sub(14), self.w(0x4434ac)?, 0x479ed8)?;
                    self.add(0x4434ac, 1)?;
                }
                if page == 48 {
                    self.di_str(sp.wrapping_sub(14), 79, 0x4034ba)?;
                    self.di_str(sp.wrapping_sub(20), 80, 0x4034ba)?;
                }
                self.word(0x4434aa, 9)?;
                while self.sw(0x4434aa)? <= 32 {
                    self.step()?;
                    self.gitem(self.l(0x4394b8)?, self.w(0x4434aa)?, 0x44292c)?;
                    self.glyph(
                        self.w(0x4434aa)?
                            .wrapping_add(9248)
                            .wrapping_add(page as u16),
                        true,
                        Some(sp.wrapping_sub(10)),
                    )?;
                    self.add(0x4434aa, 1)?;
                }
                for op in [0xa922, 0xa923] {
                    self.trap(op, vec![L(self.l(0x4394b8)?)])?;
                }
                self.first(sp.wrapping_sub(12), 130, 130)?;
                if !self.active() {
                    return Ok(());
                }
                while self.w(0x438754)? == 0 {
                    self.step()?;
                    self.modal()?;
                    self.gitem(self.l(0x4394b8)?, self.w(0x4434b2)?, 0x44292c)?;
                    if self.sw(0x4434b2)? < 3 {
                        self.gitem(self.l(0x4394b8)?, 2, 0x44292c)?;
                        self.press(sp.wrapping_sub(22), 2, 2, 0, 0x44292c)?;
                        self.word(0x438754, 1)?;
                    }
                    if self.sw(0x4434b2)? > 2 && self.sw(0x4434b2)? < 33 {
                        if mode == 2 && self.sw(0x4434b2)? > 8 {
                            self.sound(130)?;
                            self.word(
                                0x43957a,
                                self.w(0x4434b2)?.wrapping_sub(9).wrapping_add(page as u16),
                            )?;
                            self.gitem(self.l(0x4394b8)?, 81, 0x4434ba)?;
                            self.pict(sp.wrapping_sub(20), 0, 0x4434ba)?;
                            self.icon_plot(
                                sp.wrapping_sub(30),
                                self.w(0x43957a)?.wrapping_add(9257),
                                0x4434ba,
                            )?;
                        } else if mode != 2 {
                            self.sound(130)?;
                            let offset = if self.sw(0x4434b2)? < 9 {
                                race_offset.wrapping_add(9254)
                            } else {
                                page.wrapping_add(9248)
                            };
                            self.word(0x447272, self.w(0x4434b2)?.wrapping_add(offset as u16))?;
                            self.gitem(
                                self.l(0x4394b8)?,
                                self.sb(0x44273f)?.wrapping_add(33) as u16,
                                0x4434ba,
                            )?;
                            self.pict(sp.wrapping_sub(18), 154, 0x4434ba)?;
                            self.gitem(
                                self.l(0x4394b8)?,
                                self.sb(0x44273f)?.wrapping_add(45) as u16,
                                0x4434ba,
                            )?;
                            self.icon_plot(sp.wrapping_sub(28), self.w(0x447272)?, 0x4434ba)?;
                            selected = self.w(0x4434b2)?;
                        }
                    }
                    if self.w(0x4434b2)? == 57 {
                        self.sound(130)?;
                        page = if page > 95 { 0 } else { page.wrapping_add(24) };
                        continue 'pages;
                    }
                    if self.sw(0x4434b2)? > 32 && self.sw(0x4434b2)? < 39 && mode == 0 {
                        self.sound(141)?;
                        self.gitem(self.l(0x4394b8)?, selected, 0x44292c)?;
                        self.edge_button(sp.wrapping_sub(12), true)?;
                        self.gitem(
                            self.l(0x4394b8)?,
                            self.sb(0x44273f)?.wrapping_add(33) as u16,
                            0x4434ba,
                        )?;
                        self.press(sp.wrapping_sub(26), 1, 1, 0, 0x4434ba)?;
                        self.gitem(self.l(0x4394b8)?, self.w(0x4434b2)?, 0x4434ba)?;
                        self.press(sp.wrapping_sub(40), 1, 2, 0, 0x4434ba)?;
                        self.copy(0x44721c, record(0x448aa0, self.sb(0x44273f)?, 3138), 3138)?;
                        self.byte(0x44273f, self.w(0x4434b2)?.wrapping_sub(33) as u8)?;
                        self.copy(record(0x448aa0, self.sb(0x44273f)?, 3138), 0x44721c, 3138)?;
                        continue 'redraw;
                    }
                }
                break 'pages;
            }
            break 'redraw;
        }
        self.sound(141)?;
        if mode != 2 {
            self.copy(0x44721c, record(0x448aa0, self.sb(0x44273f)?, 3138), 3138)?;
        }
        self.trap(0xa983, vec![L(self.l(0x4394b8)?)])?;
        self.port(self.l(frame.wrapping_sub(4))?)?;
        self.full(0);
        Ok(())
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn glyph(&mut self, id: u16, erase: bool, up_frame: Option<u32>) -> Result<bool, RuleError> {
        let handle = self.trap(0xaa1e, vec![W(id)])?;
        self.long(0x439598, handle)?;
        if erase {
            self.trap(0xa8a3, vec![P(0x44292c)])?;
        }
        if let Some(frame) = up_frame {
            self.edge_button(frame, true)?;
        }
        let missing = self.l(0x439598)? == 0;
        if !missing {
            if self.w(self.l(self.l(0x439598)?)?.wrapping_add(74))? == 32 {
                self.trap(0xa8a9, vec![P(0x44292c), W(6), W(6)])?;
            }
            self.trap(0xaa1f, vec![P(0x44292c), L(self.l(0x439598)?)])?;
            if self.w(self.l(self.l(0x439598)?)?.wrapping_add(74))? == 32 {
                self.trap(
                    0xa8a9,
                    vec![P(0x44292c), W((-6i16) as u16), W((-6i16) as u16)],
                )?;
            }
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        Ok(missing)
    }
    fn first_word(&mut self, frame: u32, first: u16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), first)?;
        let child = prompt::first_tutorial(&self.plan.state, frame, self.limits, self.s)?;
        self.merge_prompt(child);
        Ok(())
    }
    fn selector_party_name(&mut self, sp: u32) -> Result<(), RuleError> {
        self.lib(
            0x400188,
            vec![P(record(0x448aa0, self.sw(0x4434aa)?, 3138) + 0xa21)],
        )?;
        self.trap(0xa862, vec![L(69)])?;
        self.di_str(
            sp.wrapping_sub(18),
            self.w(0x4434aa)?.wrapping_add(65),
            record(0x448aa0, self.sw(0x4434aa)?, 3138) + 0xa21,
        )?;
        self.lib(
            0x400180,
            vec![P(record(0x448aa0, self.sw(0x4434aa)?, 3138) + 0xa21)],
        )?;
        Ok(())
    }
    fn selector_name(&mut self, sp: u32, address: u32, item: u16) -> Result<(), RuleError> {
        self.lib(0x400188, vec![P(address)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.di_str(sp.wrapping_sub(18), item, address)?;
        self.lib(0x400180, vec![P(address)])?;
        Ok(())
    }
    fn portrait(&mut self, frame: u32, incoming: i16) -> Result<(), RuleError> {
        let sp = frame.wrapping_sub(28);
        let mut selected = incoming as u16;
        let mut page = 0i16;
        let mut missing = false;
        self.runtime(0x13e13c)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(4))])?;
        self.trap(
            0xa98b,
            vec![P(0x401a4c), P(0x401a4b), P(0x401a4b), P(0x401a4b)],
        )?;
        let dialog = self.new_dialog(170)?;
        self.long(0x4394b8, dialog)?;
        self.standard_port(dialog, 12)?;
        self.trap(0xa915, vec![L(dialog)])?;
        self.trap(0xa981, vec![L(dialog)])?;
        self.gitem(self.l(0x4394b8)?, 69, 0x44292c)?;
        self.edge_button(sp.wrapping_sub(10), false)?;
        self.gitem(self.l(0x4394b8)?, 70, 0x44292c)?;
        self.edge_button(sp.wrapping_sub(12), false)?;
        'redraw: loop {
            self.step()?;
            let race_offset = self.sw(0x44724e)?.wrapping_mul(6).wrapping_sub(6);
            self.trap(0xa862, vec![L(69)])?;
            let gender = match self.w(0x447254)? {
                1 => 0x401a46,
                2 => 0x401a3e,
                _ => 0x401a38,
            };
            self.di_str(sp.wrapping_sub(14), 74, gender)?;
            self.indexed(129, self.w(0x44724e)?)?;
            self.di_str(sp.wrapping_sub(14), 75, 0x479ed8)?;
            self.indexed(131, self.w(0x447250)?)?;
            self.di_str(sp.wrapping_sub(20), 76, 0x479ed8)?;
            self.word(0x4434aa, 95)?;
            while self.sw(0x4434aa)? < 100 {
                self.step()?;
                self.gitem(dialog, self.w(0x4434aa)?, 0x44292c)?;
                self.edge_button(sp.wrapping_sub(10), false)?;
                self.add(0x4434aa, 1)?;
            }
            self.word(0x4434aa, 3)?;
            while self.sw(0x4434aa)? < 9 {
                self.step()?;
                self.gitem(dialog, self.w(0x4434aa)?, 0x44292c)?;
                self.glyph(
                    self.w(0x4434aa)?
                        .wrapping_add(254)
                        .wrapping_add(race_offset as u16),
                    false,
                    None,
                )?;
                self.edge_button(sp.wrapping_sub(10), true)?;
                if self
                    .w(0x4434aa)?
                    .wrapping_add(254)
                    .wrapping_add(race_offset as u16)
                    == self.w(0x447270)?
                {
                    self.edge_button(sp.wrapping_sub(10), false)?;
                    selected = self.w(0x4434aa)?;
                }
                self.add(0x4434aa, 1)?;
            }
            self.word(0x438754, 0)?;
            if self.w(frame.wrapping_add(8))? == 0 {
                self.word(0x4434aa, 0)?;
                while self.sw(0x4434aa)? <= self.sb(0x44273a)? {
                    self.step()?;
                    self.gitem(dialog, self.w(0x4434aa)?.wrapping_add(45), 0x4434ba)?;
                    let body = record(0x448aa0, self.sw(0x4434aa)?, 3138);
                    self.portrait_plot(
                        sp.wrapping_sub(24),
                        self.w(body + 0x54)?,
                        0x4434ba,
                        self.w(body + 0x34)?,
                    )?;
                    self.gitem(dialog, self.w(0x4434aa)?.wrapping_add(51), 0x4434ba)?;
                    let body = record(0x448aa0, self.sw(0x4434aa)?, 3138);
                    self.icon_plot(sp.wrapping_sub(34), self.w(body + 0x56)?, 0x4434ba)?;
                    if self.sb(0x44273f)? == self.sw(0x4434aa)? {
                        self.gitem(dialog, self.w(0x4434aa)?.wrapping_add(33), 0x4434ba)?;
                        self.press(sp.wrapping_sub(22), 1, 2, 0, 0x4434ba)?;
                    }
                    self.selector_party_name(sp)?;
                    self.add(0x4434aa, 1)?;
                }
            } else {
                self.gitem(dialog, 33, 0x4434ba)?;
                self.pict(sp.wrapping_sub(18), 154, 0x4434ba)?;
                self.gitem(dialog, 45, 0x4434ba)?;
                self.portrait_plot(
                    sp.wrapping_sub(34),
                    self.w(0x447270)?,
                    0x4434ba,
                    self.w(0x447250)?,
                )?;
                self.selector_name(sp.wrapping_sub(26), 0x447c3d, 65)?;
            }
            'pages: loop {
                self.step()?;
                let page_group = page / 24;
                self.word(0x4434ac, 77)?;
                while self.sw(0x4434ac)? < 81 {
                    self.step()?;
                    self.gitem(dialog, self.w(0x4434aa)?, 0x44292c)?;
                    self.edge_button(sp.wrapping_sub(10), false)?;
                    self.indexed(
                        129,
                        self.w(0x4434ac)?
                            .wrapping_sub(76)
                            .wrapping_add((page_group as u16).wrapping_shl(2)),
                    )?;
                    self.di_str(sp.wrapping_sub(16), self.w(0x4434ac)?, 0x479ed8)?;
                    self.add(0x4434ac, 1)?;
                }
                if page == 48 {
                    self.di_str(sp.wrapping_sub(14), 79, 0x401a20)?;
                    self.di_str(sp.wrapping_sub(20), 80, 0x401a20)?;
                }
                self.word(0x4434aa, 9)?;
                while self.sw(0x4434aa)? <= 32 {
                    self.step()?;
                    self.gitem(dialog, self.w(0x4434aa)?, 0x44292c)?;
                    if self.glyph(
                        self.w(0x4434aa)?
                            .wrapping_add(248)
                            .wrapping_add(page as u16),
                        true,
                        None,
                    )? {
                        missing = true;
                    }
                    self.edge_button(sp.wrapping_sub(10), true)?;
                    if self
                        .w(0x4434aa)?
                        .wrapping_add(248)
                        .wrapping_add(page as u16)
                        == self.w(0x447270)?
                    {
                        self.edge_button(sp.wrapping_sub(10), false)?;
                        selected = self.w(0x4434aa)?;
                    }
                    self.add(0x4434aa, 1)?;
                }
                for op in [0xa922, 0xa923] {
                    self.trap(op, vec![L(dialog)])?;
                }
                self.first_word(sp.wrapping_sub(10), 129)?;
                if !self.active() {
                    return Ok(());
                }
                while self.w(0x438754)? == 0 {
                    self.step()?;
                    self.modal()?;
                    self.gitem(dialog, self.w(0x4434b2)?, 0x44292c)?;
                    if self.sw(0x4434b2)? < 3 {
                        self.gitem(dialog, 2, 0x44292c)?;
                        self.press(sp.wrapping_sub(22), 2, 2, 0, 0x44292c)?;
                        self.word(0x438754, 1)?;
                    }
                    if self.sw(0x4434b2)? > 2 && self.sw(0x4434b2)? < 9 {
                        selected = self.portrait_selection(
                            sp,
                            dialog,
                            selected,
                            race_offset.wrapping_add(254),
                        )?;
                    }
                    if self.sw(0x4434b2)? > 8 && self.sw(0x4434b2)? < 33 {
                        selected =
                            self.portrait_selection(sp, dialog, selected, page.wrapping_add(248))?;
                    }
                    if self.w(0x4434b2)? == 57 {
                        self.sound(130)?;
                        page = if missing || page > 95 {
                            0
                        } else {
                            page.wrapping_add(24)
                        };
                        missing = false;
                        continue 'pages;
                    }
                    if self.sw(0x4434b2)? > 32
                        && self.sw(0x4434b2)? < 39
                        && self.w(frame.wrapping_add(8))? == 0
                    {
                        self.sound(141)?;
                        self.gitem(dialog, selected, 0x44292c)?;
                        self.edge_button(sp.wrapping_sub(12), true)?;
                        self.gitem(dialog, self.sb(0x44273f)?.wrapping_add(33) as u16, 0x4434ba)?;
                        self.press(sp.wrapping_sub(26), 1, 1, 0, 0x4434ba)?;
                        self.gitem(dialog, self.w(0x4434b2)?, 0x4434ba)?;
                        self.press(sp.wrapping_sub(40), 1, 2, 0, 0x4434ba)?;
                        self.copy(0x44721c, record(0x448aa0, self.sb(0x44273f)?, 3138), 3138)?;
                        self.byte(0x44273f, self.w(0x4434b2)?.wrapping_sub(33) as u8)?;
                        self.copy(record(0x448aa0, self.sb(0x44273f)?, 3138), 0x44721c, 3138)?;
                        selected = self.w(0x447270)?.wrapping_sub(254);
                        continue 'redraw;
                    }
                }
                break 'pages;
            }
            break 'redraw;
        }
        self.sound(141)?;
        self.copy(0x44721c, record(0x448aa0, self.sb(0x44273f)?, 3138), 3138)?;
        self.trap(0xa983, vec![L(dialog)])?;
        self.port(self.l(frame.wrapping_sub(4))?)?;
        self.full(0);
        Ok(())
    }
    fn portrait_selection(
        &mut self,
        sp: u32,
        dialog: u32,
        selected: u16,
        offset: i16,
    ) -> Result<u16, RuleError> {
        self.gitem(dialog, selected, 0x44292c)?;
        self.edge_button(sp.wrapping_sub(10), true)?;
        self.gitem(dialog, self.w(0x4434b2)?, 0x44292c)?;
        self.edge_button(sp.wrapping_sub(12), false)?;
        self.sound(130)?;
        self.word(0x447270, self.w(0x4434b2)?.wrapping_add(offset as u16))?;
        self.gitem(dialog, self.sb(0x44273f)?.wrapping_add(33) as u16, 0x4434ba)?;
        self.pict(sp.wrapping_sub(24), 154, 0x4434ba)?;
        self.gitem(dialog, self.sb(0x44273f)?.wrapping_add(45) as u16, 0x4434ba)?;
        self.portrait_plot(
            sp.wrapping_sub(40),
            self.w(0x447270)?,
            0x4434ba,
            self.w(0x447250)?,
        )?;
        self.w(0x4434b2)
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn number_rows(
        &mut self,
        sp: u32,
        first: u16,
        last: i16,
        source: u32,
        nz: bool,
    ) -> Result<(), RuleError> {
        self.word(0x4434aa, first)?;
        while self.sw(0x4434aa)? <= last {
            self.step()?;
            let item = self.w(0x4434aa)?;
            let address = index(source, self.sw(0x4434aa)?.wrapping_sub(first as i16), 2);
            self.di_num(sp.wrapping_sub(12), item, self.sw(address)?, nz)?;
            self.add(0x4434aa, 1)?;
        }
        Ok(())
    }
    fn selector_finish(&mut self, sp: u32, dialog: u32) -> Result<(), RuleError> {
        self.gitem(dialog, 56, 0x44292c)?;
        self.press(sp.wrapping_sub(22), 2, 2, 0, 0x44292c)?;
        self.sound(214)?;
        if self.l(0x479fd8)? != 0 {
            self.port(self.l(0x479fd8)?)?;
        }
        self.trap(0xa983, vec![L(dialog)])?;
        if self.l(0x479fd8)? != 0 {
            self.trap(0xa981, vec![L(self.l(0x479fd8)?)])?;
        }
        self.full(0);
        Ok(())
    }
    fn race(&mut self, frame: u32) -> Result<(), RuleError> {
        let force = self.w(frame.wrapping_add(8))?;
        let sp = frame.wrapping_sub(976);
        let portraits = frame.wrapping_sub(960);
        let compatibility = frame.wrapping_sub(900);
        let sound = self.rand(2)?.wrapping_add(200);
        self.sound(sound)?;
        if force == 0 {
            self.profile_open(true)?;
        }
        self.word(0x4434ac, 0)?;
        while self.sw(0x4434ac)? < 30 {
            self.step()?;
            if force == 0 {
                self.lib(
                    0x400048,
                    vec![L(self.l(0x4434a6)?), L(1), L(408), P(0x41f85a)],
                )?;
            }
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 30 {
                self.step()?;
                let row = u32::from(self.w(0x4434ac)?) * 30;
                let col = self.sw(0x4434aa)?;
                let value = if force == 0 {
                    self.b(index(0x41f92a, col, 1))?
                } else {
                    1
                };
                self.byte(index(compatibility.wrapping_add(row), col, 1), value)?;
                self.add(0x4434aa, 1)?;
            }
            self.add(0x4434ac, 1)?;
        }
        if force == 0 {
            self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
        }
        let dialog = self.new_dialog(190)?;
        self.port(dialog)?;
        self.long(0x4394b8, dialog)?;
        self.trap(0xa915, vec![L(dialog)])?;
        self.trap(0xa981, vec![L(dialog)])?;
        self.trap(0xa88a, vec![W(12)])?;
        self.trap(0xa862, vec![L(69)])?;
        if force == 0 {
            self.word(0x44724e, 1)?;
        }
        self.word(0x4434aa, 1)?;
        while self.sw(0x4434aa)? < 30 {
            self.step()?;
            self.profile(sp.wrapping_sub(12), self.w(0x4434aa)?, 0)?;
            self.word(
                index(portraits, self.sw(0x4434aa)?.wrapping_sub(1), 2),
                self.w(0x41f9a8)?,
            )?;
            self.add(0x4434aa, 1)?;
        }
        for op in [0xa922, 0xa923] {
            self.trap(op, vec![L(dialog)])?;
        }
        self.first(sp.wrapping_sub(12), 128, 128)?;
        if !self.active() {
            return Ok(());
        }
        self.first(sp.wrapping_sub(16), 133, 133)?;
        if !self.active() {
            return Ok(());
        }
        'redraw: loop {
            self.step()?;
            for op in [0xa922, 0xa923] {
                self.trap(op, vec![L(dialog)])?;
            }
            self.gitem(dialog, 74, 0x4434ba)?;
            self.portrait_plot(
                sp.wrapping_sub(24),
                self.w(0x447270)?,
                0x4434ba,
                self.w(0x447250)?,
            )?;
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
            self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
            self.profile(sp.wrapping_sub(28), self.w(0x44724e)?, 0)?;
            self.number_rows(sp, 2, 15, 0x41f8a4, false)?;
            self.number_rows(sp, 16, 22, 0x41f896, true)?;
            self.number_rows(sp, 36, 43, 0x41f85a, true)?;
            for (n, (item, address, nz)) in [
                (44, 0x41f91e, false),
                (45, 0x41f920, true),
                (46, 0x41f922, true),
                (47, 0x41f924, true),
            ]
            .into_iter()
            .enumerate()
            {
                self.di_num(
                    sp.wrapping_sub(12 + n as u32 * 4),
                    item,
                    self.sw(address)?,
                    nz,
                )?;
            }
            self.di_str(
                sp.wrapping_sub(14),
                49,
                if self.b(0x41f9a7)? != 0 {
                    0x40197e
                } else {
                    0x40197a
                },
            )?;
            self.di_num(sp.wrapping_sub(12), 50, self.sw(0x41f926)?, true)?;
            self.di_num(sp.wrapping_sub(16), 51, self.sw(0x41f928)?, false)?;
            self.indexed(129, self.w(0x44724e)?)?;
            self.di_str(sp.wrapping_sub(22), 58, 0x479ed8)?;
            self.number_rows(sp, 59, 72, 0x41f86a, true)?;
            self.number_rows(sp, 28, 35, 0x41f886, true)?;
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 4 {
                self.step()?;
                self.di_str(
                    sp.wrapping_sub(14),
                    self.w(0x4434aa)?.wrapping_add(52),
                    0x401978,
                )?;
                self.add(0x4434aa, 1)?;
            }
            let mut shown = 0u16;
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 40 {
                self.step()?;
                if self.w(index(0x41f8ca, self.sw(0x4434aa)?, 2))? != 0 {
                    self.indexed(133, self.w(0x4434aa)?.wrapping_add(1))?;
                    self.di_str(sp.wrapping_sub(14), shown.wrapping_add(52), 0x479ed8)?;
                    shown = shown.wrapping_add(1);
                    if shown == 4 {
                        break;
                    }
                }
                self.add(0x4434aa, 1)?;
            }
            loop {
                self.step()?;
                self.modal()?;
                if self.w(0x4434b2)? == 57 || self.w(0x4434b2)? == 74 {
                    let menu = self.trap(0xa9bf, vec![W(131)])?;
                    self.trap(0xa935, vec![L(menu), W(u16::MAX)])?;
                    self.word(0x4434aa, 1)?;
                    while self.sw(0x4434aa)? <= 30 {
                        self.step()?;
                        self.indexed(129, self.w(0x4434aa)?)?;
                        if self.trap(0xa88c, vec![P(0x479ed8)])? as u16 != 0 {
                            self.trap(0xa933, vec![L(menu), P(0x479ed8)])?;
                            let id = self
                                .w(index(portraits, self.sw(0x4434aa)?.wrapping_sub(1), 2))?
                                .wrapping_mul(6)
                                .wrapping_add(251);
                            self.trap(0xa940, vec![L(menu), W(self.w(0x4434aa)?), W(id)])?;
                            let row = u32::from(self.w(0x4434aa)?.wrapping_sub(1)) * 30;
                            let col = self.sw(0x447250)?.wrapping_sub(1);
                            if self.b(index(compatibility.wrapping_add(row), col, 1))? == 0 {
                                self.trap(0xa93a, vec![L(menu), W(self.w(0x4434aa)?)])?;
                            }
                        }
                        self.add(0x4434aa, 1)?;
                    }
                    self.trap(0xa972, vec![P(0x4471ee)])?;
                    self.trap(0xa870, vec![P(0x4471ee)])?;
                    let chosen = self.trap(
                        0xa80b,
                        vec![L(menu), W(self.w(0x4471ee)?), W(self.w(0x4471f0)?), W(0)],
                    )? as u16;
                    self.word(0x4434b2, chosen)?;
                    self.trap(0xa936, vec![W(131)])?;
                    self.trap(0xa9a3, vec![L(menu)])?;
                    if self.w(0x4434b2)? != 0 {
                        self.word(0x44724e, self.w(0x4434b2)?)?;
                        self.word(
                            0x447270,
                            self.w(0x44724e)?.wrapping_mul(6).wrapping_add(251),
                        )?;
                        continue 'redraw;
                    }
                }
                if self.w(0x4434b2)? == 56 || self.w(0x4434b2)? == 1 {
                    self.selector_finish(sp, dialog)?;
                    return Ok(());
                }
            }
        }
    }
    fn caste(&mut self, frame: u32) -> Result<(), RuleError> {
        let sp = frame.wrapping_sub(68);
        let portraits = frame.wrapping_sub(60);
        let dialog = self.new_dialog(131)?;
        self.port(dialog)?;
        self.long(0x4394b8, dialog)?;
        for op in [0xa915, 0xa981, 0xa922, 0xa923] {
            self.trap(op, vec![L(dialog)])?;
        }
        self.trap(0xa862, vec![L(69)])?;
        self.trap(0xa88a, vec![W(12)])?;
        let sound = self.rand(2)?.wrapping_add(200);
        self.sound(sound)?;
        if self.w(frame.wrapping_add(8))? == 0 {
            self.word(0x447250, 1)?;
            self.word(0x447270, 257)?;
        }
        self.word(0x4434aa, 0)?;
        while self.sw(0x4434aa)? < 30 {
            self.step()?;
            self.profile(sp.wrapping_sub(12), 0, self.w(0x4434aa)?.wrapping_add(1))?;
            self.word(index(portraits, self.sw(0x4434aa)?, 2), self.w(0x41fbb4)?)?;
            self.add(0x4434aa, 1)?;
        }
        'redraw: loop {
            self.step()?;
            self.gitem(dialog, 74, 0x4434ba)?;
            self.portrait_plot(
                sp.wrapping_sub(24),
                self.w(0x447270)?,
                0x4434ba,
                self.w(0x447250)?,
            )?;
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
            self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
            self.profile(sp.wrapping_sub(28), 0, self.w(0x447250)?)?;
            self.number_rows(sp, 2, 15, 0x41fa60, false)?;
            self.number_rows(sp, 16, 22, 0x41fa3a, true)?;
            self.indexed(131, self.w(0x447250)?)?;
            self.di_str(sp.wrapping_sub(14), 58, 0x479ed8)?;
            let scalar = [
                (44, 0x41faf4, true),
                (45, 0x41faf6, true),
                (46, 0x41fafc, true),
                (49, 0x41fbb6, false),
                (50, 0x41fafe, false),
                (51, 0x41fad2, false),
                (37, 0x41fafa, false),
                (38, 0x41faf8, false),
                (40, 0x41fad0, false),
                (90, 0x41fad2, false),
                (41, 0x41fadc, false),
                (91, 0x41fade, false),
                (42, 0x41fae0, false),
                (92, 0x41fae2, false),
                (43, 0x41fae4, false),
                (93, 0x41fae6, false),
                (104, 0x41faec, false),
                (106, 0x41faee, false),
                (105, 0x41fae8, false),
                (107, 0x41faea, false),
            ];
            for (n, (item, address, nz)) in scalar.into_iter().enumerate() {
                self.di_num(
                    sp.wrapping_sub(18 + n as u32 * 4),
                    item,
                    self.sw(address)?,
                    nz,
                )?;
            }
            for (flag, first) in [(0x41fa48, 94), (0x41fa4e, 95), (0x41fa54, 96)] {
                if self.w(flag)? != 0 {
                    self.di_num(sp.wrapping_sub(12), first, self.sw(flag + 2)?, false)?;
                    self.di_num(sp.wrapping_sub(16), first + 3, self.sw(flag + 4)?, false)?;
                } else {
                    self.di_str(sp.wrapping_sub(14), first, 0x403eba)?;
                    self.di_str(sp.wrapping_sub(20), first + 3, 0x403eba)?;
                }
            }
            self.word(0x4434aa, 59)?;
            while self.sw(0x4434aa)? <= 72 {
                self.step()?;
                self.di_num(
                    sp.wrapping_sub(12),
                    self.w(0x4434aa)?,
                    self.sw(index(0x41f9f2, self.sw(0x4434aa)?.wrapping_sub(59), 2))?,
                    true,
                )?;
                self.di_num(
                    sp.wrapping_sub(16),
                    self.w(0x4434aa)?.wrapping_add(16),
                    self.sw(index(0x41fa0e, self.sw(0x4434aa)?.wrapping_sub(59), 2))?,
                    false,
                )?;
                self.add(0x4434aa, 1)?;
            }
            self.number_rows(sp, 28, 35, 0x41fa2a, true)?;
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 4 {
                self.step()?;
                self.di_str(
                    sp.wrapping_sub(14),
                    self.w(0x4434aa)?.wrapping_add(52),
                    0x403eba,
                )?;
                self.di_str(
                    sp.wrapping_sub(20),
                    self.w(0x4434aa)?.wrapping_add(100),
                    0x403eba,
                )?;
                self.add(0x4434aa, 1)?;
            }
            let mut shown = 0u16;
            self.word(0x4434aa, 0)?;
            while self.sw(0x4434aa)? < 40 {
                self.step()?;
                if self.w(index(0x41fa7c, self.sw(0x4434aa)?, 2))? != 0 {
                    self.indexed(133, self.w(0x4434aa)?.wrapping_add(1))?;
                    self.di_str(sp.wrapping_sub(14), shown.wrapping_add(52), 0x479ed8)?;
                    self.lib(
                        0x400038,
                        vec![W(self.w(index(0x41fa7c, self.sw(0x4434aa)?, 2))?)],
                    )?;
                    let value = self.d0.expect("observed abs") as u16 as i16;
                    self.di_num(sp.wrapping_sub(18), shown.wrapping_add(100), value, false)?;
                    shown = shown.wrapping_add(1);
                    if shown == 4 {
                        break;
                    }
                }
                self.add(0x4434aa, 1)?;
            }
            if shown != 4 {
                for op in [0xa922, 0xa923] {
                    self.trap(op, vec![L(dialog)])?;
                }
            }
            self.first(sp.wrapping_sub(12), 127, 127)?;
            if !self.active() {
                return Ok(());
            }
            self.first(sp.wrapping_sub(16), 132, 132)?;
            if !self.active() {
                return Ok(());
            }
            loop {
                self.step()?;
                self.modal()?;
                if self.w(0x4434b2)? == 57 || self.w(0x4434b2)? == 74 {
                    let menu = self.trap(0xa9bf, vec![W(131)])?;
                    self.long(0x438726, menu)?;
                    self.trap(0xa935, vec![L(self.l(0x438726)?), W(u16::MAX)])?;
                    self.word(0x4434aa, 1)?;
                    while self.sw(0x4434aa)? <= 30 {
                        self.step()?;
                        self.indexed(131, self.w(0x4434aa)?)?;
                        if self.trap(0xa88c, vec![P(0x479ed8)])? as u16 != 0 {
                            self.trap(0xa933, vec![L(self.l(0x438726)?), P(0x479ed8)])?;
                            self.trap(
                                0xa940,
                                vec![
                                    L(self.l(0x438726)?),
                                    W(self.w(0x4434aa)?),
                                    W(self.w(index(
                                        portraits,
                                        self.sw(0x4434aa)?.wrapping_sub(1),
                                        2,
                                    ))?),
                                ],
                            )?;
                        }
                        self.add(0x4434aa, 1)?;
                    }
                    self.trap(0xa972, vec![P(0x4471ee)])?;
                    self.trap(0xa870, vec![P(0x4471ee)])?;
                    let chosen = self.trap(
                        0xa80b,
                        vec![
                            L(self.l(0x438726)?),
                            W(self.w(0x4471ee)?),
                            W(self.w(0x4471f0)?),
                            W(0),
                        ],
                    )? as u16;
                    self.word(0x4434b2, chosen)?;
                    if self.w(0x4434b2)? != 0 {
                        self.word(
                            0x447270,
                            self.w(index(portraits, self.sw(0x4434b2)?.wrapping_sub(1), 2))?,
                        )?;
                    }
                    self.trap(0xa936, vec![W(131)])?;
                    self.trap(0xa9a3, vec![L(self.l(0x438726)?)])?;
                    if self.w(0x4434b2)? != 0 {
                        self.word(0x447250, self.w(0x4434b2)?)?;
                        continue 'redraw;
                    }
                }
                if self.w(0x4434b2)? == 56 || self.w(0x4434b2)? == 1 {
                    self.selector_finish(sp, dialog)?;
                    return Ok(());
                }
            }
        }
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn character_ui(&mut self, frame: u32, flag: u16, show: bool) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), flag)?;
        let state = CharacterUiState {
            memory: self.plan.state.memory.clone(),
            files: FileContext {
                filename: vec![0; self.limits.filename_capacity],
                race_profile: vec![0; 408],
                caste_profile: vec![0; 576],
                application_name: vec![0; 256],
                ..FileContext::default()
            },
        };
        let child = if show {
            character_ui::show_stats(&state, flag as i16, frame, self.s)?
        } else {
            character_ui::update_specialties(&state, frame, self.s)?
        };
        self.plan.state.memory = child.state.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterCreationEffect::CharacterUi),
        );
        if let Some(d) = child.native_return {
            self.full(d as u32);
        } else {
            self.d0 = None;
            self.plan.native_return = PromptReturn::Unchanged;
        }
        Ok(())
    }
    fn level_up(&mut self, frame: u32) -> Result<(), RuleError> {
        let member = self.w(frame.wrapping_add(8))?;
        let mode = self.w(frame.wrapping_add(10))?;
        let body = record(0x448aa0, member as i16, 3138);
        self.trap(0xa874, vec![P(frame.wrapping_sub(16))])?;
        self.trap(0xaa19, vec![P(frame.wrapping_sub(6))])?;
        self.trap(0xaa1a, vec![P(frame.wrapping_sub(12))])?;
        self.quiet(-1)?;
        if mode == 0 {
            self.sound(3002)?;
        }
        if mode == 2 {
            self.sound(133)?;
        }
        if self.sw(body.wrapping_add(0x3a))? > 6 {
            self.full(crate::recovered_verification::doreg() as u32);
        }
        self.add(body.wrapping_add(0x3a), 1)?;
        self.copy(body, 0x44721c, 3138)?;
        for n in [28, 26, 24, 22, 20, 18, 30] {
            self.word(frame.wrapping_sub(n), 0)?;
        }
        self.byte(0x4427b3, 1)?;
        if (self.lib(0x400080, vec![W(100)])? as u16 as i16) < 15 {
            self.full(crate::recovered_verification::doreg3() as u32);
        }
        self.profile(
            frame.wrapping_sub(62),
            self.w(body.wrapping_add(0x32))?,
            self.w(body.wrapping_add(0x34))?,
        )?;
        self.word(
            0x44722e,
            self.w(0x41f926)?
                .wrapping_add(2)
                .wrapping_add(self.w(0x41fafc)?),
        )?;
        let mut bonus = false;
        for i in 0..10u32 {
            self.step()?;
            if self.sb(0x41fba2 + i)? <= self.sw(0x447256)? && self.b(0x41fba2 + i)? != 0 {
                if self.sb(0x41fba2 + i)? == self.sw(0x447256)? {
                    bonus = true;
                }
                self.add(0x44722e, 1)?;
            }
        }
        for i in 0..40i16 {
            self.step()?;
            if self.w(index(0x41fa7c, i, 2))? == self.w(0x447256)? {
                let a = index(0x44727a, i, 2);
                if self.sw(a)? > -1 {
                    self.word(a, u16::MAX)?;
                } else {
                    self.add(a, -1)?;
                }
            }
        }
        if self.sw(0x41f928)?.wrapping_mul(2) < self.sw(0x44722e)? {
            self.word(0x44722e, self.w(0x41f928)?.wrapping_mul(2))?;
        }
        let modal = mode == 0 || (mode == 2 && bonus);
        let mut dialog = 0;
        if modal {
            dialog = self.trap(0xa97c, vec![W(150), L(0), L(u32::MAX)])?;
            self.port(dialog)?;
            self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
            self.trap(
                0xa91b,
                vec![
                    L(dialog),
                    W(self.w(0x4395a4)?.wrapping_add(18)),
                    W(self.w(0x4395a2)?.wrapping_add(44)),
                    B(0),
                ],
            )?;
            self.trap(0xa915, vec![L(dialog)])?;
            self.long(0x4394b8, dialog)?;
            self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
            self.trap(0xa88a, vec![W(14)])?;
            self.trap(0xaa14, vec![P(0x44337a)])?;
            for op in [0xa981, 0xa922, 0xa923] {
                self.trap(op, vec![L(dialog)])?;
            }
            self.gitem(dialog, 2, 0x4434ba)?;
            self.portrait_plot(
                frame.wrapping_sub(74),
                self.w(0x447270)?,
                0x4434ba,
                self.w(0x447250)?,
            )?;
            self.trap(0xaa14, vec![P(0x44337a)])?;
            self.lib(0x400188, vec![P(0x447c3d)])?;
            self.di_str(frame.wrapping_sub(84), 4, 0x447c3d)?;
            self.lib(0x400180, vec![P(0x447c3d)])?;
            self.di_num(frame.wrapping_sub(62), 5, i16::from(bonus), false)?;
        }
        self.character_ui(frame.wrapping_sub(60), 0, false)?;
        if modal {
            self.modal()?;
            self.trap(0xa983, vec![L(dialog)])?;
        }
        self.copy(0x44721c, body, 3138)?;
        self.first(frame.wrapping_sub(62), 82, 83)?;
        if !self.active() {
            return Ok(());
        }
        self.port(self.l(frame.wrapping_sub(16))?)?;
        self.trap(0xaa15, vec![P(frame.wrapping_sub(12))])?;
        self.trap(0xaa14, vec![P(frame.wrapping_sub(6))])?;
        self.full(1);
        Ok(())
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn set_file_info(&mut self, frame: u32, name: u32, kind: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(12), name)?;
        self.long(frame.wrapping_add(8), kind)?;
        self.lib(0x400188, vec![P(name)])?;
        self.lib(0x400150, vec![P(name), W(0), P(0x439cfc)])?;
        self.lib(
            0x400090,
            vec![P(self.l(frame.wrapping_add(8))?), P(0x439cfc)],
        )?;
        self.lib(0x400090, vec![P(0x402436), P(0x439d00)])?;
        self.lib(0x400158, vec![P(name), W(0), P(0x439cfc)])?;
        self.lib(0x400180, vec![P(name)])?;
        self.full(0);
        Ok(())
    }
    fn name(&mut self, frame: u32) -> Result<(), RuleError> {
        let mode = self.w(frame.wrapping_add(8))?;
        let local = frame.wrapping_sub(256);
        let base = frame.wrapping_sub(272);
        let dialog = self.new_dialog(132)?;
        self.long(0x4394b8, dialog)?;
        let mut accepted = 1u16;
        let mut handle = 0u32;
        'outer: loop {
            self.step()?;
            self.standard_port(dialog, 18)?;
            self.word(0x438754, 0)?;
            self.trap(
                0xa91b,
                vec![
                    L(dialog),
                    W(half(self.sw(0x40856c)?)
                        .wrapping_add(self.sw(0x4395a4)?)
                        .wrapping_add(161) as u16),
                    W(half(self.sw(0x40856e)?)
                        .wrapping_add(self.sw(0x4395a2)?)
                        .wrapping_add(185) as u16),
                    B(0),
                ],
            )?;
            self.trap(0xa915, vec![L(dialog)])?;
            self.trap(0xa981, vec![L(dialog)])?;
            let sound = self.rand(2)?.wrapping_add(200);
            self.sound(sound)?;
            self.gitem(dialog, 2, 0x4434ba)?;
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x401d9d)])?;
            'inner: loop {
                self.step()?;
                self.trap(0xa981, vec![L(dialog)])?;
                self.first(base.wrapping_sub(12), 125, 125)?;
                if !self.active() {
                    return Ok(());
                }
                while self.w(0x438754)? == 0 {
                    self.step()?;
                    self.modal()?;
                    if self.w(0x4434b2)? != 1 && self.w(0x4434b2)? != 4 {
                        continue;
                    }
                    self.gitem(dialog, 4, 0x44292c)?;
                    self.gitem(dialog, 2, 0x4434ba)?;
                    if mode != 0 {
                        self.trap(0xa990, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
                        self.trap(0xa990, vec![L(self.l(0x4434dc)?), P(0x4385a0)])?;
                        self.lib(0x400180, vec![P(0x479ed8)])?;
                        if self.lib(0x400088, vec![P(0x479ed8)])? as u16 == 0 {
                            self.warn(base.wrapping_sub(10), 70)?;
                            self.port(dialog)?;
                            self.trap(0xa862, vec![L(69)])?;
                            self.di_str(base.wrapping_sub(16), 2, 0x401d9d)?;
                            continue 'inner;
                        }
                        self.trap(0xa990, vec![L(self.l(0x4434dc)?), P(0x438aa4)])?;
                        break 'outer;
                    }
                    self.trap(0xa990, vec![L(self.l(0x4434dc)?), P(local)])?;
                    if self.trap(0xa88c, vec![P(local)])? as u16 == 0 {
                        self.sound(6000)?;
                        if self.question(base.wrapping_sub(14), 11, 1)? == 0 {
                            if !self.active() {
                                return Ok(());
                            }
                            continue 'inner;
                        }
                        if !self.active() {
                            return Ok(());
                        }
                        accepted = 0;
                        break 'outer;
                    }
                    self.lib(0x400180, vec![P(local)])?;
                    self.lib(0x400090, vec![P(local), P(0x447c3d)])?;
                    self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
                    self.trap(0xa88a, vec![W(15)])?;
                    self.lib(0x400188, vec![P(0x447c3d)])?;
                    let width = self.trap(0xa88c, vec![P(0x447c3d)])? as u16;
                    self.word(0x4434b6, width)?;
                    if self.sw(0x4434b6)? > 133 {
                        self.warn(base.wrapping_sub(10), 44)?;
                        continue 'inner;
                    }
                    self.lib(0x400180, vec![P(0x447c3d)])?;
                    if self.lib(0x400088, vec![P(local)])? as u16 as i16 > 28 {
                        self.warn(base.wrapping_sub(10), 44)?;
                        continue 'inner;
                    }
                    self.lib(0x400090, vec![P(0x401d92), P(0x4389a4)])?;
                    self.lib(0x4000a0, vec![W(30), P(local), P(0x4389a4)])?;
                    handle = self.lib(0x400070, vec![P(0x401d8e), P(0x4389a4)])?;
                    if handle != 0 {
                        self.lib(0x400068, vec![L(handle)])?;
                        self.flash(base.wrapping_sub(24), 0x401d2a, 100, 200, 0, 6000)?;
                        let answer = self.question(base.wrapping_sub(28), 4, 0)?;
                        if !self.active() {
                            return Ok(());
                        }
                        if answer == 0 {
                            continue 'outer;
                        }
                        self.trap(0xa981, vec![L(dialog)])?;
                        break 'outer;
                    }
                    handle = self.lib(0x400070, vec![P(0x401d14), P(0x401d18)])?;
                    if handle == 0 {
                        handle = self.lib(0x400070, vec![P(0x401d10), P(0x401d18)])?;
                        if handle == 0 {
                            self.fatal(base.wrapping_sub(10), 10)?;
                            if !self.active() {
                                return Ok(());
                            }
                        }
                    }
                    self.word(0x4434aa, 0)?;
                    while self.sw(0x4434aa)? < 5000 {
                        self.step()?;
                        self.lib(0x400090, vec![P(0x401d9d), P(local)])?;
                        self.lib(0x400048, vec![L(handle), L(1), L(30), P(local)])?;
                        self.lib(0x400048, vec![L(handle), L(1), L(2), P(0x4434b6)])?;
                        if self.lib(0x4000a8, vec![P(0x401d9d), P(local)])? as u16 == 0 {
                            self.word(0x438754, 1)?;
                            self.lib(
                                0x400078,
                                vec![
                                    W(0),
                                    L(self.sw(0x4434aa)?.wrapping_shl(5) as i32 as u32),
                                    L(handle),
                                ],
                            )?;
                            self.lib(0x400050, vec![L(handle), L(1), L(30), P(0x447c3d)])?;
                            self.lib(0x400050, vec![L(handle), L(1), L(2), P(0x447256)])?;
                            self.lib(0x400068, vec![L(handle)])?;
                            self.set_file_info(base.wrapping_sub(62), 0x401d18, 0x401d0a)?;
                            break 'outer;
                        }
                        self.add(0x4434aa, 1)?;
                    }
                }
                break 'outer;
            }
        }
        let _retained_handle = handle;
        self.trap(0xa983, vec![L(dialog)])?;
        self.low(accepted);
        Ok(())
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn gender(&mut self, frame: u32) -> Result<(), RuleError> {
        let mut selected = 2u16;
        let dialog = self.new_dialog(133)?;
        self.port(dialog)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa88a, vec![W(12)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
        self.long(0x4394b8, dialog)?;
        self.word(0x438754, 0)?;
        let sound = self.rand(2)?.wrapping_add(200);
        self.sound(sound)?;
        for op in [0xa915, 0xa981, 0xa922, 0xa923] {
            self.trap(op, vec![L(dialog)])?;
        }
        self.first(frame.wrapping_sub(22), 126, 126)?;
        if !self.active() {
            return Ok(());
        }
        while self.w(0x438754)? == 0 {
            self.step()?;
            self.modal()?;
            if self.w(0x4434b2)? != 6 && self.w(0x4434b2)? != 1 {
                self.sound(141)?;
                self.gitem(dialog, self.w(0x4434b2)?, 0x44292c)?;
                self.edge_button(frame.wrapping_sub(20), false)?;
                self.gitem(dialog, selected.wrapping_add(2), 0x4434ba)?;
                self.trap(0xa8f6, vec![L(self.l(0x439554)?), P(0x4434ba)])?;
                self.word(0x447254, self.w(0x4434b2)?.wrapping_sub(1))?;
                self.gitem(dialog, self.w(0x4434b2)?.wrapping_add(2), 0x4434ba)?;
                self.trap(0xa8f6, vec![L(self.l(0x439550)?), P(0x4434ba)])?;
                selected = self.w(0x4434b2)?;
                self.delay(20)?;
                self.edge_button(frame.wrapping_sub(24), true)?;
            } else if self.w(0x447254)? != 0 {
                self.gitem(dialog, 6, 0x44292c)?;
                self.press(frame.wrapping_sub(30), 2, 2, 0, 0x44292c)?;
                self.word(0x438754, 1)?;
            } else {
                self.sound(6000)?;
            }
        }
        self.trap(0xa983, vec![L(dialog)])?;
        self.plan.native_return = self.d0.map_or(PromptReturn::Unchanged, PromptReturn::Full);
        Ok(())
    }
    fn inventory_index(&mut self, frame: u32, plus: bool, payload: u16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(12), payload)?;
        self.long(frame.wrapping_add(8), 0x447c3d)?;
        let limits = InventoryEditLimits {
            maximum_steps: self.remaining,
            equipment: crate::recovered_equipment::EquipmentLimits {
                maximum_steps: self.remaining as usize,
                maximum_polls: self.limits.maximum_polls,
                filename_capacity: self.limits.filename_capacity,
            },
            maximum_random_draws: u64::from(self.remaining),
        };
        let child = if plus {
            inventory_edit::plus(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                limits,
                self.s,
            )?
        } else {
            inventory_edit::minus(
                &self.plan.state.memory,
                &self.plan.state.random,
                frame,
                limits,
                self.s,
            )?
        };
        self.plan.state.memory = child.memory;
        self.plan.state.random = child.random;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterCreationEffect::InventoryEdit),
        );
        self.d0 = child.native_return;
        self.plan.native_return = if let Some(d) = self.d0 {
            PromptReturn::Full(d)
        } else if self.active() {
            PromptReturn::Unchanged
        } else {
            PromptReturn::NoReturn
        };
        Ok(())
    }
    fn start_level(&mut self, frame: u32) -> Result<(), RuleError> {
        self.copy(0x401e0c, frame.wrapping_sub(14), 14)?;
        let mut selected = 0u16;
        let mut first = true;
        let dialog = self.trap(0xa97c, vec![W(161), L(0), L(u32::MAX)])?;
        self.word(0x438754, 0)?;
        self.port(dialog)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa88a, vec![W(12)])?;
        self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
        self.trap(0xa862, vec![L(69)])?;
        self.trap(
            0xa91b,
            vec![
                L(dialog),
                W(half(self.sw(0x40856c)?)
                    .wrapping_add(self.sw(0x4395a4)?)
                    .wrapping_add(230) as u16),
                W(half(self.sw(0x40856e)?)
                    .wrapping_add(self.sw(0x4395a2)?)
                    .wrapping_add(83) as u16),
                B(0),
            ],
        )?;
        self.trap(0xa915, vec![L(dialog)])?;
        let sound = self.rand(2)?.wrapping_add(200);
        self.sound(sound)?;
        for op in [0xa981, 0xa922, 0xa923] {
            self.trap(op, vec![L(dialog)])?;
        }
        self.first(frame.wrapping_sub(40), 131, 131)?;
        if !self.active() {
            return Ok(());
        }
        while self.w(0x438754)? == 0 {
            self.step()?;
            self.modal()?;
            if self.sw(0x4434b2)? > 2 && self.sw(0x4434b2)? < 15 {
                self.sound(130)?;
                if self.sw(0x4434b2)? > 4 {
                    self.full(crate::recovered_verification::doreg() as u32);
                }
                self.gitem(dialog, self.w(0x4434b2)?, 0x44292c)?;
                self.edge_button(frame.wrapping_sub(36), false)?;
                self.word(0x447256, self.w(0x4434b2)?.wrapping_sub(3))?;
                if selected != self.w(0x4434b2)? && !first {
                    self.gitem(dialog, selected, 0x44292c)?;
                    self.edge_button(frame.wrapping_sub(36), true)?;
                }
                first = false;
                selected = self.w(0x4434b2)?;
            }
            if self.w(0x4434b2)? == 1 || self.w(0x4434b2)? == 29 {
                if selected == 0 {
                    self.sound(6000)?;
                } else {
                    self.gitem(dialog, 29, 0x44292c)?;
                    self.press(frame.wrapping_sub(48), 2, 2, 0, 0x44292c)?;
                    self.sound(6001)?;
                    self.trap(0xa983, vec![L(dialog)])?;
                    self.inventory_index(frame.wrapping_sub(56), false, 1)?;
                    if !self.active() {
                        return Ok(());
                    }
                    self.inventory_index(frame.wrapping_sub(62), true, selected.wrapping_sub(2))?;
                    if !self.active() {
                        return Ok(());
                    }
                    self.low(
                        self.w(index(
                            frame.wrapping_sub(14),
                            selected.wrapping_sub(3) as i16,
                            2,
                        ))?
                        .wrapping_sub(1),
                    );
                    return Ok(());
                }
            }
        }
        self.full(0);
        Ok(())
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }
    fn merge_prompt(&mut self, child: prompt::PromptControlPlan) {
        self.plan.state = child.state;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(CharacterCreationEffect::Prompt),
        );
        self.returned(child.native_return);
    }
    fn first(&mut self, frame: u32, first: u16, last: u16) -> Result<(), RuleError> {
        self.long(
            frame.wrapping_add(8),
            (u32::from(first) << 16) | u32::from(last),
        )?;
        let child = prompt::first_tutorial(&self.plan.state, frame, self.limits, self.s)?;
        self.merge_prompt(child);
        Ok(())
    }
    fn question(&mut self, frame: u32, id: u16, flag: u16) -> Result<u16, RuleError> {
        self.long(
            frame.wrapping_add(8),
            (u32::from(id) << 16) | u32::from(flag),
        )?;
        let child = prompt::question(&self.plan.state, frame, self.limits, self.s)?;
        self.merge_prompt(child);
        match self.plan.native_return {
            PromptReturn::Full(d) => Ok(d as u16),
            PromptReturn::LowWord(w) => Ok(w),
            _ => Err(RuleError::Invalid("question has no native result".into())),
        }
    }
    fn gitem(&mut self, dialog: u32, item: u16, rect: u32) -> Result<(), RuleError> {
        self.trap(
            0xa98d,
            vec![L(dialog), W(item), P(0x4434b0), P(0x4434dc), P(rect)],
        )?;
        Ok(())
    }
    fn di_str(&mut self, frame: u32, item: u16, address: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(10), address)?;
        self.word(frame.wrapping_add(8), item)?;
        self.gitem(self.l(0x4394b8)?, self.w(frame.wrapping_add(8))?, 0x4434ba)?;
        self.trap(
            0xa98f,
            vec![L(self.l(0x4434dc)?), P(self.l(frame.wrapping_add(10))?)],
        )?;
        self.full(0);
        Ok(())
    }
    fn di_num(&mut self, frame: u32, item: u16, value: i16, nz: bool) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(10), value as u16)?;
        self.word(frame.wrapping_add(8), item)?;
        let captured = value;
        self.gitem(self.l(0x4394b8)?, item, 0x4434ba)?;
        if nz && captured == 0 {
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x4021a9)])?;
        } else {
            let number = if nz && captured > 0 {
                captured.wrapping_neg()
            } else if nz {
                captured
            } else {
                self.sw(frame.wrapping_add(10))?
            };
            self.lib(0x400168, vec![L(number as i32 as u32), P(0x479ed8)])?;
            if nz && captured > 0 {
                self.byte(0x479ed9, b'+')?;
            }
            self.trap(0xa98f, vec![L(self.l(0x4434dc)?), P(0x479ed8)])?;
        }
        self.full(0);
        Ok(())
    }
    fn indexed(&mut self, resource: u16, index: u16) -> Result<(), RuleError> {
        self.lib(0x400178, vec![P(0x479ed8), W(resource), W(index)])?;
        Ok(())
    }
    fn modal(&mut self) -> Result<(), RuleError> {
        self.flush()?;
        self.trap(0xa991, vec![L(0), P(0x4434b2)])?;
        Ok(())
    }
    fn new_dialog(&mut self, id: u16) -> Result<u32, RuleError> {
        let dialog = self.trap(0xa97c, vec![W(id), L(0), L(u32::MAX)])?;
        Ok(dialog)
    }
    fn standard_port(&mut self, dialog: u32, size: u16) -> Result<(), RuleError> {
        self.port(dialog)?;
        self.trap(0xaa0b, vec![L(self.l(0x4358ce)?)])?;
        self.trap(0xa887, vec![W(self.w(0x402f3c)?)])?;
        self.trap(0xa88a, vec![W(size)])?;
        self.trap(0xa862, vec![L(69)])?;
        Ok(())
    }
    fn copy(&mut self, source: u32, target: u32, count: u32) -> Result<(), RuleError> {
        for i in 0..count / 4 {
            self.step()?;
            self.long(
                target.wrapping_add(i * 4),
                self.l(source.wrapping_add(i * 4))?,
            )?;
        }
        if count % 4 == 2 {
            self.word(
                target.wrapping_add(count - 2),
                self.w(source.wrapping_add(count - 2))?,
            )?;
        }
        Ok(())
    }
    fn profile_open(&mut self, race: bool) -> Result<(), RuleError> {
        let limits = StorageLimits {
            filename_capacity: self.limits.filename_capacity,
            maximum_iterations: self.remaining as usize,
            maximum_polls: self.limits.maximum_polls,
        };
        let mut child = storage::Execution::new(&self.plan.state.memory, limits, self.d0);
        child.filename(if race { 0x401e02 } else { 0x401de2 }, false, self.s)?;
        self.plan.state.memory = child.plan.memory;
        self.plan.effects.extend(
            child
                .plan
                .effects
                .into_iter()
                .map(CharacterCreationEffect::Storage),
        );
        let h = self.lib(0x400070, vec![P(0x40250e), P(0x4389a4)])?;
        self.long(0x4434a6, h)?;
        if h == 0 {
            let h = self.lib(
                0x400070,
                vec![P(0x40250e), P(if race { 0x401dec } else { 0x401dcc })],
            )?;
            self.long(0x4434a6, h)?;
        }
        self.full(0);
        Ok(())
    }
    fn profile(&mut self, frame: u32, race: u16, caste: u16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(10), caste)?;
        self.word(frame.wrapping_add(8), race)?;
        for (argument, is_race, size, target) in [
            (frame.wrapping_add(8), true, 408u32, 0x41f85a),
            (frame.wrapping_add(10), false, 576, 0x41f9f2),
        ] {
            if self.w(argument)? == 0 {
                continue;
            }
            self.profile_open(is_race)?;
            self.lib(
                0x400078,
                vec![
                    W(0),
                    L(u32::from(self.w(argument)?.wrapping_sub(1)) * size),
                    L(self.l(0x4434a6)?),
                ],
            )?;
            self.lib(
                0x400048,
                vec![L(self.l(0x4434a6)?), L(1), L(size), P(target)],
            )?;
            self.lib(0x400068, vec![L(self.l(0x4434a6)?)])?;
        }
        self.full(0);
        Ok(())
    }
    fn portrait_plot(
        &mut self,
        frame: u32,
        id: u16,
        rect: u32,
        caste: u16,
    ) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(20), 0xffff0000)?;
        self.word(frame.wrapping_add(18), caste)?;
        self.long(frame.wrapping_add(14), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(10), self.l(rect)?)?;
        self.word(frame.wrapping_add(8), id)?;
        let child =
            character::plot_portrait(&self.plan.state.memory, frame, self.remaining, self.s)?;
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterCreationEffect::Prompt(PromptControlEffect::Character(e))),
        );
        self.full(child.native_return);
        Ok(())
    }
    fn icon_plot(&mut self, frame: u32, id: u16, rect: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(14), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(10), self.l(rect)?)?;
        self.word(frame.wrapping_add(8), id)?;
        self.long(0x439598, 0)?;
        let handle = self.trap(0xaa1e, vec![W(self.w(frame.wrapping_add(8))?)])?;
        self.long(0x439598, handle)?;
        if self.l(0x439598)? != 0 {
            self.trap(
                0xaa1f,
                vec![P(frame.wrapping_add(10)), L(self.l(0x439598)?)],
            )?;
            self.trap(0xaa25, vec![L(self.l(0x439598)?)])?;
        }
        self.full(0);
        Ok(())
    }
    fn edge_button(&mut self, frame: u32, up: bool) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), 0)?;
        self.trap(0xa874, vec![P(frame.wrapping_sub(22))])?;
        self.trap(0xa898, vec![P(frame.wrapping_sub(18))])?;
        if self.b(frame.wrapping_add(9))? != 0 {
            self.port(self.l(0x43878c)?)?;
        }
        self.trap(0xa89c, vec![W(0)])?;
        self.trap(0xa89b, vec![W(1), W(1)])?;
        self.trap(
            0xaa0a,
            vec![L(self.l(if up { 0x4358e2 } else { 0x4358de })?)],
        )?;
        self.trap(
            0xa893,
            vec![W(self.w(0x44292e)?), W(self.w(0x442930)?.wrapping_sub(1))],
        )?;
        self.trap(0xa891, vec![W(self.w(0x44292e)?), W(self.w(0x44292c)?)])?;
        self.trap(
            0xa891,
            vec![W(self.w(0x442932)?.wrapping_sub(1)), W(self.w(0x44292c)?)],
        )?;
        self.trap(
            0xaa0a,
            vec![L(self.l(if up { 0x4358de } else { 0x4358e2 })?)],
        )?;
        self.trap(
            0xa893,
            vec![
                W(self.w(0x442932)?.wrapping_sub(1)),
                W(self.w(0x44292c)?.wrapping_add(1)),
            ],
        )?;
        self.trap(
            0xa891,
            vec![
                W(self.w(0x442932)?.wrapping_sub(1)),
                W(self.w(0x442930)?.wrapping_sub(1)),
            ],
        )?;
        self.trap(
            0xa891,
            vec![
                W(self.w(0x44292e)?.wrapping_add(1)),
                W(self.w(0x442930)?.wrapping_sub(1)),
            ],
        )?;
        self.port(self.l(frame.wrapping_sub(22))?)?;
        self.trap(0xa899, vec![P(frame.wrapping_sub(18))])?;
        self.full(0);
        Ok(())
    }
    fn fatal(&mut self, frame: u32, code: u16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), code)?;
        let child = load::scratch2(&self.plan.state.memory, frame, self.s)?;
        self.plan.state.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterCreationEffect::Prompt(PromptControlEffect::Load(e))),
        );
        self.d0 = child.native_return;
        self.plan.native_return = if let Some(d) = self.d0 {
            PromptReturn::Full(d)
        } else if self.active() {
            PromptReturn::Unchanged
        } else {
            PromptReturn::NoReturn
        };
        Ok(())
    }
}
impl<S: CharacterCreationServices> Execution<'_, S> {
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("prompt control processing budget exhausted".into())
        })?;
        Ok(())
    }
    fn b(&self, a: u32) -> Result<u8, RuleError> {
        self.plan.state.memory.byte(a)
    }
    fn w(&self, a: u32) -> Result<u16, RuleError> {
        self.plan.state.memory.word(a)
    }
    fn sw(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.w(a)? as i16)
    }
    fn sb(&self, a: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.b(a)? as i8))
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.state.memory.long(a)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        read(&self.plan.state.memory, a, bytes.len())?;
        self.plan.state.memory.insert_region(a, bytes);
        self.plan
            .effects
            .push(CharacterCreationEffect::Prompt(PromptControlEffect::Write(
                RecordWrite {
                    address: a,
                    bytes: bytes.to_vec(),
                },
            )));
        Ok(())
    }
    fn byte(&mut self, a: u32, v: u8) -> Result<(), RuleError> {
        self.write(a, &[v])
    }
    fn word(&mut self, a: u32, v: u16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn long(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn add(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.word(a, self.w(a)?.wrapping_add(v as u16))
    }
    fn low(&mut self, v: u16) {
        self.d0 = self.d0.map(|d| (d & 0xffff0000) | u32::from(v));
        self.plan.native_return = self.d0.map_or(PromptReturn::LowWord(v), PromptReturn::Full);
    }
    fn full(&mut self, v: u32) {
        self.d0 = Some(v);
        self.plan.native_return = PromptReturn::Full(v);
    }
    fn returned(&mut self, value: PromptReturn) {
        match value {
            PromptReturn::Full(d) => self.full(d),
            PromptReturn::LowWord(w) => self.low(w),
            PromptReturn::Unchanged => {}
            PromptReturn::NoReturn => {
                self.plan.native_return = value;
                self.d0 = None;
            }
        }
    }
    fn observe(&mut self, request: FeedbackRequest) -> Result<u32, RuleError> {
        self.step()?;
        let observation = FeedbackServices::observe(self.s, &request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(CharacterCreationEffect::Prompt(
            PromptControlEffect::Feedback(FeedbackEffect::Service {
                request,
                observation,
            }),
        ));
        Ok(value)
    }
    fn trap(&mut self, opcode: u16, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.observe(FeedbackRequest::Toolbox { opcode, arguments })
    }
    fn registers(
        &mut self,
        opcode: u16,
        d0_input: Option<u32>,
        a0_input: Option<u32>,
        arguments: Vec<FeedbackArg>,
    ) -> Result<u32, RuleError> {
        self.step()?;
        let request = PromptRegisterRequest {
            opcode,
            d0_input,
            a0_input,
            arguments,
        };
        let observation = self
            .s
            .observe_registers(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(CharacterCreationEffect::Prompt(
            PromptControlEffect::RegisterToolbox {
                request,
                observation,
            },
        ));
        Ok(value)
    }
    fn flush(&mut self) -> Result<(), RuleError> {
        self.registers(0xa032, Some(0xffff), None, vec![])?;
        Ok(())
    }
    fn lib(&mut self, entry: u32, arguments: Vec<FeedbackArg>) -> Result<u32, RuleError> {
        self.step()?;
        let request = LibraryRequest { entry, arguments };
        let observation = self.s.library(&request, &self.plan.state.memory)?;
        let value = observation.value;
        self.d0 = Some(observation.d0);
        self.plan.state.memory = observation.memory.clone();
        self.plan.effects.push(CharacterCreationEffect::Prompt(
            PromptControlEffect::Library {
                request,
                observation,
            },
        ));
        Ok(value)
    }
    fn port(&mut self, pointer: u32) -> Result<(), RuleError> {
        self.trap(0xa873, vec![L(pointer)])?;
        Ok(())
    }
    fn feedback(&mut self, child: feedback::FeedbackPlan) {
        self.plan.state.memory = child.memory;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterCreationEffect::Prompt(PromptControlEffect::Feedback(e))),
        );
        if let Some(d) = child.native_return {
            self.full(d);
        } else {
            self.d0 = None;
        }
    }
    fn sound(&mut self, id: i16) -> Result<(), RuleError> {
        let child = feedback::sound(&self.plan.state.memory, id, self.s)?;
        self.feedback(child);
        Ok(())
    }
    fn warn(&mut self, frame: u32, id: i16) -> Result<(), RuleError> {
        self.word(frame.wrapping_add(8), id as u16)?;
        let child = feedback::warn(
            &self.plan.state.memory,
            frame,
            id,
            self.limits.maximum_polls,
            self.s,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn delay(&mut self, duration: i16) -> Result<(), RuleError> {
        let limit = self.limits.maximum_polls;
        let child = timing::delay(
            duration,
            0,
            &mut Clock {
                execution: self,
                remaining: limit,
            },
        )?;
        self.full(child.native_result);
        Ok(())
    }
    fn quiet(&mut self, selector: i16) -> Result<(), RuleError> {
        let last = self.sw(0x439576)?;
        let mut context = AudioContext {
            last_channel: last,
            ..AudioContext::default()
        };
        if last >= 0 {
            let reached: Vec<i16> = if selector > 0 {
                vec![selector]
            } else {
                (1..last).collect()
            };
            if let Some(maximum) = reached.iter().copied().max() {
                context.channels.resize(maximum as usize + 1, 0);
            }
            for n in reached {
                context.channels[n as usize] = self.l(index(0x4436b6, n, 4))?;
            }
        }
        let observations = PromptControlServices::quiet_observations(
            self.s,
            selector,
            &context,
            &self.plan.state.memory,
        )?;
        let child = audio::quiet(&mut context, selector, &observations)?;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterCreationEffect::Prompt(PromptControlEffect::Audio(e))),
        );
        self.full(0);
        Ok(())
    }
    fn runtime(&mut self, entry: u32) -> Result<(), RuleError> {
        let observations =
            CharacterPanelServices::runtime_observations(self.s, entry, &self.plan.state.memory)?;
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        context.memory.bytes = self.plan.state.memory.bytes.clone();
        let child = match entry {
            0x13a498 => runtime::copy_in(&mut context, &observations)?,
            0x13e13c => runtime::compact_heap(&mut context, &observations)?,
            _ => return Err(RuleError::Invalid("unknown prompt runtime child".into())),
        };
        self.plan.state.memory.bytes = context.memory.bytes;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterCreationEffect::Prompt(PromptControlEffect::Runtime(e))),
        );
        match child.result {
            RuntimeResult::Long(v) => self.full(v),
            _ => self.d0 = None,
        }
        Ok(())
    }
    fn press(
        &mut self,
        frame: u32,
        kind: u16,
        phase: u16,
        sound: u16,
        rect: u32,
    ) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(18), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(14), self.l(rect)?)?;
        self.word(frame.wrapping_add(12), sound)?;
        self.long(
            frame.wrapping_add(8),
            (u32::from(kind) << 16) | u32::from(phase),
        )?;
        let child = feedback::pressbutton(
            &self.plan.state.memory,
            frame,
            kind as i16,
            phase as i16,
            sound as i16,
            self.s,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn flash(
        &mut self,
        frame: u32,
        pointer: u32,
        h: u16,
        v: u16,
        wait: u16,
        sound: u16,
    ) -> Result<(), RuleError> {
        self.long(
            frame.wrapping_add(16),
            (u32::from(wait) << 16) | u32::from(sound),
        )?;
        self.long(frame.wrapping_add(12), (u32::from(h) << 16) | u32::from(v))?;
        self.long(frame.wrapping_add(8), pointer)?;
        let child = feedback::flashmessage(
            &self.plan.state.memory,
            frame,
            pointer,
            h as i16,
            v as i16,
            wait as i16,
            sound as i16,
            self.limits.maximum_polls,
            self.s,
        )?;
        self.feedback(child);
        Ok(())
    }
    fn pict(&mut self, frame: u32, id: u16, rect: u32) -> Result<(), RuleError> {
        self.long(frame.wrapping_add(14), self.l(rect.wrapping_add(4))?)?;
        self.long(frame.wrapping_add(10), self.l(rect)?)?;
        self.word(frame.wrapping_add(8), id)?;
        let handle = self.trap(0xa9bc, vec![W(id)])?;
        self.long(frame.wrapping_sub(4), handle)?;
        let bounds = self.plan.state.memory.rect(frame.wrapping_add(10))?;
        let child = drawing::pict(id as i16, bounds, &mut FixedDrawing { handle })?;
        if !matches!(
            child.as_slice(),
            [
                DrawingEffect::GetPicture(_),
                DrawingEffect::DrawPicture { .. }
            ]
        ) {
            return Err(RuleError::Invalid("unexpected pict child plan".into()));
        }
        self.trap(
            0xa8f6,
            vec![L(self.l(frame.wrapping_sub(4))?), P(frame.wrapping_add(10))],
        )?;
        Ok(())
    }
    fn rand(&mut self, bound: i16) -> Result<i16, RuleError> {
        let mut context =
            runtime::RuntimeContext::new(self.plan.state.random.clone(), self.remaining as usize);
        let child = runtime::rand(&mut context, bound)?;
        self.plan.state.random = context.random;
        self.plan.effects.extend(
            child
                .effects
                .into_iter()
                .map(|e| CharacterCreationEffect::Prompt(PromptControlEffect::Runtime(e))),
        );
        let RuntimeResult::Word { value, native_d0 } = child.result else {
            return Err(RuleError::Invalid("unexpected native Rand result".into()));
        };
        self.full(native_d0);
        Ok(value)
    }
}
fn index(base: u32, n: i16, stride: i32) -> u32 {
    base.wrapping_add((i32::from(n).wrapping_mul(stride)) as u32)
}
fn record(base: u32, n: i16, stride: u32) -> u32 {
    base.wrapping_add(u32::from(n as u16) * stride)
}
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
struct FixedDrawing {
    handle: u32,
}
impl DrawingServices for FixedDrawing {
    fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
        Ok(self.handle)
    }
    fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
        Ok(PictureObservation {
            handle: self.handle,
            bounds: None,
        })
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid("unused prompt drawing string".into()))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused prompt drawing world".into()))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid(
            "unused prompt drawing allocation".into(),
        ))
    }
}
struct Clock<'a, 'b, S> {
    execution: &'a mut Execution<'b, S>,
    remaining: usize,
}
fn execution<'a, S: CharacterCreationServices>(
    state: &PromptControlState,
    limits: PromptLimits,
    services: &'a mut S,
) -> Execution<'a, S> {
    Execution {
        plan: CharacterCreationPlan {
            state: state.clone(),
            effects: vec![],
            native_return: PromptReturn::Unchanged,
            flow: LoadFlow::Returned,
            qualification:
                CharacterCreationQualification::SourcePortWithObservedBoundariesAndProjectedChildren,
        },
        s: services,
        limits,
        remaining: limits.maximum_steps,
        d0: None,
    }
}
macro_rules! entry {
    ($name:ident) => {
        pub fn $name(
            state: &PromptControlState,
            frame: u32,
            limits: PromptLimits,
            services: &mut impl CharacterCreationServices,
        ) -> Result<CharacterCreationPlan, RuleError> {
            let mut e = execution(state, limits, services);
            e.$name(frame)?;
            Ok(e.plan)
        }
    };
}
entry!(race);
entry!(caste);
entry!(gender);
entry!(start_level);
entry!(name);
entry!(level_up);
entry!(new_character);
pub fn portrait(
    state: &PromptControlState,
    frame: u32,
    incoming_d3: i16,
    limits: PromptLimits,
    services: &mut impl CharacterCreationServices,
) -> Result<CharacterCreationPlan, RuleError> {
    let mut e = execution(state, limits, services);
    e.portrait(frame, incoming_d3)?;
    Ok(e.plan)
}
pub fn icon_picture(
    state: &PromptControlState,
    frame: u32,
    incoming_d6: i16,
    limits: PromptLimits,
    services: &mut impl CharacterCreationServices,
) -> Result<CharacterCreationPlan, RuleError> {
    let mut e = execution(state, limits, services);
    e.icon_picture(frame, incoming_d6)?;
    Ok(e.plan)
}
impl<S: CharacterCreationServices> TimingServices for Clock<'_, '_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("prompt delay observation budget exhausted".into())
        })?;
        self.execution.observe(FeedbackRequest::TickCount)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(self.execution.observe(FeedbackRequest::Button)? as u8 != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_audio::AudioObservation;
    use crate::recovered_bodies::BodyState;
    use crate::recovered_character_panels::{CharacterPanelObservation, CharacterPanelRequest};
    use crate::recovered_character_ui::{CharacterUiObservation, CharacterUiRequest};
    use crate::recovered_combat_center::CombatCenterServices;
    use crate::recovered_combat_panel::{
        CombatPanelObservation, CombatPanelRequest, CombatPanelServices,
    };
    use crate::recovered_combat_ui::{CombatUiObservation, CombatUiRequest, CombatUiServices};
    use crate::recovered_detail_ui::{DetailReply, DetailRequest, DetailServices};
    use crate::recovered_equipment::EquipmentServices;
    use crate::recovered_equipment_ui::EquipmentUiServices;
    use crate::recovered_feedback::FeedbackObservation;
    use crate::recovered_files::{FileObservation, FileReply, FileRequest};
    use crate::recovered_gamma::{
        GammaError, GammaMemory, GammaObservation, GammaRequest, GammaServices,
    };
    use crate::recovered_gamma_control::{GammaControlRequest, GammaControlServices};
    use crate::recovered_install::InstallMemory;
    use crate::recovered_interface::{InterfaceServices, Point};
    use crate::recovered_inventory_edit::{InventoryEditObservation, InventoryEditRequest};
    use crate::recovered_inventory_rules::InventoryServices;
    use crate::recovered_inventory_rules::SaneRequest;
    use crate::recovered_item_ui::{ItemUiReply, ItemUiRequest, ItemUiServices};
    use crate::recovered_load_control::LoadServices;
    use crate::recovered_load_control::{LoadObservation, LoadRequest, VersionInputs};
    use crate::recovered_prompt_control::LibraryObservation;
    use crate::recovered_rendering::{RenderingObservation, RenderingRequest, RenderingServices};
    use crate::recovered_runtime::RuntimeObservation;
    use crate::recovered_runtime::{RuntimeReply, RuntimeRequest};
    use crate::recovered_status_panels::{
        StatusPanelObservation, StatusPanelRequest, StatusPanelServices,
    };
    use crate::recovered_storage::{StorageObservation, StorageRequest};
    use crate::recovered_text::NumericServices;
    use crate::recovered_world_display::{
        WorldDisplayObservation, WorldDisplayRequest, WorldDisplayServices,
    };
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    const FRAME: u32 = 0x10000;
    fn unused() -> RuleError {
        RuleError::Invalid("unexpected focused creation observation".into())
    }
    fn value(arg: &FeedbackArg) -> u32 {
        match arg {
            W(w) => u32::from(*w),
            L(l) | P(l) => *l,
            B(b) => u32::from(*b),
        }
    }
    struct Services {
        choices: RefCell<VecDeque<u16>>,
        menu: u16,
        ticks: Cell<u32>,
        name: Vec<u8>,
        empty_slot: bool,
        calls: RefCell<Vec<FeedbackRequest>>,
        imports: Vec<LibraryRequest>,
        portrait_selection_change: Cell<bool>,
        modal_item_calls: Cell<u16>,
    }
    impl Services {
        fn new(choices: &[u16]) -> Self {
            Self {
                choices: RefCell::new(choices.iter().copied().collect()),
                menu: 30,
                ticks: Cell::new(0),
                name: b"Aster".to_vec(),
                empty_slot: true,
                calls: RefCell::new(vec![]),
                imports: vec![],
                portrait_selection_change: Cell::new(false),
                modal_item_calls: Cell::new(0),
            }
        }
    }
    impl RenderingServices for Services {
        fn observe(
            &self,
            _: &RenderingRequest,
            _: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError> {
            Err(unused())
        }
    }
    impl CombatUiServices for Services {
        fn observe(
            &self,
            _: &CombatUiRequest,
            _: &RenderingMemory,
        ) -> Result<CombatUiObservation, RuleError> {
            Err(unused())
        }
    }
    impl WorldDisplayServices for Services {
        fn observe(
            &mut self,
            _: &WorldDisplayRequest,
            _: &RenderingMemory,
        ) -> Result<WorldDisplayObservation, RuleError> {
            Err(unused())
        }
    }
    impl CombatCenterServices for Services {}
    impl InventoryServices for Services {
        fn sane(
            &self,
            _: &SaneRequest,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            Err(unused())
        }
    }
    impl GammaServices for Services {
        fn observe(
            &self,
            _: &GammaRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService("unused prompt gamma".into()))
        }
    }
    impl GammaControlServices for Services {
        fn observe_control(
            &self,
            _: &GammaControlRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService(
                "unused prompt gamma control".into(),
            ))
        }
    }
    impl DrawingServices for Services {
        fn color_icon(&mut self, _: i16) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
            Err(unused())
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            Err(unused())
        }
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
            Err(unused())
        }
        fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            Err(unused())
        }
    }
    impl NumericServices for Services {
        fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
            Err(unused())
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            Err(unused())
        }
    }
    impl ItemUiServices for Services {
        fn observe(
            &mut self,
            _: &ItemUiRequest,
            _: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            Err(unused())
        }
    }
    impl EquipmentUiServices for Services {
        fn pen_position(
            &mut self,
            _: u32,
            _: &RenderingMemory,
        ) -> Result<Vec<RecordWrite>, RuleError> {
            Err(unused())
        }
    }
    impl InterfaceServices for Services {
        fn word(&mut self, _: u32) -> Result<i16, RuleError> {
            Err(unused())
        }
        fn current_port(&mut self) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
            Err(unused())
        }
        fn local_to_global(&mut self, _: Point) -> Result<Point, RuleError> {
            Err(unused())
        }
        fn point_in_rect(&mut self, _: Point, _: BodyRect) -> Result<u8, RuleError> {
            Err(unused())
        }
        fn new_region(&mut self) -> Result<u32, RuleError> {
            Err(unused())
        }
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
            Err(unused())
        }
    }
    impl crate::recovered_inventory_selection::InventorySelectionServices for Services {}

    impl FeedbackServices for Services {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.calls.borrow_mut().push(request.clone());
            let mut memory = memory.clone();
            let mut result = 0;
            let d0 = 0xaabb7777;
            match request {
                FeedbackRequest::Toolbox { opcode, arguments } => {
                    let at = |n: usize| value(&arguments[n]);
                    match opcode {
                        0xa874 => {
                            memory.insert_region(at(0), &memory.long(0x3fe302)?.to_be_bytes());
                        }
                        0xa873 => {
                            memory.insert_region(0x3fe302, &at(0).to_be_bytes());
                        }
                        0xaa19 | 0xaa1a => memory.insert_region(at(0), &[1, 2, 3, 4, 5, 6]),
                        0xa898 => memory.insert_region(at(0), &[0; 18]),
                        0xa97c => result = 0xd000,
                        0xa98d => {
                            if self.portrait_selection_change.get() {
                                let count = self.modal_item_calls.get().wrapping_add(1);
                                self.modal_item_calls.set(count);
                                if count == 2 && memory.word(0x4434b2)? == 3 {
                                    memory.insert_region(0x4434b2, &9u16.to_be_bytes());
                                    self.portrait_selection_change.set(false);
                                }
                            }
                            memory.insert_region(at(2), &4u16.to_be_bytes());
                            memory.insert_region(at(3), &(0x60000 + at(1)).to_be_bytes());
                            memory.insert_region(at(4), &[0, 10, 0, 20, 0, 40, 0, 60]);
                        }
                        0xa990 => {
                            let text = PascalString::from_payload(&self.name)?;
                            memory.insert_region(at(1), text.raw());
                        }
                        0xa88c => result = u32::from(memory.byte(at(0))?),
                        0xa991 => {
                            let choice =
                                self.choices.borrow_mut().pop_front().ok_or_else(unused)?;
                            memory.insert_region(at(1), &choice.to_be_bytes());
                            self.modal_item_calls.set(0);
                        }
                        0xa9bf => result = 0x70000,
                        0xa80b => result = 0x00830000 | u32::from(self.menu),
                        0xa9bc => result = 0x70000,
                        0xaa1e => result = 0x90000,
                        _ => {}
                    }
                }
                FeedbackRequest::TickCount => {
                    result = self.ticks.get();
                    self.ticks.set(result + 40);
                }
                FeedbackRequest::Button => {}
                _ => return Err(unused()),
            }
            Ok(FeedbackObservation {
                memory,
                value: result,
                d0,
            })
        }
        fn sound_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(unused())
        }
    }
    fn cstring(memory: &RenderingMemory, at: u32) -> Result<Vec<u8>, RuleError> {
        let mut b = vec![];
        for i in 0..4096 {
            let n = memory.byte(at + i)?;
            if n == 0 {
                return Ok(b);
            }
            b.push(n);
        }
        Err(unused())
    }
    fn pascal(memory: &RenderingMemory, at: u32) -> Result<Vec<u8>, RuleError> {
        read(memory, at + 1, usize::from(memory.byte(at)?))
    }
    fn race_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 408];
        for i in 0..14 {
            bytes[0x4a + i * 2..0x4c + i * 2].copy_from_slice(&10u16.to_be_bytes());
        }
        bytes[0xd0..0xee].fill(1);
        bytes[0xee..0xf0].copy_from_slice(&1u16.to_be_bytes());
        bytes[0xf0..0xf2].copy_from_slice(&2u16.to_be_bytes());
        bytes[0x148..0x14a].copy_from_slice(&2u16.to_be_bytes());
        bytes[0xce..0xd0].copy_from_slice(&100u16.to_be_bytes());
        bytes
    }
    fn caste_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 576];
        for i in 0..14 {
            bytes[0x6e + i * 2..0x70 + i * 2].copy_from_slice(&10u16.to_be_bytes());
        }
        bytes[0x100..0x102].copy_from_slice(&1u16.to_be_bytes());
        bytes[0xe0..0xe2].copy_from_slice(&1u16.to_be_bytes());
        bytes[0x1c2..0x1c4].copy_from_slice(&257u16.to_be_bytes());
        bytes[0xde..0xe0].copy_from_slice(&2u16.to_be_bytes());
        bytes
    }
    impl PromptControlServices for Services {
        fn observe_registers(
            &mut self,
            request: &PromptRegisterRequest,
            memory: &RenderingMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            assert_eq!(request.opcode, 0xa032);
            assert_eq!(request.d0_input, Some(0xffff));
            Ok(FeedbackObservation {
                memory: memory.clone(),
                value: 0,
                d0: 0xffff,
            })
        }
        fn library(
            &mut self,
            request: &LibraryRequest,
            memory: &RenderingMemory,
        ) -> Result<LibraryObservation, RuleError> {
            self.imports.push(request.clone());
            let mut memory = memory.clone();
            let at = |n: usize| value(&request.arguments[n]);
            let mut result = 0;
            match request.entry {
                0x400038 => result = i32::from((at(0) as u16 as i16).wrapping_abs()) as u32,
                0x400080 => result = 100,
                0x400070 => {
                    result = if at(1) == 0x4389a4
                        && cstring(&memory, at(1))?.starts_with(b":Warriors:")
                    {
                        0
                    } else {
                        42
                    }
                }
                0x400168 => {
                    let text = PascalString::from_payload((at(0) as i32).to_string().as_bytes())?;
                    memory.insert_region(at(1), text.raw());
                }
                0x400178 => {
                    let text =
                        PascalString::from_payload(format!("{} {}", at(1), at(2)).as_bytes())?;
                    memory.insert_region(at(0), text.raw());
                }
                0x400180 => {
                    let mut bytes = pascal(&memory, at(0))?;
                    bytes.push(0);
                    memory.insert_region(at(0), &bytes);
                }
                0x400188 => {
                    let bytes = cstring(&memory, at(0))?;
                    let text = PascalString::from_payload(&bytes)?;
                    memory.insert_region(at(0), text.raw());
                }
                0x400090 => {
                    let mut b = cstring(&memory, at(0))?;
                    b.push(0);
                    memory.insert_region(at(1), &b);
                }
                0x400098 | 0x4000a0 => {
                    let offset = usize::from(request.entry == 0x4000a0);
                    let mut d = cstring(&memory, at(1 + offset))?;
                    let mut src = cstring(&memory, at(offset))?;
                    if offset == 1 {
                        src.truncate(at(0) as usize);
                    }
                    d.extend(src);
                    d.push(0);
                    memory.insert_region(at(1 + offset), &d);
                }
                0x400088 => result = cstring(&memory, at(0))?.len() as u32,
                0x4000a8 => {
                    result = u32::from(cstring(&memory, at(0))? != cstring(&memory, at(1))?)
                }
                0x400048 => {
                    match at(3) {
                        0x41f85a => memory.insert_region(at(3), &race_bytes()),
                        0x41f9f2 => memory.insert_region(at(3), &caste_bytes()),
                        _ => {
                            let bytes = if at(2) == 30 {
                                if self.empty_slot {
                                    vec![0; 30]
                                } else {
                                    let mut b = vec![0; 30];
                                    b[..5].copy_from_slice(b"Aster");
                                    b
                                }
                            } else {
                                vec![0; at(2) as usize]
                            };
                            memory.insert_region(at(3), &bytes);
                        }
                    }
                    result = 1;
                }
                0x400078 | 0x400068 | 0x400050 | 0x400150 | 0x400158 => {}
                _ => return Err(unused()),
            }
            Ok(LibraryObservation {
                memory,
                value: result,
                d0: result,
            })
        }
        fn quiet_observations(
            &mut self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Ok(vec![])
        }
        fn incoming_mechanical_mode(&mut self, _: &RenderingMemory) -> Result<i16, RuleError> {
            Err(unused())
        }
    }
    impl CharacterPanelServices for Services {
        fn panel_observe(
            &mut self,
            request: &CharacterPanelRequest,
            memory: &RenderingMemory,
        ) -> Result<CharacterPanelObservation, RuleError> {
            let (value, memory) = match request {
                CharacterPanelRequest::Toolbox {
                    selector,
                    arguments,
                } => {
                    let args = arguments
                        .iter()
                        .map(|a| match a {
                            crate::recovered_character_panels::PanelArgument::Byte(b) => B(*b),
                            crate::recovered_character_panels::PanelArgument::Word(w) => W(*w),
                            crate::recovered_character_panels::PanelArgument::Long(l) => L(*l),
                            crate::recovered_character_panels::PanelArgument::Pointer(p) => P(*p),
                        })
                        .collect();
                    let o = FeedbackServices::observe(
                        self,
                        &FeedbackRequest::Toolbox {
                            opcode: *selector,
                            arguments: args,
                        },
                        memory,
                    )?;
                    (o.value, o.memory)
                }
                CharacterPanelRequest::RuntimeString { .. } => return Err(unused()),
            };
            Ok(CharacterPanelObservation { value, memory })
        }
        fn runtime_observations(
            &mut self,
            entry: u32,
            memory: &RenderingMemory,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            assert_eq!(entry, 0x13e13c);
            let mut trace = vec![];
            for a in [0x4386d8, 0x4386e0, 0x4386e4, 0x4386e8, 0x4386dc] {
                trace.push(RuntimeObservation {
                    request: RuntimeRequest::UnlockPixels(memory.long(a)?),
                    reply: RuntimeReply::Ignored,
                });
            }
            trace.push(RuntimeObservation {
                request: RuntimeRequest::MaxMem,
                reply: RuntimeReply::MaxMem { d0: 100, a0: 200 },
            });
            for (a, id) in [0x4386b4, 0x4386bc, 0x4386c0, 0x4386c4, 0x4386b8]
                .into_iter()
                .zip(1..=5)
            {
                trace.push(RuntimeObservation {
                    request: RuntimeRequest::GetGWorldPixMap(memory.long(a)?),
                    reply: RuntimeReply::Pointer(id),
                });
                trace.push(RuntimeObservation {
                    request: RuntimeRequest::LockPixels(id),
                    reply: RuntimeReply::LockPixels {
                        boolean: 1,
                        native_d0: 0xaabbcc00,
                    },
                });
            }
            Ok(trace)
        }
    }
    impl LoadServices for Services {
        fn observe_load(
            &self,
            _: &LoadRequest,
            _: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            Err(unused())
        }
        fn version_inputs(
            &self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            Err(unused())
        }
        fn filename_number(
            &self,
            variant: u8,
            _: &RenderingMemory,
        ) -> Result<PascalString, RuleError> {
            PascalString::from_payload(variant.to_string().as_bytes())
        }
    }
    impl StorageServices for Services {
        fn observe_storage(
            &self,
            _: &StorageRequest,
            _: &RenderingMemory,
        ) -> Result<StorageObservation, RuleError> {
            Err(unused())
        }
    }
    impl EquipmentServices for Services {
        fn profile_observations(
            &mut self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<Vec<FileObservation>, RuleError> {
            Err(unused())
        }
    }
    impl InventoryEditServices for Services {
        fn observe_edit(
            &mut self,
            request: &InventoryEditRequest,
            memory: &RenderingMemory,
        ) -> Result<InventoryEditObservation, RuleError> {
            let o = match request {
                InventoryEditRequest::Toolbox {
                    opcode, arguments, ..
                } => {
                    let o = FeedbackServices::observe(
                        self,
                        &FeedbackRequest::Toolbox {
                            opcode: *opcode,
                            arguments: arguments.clone(),
                        },
                        memory,
                    )?;
                    return Ok(InventoryEditObservation {
                        memory: o.memory,
                        value: o.value,
                        d0: o.d0,
                    });
                }
                InventoryEditRequest::Imported { address, arguments } => {
                    PromptControlServices::library(
                        self,
                        &LibraryRequest {
                            entry: *address,
                            arguments: arguments.clone(),
                        },
                        memory,
                    )?
                }
            };
            Ok(InventoryEditObservation {
                memory: o.memory,
                value: o.value,
                d0: o.d0,
            })
        }
    }
    impl DetailServices for Services {
        fn observe(
            &mut self,
            _: &DetailRequest,
            _: &RenderingMemory,
        ) -> Result<DetailReply, RuleError> {
            Err(unused())
        }
    }
    impl CombatPanelServices for Services {
        fn observe_panel(
            &mut self,
            _: &CombatPanelRequest,
            _: &RenderingMemory,
        ) -> Result<CombatPanelObservation, RuleError> {
            Err(unused())
        }
        fn visibility_memory(&mut self, _: &RenderingMemory) -> Result<InstallMemory, RuleError> {
            Err(unused())
        }
    }
    impl StatusPanelServices for Services {
        fn observe_status(
            &self,
            _: &StatusPanelRequest,
            _: &RenderingMemory,
        ) -> Result<StatusPanelObservation, RuleError> {
            Err(unused())
        }
        fn body_ground_state(&self, _: &RenderingMemory) -> Result<BodyState, RuleError> {
            Err(unused())
        }
        fn quiet_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &RenderingMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(unused())
        }
        fn heap_observations(
            &self,
            _: &runtime::RuntimeContext,
        ) -> Result<Vec<RuntimeObservation>, RuleError> {
            Err(unused())
        }
    }

    impl CharacterUiServices for Services {
        fn observe(
            &mut self,
            request: &CharacterUiRequest,
            memory: &RenderingMemory,
        ) -> Result<CharacterUiObservation, RuleError> {
            let mut library = None;
            let (opcode, args) = match *request {
                CharacterUiRequest::SetPort(p) => (0xa873, vec![L(p)]),
                CharacterUiRequest::BackPixPat(p) => (0xaa0b, vec![L(p)]),
                CharacterUiRequest::ForeColor(c) => (0xa862, vec![L(c as u32)]),
                CharacterUiRequest::TextSize(n) => (0xa88a, vec![W(n as u16)]),
                CharacterUiRequest::TextFont(n) => (0xa887, vec![W(n as u16)]),
                CharacterUiRequest::RgbForeColor(p) => (0xaa14, vec![P(p)]),
                CharacterUiRequest::GetDialog {
                    id,
                    storage,
                    behind,
                } => (0xa97c, vec![W(id as u16), L(storage), L(behind)]),
                CharacterUiRequest::DrawControls(p) => (0xa915, vec![L(p)]),
                CharacterUiRequest::DrawDialog(p) => (0xa981, vec![L(p)]),
                CharacterUiRequest::BeginUpdate(p) => (0xa922, vec![L(p)]),
                CharacterUiRequest::EndUpdate(p) => (0xa923, vec![L(p)]),
                CharacterUiRequest::DisposeDialog(p) => (0xa983, vec![L(p)]),
                CharacterUiRequest::FlushEvents(_) => (0xa032, vec![]),
                CharacterUiRequest::ModalDialog {
                    filter,
                    item_address,
                } => (0xa991, vec![L(filter), P(item_address)]),
                CharacterUiRequest::GetDialogItem {
                    dialog,
                    item,
                    type_address,
                    handle_address,
                    rect_address,
                } => (
                    0xa98d,
                    vec![
                        L(dialog),
                        W(item as u16),
                        P(type_address),
                        P(handle_address),
                        P(rect_address),
                    ],
                ),
                CharacterUiRequest::SetControlValue { handle, value } => {
                    (0xa963, vec![L(handle), W(value as u16)])
                }
                CharacterUiRequest::SetDialogItemText {
                    handle,
                    text_address,
                } => (0xa98f, vec![L(handle), P(text_address)]),
                CharacterUiRequest::NumberToString { value, destination } => {
                    library = Some(LibraryRequest {
                        entry: 0x400168,
                        arguments: vec![L(value as u32), P(destination)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::GetIndexedString {
                    destination,
                    resource,
                    index,
                } => {
                    library = Some(LibraryRequest {
                        entry: 0x400178,
                        arguments: vec![P(destination), W(resource as u16), W(index as u16)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::CopyCString {
                    destination,
                    source,
                } => {
                    library = Some(LibraryRequest {
                        entry: 0x400090,
                        arguments: vec![P(source), P(destination)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::AppendCString {
                    destination,
                    source,
                    count,
                } => {
                    library = Some(LibraryRequest {
                        entry: 0x4000a0,
                        arguments: vec![W(count), P(source), P(destination)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::CToPascal(p) => {
                    library = Some(LibraryRequest {
                        entry: 0x400188,
                        arguments: vec![P(p)],
                    });
                    (0, vec![])
                }
                CharacterUiRequest::PascalToC(p) => {
                    library = Some(LibraryRequest {
                        entry: 0x400180,
                        arguments: vec![P(p)],
                    });
                    (0, vec![])
                }
            };
            if let Some(r) = library {
                let o = PromptControlServices::library(self, &r, memory)?;
                Ok(CharacterUiObservation {
                    value: o.value,
                    memory: o.memory,
                })
            } else {
                let o = FeedbackServices::observe(
                    self,
                    &FeedbackRequest::Toolbox {
                        opcode,
                        arguments: args,
                    },
                    memory,
                )?;
                Ok(CharacterUiObservation {
                    value: o.value,
                    memory: o.memory,
                })
            }
        }
        fn profile_observations(
            &mut self,
            race: i16,
            caste: i16,
            context: &FileContext,
        ) -> Result<Vec<FileObservation>, RuleError> {
            let mut out = vec![];
            for (id, is_race, size, destination) in [
                (
                    race,
                    true,
                    408,
                    crate::recovered_files::FileBuffer::RaceProfile,
                ),
                (
                    caste,
                    false,
                    576,
                    crate::recovered_files::FileBuffer::CasteProfile,
                ),
            ] {
                if id == 0 {
                    continue;
                }
                let number = PascalString::from_payload(context.variant.to_string().as_bytes())?;
                let n = FileObservation {
                    request: FileRequest::NumberToString(i32::from(context.variant)),
                    reply: FileReply::Pascal(number),
                };
                let mut file = context.clone();
                crate::recovered_files::get_filename(
                    &mut file,
                    if is_race {
                        b"Data Race\0"
                    } else {
                        b"Data MOS\0"
                    },
                    std::slice::from_ref(&n),
                )?;
                out.push(n);
                out.push(FileObservation {
                    request: FileRequest::Open {
                        name: file
                            .filename
                            .into_iter()
                            .take_while(|b| *b != 0)
                            .chain([0])
                            .collect(),
                        mode: b"rb\0".to_vec(),
                    },
                    reply: FileReply::Handle(42),
                });
                out.push(FileObservation {
                    request: FileRequest::Seek {
                        handle: 42,
                        offset: u32::from(id.wrapping_sub(1) as u16) * size,
                        origin: 0,
                    },
                    reply: FileReply::Ignored,
                });
                out.push(FileObservation {
                    request: FileRequest::Read {
                        handle: 42,
                        size,
                        count: 1,
                        destination,
                    },
                    reply: FileReply::Read {
                        bytes_written: if is_race { race_bytes() } else { caste_bytes() },
                        elements_read: 1,
                    },
                });
                out.push(FileObservation {
                    request: FileRequest::Close(42),
                    reply: FileReply::Ignored,
                });
            }
            Ok(out)
        }
    }
    fn fixture() -> PromptControlState {
        let mut memory = RenderingMemory::default();
        for (a, n) in [
            (0xf000, 0x1200),
            (0x3fe300, 32),
            (0x400600, 0x600),
            (0x401000, 0x1100),
            (0x402f30, 0x20),
            (0x403400, 0xd00),
            (0x408560, 32),
            (0x41f800, 0x900),
            (0x435600, 0x400),
            (0x438500, 0x600),
            (0x439400, 0x300),
            (0x439cf0, 0x30),
            (0x442600, 0x200),
            (0x443300, 0x500),
            (0x447000, 0xf00),
            (0x448aa0, 3138),
            (0x479e00, 0x500),
            (0x60000, 0x400),
            (0x70000, 0x100),
            (0x90000, 4),
            (0x91000, 0x100),
        ] {
            memory.insert_region(a, &vec![0; n]);
        }
        memory.insert_region(0x3fe302, &0xa000u32.to_be_bytes());
        memory.insert_region(0x90000, &0x91000u32.to_be_bytes());
        memory.insert_region(0x9104a, &32u16.to_be_bytes());
        memory.insert_region(0x439576, &u16::MAX.to_be_bytes());
        memory.insert_region(0x44724e, &1u16.to_be_bytes());
        memory.insert_region(0x447250, &1u16.to_be_bytes());
        memory.insert_region(0x447254, &1u16.to_be_bytes());
        memory.insert_region(0x447270, &257u16.to_be_bytes());
        memory.insert_region(0x447272, &9257u16.to_be_bytes());
        memory.insert_region(0x44260c, b":Scenario\0");
        memory.insert_region(0x435664, &[10]);
        for (a, bytes) in [
            (0x401e02, b"Data Race\0".as_slice()),
            (0x401de2, b"Data MOS\0"),
            (0x401dec, b":Data Files:Data Race\0"),
            (0x401dcc, b":Data Files:Data MOS\0"),
            (0x40250e, b"rb\0"),
            (0x401d92, b":Warriors:\0"),
            (0x401d9d, b"\0"),
            (0x401d18, b":Data Files:Data CD\0"),
            (0x401a66, b":Warriors:\0"),
            (0x401a71, b"\0"),
            (0x401a7a, b":Data Files:Data CD\0"),
            (0x401a76, b"r+b\0"),
            (0x401a72, b"w+b\0"),
            (0x402436, b"NEWC\0"),
        ] {
            memory.insert_region(a, bytes);
        }
        memory.insert_region(0x401e0c, &[0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7]);
        memory.insert_region(0x41f85a, &race_bytes());
        memory.insert_region(0x41f9f2, &caste_bytes());
        memory.insert_region(0x447c3d, b"Aster\0");
        memory.insert_region(0x448ad2, &1u16.to_be_bytes());
        memory.insert_region(0x448ad4, &1u16.to_be_bytes());
        memory.insert_region(0x448b74, b"Aster\0");
        PromptControlState {
            memory,
            random: crate::RandomState::new(4),
        }
    }
    fn argument(state: &mut PromptControlState, offset: u32, value: u16) {
        state
            .memory
            .insert_region(FRAME + offset, &value.to_be_bytes());
    }
    fn dialog_fixture() -> PromptControlState {
        let mut state = fixture();
        let spans = [
            (FRAME - 334, 344),
            (0x3fe302, 4),
            (0x401a66, 11),
            (0x401a71, 1),
            (0x401a72, 4),
            (0x401a76, 4),
            (0x401a7a, 19),
            (0x401d0a, 1),
            (0x401d18, 20),
            (0x401d92, 11),
            (0x401d9d, 1),
            (0x401e0c, 14),
            (0x402436, 5),
            (0x402f3c, 2),
            (0x40856c, 4),
            (0x4358ce, 4),
            (0x4358de, 8),
            (0x4386cc, 4),
            (0x438754, 2),
            (0x4394b8, 4),
            (0x439550, 8),
            (0x439570, 4),
            (0x439596, 2),
            (0x4395a2, 4),
            (0x439cfc, 8),
            (0x44292c, 8),
            (0x4434aa, 2),
            (0x4434b0, 10),
            (0x4434ba, 8),
            (0x4434dc, 4),
            (0x447254, 4),
            (0x447c3d, 30),
        ];
        state.memory.bytes.retain(|address, _| {
            spans
                .iter()
                .any(|(start, length)| *address >= *start && *address - *start < *length)
        });
        state
    }
    fn portrait_fixture() -> PromptControlState {
        let mut state = fixture();
        let spans = [
            (FRAME - 366, 376),
            (0x3fe302, 4),
            (0x402f3c, 2),
            (0x4358ce, 4),
            (0x4358de, 8),
            (0x4386b4, 20),
            (0x4386cc, 4),
            (0x4386d8, 20),
            (0x438754, 2),
            (0x4394b8, 4),
            (0x439570, 4),
            (0x43957a, 2),
            (0x439584, 2),
            (0x439596, 6),
            (0x44273f, 1),
            (0x44292c, 8),
            (0x4434aa, 4),
            (0x4434b0, 10),
            (0x4434ba, 8),
            (0x4434dc, 4),
            (0x44721c, 3138),
            (0x448aa0, 3138),
            (0x90000, 4),
            (0x9104a, 2),
        ];
        state.memory.bytes.retain(|address, _| {
            spans
                .iter()
                .any(|(start, length)| *address >= *start && *address - *start < *length)
        });
        state
    }
    fn selector_fixture() -> PromptControlState {
        let mut state = fixture();
        let spans = [
            (FRAME - 1294, 1304),
            (0x3fe302, 4),
            (0x401dcc, 21),
            (0x401de2, 9),
            (0x401dec, 22),
            (0x401e02, 10),
            (0x40250e, 3),
            (0x402f3c, 2),
            (0x4358ce, 4),
            (0x4386cc, 4),
            (0x438726, 4),
            (0x4389a4, 256),
            (0x4394b8, 4),
            (0x439570, 4),
            (0x439584, 2),
            (0x439596, 6),
            (0x41f85a, 984),
            (0x44260c, 10),
            (0x44274c, 1),
            (0x44292c, 8),
            (0x4434a6, 4),
            (0x4434aa, 4),
            (0x4434b0, 10),
            (0x4434ba, 8),
            (0x4434dc, 4),
            (0x4471ee, 4),
            (0x44724e, 4),
            (0x447270, 2),
            (0x479fd8, 4),
            (0x90000, 4),
            (0x9104a, 2),
        ];
        state.memory.bytes.retain(|address, _| {
            spans
                .iter()
                .any(|(start, length)| *address >= *start && *address - *start < *length)
        });
        state
    }
    fn character_fixture() -> PromptControlState {
        let mut state = fixture();
        let spans = [
            (FRAME - 3174, 3186),
            (0x3fe302, 4),
            (0x400634, 1080),
            (0x400af8, 8),
            (0x400b12, 1),
            (0x401de2, 9),
            (0x401e02, 10),
            (0x40250e, 3),
            (0x402f3c, 2),
            (0x435664, 1),
            (0x4358ce, 4),
            (0x4389a4, 256),
            (0x4394b8, 8),
            (0x439570, 4),
            (0x439576, 2),
            (0x439584, 2),
            (0x439596, 6),
            (0x41f85a, 1006),
            (0x44260c, 10),
            (0x44273a, 1),
            (0x44274c, 1),
            (0x442788, 4),
            (0x4427b3, 1),
            (0x4427b6, 4),
            (0x44292c, 8),
            (0x4434a6, 4),
            (0x4434aa, 4),
            (0x4434b0, 10),
            (0x4434ba, 8),
            (0x4434dc, 4),
            (0x44721c, 3138),
            (0x448aa0, 3138),
            (0x90000, 4),
            (0x9104a, 2),
        ];
        state.memory.bytes.retain(|address, _| {
            spans
                .iter()
                .any(|(start, length)| *address >= *start && *address - *start < *length)
        });
        state
    }
    fn limits() -> PromptLimits {
        PromptLimits {
            maximum_steps: 20000,
            maximum_polls: 64,
            filename_capacity: 256,
        }
    }
    fn traps(plan: &CharacterCreationPlan, opcode: u16) -> Vec<&[FeedbackArg]> {
        plan.effects
            .iter()
            .filter_map(|e| match e {
                CharacterCreationEffect::Prompt(PromptControlEffect::Feedback(
                    FeedbackEffect::Service {
                        request:
                            FeedbackRequest::Toolbox {
                                opcode: o,
                                arguments,
                            },
                        ..
                    },
                )) if *o == opcode => Some(arguments.as_slice()),
                _ => None,
            })
            .collect()
    }
    fn first_trap(plan: &CharacterCreationPlan, opcode: u16) -> usize {
        plan.effects
            .iter()
            .position(|e| {
                matches!(e,
                    CharacterCreationEffect::Prompt(PromptControlEffect::Feedback(
                        FeedbackEffect::Service {
                            request: FeedbackRequest::Toolbox { opcode: actual, .. }, ..
                        }
                    )) if *actual == opcode
                )
            })
            .unwrap()
    }
    fn first_dialog_write(plan: &CharacterCreationPlan) -> usize {
        plan.effects
            .iter()
            .position(|e| {
                matches!(e,
                    CharacterCreationEffect::Prompt(PromptControlEffect::Write(write))
                        if write.address == 0x4394b8
                )
            })
            .unwrap()
    }
    #[test]
    fn race_and_caste_menus_keep_native_cache_bounds_and_icon_selection() {
        let mut state = selector_fixture();
        argument(&mut state, 8, 1);
        state
            .memory
            .insert_region(FRAME - 960 + 58, &0x2222u16.to_be_bytes());
        let mut services = Services::new(&[57, 56]);
        let race_plan = race(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(race_plan.state.memory.word(0x44724e).unwrap(), 30);
        assert_eq!(race_plan.state.memory.word(0x447270).unwrap(), 431);
        let icons = traps(&race_plan, 0xa940);
        assert_eq!(icons.len(), 30);
        assert_eq!(
            icons[29],
            &[
                L(0x70000),
                W(30),
                W(0x2222u16.wrapping_mul(6).wrapping_add(251))
            ]
        );
        let mut services = Services::new(&[57, 56]);
        let caste_plan = caste(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(caste_plan.state.memory.word(0x447250).unwrap(), 30);
        assert_eq!(caste_plan.state.memory.word(0x447270).unwrap(), 257);
        assert_eq!(traps(&caste_plan, 0xa940).len(), 30);
        assert_eq!(race_plan.state.random.draws, 1);
        assert_eq!(caste_plan.state.random.draws, 1);
        for plan in [&race_plan, &caste_plan] {
            let write = first_dialog_write(plan);
            assert!(first_trap(plan, 0xa873) < write);
            assert!(write < first_trap(plan, 0xa915));
        }
    }
    #[test]
    fn gender_start_level_and_name_use_real_buttons_and_index_writes() {
        let state = dialog_fixture();
        let mut services = Services::new(&[3, 6]);
        let gender_plan = gender(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(gender_plan.state.memory.word(0x447254).unwrap(), 2);
        assert_eq!(gender_plan.native_return, PromptReturn::Full(0xaabb7777));
        let random = gender_plan
            .effects
            .iter()
            .position(|e| {
                matches!(
                    e,
                    CharacterCreationEffect::Prompt(PromptControlEffect::Runtime(
                        runtime::RuntimeEffect::Random { bound: 2, .. }
                    ))
                )
            })
            .unwrap();
        assert!(first_trap(&gender_plan, 0xa887) < first_dialog_write(&gender_plan));
        assert!(first_dialog_write(&gender_plan) < random);
        let windows =
            [0xa915, 0xa981, 0xa922, 0xa923].map(|opcode| first_trap(&gender_plan, opcode));
        assert!(random < windows[0]);
        assert!(windows.windows(2).all(|pair| pair[0] < pair[1]));
        let mut services = Services::new(&[4, 29]);
        let level = start_level(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(level.native_return, PromptReturn::Full(1));
        assert_eq!(level.state.memory.word(0x447256).unwrap(), 1);
        assert!(level.effects.iter().any(|e| matches!(
            e,
            CharacterCreationEffect::InventoryEdit(InventoryEditEffect::Service {
                request: InventoryEditRequest::Imported {
                    address: 0x400050,
                    ..
                },
                ..
            })
        )));
        let mut services = Services::new(&[4]);
        let name_plan = name(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(name_plan.native_return, PromptReturn::Full(0xaabb0001));
        assert_eq!(name_plan.state.memory.word(0x438754).unwrap(), 1);
        assert!(
            services.imports.iter().any(
                |r| r.entry == 0x400050 && r.arguments == vec![L(42), L(1), L(30), P(0x447c3d)]
            )
        );
        assert!(services.imports.iter().any(|r| r.entry == 0x400150));
    }
    #[test]
    fn portrait_and_icon_pages_preserve_declared_modes_and_copy_boundaries() {
        let mut state = portrait_fixture();
        argument(&mut state, 8, 1);
        let mut services = Services::new(&[9, 57, 2]);
        let portrait_plan = portrait(&state, FRAME, 77, limits(), &mut services).unwrap();
        assert_eq!(portrait_plan.state.memory.word(0x447270).unwrap(), 257);
        assert_eq!(portrait_plan.native_return, PromptReturn::Full(0));
        assert_eq!(traps(&portrait_plan, 0xa9bc).len(), 2);
        drop(portrait_plan);
        let mut services = Services::new(&[3, 2]);
        services.portrait_selection_change.set(true);
        let changed = portrait(&state, FRAME, 77, limits(), &mut services).unwrap();
        let selections: Vec<u16> = changed
            .effects
            .iter()
            .filter_map(|effect| match effect {
                CharacterCreationEffect::Prompt(PromptControlEffect::Write(write))
                    if write.address == 0x447270 =>
                {
                    Some(u16::from_be_bytes(
                        write.bytes.as_slice().try_into().unwrap(),
                    ))
                }
                _ => None,
            })
            .collect();
        assert_eq!(selections, vec![263, 257]);
        drop(changed);
        argument(&mut state, 8, 2);
        let before = read(&state.memory, 0x448aa0, 3138).unwrap();
        let mut services = Services::new(&[10, 57, 2]);
        let icon = icon_picture(&state, FRAME, 77, limits(), &mut services).unwrap();
        assert_eq!(icon.state.memory.word(0x43957a).unwrap(), 1);
        assert_eq!(read(&icon.state.memory, 0x448aa0, 3138).unwrap(), before);
        assert_eq!(traps(&icon, 0xa9bc).len(), 1);
    }
    #[test]
    fn level_up_increases_level_applies_skills_and_copies_entire_record() {
        let mut state = character_fixture();
        argument(&mut state, 8, 0);
        argument(&mut state, 10, 1);
        state.memory.insert_region(0x448ada, &6u16.to_be_bytes());
        state.memory.insert_region(0x447256, &7u16.to_be_bytes());
        let mut services = Services::new(&[]);
        let plan = level_up(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(plan.state.memory.word(0x448ada).unwrap(), 7);
        assert_eq!(plan.native_return, PromptReturn::Full(1));
        assert_eq!(
            read(&plan.state.memory, 0x44721c, 3138).unwrap(),
            read(&plan.state.memory, 0x448aa0, 3138).unwrap()
        );
        assert_eq!(traps(&plan, 0xa97c).len(), 0);
        assert_eq!(plan.state.random, state.random);
    }
    #[test]
    fn new_character_reroll_keeps_forward_copies_and_rng_consumption() {
        let mut state = character_fixture();
        argument(&mut state, 8, 0);
        let mut services = Services::new(&[65, 1]);
        let plan = new_character(&state, FRAME, limits(), &mut services).unwrap();
        assert_eq!(plan.native_return, PromptReturn::Full(0));
        assert_eq!(plan.state.memory.word(0x447256).unwrap(), 1);
        assert_eq!(
            read(&plan.state.memory, 0x44721c, 3138).unwrap(),
            read(&plan.state.memory, 0x448aa0, 3138).unwrap()
        );
        assert_eq!(
            plan.effects
                .iter()
                .filter(|e| matches!(e, CharacterCreationEffect::Experience(_)))
                .count(),
            2
        );
        let draws = plan
            .effects
            .iter()
            .filter_map(|e| match e {
                CharacterCreationEffect::Prompt(PromptControlEffect::Runtime(
                    runtime::RuntimeEffect::Random { bound, .. },
                )) => Some(*bound),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(draws.len() as u64, plan.state.random.draws);
        assert_eq!(draws.iter().filter(|v| **v == 100).count(), 6);
        assert!(draws.iter().filter(|v| **v == 1).count() >= 14);
    }
}
