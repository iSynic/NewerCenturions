//! Spell display bodies with native memory and explicit Toolbox/library observations.

use crate::{
    RuleError,
    recovered_drawing::{self, DrawingContext, DrawingEffect, DrawingServices},
    recovered_geometry::BodyRect,
    recovered_interface::{self, ButtonInput, InterfaceEffect, InterfaceServices},
    recovered_records::{self, RecordWrite, SPELL_BYTES},
    recovered_rendering::RenderingMemory,
    recovered_spatial::{self, Position, SpatialServices},
    recovered_text::{self, NumericServices, PascalString, TextContext, TextEffect},
    recovered_timing::{self, TimingEffect, TimingServices},
};

const ACTION: &str = "spell_ui";

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: ACTION,
        dependency,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellUiState {
    pub memory: RenderingMemory,
    /// The existing icon helper's work context; unrelated fields are preserved.
    pub drawing: DrawingContext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogItemObservation {
    pub item_type: i16,
    pub handle: u32,
    pub rect: BodyRect,
}

/// These methods observe OS/library results; none substitutes a gameplay command.
pub trait SpellUiServices:
    NumericServices + DrawingServices + InterfaceServices + TimingServices
{
    fn dialog_item(&mut self, dialog: u32, item: i16) -> Result<DialogItemObservation, RuleError>;
    fn string_width(&mut self, text: &PascalString) -> Result<i16, RuleError>;
    fn color_cursor(&mut self, id: i16) -> Result<u32, RuleError>;
    fn square_root_extended(&mut self, value: [u8; 10]) -> Result<[u8; 10], RuleError>;
    /// Observe Fintrz followed by EXT-to-LONG, including original SANE exceptions.
    fn truncate_extended(&mut self, value: [u8; 10]) -> Result<i32, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpellUiEffect {
    Write(RecordWrite),
    Drawing(DrawingEffect),
    Interface(InterfaceEffect),
    Text(TextEffect),
    Timing(TimingEffect),
    DialogItem {
        dialog: u32,
        item: i16,
        rect_address: u32,
        observation: DialogItemObservation,
    },
    SetDialogItemText {
        item: i16,
        handle: u32,
        text: PascalString,
    },
    RgbForeColor {
        address: u32,
        components: [u16; 3],
    },
    TextMode(i16),
    OffsetRect {
        address: u32,
        horizontal: i16,
        vertical: i16,
        before: BodyRect,
        after: BodyRect,
    },
    StringWidth {
        text: PascalString,
        width: i16,
    },
    GetColorCursor {
        id: i16,
        handle: u32,
    },
    SetColorCursor(u32),
    DisposeColorCursor(u32),
    SquareRootExtended {
        input: [u8; 10],
        output: [u8; 10],
    },
    TruncateExtended {
        input: [u8; 10],
        output: i32,
    },
    LocalWrite {
        offset: usize,
        bytes: Vec<u8>,
    },
    ArgumentWordWrite {
        offset: usize,
        value: i16,
    },
    LoadSpell {
        coordinates: [i8; 3],
        source_address: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellUiQualification {
    SourcePortWithObservedToolboxAndSane,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellUiResult {
    pub state: SpellUiState,
    pub effects: Vec<SpellUiEffect>,
    /// spellinfoupdate is void and does not assign a native return register.
    pub native_return: Option<i32>,
    pub qualification: SpellUiQualification,
}

struct Execution {
    state: SpellUiState,
    effects: Vec<SpellUiEffect>,
}

impl Execution {
    fn new(state: &SpellUiState) -> Self {
        Self {
            state: state.clone(),
            effects: Vec::new(),
        }
    }

    fn finish(self, native_return: Option<i32>) -> SpellUiResult {
        SpellUiResult {
            state: self.state,
            effects: self.effects,
            native_return,
            qualification: SpellUiQualification::SourcePortWithObservedToolboxAndSane,
        }
    }

    fn bytes(&self, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        (0..length)
            .map(|i| {
                self.state
                    .memory
                    .byte(address.wrapping_add(i as u32))
                    .map_err(|_| unavailable("reached original memory byte is unavailable"))
            })
            .collect()
    }

    fn byte(&self, address: u32) -> Result<u8, RuleError> {
        Ok(self.bytes(address, 1)?[0])
    }
    fn signed_byte(&self, address: u32) -> Result<i16, RuleError> {
        Ok(i16::from(self.byte(address)? as i8))
    }
    fn word(&self, address: u32) -> Result<i16, RuleError> {
        let bytes = self.bytes(address, 2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }
    fn long(&self, address: u32) -> Result<u32, RuleError> {
        Ok(u32::from_be_bytes(
            self.bytes(address, 4)?.try_into().unwrap(),
        ))
    }
    fn rect(&self, address: u32) -> Result<BodyRect, RuleError> {
        Ok(BodyRect {
            top: self.word(address)?,
            left: self.word(address + 2)?,
            bottom: self.word(address + 4)?,
            right: self.word(address + 6)?,
        })
    }
    fn store(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(address, bytes.len())?;
        self.state.memory.insert_region(address, bytes);
        Ok(())
    }
    fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.store(address, bytes)?;
        self.effects.push(SpellUiEffect::Write(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
    fn word_write(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    fn long_write(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    fn draw(&mut self, effect: DrawingEffect) {
        self.effects.push(SpellUiEffect::Drawing(effect));
    }
    fn pascal(&self, address: u32) -> Result<PascalString, RuleError> {
        PascalString::from_raw(&self.bytes(address, usize::from(self.byte(address)?) + 1)?)
    }
    fn scratch(&mut self, text: &PascalString) -> Result<(), RuleError> {
        self.write(0x479ed8, text.raw())
    }

    fn dialog_item(
        &mut self,
        item: i16,
        rect_address: u32,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        let dialog = self.long(0x4394b8)?;
        let observation = services.dialog_item(dialog, item)?;
        self.store(0x4434b0, &observation.item_type.to_be_bytes())?;
        self.store(0x4434dc, &observation.handle.to_be_bytes())?;
        self.store(rect_address, &rect_bytes(observation.rect))?;
        self.effects.push(SpellUiEffect::DialogItem {
            dialog,
            item,
            rect_address,
            observation,
        });
        Ok(())
    }

    fn dialog_number(
        &mut self,
        item: i16,
        value: i16,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.dialog_item(item, 0x4434ba, services)?;
        let mut text = TextContext::default();
        for effect in recovered_text::dialog_number(&mut text, item, value, services)? {
            match effect {
                TextEffect::GetDialogItem { .. } => {}
                TextEffect::SetDialogItemText { item, text } => {
                    self.effects.push(SpellUiEffect::SetDialogItemText {
                        item,
                        handle: self.long(0x4434dc)?,
                        text,
                    });
                }
                effect @ TextEffect::NumberToString(_) => {
                    self.effects.push(SpellUiEffect::Text(effect));
                    self.scratch(&text.scratch)?;
                }
                effect => self.effects.push(SpellUiEffect::Text(effect)),
            }
        }
        Ok(())
    }

    fn dialog_string(
        &mut self,
        item: i16,
        address: u32,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.dialog_item(item, 0x4434ba, services)?;
        let text = self.pascal(address)?;
        for effect in recovered_text::dialog_string(item, &text) {
            if let TextEffect::SetDialogItemText { item, text } = effect {
                self.effects.push(SpellUiEffect::SetDialogItemText {
                    item,
                    handle: self.long(0x4434dc)?,
                    text,
                });
            }
        }
        Ok(())
    }

    fn number(
        &mut self,
        value: i16,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        let mut text = TextContext::default();
        for effect in recovered_text::string(&mut text, value, services)? {
            if matches!(effect, TextEffect::NumberToString(_)) {
                self.effects.push(SpellUiEffect::Text(effect));
                self.scratch(&text.scratch)?;
            } else {
                self.effects.push(SpellUiEffect::Text(effect));
            }
        }
        Ok(())
    }

    fn indexed_string(
        &mut self,
        resource_id: i16,
        index: i16,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.draw(DrawingEffect::GetIndexedString { resource_id, index });
        let text = services.indexed_string(resource_id, index)?;
        self.scratch(&text)
    }

    fn rgb_forecolor(&mut self, address: u32) -> Result<(), RuleError> {
        self.effects.push(SpellUiEffect::RgbForeColor {
            address,
            components: [
                self.word(address)? as u16,
                self.word(address + 2)? as u16,
                self.word(address + 4)? as u16,
            ],
        });
        Ok(())
    }

    fn offset_rect(
        &mut self,
        address: u32,
        horizontal: i16,
        vertical: i16,
    ) -> Result<(), RuleError> {
        let before = self.rect(address)?;
        let after = BodyRect {
            top: before.top.wrapping_add(vertical),
            left: before.left.wrapping_add(horizontal),
            bottom: before.bottom.wrapping_add(vertical),
            right: before.right.wrapping_add(horizontal),
        };
        self.store(address, &rect_bytes(after))?;
        self.effects.push(SpellUiEffect::OffsetRect {
            address,
            horizontal,
            vertical,
            before,
            after,
        });
        Ok(())
    }

    fn copy_bits(
        &mut self,
        source_port: u32,
        destination_port: u32,
        source: BodyRect,
        destination: BodyRect,
    ) -> Result<(), RuleError> {
        self.draw(DrawingEffect::CopyBits {
            source_bitmap: self.long(source_port)?.wrapping_add(2),
            destination_bitmap: self.long(destination_port)?.wrapping_add(2),
            source,
            destination,
            mode: 0,
            mask: 0,
        });
        Ok(())
    }

    fn load_icon(
        &mut self,
        id: i16,
        clear: bool,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<u32, RuleError> {
        if clear {
            self.long_write(0x439598, 0)?;
        }
        self.draw(DrawingEffect::GetColorIcon(id));
        let handle = services.color_icon(id)?;
        self.state.drawing.icon_handle = handle;
        self.long_write(0x439598, handle)?;
        Ok(handle)
    }

    fn plot_icon(
        &mut self,
        id: i16,
        rect: BodyRect,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        let child = recovered_drawing::ploticon3(&mut self.state.drawing, id, rect, services)?;
        for effect in child.effects {
            if let DrawingEffect::Write(ref write) = effect {
                self.store(write.address, &write.bytes)?;
            }
            self.draw(effect);
        }
        Ok(())
    }

    fn direct_icon(
        &mut self,
        id: i16,
        rect: BodyRect,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        let handle = self.load_icon(id, false, services)?;
        self.draw(DrawingEffect::PlotColorIcon { handle, rect });
        self.draw(DrawingEffect::DisposeColorIcon(handle));
        Ok(())
    }

    fn button(
        &mut self,
        pressed: bool,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        let input = ButtonInput {
            rect: self.rect(0x44292c)?,
            target_port: 0,
            dark_pattern: self.long(0x4358de)?,
            light_pattern: self.long(0x4358e2)?,
        };
        let child = if pressed {
            recovered_interface::downbutton(input, 0, services)?
        } else {
            recovered_interface::upbutton(input, 0, services)?
        };
        self.effects
            .extend(child.effects.into_iter().map(SpellUiEffect::Interface));
        Ok(())
    }

    fn overlay(&mut self, services: &mut (impl SpellUiServices + ?Sized)) -> Result<(), RuleError> {
        self.word_write(0x4434be, self.word(0x4434be)?.wrapping_sub(32))?;
        self.word_write(0x4434c0, self.word(0x4434c0)?.wrapping_sub(32))?;
        // FFFB0000 packs the final vertical WORD -5 above horizontal WORD zero.
        self.offset_rect(0x4434ba, 0, -5)?;
        self.direct_icon(14022, self.rect(0x4434ba)?, services)
    }

    fn truncate_distance(
        &mut self,
        services: &mut (impl SpellUiServices + ?Sized),
    ) -> Result<i16, RuleError> {
        let input = self.bytes(0x44349c, 10)?.try_into().unwrap();
        let output = services.truncate_extended(input)?;
        self.effects
            .push(SpellUiEffect::TruncateExtended { input, output });
        Ok(output as i16)
    }
}

fn rect_bytes(rect: BodyRect) -> [u8; 8] {
    let mut bytes = [0; 8];
    for (index, value) in [rect.top, rect.left, rect.bottom, rect.right]
        .into_iter()
        .enumerate()
    {
        bytes[index * 2..index * 2 + 2].copy_from_slice(&value.to_be_bytes());
    }
    bytes
}

fn icon_id(execution: &Execution) -> Result<i16, RuleError> {
    Ok(match execution.byte(0x44374f)? {
        3 => i16::from(execution.byte(0x44376d)?).wrapping_add(9999),
        4 => execution.word(0x4435cc)?.wrapping_add(9999),
        5 => {
            let extra = i16::from(execution.byte(0x44376d)?);
            if extra == 0 {
                10215
            } else {
                extra.wrapping_add(9999)
            }
        }
        6 => 10213,
        7 => 10214,
        9 => 10211,
        10 => 10210,
        11 => 10212,
        _ => 10000,
    })
}

fn positive_power(execution: &Execution) -> Result<bool, RuleError> {
    Ok(execution
        .signed_byte(0x443751)?
        .wrapping_add(execution.signed_byte(0x443750)?)
        > 0)
}

/// showspelllist, 0x10b6ba: every one of the seven button/label branches executes.
pub fn showspelllist(
    state: &SpellUiState,
    services: &mut (impl SpellUiServices + ?Sized),
) -> Result<SpellUiResult, RuleError> {
    let mut execution = Execution::new(state);
    let member = execution.signed_byte(0x44273f)?;
    let warrior = 0x448aa0_u32.wrapping_add(u32::from(member as u16) * 3138);
    let school = execution.word(warrior.wrapping_add(0x36))?.wrapping_sub(1);
    for slot in 0..7_i16 {
        execution.dialog_item(slot + 58, 0x44292c, services)?;
        let tier = execution.word(0x4435c8)?;
        let current_member = execution.signed_byte(0x44273f)?;
        let learned = 0x448aa0_u32
            .wrapping_add(u32::from(current_member as u16) * 3138)
            .wrapping_add(u32::from(tier as u16) * 7)
            .wrapping_add(slot as u32)
            .wrapping_add(0x9f0);
        if execution.byte(learned)? == 0 {
            execution.button(true, services)?;
            execution.dialog_string(slot + 1, 0x403f22, services)?;
            continue;
        }
        let coordinates = [school as i8, tier as i8, slot as i8];
        let source = 0x443788_u32
            .wrapping_add(u32::from(i16::from(coordinates[0]) as u16) * 2842)
            .wrapping_add(u32::from(i16::from(coordinates[1]) as u16) * 406)
            .wrapping_add(u32::from(i16::from(coordinates[2]) as u16) * SPELL_BYTES as u32);
        let bytes = execution.bytes(source, SPELL_BYTES)?;
        execution.effects.push(SpellUiEffect::LoadSpell {
            coordinates,
            source_address: source,
        });
        // Rebase the reached source record onto the shared loader's copy domain.
        for write in recovered_records::loadspell(&bytes, 0, 0, 0)?.writes {
            execution.write(write.address, &write.bytes)?;
        }
        let resource_id = school
            .wrapping_add(1)
            .wrapping_mul(1000)
            .wrapping_add(execution.word(0x4435c8)?);
        execution.indexed_string(resource_id, slot + 1, services)?;
        let usable = (execution.word(0x438776)? != 0 && execution.byte(0x443763)? != 0)
            || (execution.word(0x438776)? == 0 && execution.byte(0x443764)? != 0)
            || execution.word(0x447198)? != 0;
        execution.button(!usable, services)?;
        if usable {
            execution.draw(DrawingEffect::ForeColor(69));
            execution.dialog_string(slot + 1, 0x479ed8, services)?;
        } else {
            execution.rgb_forecolor(0x443380)?;
            execution.dialog_string(slot + 1, 0x479ed8, services)?;
            execution.draw(DrawingEffect::ForeColor(69));
        }
    }
    Ok(execution.finish(Some(0)))
}

/// showspellinfo, 0x159192: item banner, spell mask, overlay, name and WORD count.
pub fn showspellinfo(
    state: &SpellUiState,
    services: &mut (impl SpellUiServices + ?Sized),
) -> Result<SpellUiResult, RuleError> {
    let mut execution = Execution::new(state);
    execution.draw(DrawingEffect::SetPort(execution.long(0x43878c)?));
    if execution.byte(0x443761)? == 9 {
        let handle = execution.load_icon(execution.word(0x453772)?, true, services)?;
        let rect = execution.rect(0x4436f2)?;
        execution.draw(DrawingEffect::EraseRect(rect));
        if handle != 0 {
            execution.draw(DrawingEffect::PlotColorIcon { handle, rect });
            execution.draw(DrawingEffect::DisposeColorIcon(handle));
        }
    }
    execution.draw(DrawingEffect::TextSize(12));
    execution.long_write(0x443702, execution.long(0x4436fa)?)?;
    execution.long_write(0x443706, execution.long(0x4436fe)?)?;
    let mut id = icon_id(&execution)?;
    if execution.word(0x443776)? == 58 {
        id = i16::from(execution.byte(0x44376d)?).wrapping_add(10013);
    }
    execution.draw(DrawingEffect::BackColor(30));
    let handle = execution.load_icon(id, true, services)?;
    if handle != 0 {
        execution.draw(DrawingEffect::PlotColorIcon {
            handle,
            rect: execution.rect(0x4436fa)?,
        });
        let pointer = execution.long(execution.long(0x439598)?)?;
        let source_bitmap = pointer.wrapping_add(64);
        let destination_bitmap = execution.long(0x4386b8)?.wrapping_add(2);
        let pointer = execution.long(execution.long(0x439598)?)?;
        let source = execution.rect(pointer.wrapping_add(6))?;
        let destination = execution.rect(0x439d0c)?;
        execution.draw(DrawingEffect::CopyBits {
            source_bitmap,
            destination_bitmap,
            source,
            destination,
            mode: 0,
            mask: 0,
        });
        execution.draw(DrawingEffect::DisposeColorIcon(execution.long(0x439598)?));
    }
    if positive_power(&execution)? {
        execution.long_write(0x4434ba, execution.long(0x4436fa)?)?;
        execution.long_write(0x4434be, execution.long(0x4436fe)?)?;
        execution.overlay(services)?;
    }
    execution.draw(DrawingEffect::ForeColor(69));
    let vertical = execution
        .word(0x40856e)?
        .wrapping_add(402)
        .wrapping_sub(execution.word(0x439582)?.wrapping_mul(32));
    execution.draw(DrawingEffect::MoveTo {
        horizontal: execution.word(0x40856c)?.wrapping_add(369),
        vertical,
    });
    execution.word_write(0x44719a, 0)?;
    if execution.byte(0x443761)? == 9 {
        let selection = crate::rules::selection_base(execution.word(0x4537ce)?);
        execution.word_write(0x4471e8, selection)?;
        execution.indexed_string(
            selection.wrapping_add(1),
            execution
                .word(0x4536a2)?
                .wrapping_sub(execution.word(0x4471e8)?)
                .wrapping_add(1),
            services,
        )?;
    } else {
        execution.indexed_string(
            execution
                .word(0x4435c6)?
                .wrapping_add(1)
                .wrapping_mul(1000)
                .wrapping_add(execution.word(0x4435c8)?),
            execution.word(0x4435ca)?.wrapping_add(1),
            services,
        )?;
    }
    execution.draw(DrawingEffect::DrawString(execution.pascal(0x479ed8)?));
    execution.effects.push(SpellUiEffect::TextMode(1));
    execution.draw(DrawingEffect::ForeColor(205));
    let vertical = execution
        .word(0x40856e)?
        .wrapping_add(386)
        .wrapping_sub(execution.word(0x439582)?.wrapping_mul(32));
    execution.draw(DrawingEffect::MoveTo {
        horizontal: execution.word(0x40856c)?.wrapping_add(548),
        vertical,
    });
    let amount = execution.word(if execution.byte(0x443761)? == 9 {
        0x4435be
    } else {
        0x4435cc
    })?;
    execution.number(amount, services)?;
    Ok(execution.finish(Some(0)))
}

/// spellinfoupdate, 0x15947c: complete dialog fields, type legend and icon branches.
pub fn spellinfoupdate(
    state: &SpellUiState,
    services: &mut (impl SpellUiServices + ?Sized),
) -> Result<SpellUiResult, RuleError> {
    let mut execution = Execution::new(state);
    execution.draw(DrawingEffect::ForeColor(69));
    for (modifier, base, item) in [(0x443759, 0x44376e, 13), (0x44375a, 0x443770, 50)] {
        let value = execution
            .signed_byte(modifier)?
            .wrapping_mul(execution.word(0x4435cc)?)
            .wrapping_add(execution.word(base)?);
        execution.dialog_number(item, value, services)?;
    }
    for (modifier, base, item) in [
        (0x44375d, 0x44375b, 14),
        (0x44375e, 0x44375c, 15),
        (0x443751, 0x443750, 18),
    ] {
        let value = execution
            .signed_byte(modifier)?
            .wrapping_mul(execution.word(0x4435cc)?)
            .wrapping_add(execution.signed_byte(base)?)
            .wrapping_abs();
        execution.dialog_number(item, value, services)?;
    }
    let quantity = if execution.signed_byte(0x44374f)? < 1 {
        execution.word(0x4435cc)?
    } else {
        1
    };
    execution.dialog_number(19, quantity, services)?;
    let damage_label = match execution.byte(0x443762)? {
        0 => 0x400c38,
        1 => 0x400c2e,
        2 => 0x400c28,
        3 => 0x400c22,
        4 => 0x400c18,
        5 => 0x400c10,
        6 => 0x400c08,
        7 => 0x400bfe,
        8 => 0x400bf6,
        9 => 0x400c42,
        10 => 0x400bee,
        _ => 0x400c42,
    };
    execution.dialog_string(52, damage_label, services)?;
    let duration_kind = execution.signed_byte(0x443757)?;
    if duration_kind == 1 || duration_kind > 2 {
        execution.dialog_item(65, 0x4434ba, services)?;
        execution.direct_icon(129, execution.rect(0x4434ba)?, services)?;
    } else if execution.byte(0x443758)? == 0 {
        execution.dialog_string(65, 0x400bed, services)?;
    } else {
        let value = execution
            .signed_byte(0x443758)?
            .wrapping_mul(execution.word(0x4435cc)?);
        execution.dialog_number(65, value, services)?;
    }
    if execution.signed_byte(0x443757)? > 1 {
        execution.dialog_item(66, 0x4434ba, services)?;
        execution.direct_icon(130, execution.rect(0x4434ba)?, services)?;
    } else if execution.byte(0x443756)? == 0 && execution.byte(0x443754)? == 0 {
        execution.dialog_string(66, 0x400bed, services)?;
    } else {
        let value = execution
            .signed_byte(0x443756)?
            .wrapping_mul(execution.word(0x4435cc)?)
            .wrapping_add(execution.signed_byte(0x443754)?);
        execution.dialog_number(66, value, services)?;
    }
    execution.dialog_number(54, execution.word(0x4435cc)?, services)?;
    execution.draw(DrawingEffect::ForeColor(69));
    if execution.word(0x443776)? != 0 && execution.byte(0x44374f)? != 7 {
        execution.indexed_string(134, execution.word(0x443776)?, services)?;
        let text = execution.pascal(0x479ed8)?;
        let width = services.string_width(&text)?;
        execution
            .effects
            .push(SpellUiEffect::StringWidth { text, width });
        if width == 0 {
            execution.dialog_string(67, 0x400bed, services)?;
        }
        execution.dialog_string(67, 0x479ed8, services)?;
    } else {
        let label = match execution.byte(0x44374f)? {
            0 => 0x400bd8,
            1 => 0x400bca,
            3 => 0x400bb8,
            4 => 0x400ba2,
            5 => 0x400b8e,
            6 => 0x400b7a,
            7 => 0x400b64,
            8 => 0x400b52,
            9 => 0x400b3c,
            10 => 0x400b28,
            11 => 0x400b18,
            _ => 0x400bed,
        };
        execution.dialog_string(67, label, services)?;
    }
    let id = icon_id(&execution)?;
    execution.dialog_item(16, 0x4434ba, services)?;
    execution.plot_icon(id, execution.rect(0x4434ba)?, services)?;
    if execution.word(0x438776)? != 0 && positive_power(&execution)? {
        execution.overlay(services)?;
    }
    Ok(execution.finish(None))
}

struct Positions<'a>(&'a RenderingMemory);

impl Positions<'_> {
    fn position(&self, base: u32, index: i16) -> Option<Position> {
        let address = base.wrapping_add((i32::from(index) * 4) as u32);
        Some((
            self.0.word(address).ok()? as i16,
            self.0.word(address.wrapping_add(2)).ok()? as i16,
        ))
    }
}

impl SpatialServices for Positions<'_> {
    fn warrior_position(&self, index: i16) -> Option<Position> {
        self.position(0x44709a, index)
    }
    fn enemy_position(&self, index: i16) -> Option<Position> {
        self.position(0x446f0a, index)
    }
    fn warrior_byte(&self, _: i16, _: usize) -> Option<u8> {
        None
    }
    fn warrior_word(&self, _: i16, _: usize) -> Option<i16> {
        None
    }
    fn enemy_byte(&self, _: i16, _: usize) -> Option<u8> {
        None
    }
    fn enemy_word(&self, _: i16, _: usize) -> Option<i16> {
        None
    }
    fn map_word(&self, _: i32, _: i32) -> Option<i16> {
        None
    }
}

// The reached squared distance is an exact nonnegative integer below 2^32.
fn squared_extended(value: f64) -> [u8; 10] {
    if value == 0.0 {
        return [0; 10];
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023 + 16383;
    let significand = ((1_u64 << 52) | (bits & 0x000f_ffff_ffff_ffff)) << 11;
    let mut bytes = [0; 10];
    bytes[..2].copy_from_slice(&(exponent as u16).to_be_bytes());
    bytes[2..].copy_from_slice(&significand.to_be_bytes());
    bytes
}

fn check_range(
    execution: &mut Execution,
    services: &mut (impl SpellUiServices + ?Sized),
) -> Result<(), RuleError> {
    let body = execution.word(0x4435d2)?;
    let target = (execution.word(0x44719e)?, execution.word(0x4471a0)?);
    let work = recovered_spatial::getrangetobody(body, target, &Positions(&execution.state.memory))
        .map_err(|_| unavailable("checkrange reached position WORDs are unavailable"))?;
    let squared = squared_extended(work.squared_distance);
    execution.write(0x44349c, &squared[..4])?;
    execution.write(0x4434a0, &squared[4..8])?;
    execution.write(0x4434a4, &squared[8..])?;
    let argument = execution.bytes(0x44349c, 10)?.try_into().unwrap();
    let output = services.square_root_extended(argument)?;
    execution.store(0x44349c, &output)?;
    execution.effects.push(SpellUiEffect::SquareRootExtended {
        input: argument,
        output,
    });
    let range_word = execution.truncate_distance(services)?;
    execution.word_write(0x4434b6, range_word)?;
    // The original compares this WORD with maximum 4435bc; caller discards the Boolean.
    let _within_range = range_word <= execution.word(0x4435bc)?;
    Ok(())
}

/// updatespell, 0x162084: range geometry is composed before the redraw gate.
pub fn updatespell(
    state: &SpellUiState,
    force: i8,
    services: &mut (impl SpellUiServices + ?Sized),
) -> Result<SpellUiResult, RuleError> {
    let mut execution = Execution::new(state);
    check_range(&mut execution, services)?;
    let distance = execution.truncate_distance(services)?;
    if distance != execution.word(0x4042c0)? || force != 0 {
        execution.draw(DrawingEffect::SetPort(execution.long(0x43878c)?));
        execution.draw(DrawingEffect::TextSize(17));
        execution.draw(DrawingEffect::ForeColor(
            if distance > execution.word(0x4435bc)? {
                205
            } else {
                69
            },
        ));
        let shift = execution.word(0x439582)?.wrapping_mul(32);
        let width = execution.word(0x40856c)?;
        let height = execution.word(0x40856e)?;
        let rect = BodyRect {
            top: height.wrapping_add(427).wrapping_sub(shift),
            left: width.wrapping_add(479),
            bottom: height.wrapping_add(447).wrapping_sub(shift),
            right: width.wrapping_add(504),
        };
        execution.draw(DrawingEffect::MoveTo {
            horizontal: width.wrapping_add(480),
            vertical: height.wrapping_add(445).wrapping_sub(shift),
        });
        execution.draw(DrawingEffect::EraseRect(rect));
        execution.number(distance, services)?;
        execution.rgb_forecolor(0x443374)?;
        execution.draw(DrawingEffect::MoveTo {
            horizontal: execution.word(0x40856c)?.wrapping_add(605),
            vertical: execution
                .word(0x40856e)?
                .wrapping_add(445)
                .wrapping_sub(execution.word(0x439582)?.wrapping_mul(32)),
        });
        execution.number(execution.word(0x4435bc)?, services)?;
        let id = execution
            .word(0x4435be)?
            .wrapping_add(147)
            .wrapping_sub(execution.word(0x4435c0)?);
        let handle = services.color_cursor(id)?;
        execution
            .effects
            .push(SpellUiEffect::GetColorCursor { id, handle });
        execution
            .effects
            .push(SpellUiEffect::SetColorCursor(handle));
        execution
            .effects
            .push(SpellUiEffect::DisposeColorCursor(handle));
        execution.draw(DrawingEffect::SetPort(execution.long(0x43879c)?));
    }
    // Native rereads all ten global bytes; it does not reuse the first local copy.
    let last_distance = execution.truncate_distance(services)?;
    execution.word_write(0x4042c0, last_distance)?;
    Ok(execution.finish(Some(0)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutdoorsSpellInput {
    pub animation: i16,
    pub horizontal: i16,
    pub vertical: i16,
    pub extent: i16,
    pub delay: i16,
}

/// spelleffectoutdoors, 0x15b84c: back up pixels, restore before each icon and once after.
pub fn spelleffectoutdoors(
    state: &SpellUiState,
    input: OutdoorsSpellInput,
    services: &mut (impl SpellUiServices + ?Sized),
) -> Result<SpellUiResult, RuleError> {
    let mut execution = Execution::new(state);
    for (offset, value) in [(2, 0_i16), (0, 0), (6, 64), (4, 64)] {
        execution.effects.push(SpellUiEffect::LocalWrite {
            offset,
            bytes: value.to_be_bytes().to_vec(),
        });
    }
    let first_id = input.animation.wrapping_shl(3).wrapping_add(12000);
    let horizontal =
        (i32::from(input.horizontal) as u32).wrapping_sub(execution.long(0x442778)?) as i16;
    execution.effects.push(SpellUiEffect::ArgumentWordWrite {
        offset: 10,
        value: horizontal,
    });
    let vertical =
        (i32::from(input.vertical) as u32).wrapping_sub(execution.long(0x44277c)?) as i16;
    execution.word_write(0x44371a, vertical.wrapping_shl(5))?;
    execution.word_write(0x44371c, horizontal.wrapping_shl(5))?;
    execution.word_write(
        0x44371e,
        execution.word(0x44371a)?.wrapping_add(input.extent),
    )?;
    execution.word_write(
        0x443720,
        execution.word(0x44371c)?.wrapping_add(input.extent),
    )?;
    let half = input
        .extent
        .wrapping_add(((input.extent as u16) >> 15) as i16)
        >> 1;
    execution.offset_rect(0x44371a, half.wrapping_neg(), half.wrapping_neg())?;
    execution.draw(DrawingEffect::SetPort(execution.long(0x43879c)?));
    let bottom = execution
        .word(0x44371e)?
        .wrapping_sub(execution.word(0x44371a)?);
    execution.effects.push(SpellUiEffect::LocalWrite {
        offset: 4,
        bytes: bottom.to_be_bytes().to_vec(),
    });
    let right = execution
        .word(0x443720)?
        .wrapping_sub(execution.word(0x44371c)?);
    execution.effects.push(SpellUiEffect::LocalWrite {
        offset: 6,
        bytes: right.to_be_bytes().to_vec(),
    });
    let backup = BodyRect {
        top: 0,
        left: 0,
        bottom,
        right,
    };
    execution.copy_bits(0x43879c, 0x4386b4, execution.rect(0x44371a)?, backup)?;
    let mut id = first_id;
    let end = first_id.wrapping_add(8);
    while id < end {
        execution.copy_bits(0x4386b4, 0x43879c, backup, execution.rect(0x44371a)?)?;
        execution.plot_icon(id, execution.rect(0x44371a)?, services)?;
        if input.delay != 0 {
            let child = recovered_timing::delay(input.delay, 0, services)?;
            execution
                .effects
                .extend(child.effects.into_iter().map(SpellUiEffect::Timing));
        }
        id = id.wrapping_add(1);
    }
    execution.copy_bits(0x4386b4, 0x43879c, backup, execution.rect(0x44371a)?)?;
    Ok(execution.finish(Some(0)))
}
