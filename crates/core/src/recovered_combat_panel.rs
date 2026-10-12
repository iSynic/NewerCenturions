//! Combat and character panels composed from complete CODE 1 bodies.

use crate::{
    RuleError,
    recovered_attack::{self, AttackEffect, AttackTables, AttackWork, DamageFrame},
    recovered_combat_ui::{
        self, CombatUiEffect, CombatUiObservation, CombatUiRequest, CombatUiServices,
    },
    recovered_detail_ui::{self, DetailEffect, DetailServices},
    recovered_drawing::{
        self, BuildingWorld, DrawingContext, DrawingEffect, DrawingServices, PictureObservation,
    },
    recovered_economy::ArithmeticQualification,
    recovered_geometry::BodyRect,
    recovered_install::InstallMemory,
    recovered_interface::{self, ButtonInput, InterfaceEffect, InterfaceServices},
    recovered_navigation::{self, VisibilityResult},
    recovered_records::{self, ITEM_TABLE_ADDRESSES, ItemTables, RecordWrite, SPELL_BYTES},
    recovered_rendering::RenderingMemory,
    recovered_text::{self, NumberContext, NumericServices, PascalString, TextContext, TextEffect},
    rules,
};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombatPanelRequest {
    GetPort {
        destination: u32,
    },
    GetDialogItem {
        dialog: u32,
        item: i16,
        type_address: u32,
        handle_address: u32,
        rect_address: u32,
    },
    SetDialogItemText {
        handle: u32,
        text: u32,
    },
    GetIndexedString {
        resource: i16,
        index: i16,
        destination: u32,
    },
    CToPascal(u32),
    PascalToC(u32),
    CopyCString {
        source: u32,
        destination: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatPanelObservation {
    pub memory: RenderingMemory,
}

pub trait CombatPanelServices: CombatUiServices + DetailServices + InterfaceServices {
    fn observe_panel(
        &mut self,
        request: &CombatPanelRequest,
        memory: &RenderingMemory,
    ) -> Result<CombatPanelObservation, RuleError>;
    /// Original addressed input bytes for the real cansee helper, not its result.
    fn visibility_memory(&mut self, memory: &RenderingMemory) -> Result<InstallMemory, RuleError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum CombatPanelEffect {
    Write(RecordWrite),
    Service {
        request: CombatPanelRequest,
        observation: CombatPanelObservation,
    },
    Combat(CombatUiEffect),
    Detail(DetailEffect),
    Interface(InterfaceEffect),
    Visibility {
        from: (i16, i16),
        to: (i16, i16),
        result: VisibilityResult,
    },
    Damage {
        member: u16,
        mode: i16,
        result: i16,
        qualification: ArithmeticQualification,
    },
    Selection {
        input: i16,
        base: i16,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatPanelPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<CombatPanelEffect>,
    pub native_return: i32,
    /// The inherited cansee/calcdamage adapters use source-derived binary32.
    pub arithmetic_qualifications: Vec<ArithmeticQualification>,
    pub qualification: CombatPanelQualification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatPanelQualification {
    SourcePortWithObservedToolboxAndProjectedChildViews,
}

impl CombatPanelPlan {
    fn new(memory: &RenderingMemory) -> Self {
        Self {
            memory: memory.clone(),
            effects: vec![],
            native_return: 0,
            arithmetic_qualifications: vec![],
            qualification:
                CombatPanelQualification::SourcePortWithObservedToolboxAndProjectedChildViews,
        }
    }
    fn bytes(&self, a: u32, n: usize) -> Result<Vec<u8>, RuleError> {
        (0..n)
            .map(|i| self.memory.byte(a.wrapping_add(i as u32)))
            .collect()
    }
    fn word(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.memory.word(a)? as i16)
    }
    fn byte(&self, a: u32) -> Result<u8, RuleError> {
        self.memory.byte(a)
    }
    fn long(&self, a: u32) -> Result<u32, RuleError> {
        self.memory.long(a)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(a, bytes.len())?;
        self.memory.insert_region(a, bytes);
        self.effects.push(CombatPanelEffect::Write(RecordWrite {
            address: a,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
    fn ww(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn lw(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn add(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.ww(a, self.word(a)?.wrapping_add(v))
    }
    fn outer(
        &mut self,
        request: CombatPanelRequest,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let observation = services.observe_panel(&request, &self.memory)?;
        self.memory = observation.memory.clone();
        self.effects.push(CombatPanelEffect::Service {
            request,
            observation,
        });
        Ok(())
    }
    fn observed(&mut self, request: CombatUiRequest, observation: CombatUiObservation) -> u32 {
        let value = observation.value;
        self.memory = observation.memory.clone();
        self.effects
            .push(CombatPanelEffect::Combat(CombatUiEffect::Service {
                request,
                observation,
            }));
        value
    }
    fn combat_service(
        &mut self,
        request: CombatUiRequest,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<u32, RuleError> {
        let observation = CombatUiServices::observe(services, &request, &self.memory)?;
        Ok(self.observed(request, observation))
    }
    fn draw(
        &mut self,
        effect: DrawingEffect,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.combat_service(CombatUiRequest::Drawing(effect), services)
    }
    fn pascal(&self, a: u32) -> Result<PascalString, RuleError> {
        PascalString::from_raw(&self.bytes(a, usize::from(self.byte(a)?) + 1)?)
    }
    fn literal(
        &mut self,
        a: u32,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.draw(DrawingEffect::DrawString(self.pascal(a)?), services)?;
        Ok(())
    }
    fn y(&self, addition: i16) -> Result<i16, RuleError> {
        Ok(self
            .word(0x40856e)?
            .wrapping_add(addition)
            .wrapping_sub(self.word(0x439582)?.wrapping_shl(5)))
    }
    fn move_to(
        &mut self,
        x: i16,
        addition: i16,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let y = self.y(addition)?;
        self.draw(
            DrawingEffect::MoveTo {
                horizontal: x,
                vertical: y,
            },
            services,
        )?;
        Ok(())
    }
    fn string(
        &mut self,
        value: i16,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let mut bridge = NumericBridge {
            services,
            memory: self.memory.clone(),
            replies: VecDeque::new(),
        };
        let effects = recovered_text::string(&mut TextContext::default(), value, &mut bridge)?;
        self.text_effects(effects, &mut bridge.replies, services)
    }
    fn text_effects(
        &mut self,
        effects: Vec<TextEffect>,
        replies: &mut VecDeque<CombatUiObservation>,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        for effect in effects {
            let request = CombatUiRequest::Text(effect.clone());
            if matches!(effect, TextEffect::NumberToString(_)) {
                let reply = replies.pop_front().ok_or_else(|| {
                    RuleError::Invalid("missing panel numeric observation".into())
                })?;
                self.observed(request, reply);
            } else {
                self.combat_service(request, services)?;
            }
        }
        Ok(())
    }
    fn item(
        &mut self,
        item: i16,
        rect_address: u32,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.outer(
            CombatPanelRequest::GetDialogItem {
                dialog: self.long(0x4394b8)?,
                item,
                type_address: 0x4434b0,
                handle_address: 0x4434dc,
                rect_address,
            },
            services,
        )
    }
    fn dialog_number(
        &mut self,
        item: i16,
        value: i16,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.item(item, 0x4434ba, services)?;
        let mut bridge = NumericBridge {
            services: &*services,
            memory: self.memory.clone(),
            replies: VecDeque::new(),
        };
        let effects =
            recovered_text::dialog_number(&mut TextContext::default(), item, value, &mut bridge)?;
        let reply = bridge
            .replies
            .pop_front()
            .ok_or_else(|| RuleError::Invalid("missing panel numeric observation".into()))?;
        self.observed(
            CombatUiRequest::Text(TextEffect::NumberToString(i32::from(value))),
            reply,
        );
        // The child owns the numeric semantics; native SetIText still receives pointers.
        if !matches!(effects.last(), Some(TextEffect::SetDialogItemText { .. })) {
            return Err(RuleError::Invalid("unexpected dialog child effects".into()));
        }
        self.outer(
            CombatPanelRequest::SetDialogItemText {
                handle: self.long(0x4434dc)?,
                text: 0x479ed8,
            },
            services,
        )
    }
    fn dialog_string(
        &mut self,
        item: i16,
        address: u32,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.item(item, 0x4434ba, services)?;
        let effects = recovered_text::dialog_string(item, &self.pascal(address)?);
        if !matches!(effects.last(), Some(TextEffect::SetDialogItemText { .. })) {
            return Err(RuleError::Invalid("unexpected string child effects".into()));
        }
        self.outer(
            CombatPanelRequest::SetDialogItemText {
                handle: self.long(0x4434dc)?,
                text: address,
            },
            services,
        )
    }
    fn indexed(
        &mut self,
        resource: i16,
        index: i16,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.outer(
            CombatPanelRequest::GetIndexedString {
                resource,
                index,
                destination: 0x479ed8,
            },
            services,
        )
    }
    fn selection(&mut self) -> Result<i16, RuleError> {
        let input = self.word(0x4536a2)?;
        let base = rules::selection_base(input);
        self.ww(0x4471e8, base)?;
        self.effects
            .push(CombatPanelEffect::Selection { input, base });
        Ok(base)
    }
    fn picture(
        &mut self,
        id: i16,
        rect: BodyRect,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let mut bridge = DrawBridge {
            services,
            memory: self.memory.clone(),
            replies: VecDeque::new(),
        };
        let effects = recovered_drawing::pict(id, rect, &mut bridge)?;
        self.drawing_effects(effects, &mut bridge.replies, services)
    }
    fn picture_at(
        &mut self,
        id: i16,
        address: u32,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.picture(id, self.memory.rect(address)?, services)
    }
    fn drawing_effects(
        &mut self,
        effects: Vec<DrawingEffect>,
        replies: &mut VecDeque<CombatUiObservation>,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        for effect in effects {
            match effect {
                DrawingEffect::Write(write) => self.write(write.address, &write.bytes)?,
                DrawingEffect::GetColorIcon(_) | DrawingEffect::GetPicture(_) => {
                    let reply = replies.pop_front().ok_or_else(|| {
                        RuleError::Invalid("missing panel resource observation".into())
                    })?;
                    self.observed(CombatUiRequest::Drawing(effect), reply);
                }
                DrawingEffect::PlotColorIcon { rect, .. } => {
                    self.draw(
                        DrawingEffect::PlotColorIcon {
                            handle: self.long(0x439598)?,
                            rect,
                        },
                        services,
                    )?;
                }
                DrawingEffect::DisposeColorIcon(_) => {
                    self.draw(
                        DrawingEffect::DisposeColorIcon(self.long(0x439598)?),
                        services,
                    )?;
                }
                effect => {
                    self.draw(effect, services)?;
                }
            }
        }
        Ok(())
    }
    fn plot_icon(
        &mut self,
        id: i16,
        rect: BodyRect,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let mut memory = self.memory.clone();
        memory.insert_region(0x439598, &0u32.to_be_bytes());
        let mut bridge = DrawBridge {
            services,
            memory,
            replies: VecDeque::new(),
        };
        let child = recovered_drawing::ploticon3(&mut context(rect), id, rect, &mut bridge)?;
        self.drawing_effects(child.effects, &mut bridge.replies, services)
    }
    fn load_icon(
        &mut self,
        id: i16,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let handle = self.draw(DrawingEffect::GetColorIcon(id), services)?;
        self.lw(0x439598, handle)
    }
    fn plot_loaded(
        &mut self,
        rect_address: u32,
        services: &(impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        if self.long(0x439598)? != 0 {
            self.draw(
                DrawingEffect::PlotColorIcon {
                    handle: self.long(0x439598)?,
                    rect: self.memory.rect(rect_address)?,
                },
                services,
            )?;
            self.draw(
                DrawingEffect::DisposeColorIcon(self.long(0x439598)?),
                services,
            )?;
        }
        Ok(())
    }
    fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
        let current = self.bytes(0x45363c, ITEM_BYTES)?.try_into().unwrap();
        let backing = item_backing(self, id, false)?;
        let child = recovered_records::loaditem(
            &ItemTables {
                families: std::array::from_fn(|i| backing[i].as_slice()),
            },
            &current,
            id,
        )?;
        for effect in child.writes {
            self.write(effect.address, &effect.bytes)?;
        }
        Ok(())
    }
    fn upbutton(
        &mut self,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        let input = ButtonInput {
            rect: self.memory.rect(0x44292c)?,
            target_port: self.long(0x43878c)?,
            dark_pattern: self.long(0x4358de)?,
            light_pattern: self.long(0x4358e2)?,
        };
        let child = recovered_interface::upbutton(input, 1, services)?;
        self.effects
            .extend(child.effects.into_iter().map(CombatPanelEffect::Interface));
        Ok(())
    }
    fn copy_button_rect(&mut self, address: u32) -> Result<(), RuleError> {
        let pointer = self.long(self.long(address)?)?;
        self.lw(0x44292c, self.long(pointer.wrapping_add(8))?)?;
        self.lw(0x442930, self.long(pointer.wrapping_add(12))?)
    }
    fn merge_combat(&mut self, child: recovered_combat_ui::CombatUiPlan) {
        self.memory = child.memory;
        self.effects
            .extend(child.effects.into_iter().map(CombatPanelEffect::Combat));
    }
    fn current_member(&self) -> Result<u16, RuleError> {
        Ok(i16::from(self.byte(0x44273f)? as i8) as u16)
    }
    fn member_address(&self, offset: u32) -> Result<u32, RuleError> {
        Ok(warrior(self.current_member()? as i16).wrapping_add(offset))
    }
    fn member_word(&self, offset: u32) -> Result<i16, RuleError> {
        self.word(self.member_address(offset)?)
    }
    fn member_number(
        &mut self,
        item: i16,
        offset: u32,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.dialog_number(item, self.member_word(offset)?, services)
    }
    fn member_sum(
        &mut self,
        item: i16,
        base: u32,
        bonus: u32,
        services: &mut (impl CombatPanelServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.dialog_number(
            item,
            self.member_word(base)?
                .wrapping_add(self.member_word(bonus)?),
            services,
        )
    }
    fn damage(&mut self, mode: i16, frame: DamageFrame) -> Result<i16, RuleError> {
        let member = self.current_member()?;
        let bytes = self.bytes(warrior(member as i16), WARRIOR_BYTES)?;
        let id = i16::from_be_bytes([
            bytes[if mode == 0 { 0x12c } else { 0x12e }],
            bytes[if mode == 0 { 0x12d } else { 0x12f }],
        ]);
        let ammo = i16::from_be_bytes([bytes[0x126], bytes[0x127]]);
        let mut backing = item_backing(self, id, true)?;
        if mode == 0 && id != 0 && ammo != 0 {
            let additional = item_backing(self, ammo, true)?;
            merge_backing(&mut backing, additional);
        }
        let work = AttackWork {
            item: self.bytes(0x45363c, ITEM_BYTES)?.try_into().unwrap(),
            ammunition: self.bytes(0x4536d2, ITEM_BYTES)?.try_into().unwrap(),
            weapon: [0; ITEM_BYTES],
            packet: [0; SPELL_BYTES],
            spell_coordinates: [0; 3],
            attack_count: 0,
            damage_total: self.word(0x41f714)?,
        };
        // Only this member is reached. Rebase its complete raw record to slot zero.
        let child = recovered_attack::calcdamage(
            &work,
            &AttackTables {
                warrior_memory: &bytes,
                items: ItemTables {
                    families: std::array::from_fn(|i| backing[i].as_slice()),
                },
                spells: &[],
            },
            0,
            mode,
            0,
            frame,
        )?;
        for effect in child.effects {
            match effect {
                AttackEffect::Write(write) => self.write(write.address, &write.bytes)?,
                AttackEffect::LocationLog(_) => {
                    return Err(RuleError::Invalid(
                        "unexpected damage child location log".into(),
                    ));
                }
            }
        }
        self.arithmetic_qualifications.push(child.qualification);
        self.effects.push(CombatPanelEffect::Damage {
            member,
            mode,
            result: child.native_return,
            qualification: child.qualification,
        });
        Ok(child.native_return)
    }
}

fn warrior(body: i16) -> u32 {
    0x448aa0u32.wrapping_add(u32::from(body as u16) * 3138)
}
fn npc(body: i16) -> u32 {
    0x439d3cu32.wrapping_add(u32::from(body.wrapping_sub(10) as u16) * 288)
}
fn item_backing(e: &CombatPanelPlan, id: i16, skip_zero: bool) -> Result<[Vec<u8>; 5], RuleError> {
    let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
    if skip_zero && id == 0 {
        return Ok(backing);
    }
    let id = id.wrapping_abs();
    if let Ok(family) = usize::try_from(id / 200)
        && family < 5
    {
        let offset = (id % 200) as usize * ITEM_BYTES;
        let bytes = e.bytes(
            ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32),
            ITEM_BYTES,
        )?;
        backing[family].resize(offset + ITEM_BYTES, 0);
        backing[family][offset..].copy_from_slice(&bytes);
    }
    Ok(backing)
}
fn merge_backing(target: &mut [Vec<u8>; 5], source: [Vec<u8>; 5]) {
    for (target, source) in target.iter_mut().zip(source) {
        if !source.is_empty() {
            let offset = source.len() - ITEM_BYTES;
            target.resize(target.len().max(source.len()), 0);
            target[offset..offset + ITEM_BYTES].copy_from_slice(&source[offset..]);
        }
    }
}
fn context(rect: BodyRect) -> DrawingContext {
    DrawingContext {
        rect,
        banner_rect: rect,
        source_rect: rect,
        source_row: 0,
        source_column: 0,
        icon_handle: 0,
        item_work: [0; ITEM_BYTES],
        text_work: PascalString::default(),
        window: 0,
        item_window: 0,
        half_width_input: 0,
        active_member: 0,
        omit_banner: 0,
        show_item_background: 0,
        text_in_other_port: 0,
        erase_pattern: 0,
        default_font: 0,
        selection: 0,
        map_port: 0,
        text_port: 0,
        secondary_port: 0,
        backing_port: 0,
        tile_port: 0,
        depth: 0,
        old_world: 0,
        old_device: 0,
        buildings: vec![],
    }
}
struct NumericBridge<'a, S: CombatPanelServices + ?Sized> {
    services: &'a S,
    memory: RenderingMemory,
    replies: VecDeque<CombatUiObservation>,
}
impl<S: CombatPanelServices + ?Sized> NumericServices for NumericBridge<'_, S> {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
        let reply = CombatUiServices::observe(
            self.services,
            &CombatUiRequest::Text(TextEffect::NumberToString(value)),
            &self.memory,
        )?;
        let length = usize::from(reply.memory.byte(0x479ed8)?) + 1;
        let bytes = (0..length)
            .map(|i| reply.memory.byte(0x479ed8 + i as u32))
            .collect::<Result<Vec<_>, _>>()?;
        let text = PascalString::from_raw(&bytes)?;
        self.memory = reply.memory.clone();
        self.replies.push_back(reply);
        Ok(text)
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(RuleError::Invalid("unused panel inverse conversion".into()))
    }
}
struct DrawBridge<'a, S: CombatPanelServices + ?Sized> {
    services: &'a S,
    memory: RenderingMemory,
    replies: VecDeque<CombatUiObservation>,
}
impl<S: CombatPanelServices + ?Sized> DrawingServices for DrawBridge<'_, S> {
    fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
        let reply = CombatUiServices::observe(
            self.services,
            &CombatUiRequest::Drawing(DrawingEffect::GetColorIcon(id)),
            &self.memory,
        )?;
        let value = reply.value;
        self.memory = reply.memory.clone();
        self.replies.push_back(reply);
        Ok(value)
    }
    fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
        let reply = CombatUiServices::observe(
            self.services,
            &CombatUiRequest::Drawing(DrawingEffect::GetPicture(id)),
            &self.memory,
        )?;
        let handle = reply.value;
        self.memory = reply.memory.clone();
        self.replies.push_back(reply);
        Ok(PictureObservation {
            handle,
            bounds: None,
        })
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(RuleError::Invalid(
            "unused panel drawing indexed string".into(),
        ))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(RuleError::Invalid("unused panel world query".into()))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(RuleError::Invalid("unused panel world allocation".into()))
    }
}

fn budget(count: &mut u32) -> Result<(), RuleError> {
    *count += 1;
    if *count > 65536 {
        return Err(RuleError::Unsupported {
            action: "combat_panel",
            dependency: "observed memory produced a nonterminating WORD loop",
        });
    }
    Ok(())
}

/// updatecharinfo, 0x15fa20. Five traits return before restoring the saved port.
pub fn update_char_info(
    memory: &RenderingMemory,
    frame: u32,
    damage_frames: [DamageFrame; 2],
    services: &mut (impl CombatPanelServices + ?Sized),
) -> Result<CombatPanelPlan, RuleError> {
    let mut e = CombatPanelPlan::new(memory);
    e.outer(
        CombatPanelRequest::GetPort {
            destination: frame.wrapping_sub(4),
        },
        services,
    )?;
    e.draw(DrawingEffect::SetPort(e.long(0x4394c8)?), services)?;
    e.lw(0x4394b8, e.long(0x4394c8)?)?;
    e.draw(DrawingEffect::TextSize(12), services)?;
    e.draw(DrawingEffect::TextFont(e.word(0x402f3c)?), services)?;
    e.draw(DrawingEffect::ForeColor(69), services)?;
    for i in 0..7 {
        e.member_number(2 + i as i16, 0x9e2 + 2 * i, services)?;
    }
    for (item, offset) in [(28, 0x3e), (9, 0x9d6), (47, 0x9d8)] {
        e.member_number(item, offset, services)?;
    }
    let damage = e.damage(0, damage_frames[0])?;
    if damage < 1 {
        e.dialog_string(10, 0x400a75, services)?;
        e.item(46, 0x4471f2, services)?;
        if e.member_word(0xe4)? != 0 {
            e.plot_icon(2109, e.memory.rect(0x4471f2)?, services)?;
        }
    } else {
        e.dialog_number(10, damage, services)?;
    }
    let damage = e.damage(1, damage_frames[1])?;
    if damage < 1 {
        e.dialog_string(29, 0x400a75, services)?;
    } else {
        e.dialog_number(29, damage, services)?;
    }
    for (item, base, bonus) in [
        (17, 4, 0xb58),
        (27, 6, 0xb5a),
        (11, 8, 0xb5c),
        (12, 10, 0xb5e),
        (16, 0x22, 0xb60),
    ] {
        e.member_sum(item, base, bonus, services)?;
    }
    if e.member_word(0x52)? > 999 {
        e.draw(DrawingEffect::TextSize(10), services)?;
    }
    e.member_number(13, 0x50, services)?;
    e.member_number(14, 0x52, services)?;
    e.draw(DrawingEffect::TextSize(12), services)?;
    if e.member_word(0x5a)? != 0 {
        if e.member_word(0x5a)? > 999 {
            e.draw(DrawingEffect::TextSize(10), services)?;
        }
        e.member_number(44, 0x58, services)?;
        e.member_number(15, 0x5a, services)?;
    } else {
        e.dialog_string(44, 0x400a75, services)?;
        e.dialog_string(15, 0x400a75, services)?;
    }
    e.draw(DrawingEffect::TextSize(12), services)?;
    e.member_number(45, 0x24, services)?;
    if e.member_word(0x28)? > 999 {
        e.draw(DrawingEffect::TextSize(10), services)?;
    }
    e.member_number(40, 0x26, services)?;
    e.member_number(42, 0x28, services)?;
    e.draw(DrawingEffect::TextSize(12), services)?;
    e.member_number(41, 0x2c, services)?;
    e.member_number(43, 0x2a, services)?;
    e.ww(0x4434aa, 32)?;
    let mut count = 0;
    while e.word(0x4434aa)? < 40 {
        budget(&mut count)?;
        e.draw(DrawingEffect::TextSize(12), services)?;
        let offset = i32::from(e.word(0x4434aa)?.wrapping_sub(32)).wrapping_mul(2) as u32;
        if e.member_word(0x112u32.wrapping_add(offset))? > 999
            || e.member_word(0x112u32.wrapping_add(offset))? < -99
        {
            e.draw(DrawingEffect::TextSize(10), services)?;
        }
        let offset = i32::from(e.word(0x4434aa)?.wrapping_sub(32)).wrapping_mul(2) as u32;
        let value = e.member_word(0x112u32.wrapping_add(offset))?;
        let item = e.word(0x4434aa)?;
        e.dialog_number(item, value, services)?;
        e.add(0x4434aa, 1)?;
    }
    e.combat_service(CombatUiRequest::RgbForeColor(0x44337a), services)?;
    e.ww(0x4434aa, 20)?;
    while e.word(0x4434aa)? < 25 {
        budget(&mut count)?;
        e.dialog_string(e.word(0x4434aa)?, 0x400a75, services)?;
        e.add(0x4434aa, 1)?;
    }
    e.ww(0x4434aa, 0)?;
    let mut shown = 0i8;
    while e.word(0x4434aa)? < 40 {
        budget(&mut count)?;
        let offset = (i32::from(e.word(0x4434aa)?) * 2) as u32;
        if e.member_word(0x5eu32.wrapping_add(offset))? != 0 {
            e.indexed(133, e.word(0x4434aa)?.wrapping_add(1), services)?;
            let item = i16::from(shown).wrapping_add(20);
            shown = shown.wrapping_add(1);
            e.dialog_string(item, 0x479ed8, services)?;
            if shown == 5 {
                return Ok(e);
            }
        }
        e.add(0x4434aa, 1)?;
    }
    e.draw(
        DrawingEffect::SetPort(e.long(frame.wrapping_sub(4))?),
        services,
    )?;
    Ok(e)
}

fn local_rect(
    e: &mut CombatPanelPlan,
    frame: u32,
    top_add: i16,
    left: i16,
) -> Result<(), RuleError> {
    e.ww(frame.wrapping_sub(8), e.y(top_add)?)?;
    e.ww(frame.wrapping_sub(6), left)?;
    e.ww(
        frame.wrapping_sub(2),
        e.word(frame.wrapping_sub(6))?.wrapping_add(32),
    )?;
    e.ww(
        frame.wrapping_sub(4),
        e.word(frame.wrapping_sub(8))?.wrapping_add(32),
    )
}

/// combatupdate2, 0x11f1a8. Body WORD is captured before presentation services.
pub fn combat_update2(
    memory: &RenderingMemory,
    frame: u32,
    body: i16,
    odds_damage_frame: DamageFrame,
    services: &mut (impl CombatPanelServices + ?Sized),
) -> Result<CombatPanelPlan, RuleError> {
    let mut e = CombatPanelPlan::new(memory);
    if e.word(frame.wrapping_add(8))? != body {
        return Err(RuleError::Invalid(
            "combat panel caller WORD disagrees with body".into(),
        ));
    }
    e.draw(DrawingEffect::SetPort(e.long(0x43878c)?), services)?;
    e.draw(DrawingEffect::TextFont(e.word(0x402f3c)?), services)?;
    e.ww(0x443318, body)?;
    if e.word(0x438782)? == 0 {
        let id = 153i16.wrapping_sub(e.word(0x438760)?.wrapping_shl(3));
        e.picture_at(id, 0x43872e, services)?;
        let id = e.word(0x438760)?.wrapping_mul(18).wrapping_add(156);
        e.picture_at(id, 0x44370a, services)?;
        panel_number(&mut e, services)?;
    } else if e.word(0x438760)? == 0 {
        let actor = e.word(0x4435d2)?;
        let child = recovered_combat_ui::flashrange(
            &e.memory,
            frame.wrapping_sub(24),
            body,
            actor,
            &*services,
        )?;
        e.merge_combat(child);
        let target = if body < 9 {
            0x44709au32.wrapping_add((i32::from(body) * 4) as u32)
        } else {
            0x446f0au32.wrapping_add((i32::from(body.wrapping_sub(10)) * 4) as u32)
        };
        let to = (e.word(target)?, e.word(target.wrapping_add(2))?);
        let actor = e.word(0x4435d2)?;
        let from_address = 0x44709au32.wrapping_add((i32::from(actor) * 4) as u32);
        let from = (e.word(from_address)?, e.word(from_address.wrapping_add(2))?);
        let dense = services.visibility_memory(&e.memory)?;
        for (&address, &byte) in &e.memory.bytes {
            if let Some(offset) = address.checked_sub(dense.base_address).map(|n| n as usize)
                && let Some(observed) = dense.bytes.get(offset)
                && *observed != byte
            {
                return Err(RuleError::Invalid(
                    "visibility memory disagrees with candidate bytes".into(),
                ));
            }
        }
        let result = recovered_navigation::cansee(&dense, from, to)?;
        e.arithmetic_qualifications.push(result.qualification);
        e.effects
            .push(CombatPanelEffect::Visibility { from, to, result });
        if result.native_return as i16 == 0 {
            e.literal(0x403aa8, services)?;
        }
    }
    e.picture_at(166, 0x4471c4, services)?;
    if e.word(0x439582)? != 0 && e.word(0x438778)? == 0 {
        let child_frame = frame.wrapping_sub(22);
        e.ww(
            child_frame.wrapping_add(8),
            if body > 9 { body.wrapping_sub(10) } else { -1 },
        )?;
        let child =
            recovered_detail_ui::update_odds(&e.memory, child_frame, odds_damage_frame, services)?;
        e.memory = child.memory;
        e.effects
            .extend(child.effects.into_iter().map(CombatPanelEffect::Detail));
        if let Some(qualification) = child.arithmetic_qualification {
            e.arithmetic_qualifications.push(qualification);
        }
    }
    if e.word(0x439582)? != 0 {
        e.ww(0x4434bc, 0)?;
        e.ww(0x4434c0, 480)?;
        e.ww(0x4434ba, 416)?;
        e.ww(0x4434be, e.word(0x4434ba)?.wrapping_add(11))?;
        e.picture_at(206, 0x4434ba, services)?;
        e.ww(0x4434bc, 0)?;
        e.ww(0x4434c0, 480)?;
        e.ww(0x4434ba, 589)?;
        e.ww(0x4434be, 580)?;
        e.picture_at(206, 0x4434ba, services)?;
        e.add(0x4434bc, 480)?;
        e.add(0x4434c0, 480)?;
        e.picture_at(206, 0x4434ba, services)?;
        e.picture_at(228, 0x438736, services)?;
        if body > 9 {
            for (left, negative) in [(339, false), (426, true)] {
                e.ww(0x4434bc, left)?;
                e.ww(0x4434c0, e.word(0x4434bc)?.wrapping_add(9))?;
                e.ww(
                    0x4434ba,
                    484i16.wrapping_sub(e.word(0x439582)?.wrapping_shl(5)),
                )?;
                e.ww(0x4434be, e.word(0x4434ba)?.wrapping_add(9))?;
                e.ww(0x4434aa, 1)?;
                let mut count = 0;
                while e.word(0x4434aa)? < 8 {
                    budget(&mut count)?;
                    let a = npc(body)
                        .wrapping_add(0x8b)
                        .wrapping_add(i32::from(e.word(0x4434aa)?) as u32);
                    let value = e.byte(a)? as i8;
                    let id = if if negative { value < 0 } else { value > 100 } {
                        150
                    } else {
                        146
                    };
                    e.picture_at(id, 0x4434ba, services)?;
                    e.add(0x4434ba, 16)?;
                    e.add(0x4434be, 16)?;
                    e.add(0x4434aa, 1)?;
                }
            }
        }
    }
    e.draw(DrawingEffect::TextSize(12), services)?;
    local_rect(&mut e, frame, 344, 24)?;
    e.lw(0x439598, 0)?;
    if body < 10 {
        let base = warrior(body);
        e.load_icon(e.word(base + 0x56)?, services)?;
        e.plot_loaded(frame.wrapping_sub(8), services)?;
        let turn = e.word(0x447194)?;
        if body == e.word(0x4470b8u32.wrapping_add((i32::from(turn) * 2) as u32))? {
            if e.word(base + 0xe4)? != 0 {
                e.copy_button_rect(0x4393f8)?;
                e.upbutton(services)?;
            }
            let id = if e.byte(base + 0x9dd)? == 0 {
                e.word(base + 0xca)?
            } else {
                e.word(base + 0xe4)?
            };
            e.load_item(id)?;
            if e.word(0x453668)? > 1100 || e.word(0x453656)? == 15 {
                find_inventory(&mut e, base, 0x4536a2, true)?;
                let pack = inventory(base, e.word(0x4434aa)?);
                if e.word(pack + 2)? != 0 && e.word(0x438760)? == 0 {
                    e.copy_button_rect(0x43942c)?;
                    e.upbutton(services)?;
                }
            }
        }
    } else {
        let base = npc(body);
        e.ww(0x4435ce, e.word(base + 16)?.wrapping_abs())?;
        e.ww(0x4434b6, e.word(base + 0x50)?)?;
        if e.word(base + 16)? != -3 && e.byte(base + 0x107)? == 1 {
            e.add(0x4434b6, 1000)?;
        }
        if matches!(e.word(0x4435ce)?, 1 | 3) {
            e.add(frame.wrapping_sub(8), -16)?;
            e.add(frame.wrapping_sub(4), 16)?;
        }
        if e.word(0x4435ce)? > 1 {
            e.add(frame.wrapping_sub(6), -16)?;
            e.add(frame.wrapping_sub(2), 16)?;
        }
        e.load_icon(e.word(0x4434b6)?, services)?;
        e.plot_loaded(frame.wrapping_sub(8), services)?;
    }
    let child =
        recovered_combat_ui::combatupdate(&e.memory, frame.wrapping_sub(22), body, &*services)?;
    e.merge_combat(child);
    e.draw(DrawingEffect::ForeColor(69), services)?;
    e.move_to(112, 367, services)?;
    if body < 10 {
        let base = warrior(body);
        if e.word(base + 0x58)? < e.word(base + 0x5a)? {
            e.combat_service(CombatUiRequest::RgbForeColor(0x44337a), services)?;
        } else {
            e.draw(DrawingEffect::ForeColor(69), services)?;
        }
        e.string(e.word(base + 0x58)?, services)?;
        for (x, y, field) in [(212, 389, 0x40), (172, 344, 0x24), (276, 344, 0x2c)] {
            e.move_to(x, y, services)?;
            e.string(e.word(base + field)?, services)?;
        }
    } else {
        let base = npc(body);
        if e.word(base + 0x10e)? < e.word(base + 0x52)? {
            e.combat_service(CombatUiRequest::RgbForeColor(0x44337a), services)?;
        } else {
            e.draw(DrawingEffect::ForeColor(69), services)?;
        }
        e.string(e.word(base + 0x10e)?, services)?;
        e.move_to(212, 389, services)?;
        e.string(i16::from(e.byte(base + 0x109)? as i8), services)?;
        for (x, field) in [(172, 0x56), (217, 0x58), (276, 8)] {
            e.move_to(x, 344, services)?;
            e.string(e.word(base + field)?, services)?;
        }
    }
    e.draw(DrawingEffect::ForeColor(69), services)?;
    local_rect(&mut e, frame, 399, 8)?;
    e.move_to(128, 432, services)?;
    e.lw(0x439598, 0)?;
    if body < 10 {
        let base = warrior(body);
        if e.byte(base + 0x9dd)? != 0 {
            if e.word(base + 0xe4)? != 0 {
                e.load_item(e.word(base + 0xe4)?)?;
                e.ww(0x4471ea, 0)?;
                if e.word(base + 0x12c)? != 0 {
                    e.ww(0x4471ea, 1)?;
                }
                selected_item_name(&mut e, false, services)?;
            } else {
                e.load_icon(7002, services)?;
                e.indexed(4, 3, services)?;
                e.write(base + 0x9dd, &[0])?;
            }
        } else if e.word(base + 0xca)? != 0 {
            e.load_item(e.word(base + 0xca)?)?;
            e.ww(0x4471ea, 0)?;
            if e.word(base + 0x12e)? > -1 {
                e.ww(0x4471ea, 1)?;
            }
            selected_item_name(&mut e, false, services)?;
        } else {
            e.load_icon(7002, services)?;
            e.indexed(4, 3, services)?;
        }
        e.literal(0x479ed8, services)?;
        e.move_to(64, 430, services)?;
        e.draw(DrawingEffect::TextSize(12), services)?;
        let attacks = if e.word(base + 0xe4)? != 0 && e.byte(base + 0x9dd)? != 0 {
            e.word(0x453688)?
        } else {
            1
        };
        e.string(attacks, services)?;
        e.draw(DrawingEffect::ForeColor(69), services)?;
        e.plot_loaded(frame.wrapping_sub(8), services)?;
        e.move_to(128, 411, services)?;
        e.outer(CombatPanelRequest::CToPascal(base + 0xa21), services)?;
        e.literal(base + 0xa21, services)?;
        e.outer(CombatPanelRequest::PascalToC(base + 0xa21), services)?;
    } else {
        let base = npc(body);
        if e.word(base + 0x4e)? != 0 {
            e.load_item(e.word(base + 0x4e)?)?;
            selected_item_name(&mut e, true, services)?;
        } else {
            e.load_icon(7002, services)?;
            e.indexed(4, 3, services)?;
        }
        e.literal(0x479ed8, services)?;
        e.plot_loaded(frame.wrapping_sub(8), services)?;
        e.move_to(64, 430, services)?;
        e.draw(DrawingEffect::TextSize(12), services)?;
        e.string(
            if e.word(base + 0x4e)? != 0 {
                e.word(0x453688)?
            } else {
                1
            },
            services,
        )?;
        e.outer(
            CombatPanelRequest::CopyCString {
                source: base + 0xda,
                destination: 0x438aa4,
            },
            services,
        )?;
        e.move_to(128, 411, services)?;
        e.literal(0x438aa4, services)?;
    }
    e.move_to(73, 453, services)?;
    if body < 10 {
        let base = warrior(body);
        let id = if e.byte(base + 0x9dd)? != 0 {
            e.word(base + 0xe4)?
        } else {
            e.word(base + 0xca)?
        };
        e.ww(0x4434b6, id)?;
        find_inventory(&mut e, base, 0x4434b6, false)?;
        let pack = inventory(base, e.word(0x4434aa)?);
        if e.word(pack + 2)? < 1 {
            e.literal(0x403aa4, services)?;
        } else {
            e.string(e.word(pack + 2)?, services)?;
        }
    } else {
        e.literal(
            if e.word(npc(body) + 0x4e)? != 0 {
                0x403aa0
            } else {
                0x403aa4
            },
            services,
        )?;
    }
    e.ww(0x4434aa, 0)?;
    let mut count = 0;
    while e.word(0x4434aa)? < 40 {
        budget(&mut count)?;
        let i = e.word(0x4434aa)?;
        let found = if body < 10 {
            e.word(
                warrior(body)
                    .wrapping_add(0x5e)
                    .wrapping_add((i32::from(i) * 2) as u32),
            )? != 0
        } else {
            e.byte(
                npc(body)
                    .wrapping_add(0xb2)
                    .wrapping_add(i32::from(i) as u32),
            )? != 0
        };
        if found {
            e.indexed(133, i.wrapping_add(1), services)?;
            e.move_to(117, 453, services)?;
            e.literal(0x479ed8, services)?;
            return Ok(e);
        }
        e.add(0x4434aa, 1)?;
    }
    Ok(e)
}

fn inventory(base: u32, index: i16) -> u32 {
    base.wrapping_add(0x132)
        .wrapping_add(u32::from(index as u16) * 22)
}
fn find_inventory(
    e: &mut CombatPanelPlan,
    base: u32,
    id_address: u32,
    equipped: bool,
) -> Result<(), RuleError> {
    e.ww(0x4434aa, 0)?;
    let mut count = 0;
    while e.word(0x4434aa)? < e.word(base + 0x1a)? {
        budget(&mut count)?;
        let pack = inventory(base, e.word(0x4434aa)?);
        if e.word(id_address)? == e.word(pack)? && (!equipped || e.byte(pack + 9)? != 0) {
            break;
        }
        e.add(0x4434aa, 1)?;
    }
    Ok(())
}
fn selected_item_name(
    e: &mut CombatPanelPlan,
    npc: bool,
    services: &mut (impl CombatPanelServices + ?Sized),
) -> Result<(), RuleError> {
    let base = e.selection()?;
    let resource = base.wrapping_add(if npc { 1 } else { e.word(0x4471ea)? });
    let index = e
        .word(0x4536a2)?
        .wrapping_sub(e.word(0x4471e8)?)
        .wrapping_add(1);
    e.indexed(resource, index, services)?;
    e.load_icon(e.word(0x453646)?, services)
}
fn panel_number(
    e: &mut CombatPanelPlan,
    services: &(impl CombatPanelServices + ?Sized),
) -> Result<(), RuleError> {
    let suppressed = e.word(0x438760)?;
    if suppressed != 0 {
        // Other typed fields are not read by the suppressed child branch.
        let input = NumberContext {
            suppressed,
            origin_x: 0,
            origin_y: 0,
            line: 0,
            font: 0,
            enemy_total: 0,
            enemy_removed: 0,
            label: PascalString::default(),
        };
        let mut bridge = NumericBridge {
            services,
            memory: e.memory.clone(),
            replies: VecDeque::new(),
        };
        let effects = recovered_text::number(&mut TextContext::default(), &input, &mut bridge)?;
        return e.text_effects(effects, &mut bridge.replies, services);
    }
    // Refine the pure child's captured prefix at native global-read boundaries.
    // Observed drawing calls may change globals before later arguments are read.
    e.draw(DrawingEffect::TextSize(20), services)?;
    e.draw(DrawingEffect::ForeColor(69), services)?;
    e.draw(DrawingEffect::TextFace(1), services)?;
    let font = e.word(0x402f3c)?;
    e.draw(DrawingEffect::TextFont(font), services)?;
    let origin_x = e.word(0x40856c)?;
    let line = e.word(0x439582)?;
    let origin_y = e.word(0x40856e)?;
    e.draw(
        DrawingEffect::MoveTo {
            horizontal: origin_x.wrapping_add(430),
            vertical: origin_y
                .wrapping_add(445)
                .wrapping_sub(line.wrapping_shl(5)),
        },
        services,
    )?;
    let label = e.pascal(0x401b6a)?;
    e.draw(DrawingEffect::DrawString(label.clone()), services)?;
    let x = e.word(0x40856c)?.wrapping_add(580);
    e.move_to(x, 445, services)?;
    let input = NumberContext {
        suppressed: 0,
        origin_x,
        origin_y,
        line,
        font,
        enemy_total: e.word(0x4435c4)?,
        enemy_removed: e.word(0x44374c)?,
        label,
    };
    let mut bridge = NumericBridge {
        services,
        memory: e.memory.clone(),
        replies: VecDeque::new(),
    };
    let effects = recovered_text::number(&mut TextContext::default(), &input, &mut bridge)?;
    // The seven prefix effects have already been observed in native order.
    e.text_effects(
        effects.into_iter().skip(7).collect(),
        &mut bridge.replies,
        services,
    )
}
