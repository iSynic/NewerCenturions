//! Character displays and specialty adjustment from ordered CODE 1 instructions.

use crate::{
    RuleError, party,
    recovered_files::{self, FileBuffer, FileContext, FileEffect, FileObservation},
    recovered_records::RecordWrite,
    recovered_rendering::RenderingMemory,
    recovered_text::{self, NumericServices, PascalString, TextContext, TextEffect},
    rules,
};
use centurions_content::{WARRIOR_BYTES, Warrior};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterUiState {
    pub memory: RenderingMemory,
    /// Explicit buffers used by the owned file helper; no host files are opened.
    pub files: FileContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterUiRequest {
    SetPort(u32),
    BackPixPat(u32),
    ForeColor(i32),
    TextSize(i16),
    TextFont(i16),
    RgbForeColor(u32),
    GetDialog {
        id: i16,
        storage: u32,
        behind: u32,
    },
    DrawControls(u32),
    DrawDialog(u32),
    BeginUpdate(u32),
    EndUpdate(u32),
    DisposeDialog(u32),
    FlushEvents(u32),
    ModalDialog {
        filter: u32,
        item_address: u32,
    },
    GetDialogItem {
        dialog: u32,
        item: i16,
        type_address: u32,
        handle_address: u32,
        rect_address: u32,
    },
    SetControlValue {
        handle: u32,
        value: i16,
    },
    SetDialogItemText {
        handle: u32,
        text_address: u32,
    },
    NumberToString {
        value: i32,
        destination: u32,
    },
    GetIndexedString {
        destination: u32,
        resource: i16,
        index: i16,
    },
    CopyCString {
        destination: u32,
        source: u32,
    },
    AppendCString {
        destination: u32,
        source: u32,
        count: u16,
    },
    CToPascal(u32),
    PascalToC(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterUiObservation {
    pub value: u32,
    /// Complete observed memory, retaining every supplied unrelated byte.
    pub memory: RenderingMemory,
}

pub trait CharacterUiServices {
    /// Toolbox and imported numeric/string calls only, never application actions.
    fn observe(
        &mut self,
        request: &CharacterUiRequest,
        memory: &RenderingMemory,
    ) -> Result<CharacterUiObservation, RuleError>;
    fn profile_observations(
        &mut self,
        race: i16,
        caste: i16,
        context: &FileContext,
    ) -> Result<Vec<FileObservation>, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterUiEffect {
    Write(RecordWrite),
    Service {
        request: CharacterUiRequest,
        observation: CharacterUiObservation,
    },
    Text(TextEffect),
    File(FileEffect),
    SpellCount {
        level: i16,
        result: i32,
    },
    Pin {
        value: i16,
        minimum: i16,
        maximum: i16,
        result: i16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterUiResult {
    pub state: CharacterUiState,
    pub effects: Vec<CharacterUiEffect>,
    pub native_return: Option<i32>,
    /// File services retain the existing helper's declared buffer projection.
    pub qualification: CharacterUiQualification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterUiQualification {
    SourcePortWithObservedToolboxAndProjectedFileServices,
}

struct Execution {
    state: CharacterUiState,
    effects: Vec<CharacterUiEffect>,
}

impl Execution {
    fn new(state: &CharacterUiState) -> Self {
        Self {
            state: state.clone(),
            effects: vec![],
        }
    }
    fn finish(self, native_return: Option<i32>) -> CharacterUiResult {
        CharacterUiResult {
            state: self.state,
            effects: self.effects,
            native_return,
            qualification:
                CharacterUiQualification::SourcePortWithObservedToolboxAndProjectedFileServices,
        }
    }
    fn bytes(&self, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        (0..length)
            .map(|i| {
                self.state
                    .memory
                    .byte(address.wrapping_add(i as u32))
                    .map_err(|_| RuleError::Unsupported {
                        action: "character_ui",
                        dependency: "reached original memory is unavailable",
                    })
            })
            .collect()
    }
    fn byte(&self, a: u32) -> Result<u8, RuleError> {
        Ok(self.bytes(a, 1)?[0])
    }
    fn word(&self, a: u32) -> Result<i16, RuleError> {
        Ok(i16::from_be_bytes(self.bytes(a, 2)?.try_into().unwrap()))
    }
    fn long(&self, a: u32) -> Result<u32, RuleError> {
        Ok(u32::from_be_bytes(self.bytes(a, 4)?.try_into().unwrap()))
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.state.memory.insert_region(a, bytes);
        self.effects.push(CharacterUiEffect::Write(RecordWrite {
            address: a,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
    fn ww(&mut self, a: u32, value: i16) -> Result<(), RuleError> {
        self.write(a, &value.to_be_bytes())
    }
    fn lw(&mut self, a: u32, value: u32) -> Result<(), RuleError> {
        self.write(a, &value.to_be_bytes())
    }
    fn service(
        &mut self,
        request: CharacterUiRequest,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<u32, RuleError> {
        let observation = services.observe(&request, &self.state.memory)?;
        self.state.memory = observation.memory.clone();
        let value = observation.value;
        self.effects.push(CharacterUiEffect::Service {
            request,
            observation,
        });
        Ok(value)
    }
    fn pascal(&self, a: u32) -> Result<PascalString, RuleError> {
        PascalString::from_raw(&self.bytes(a, usize::from(self.byte(a)?) + 1)?)
    }
    fn item(
        &mut self,
        item: i16,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.service(
            CharacterUiRequest::GetDialogItem {
                dialog: self.long(0x4394b8)?,
                item,
                type_address: 0x4434b0,
                handle_address: 0x4434dc,
                rect_address: 0x4434ba,
            },
            services,
        )?;
        Ok(())
    }
    fn number(
        &mut self,
        item: i16,
        value: i32,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.item(item, services)?;
        self.service(
            CharacterUiRequest::NumberToString {
                value,
                destination: 0x479ed8,
            },
            services,
        )?;
        let mut bridge = ObservedNumber(self.pascal(0x479ed8)?);
        let effects = recovered_text::dialog_number_long(
            &mut TextContext::default(),
            item,
            value,
            &mut bridge,
        )?;
        self.effects
            .extend(effects.into_iter().map(CharacterUiEffect::Text));
        self.service(
            CharacterUiRequest::SetDialogItemText {
                handle: self.long(0x4434dc)?,
                text_address: 0x479ed8,
            },
            services,
        )?;
        Ok(())
    }
    fn number_at(
        &mut self,
        item: i16,
        address: u32,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.number(item, i32::from(self.word(address)?), services)
    }
    fn string(
        &mut self,
        item: i16,
        address: u32,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.item(item, services)?;
        let effects = recovered_text::dialog_string(item, &self.pascal(address)?);
        self.effects
            .extend(effects.into_iter().map(CharacterUiEffect::Text));
        self.service(
            CharacterUiRequest::SetDialogItemText {
                handle: self.long(0x4434dc)?,
                text_address: address,
            },
            services,
        )?;
        Ok(())
    }
    fn indexed(
        &mut self,
        resource: i16,
        index: i16,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.service(
            CharacterUiRequest::GetIndexedString {
                destination: 0x479ed8,
                resource,
                index,
            },
            services,
        )?;
        Ok(())
    }
    fn summed(
        &mut self,
        item: i16,
        a: u32,
        b: u32,
        services: &mut (impl CharacterUiServices + ?Sized),
    ) -> Result<(), RuleError> {
        let value = self.word(a)?.wrapping_add(self.word(b)?);
        self.number(item, i32::from(value), services)
    }
    fn increment(&mut self, a: u32) -> Result<(), RuleError> {
        self.ww(a, self.word(a)?.wrapping_add(1))
    }
    fn pin(&mut self, a: u32, minimum: i16, maximum: i16) -> Result<(), RuleError> {
        let value = self.word(a)?;
        let result = rules::pin(value, minimum, maximum);
        self.effects.push(CharacterUiEffect::Pin {
            value,
            minimum,
            maximum,
            result,
        });
        self.ww(a, result)
    }
}

struct ObservedNumber(PascalString);
impl NumericServices for ObservedNumber {
    fn number_to_string(&mut self, _: i32) -> Result<PascalString, RuleError> {
        Ok(self.0.clone())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid(
            "unexpected inverse numeric conversion".into(),
        ))
    }
}

/// ShowStats, 0x159a96. The WORD flag is captured in D7 before services.
pub fn show_stats(
    state: &CharacterUiState,
    flag: i16,
    frame: u32,
    services: &mut (impl CharacterUiServices + ?Sized),
) -> Result<CharacterUiResult, RuleError> {
    let mut e = Execution::new(state);
    if e.word(frame.wrapping_add(8))? != flag {
        return Err(RuleError::Invalid(
            "ShowStats caller WORD disagrees with captured flag".into(),
        ));
    }
    e.lw(0x4394b8, e.long(0x4394bc)?)?;
    e.service(CharacterUiRequest::SetPort(e.long(0x4394bc)?), services)?;
    e.service(CharacterUiRequest::BackPixPat(e.long(0x4358ce)?), services)?;
    e.service(CharacterUiRequest::ForeColor(69), services)?;
    e.service(CharacterUiRequest::TextSize(12), services)?;
    if flag != 0 {
        e.service(CharacterUiRequest::TextFont(e.word(0x402f44)?), services)?;
        e.service(CharacterUiRequest::RgbForeColor(0x44337a), services)?;
        e.number(103, e.long(0x447c76)? as i32, services)?;
        e.number(104, e.long(0x447c62)? as i32, services)?;
        let mut total = 0u32;
        total = total.wrapping_add(
            e.long(0x447c66)?
                .wrapping_sub(e.long(0x447c6a)?.wrapping_mul(2)),
        );
        total = total.wrapping_add(
            e.long(0x447c72)?
                .wrapping_sub(e.long(0x447c6e)?.wrapping_mul(2)),
        );
        total = total.wrapping_add(
            e.long(0x447c8a)?
                .wrapping_mul(6)
                .wrapping_add(e.long(0x447c82)?.wrapping_mul((-3i32) as u32))
                .wrapping_add(e.long(0x447c86)?.wrapping_mul(4)),
        );
        total = total.wrapping_add(
            e.long(0x447c7a)?
                .wrapping_mul((-75i32) as u32)
                .wrapping_add(e.long(0x447c76)?.wrapping_mul(3))
                .wrapping_add(e.long(0x447c7e)?.wrapping_mul((-35i32) as u32)),
        );
        let difference = e.long(0x447c62)?.wrapping_sub(e.long(0x447c5e)?) as i32;
        total = total
            .wrapping_add((difference / 20) as u32)
            .wrapping_sub(e.long(0x447c8e)?);
        e.number(101, total as i32, services)?;
    }
    e.service(CharacterUiRequest::TextFont(e.word(0x402f3c)?), services)?;
    e.service(CharacterUiRequest::ForeColor(69), services)?;
    for (item, a) in [
        (19, 0x447bfe),
        (20, 0x447c00),
        (21, 0x447c02),
        (22, 0x447c04),
        (23, 0x447c06),
        (24, 0x447c08),
        (52, 0x447c0a),
    ] {
        e.number_at(item, a, services)?;
    }
    e.lw(0x4427b6, ((e.long(0x447be6)? as i32) / 365) as u32)?;
    e.number(71, e.long(0x4427b6)? as i32, services)?;
    let mut index = 0i16;
    while index <= i16::from(e.byte(0x44273a)? as i8) {
        let a = 0x448aa0u32.wrapping_add(u32::from(index as u16) * 3138 + 0x9da);
        // Only the carried WORD is projected into the owned credit primitive.
        let mut bytes = vec![0; WARRIOR_BYTES];
        bytes[0x9da..0x9dc].copy_from_slice(&e.bytes(a, 2)?);
        let mut warrior = Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes,
        };
        let mut pool = e.long(0x442788)? as i32;
        party::pool_credits(std::slice::from_mut(&mut warrior), &mut pool)?;
        e.lw(0x442788, pool as u32)?;
        e.write(a, &warrior.bytes[0x9da..0x9dc])?;
        index = index.wrapping_add(1);
    }
    if (e.long(0x442788)? as i32) > 9_999_999 {
        e.service(CharacterUiRequest::TextSize(10), services)?;
    }
    e.number(64, e.long(0x442788)? as i32, services)?;
    e.service(CharacterUiRequest::TextSize(12), services)?;
    e.indexed(131, e.word(0x447250)?, services)?;
    e.string(45, 0x479ed8, services)?;
    e.indexed(129, e.word(0x44724e)?, services)?;
    e.string(46, 0x479ed8, services)?;
    let gender = match e.word(0x447254)? {
        1 => 0x400b12,
        2 => 0x400b0a,
        _ => 0x400b04,
    };
    e.string(47, gender, services)?;
    for (item, a) in [(48, 0x44722e), (49, 0x447ce6), (25, 0x447ce2)] {
        e.number_at(item, a, services)?;
    }
    if e.word(0x447244)? > 999 {
        e.service(CharacterUiRequest::TextSize(10), services)?;
    }
    e.service(CharacterUiRequest::TextSize(12), services)?;
    for (item, a) in [(113, 0x447242), (106, 0x447244)] {
        e.number_at(item, a, services)?;
    }
    e.number(81, e.long(0x447bea)? as i32, services)?;
    e.number(83, e.long(0x447bee)? as i32, services)?;
    for (item, a) in [
        (34, 0x447240),
        (35, 0x447246),
        (36, 0x447248),
        (32, 0x447256),
        (50, 0x447bf2),
        (51, 0x447bf4),
    ] {
        e.number_at(item, a, services)?;
    }
    if e.word(0x447276)? != 0 {
        let level = e.word(0x447256)?;
        let result = rules::spell_count(level);
        e.effects
            .push(CharacterUiEffect::SpellCount { level, result });
        e.number(33, i32::from(result as i16), services)?;
    } else {
        e.string(33, 0x400afe, services)?;
    }
    e.service(CharacterUiRequest::CToPascal(0x447c3d), services)?;
    e.string(67, 0x447c3d, services)?;
    e.service(CharacterUiRequest::PascalToC(0x447c3d), services)?;
    e.number_at(108, 0x44722a, services)?;
    for (item, a, b) in [
        (109, 0x447228, 0x447d96),
        (110, 0x447220, 0x447d74),
        (111, 0x447222, 0x447d76),
        (54, 0x447224, 0x447d78),
        (55, 0x447226, 0x447d7a),
    ] {
        e.summed(item, a, b, services)?;
    }
    if e.word(0x447252)? != 0 && e.word(0x447276)? != 0 {
        if e.word(0x447276)? > 999 {
            e.service(CharacterUiRequest::TextSize(10), services)?;
        }
        e.number_at(112, 0x447274, services)?;
        e.number_at(58, 0x447276, services)?;
    } else {
        e.string(112, 0x400afc, services)?;
        e.string(58, 0x400afc, services)?;
    }
    e.service(CharacterUiRequest::TextSize(12), services)?;
    if e.word(0x44726e)? > 999 {
        e.service(CharacterUiRequest::TextSize(10), services)?;
    }
    e.number_at(56, 0x44726c, services)?;
    e.number_at(57, 0x44726e, services)?;
    e.summed(59, 0x44723e, 0x447d7c, services)?;
    e.number_at(26, 0x44725a, services)?;
    e.service(CharacterUiRequest::TextSize(12), services)?;
    for i in 0..8 {
        e.summed(37 + i as i16, 0x44732e + 2 * i, 0x447d64 + 2 * i, services)?;
    }
    if flag == 0 {
        for i in 0..8 {
            e.summed(117 + i as i16, 0x447310 + 2 * i, 0x447d7e + 2 * i, services)?;
        }
    }
    for item in 60..64 {
        e.string(item, 0x400afc, services)?;
    }
    let local = frame.wrapping_sub(50);
    let mut shown = 0;
    for i in 0..12 {
        let a = 0x4472ca + 2 * i;
        if e.word(a)? == 0 {
            continue;
        }
        e.service(
            CharacterUiRequest::CopyCString {
                destination: local,
                source: 0x400afc,
            },
            services,
        )?;
        let sign = if e.word(a)? > 0 { 0x400afa } else { 0x400af9 };
        e.service(
            CharacterUiRequest::CopyCString {
                destination: local,
                source: sign,
            },
            services,
        )?;
        e.service(
            CharacterUiRequest::NumberToString {
                value: i32::from(e.word(a)?),
                destination: 0x479ed8,
            },
            services,
        )?;
        e.service(CharacterUiRequest::PascalToC(0x479ed8), services)?;
        e.service(
            CharacterUiRequest::AppendCString {
                destination: local,
                source: 0x479ed8,
                count: 3,
            },
            services,
        )?;
        e.indexed(132, i as i16 + 1, services)?;
        e.service(CharacterUiRequest::PascalToC(0x479ed8), services)?;
        e.service(
            CharacterUiRequest::AppendCString {
                destination: local,
                source: 0x479ed8,
                count: 45,
            },
            services,
        )?;
        e.service(CharacterUiRequest::CToPascal(local), services)?;
        if shown < 4 {
            e.string(60 + shown, local, services)?;
            shown += 1;
        }
    }
    for item in 27..32 {
        e.string(item, 0x400afc, services)?;
    }
    let mut shown = 0;
    for i in 0..40 {
        if e.word(0x44727a + 2 * i)? != 0 {
            e.indexed(133, i as i16 + 1, services)?;
            e.string(27 + shown, 0x479ed8, services)?;
            shown += 1;
            if shown == 5 {
                break;
            }
        }
    }
    Ok(e.finish(Some(0)))
}

/// showupgrade, 0x15a49a. Member WORD is unsigned for the 3138-byte stride.
pub fn show_upgrade(
    state: &CharacterUiState,
    member: u16,
    services: &mut (impl CharacterUiServices + ?Sized),
) -> Result<CharacterUiResult, RuleError> {
    let mut e = Execution::new(state);
    let base = 0x448aa0 + u32::from(member) * 3138;
    e.number(43, e.long(base + 0x9d2)? as i32, services)?;
    e.service(CharacterUiRequest::CToPascal(base + 0xa21), services)?;
    e.string(2, base + 0xa21, services)?;
    e.service(CharacterUiRequest::PascalToC(base + 0xa21), services)?;
    for i in 0..7 {
        e.number_at(5 + i as i16, base + 0x9e2 + 2 * i, services)?;
    }
    e.number_at(12, base + 0x9d8, services)?;
    for i in 0..8 {
        e.summed(
            13 + i as i16,
            base + 0x112 + 2 * i,
            base + 0xb48 + 2 * i,
            services,
        )?;
    }
    for i in 0..4 {
        e.summed(
            21 + i as i16,
            base + 4 + 2 * i,
            base + 0xb58 + 2 * i,
            services,
        )?;
    }
    e.summed(25, base + 0xb42, base + 0xb46, services)?;
    if e.word(base + 0x36)? == 0 {
        e.string(26, 0x400af8, services)?;
    } else {
        e.summed(26, base + 0xb40, base + 0xb44, services)?;
    }
    e.summed(27, base + 0x22, base + 0xb60, services)?;
    e.ww(0x4434aa, 0)?;
    let mut budget = 0;
    while e.word(0x4434aa)? < 9 {
        guard(&mut budget)?;
        let offset = (i32::from(e.word(0x4434aa)?) * 2) as u32;
        let item = e.word(0x4434aa)?.wrapping_add(28);
        if e.word(base.wrapping_add(0xf4).wrapping_add(offset))? > 0 {
            e.summed(
                item,
                base.wrapping_add(0xf4).wrapping_add(offset),
                base.wrapping_add(0xb62).wrapping_add(offset),
                services,
            )?;
        } else {
            e.string(item, 0x400af8, services)?;
        }
        e.increment(0x4434aa)?;
    }
    e.summed(37, base + 12, base + 0xb7a, services)?;
    Ok(e.finish(Some(0)))
}

fn guard(budget: &mut u32) -> Result<(), RuleError> {
    *budget += 1;
    if *budget > 65536 {
        return Err(RuleError::Unsupported {
            action: "character_ui",
            dependency: "observed memory produced a nonterminating WORD loop",
        });
    }
    Ok(())
}

/// viewrestrictions, 0x148f9e. The returned dialog handle remains in A2.
pub fn view_restrictions(
    state: &CharacterUiState,
    services: &mut (impl CharacterUiServices + ?Sized),
) -> Result<CharacterUiResult, RuleError> {
    let mut e = Execution::new(state);
    let dialog = e.service(
        CharacterUiRequest::GetDialog {
            id: 0xe105u16 as i16,
            storage: 0,
            behind: u32::MAX,
        },
        services,
    )?;
    e.service(CharacterUiRequest::SetPort(dialog), services)?;
    e.lw(0x4394b8, dialog)?;
    e.write(0x4427b0, &[0])?;
    e.service(CharacterUiRequest::DrawDialog(dialog), services)?;
    e.service(CharacterUiRequest::DrawControls(dialog), services)?;
    e.string(4, 0x41f71a, services)?;
    e.number_at(65, 0x41f81a, services)?;
    e.number_at(66, 0x41f81c, services)?;
    e.ww(0x4434aa, 0)?;
    let mut budget = 0;
    while e.word(0x4434aa)? < 30 {
        guard(&mut budget)?;
        e.item(e.word(0x4434aa)?.wrapping_add(5), services)?;
        let value =
            i16::from(e.byte(0x41f81eu32.wrapping_add(i32::from(e.word(0x4434aa)?) as u32))? as i8);
        e.service(
            CharacterUiRequest::SetControlValue {
                handle: e.long(0x4434dc)?,
                value,
            },
            services,
        )?;
        e.item(e.word(0x4434aa)?.wrapping_add(35), services)?;
        let value =
            i16::from(e.byte(0x41f83cu32.wrapping_add(i32::from(e.word(0x4434aa)?) as u32))? as i8);
        e.service(
            CharacterUiRequest::SetControlValue {
                handle: e.long(0x4434dc)?,
                value,
            },
            services,
        )?;
        e.indexed(129, e.word(0x4434aa)?.wrapping_add(1), services)?;
        e.string(e.word(0x4434aa)?.wrapping_add(67), 0x479ed8, services)?;
        e.indexed(131, e.word(0x4434aa)?.wrapping_add(1), services)?;
        e.string(e.word(0x4434aa)?.wrapping_add(97), 0x479ed8, services)?;
        e.increment(0x4434aa)?;
    }
    e.service(CharacterUiRequest::BeginUpdate(dialog), services)?;
    e.service(CharacterUiRequest::EndUpdate(dialog), services)?;
    e.service(CharacterUiRequest::FlushEvents(0xffff), services)?;
    e.service(
        CharacterUiRequest::ModalDialog {
            filter: 0,
            item_address: 0x4434b2,
        },
        services,
    )?;
    e.service(CharacterUiRequest::DisposeDialog(dialog), services)?;
    Ok(e.finish(Some(0)))
}

fn file_address(buffer: FileBuffer) -> Result<u32, RuleError> {
    match buffer {
        FileBuffer::Filename => Ok(0x4389a4),
        FileBuffer::RaceProfile => Ok(0x41f85a),
        FileBuffer::CasteProfile => Ok(0x41f9f2),
        _ => Err(RuleError::Invalid(
            "unexpected profile helper buffer".into(),
        )),
    }
}

/// updatespec, 0x161d86. Actual table copies use consecutive LONGs.
pub fn update_specialties(
    state: &CharacterUiState,
    frame: u32,
    services: &mut (impl CharacterUiServices + ?Sized),
) -> Result<CharacterUiResult, RuleError> {
    let mut e = Execution::new(state);
    for (source, offset, count) in [
        (0x4008a4u32, 456u32, 114u32),
        (0x400874, 1080, 12),
        (0x400634, 1032, 144),
    ] {
        for i in 0..count {
            let bytes = e.bytes(source + 4 * i, 4)?;
            e.write(frame.wrapping_sub(offset).wrapping_add(4 * i), &bytes)?;
        }
    }
    let caste = e.word(0x447250)?;
    let race = e.word(0x44724e)?;
    if race != 0 || caste != 0 {
        let mut directory = Vec::new();
        for i in 0..65536u32 {
            let byte = e.byte(0x44260c + i)?;
            directory.push(byte);
            if byte == 0 {
                break;
            }
        }
        if directory.last() != Some(&0) {
            return Err(RuleError::Unsupported {
                action: "character_ui",
                dependency: "unterminated supplied scenario directory",
            });
        }
        e.state.files.scenario_directory = directory;
        e.state.files.variant = e.byte(0x44274c)?;
        e.state.files.file_handle = e.long(0x4434a6)?;
        e.state.files.filename = e.bytes(0x4389a4, e.state.files.filename.len())?;
        if race != 0 {
            e.state.files.race_profile = e.bytes(0x41f85a, e.state.files.race_profile.len())?;
        }
        if caste != 0 {
            e.state.files.caste_profile = e.bytes(0x41f9f2, e.state.files.caste_profile.len())?;
        }
    }
    let observations = services.profile_observations(race, caste, &e.state.files)?;
    let plan = recovered_files::load_profile(&mut e.state.files, race, caste, &observations)?;
    for effect in plan.effects {
        match &effect {
            FileEffect::Write {
                destination,
                offset,
                bytes,
            } => {
                e.write(
                    file_address(*destination)?.wrapping_add(*offset as u32),
                    bytes,
                )?;
            }
            FileEffect::FileHandle(value) => {
                e.lw(0x4434a6, *value)?;
            }
            _ => {}
        }
        e.effects.push(CharacterUiEffect::File(effect));
    }
    if e.word(frame.wrapping_add(8))? == 1 {
        e.ww(0x4434aa, 0)?;
        while e.word(0x4434aa)? < 15 {
            let i = i32::from(e.word(0x4434aa)?) as u32;
            let value = if e.word(0x41f9f2u32.wrapping_add(2 * i))? != 0 {
                e.word(0x41f86au32.wrapping_add(2 * i))?
                    .wrapping_add(e.word(0x41f9f2u32.wrapping_add(2 * i))?)
            } else {
                0
            };
            e.ww(0x447310u32.wrapping_add(2 * i), value)?;
            e.increment(0x4434aa)?;
        }
        e.ww(0x4434ac, 0)?;
        while e.word(0x4434ac)? < 24 {
            let row = e.word(0x4434ac)?;
            if e.word(0x447bfe)?
                == e.word(
                    frame
                        .wrapping_sub(1080)
                        .wrapping_add((i32::from(row) * 2) as u32),
                )?
            {
                add_specialty_row(&mut e, frame.wrapping_sub(1032), 48)?;
            }
            e.increment(0x4434ac)?;
        }
        e.ww(0x4434ac, 0)?;
        while e.word(0x4434ac)? < 19 {
            let value = e.word(0x447c04)?.wrapping_add(e.word(0x447c06)?) / 2;
            e.ww(0x4434b6, value)?;
            let row = e.word(0x4434ac)?;
            if e.word(0x4434b6)?
                == e.word(
                    frame
                        .wrapping_sub(1080)
                        .wrapping_add((i32::from(row) * 2) as u32),
                )?
            {
                add_specialty_row(&mut e, frame.wrapping_sub(456), 38)?;
            }
            e.increment(0x4434ac)?;
        }
    }
    e.ww(0x4434aa, 0)?;
    while e.word(0x4434aa)? < 12 {
        let a = 0x447310u32.wrapping_add((i32::from(e.word(0x4434aa)?) * 2) as u32);
        e.pin(a, -99, 999)?;
        e.increment(0x4434aa)?;
    }
    for a in [
        0x447220, 0x447222, 0x447224, 0x447226, 0x44723e, 0x447228, 0x44722a,
    ] {
        e.pin(a, 0, 999)?;
    }
    Ok(e.finish(None))
}

fn add_specialty_row(e: &mut Execution, table: u32, stride: u32) -> Result<(), RuleError> {
    e.ww(0x4434aa, 0)?;
    while e.word(0x4434aa)? < 14 {
        let i = e.word(0x4434aa)?;
        if e.word(0x41f9f2u32.wrapping_add((i32::from(i) * 2) as u32))? != 0 {
            let source = table
                .wrapping_add(u32::from(i as u16) * stride)
                .wrapping_add((i32::from(e.word(0x4434ac)?) * 2) as u32);
            let a = 0x447310u32.wrapping_add((i32::from(i) * 2) as u32);
            e.ww(a, e.word(a)?.wrapping_add(e.word(source)?))?;
        }
        e.increment(0x4434aa)?;
    }
    Ok(())
}
