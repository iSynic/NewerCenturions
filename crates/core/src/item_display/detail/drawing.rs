//! Original panel coordinates and text formatting. Addresses in compatibility
//! effects identify output receipts only; they are never read as machine state.
use super::{
    BTreeMap, BodyRect, Draw, NativeText, PanelEffect, PanelPlan, PascalString, RuleError,
};

pub(super) struct Panel<'a> {
    pub plan: PanelPlan,
    native: &'a NativeText,
    strings: &'a BTreeMap<i16, Vec<u8>>,
}

impl<'a> Panel<'a> {
    pub fn new(native: &'a NativeText, strings: &'a BTreeMap<i16, Vec<u8>>) -> Self {
        Self {
            native,
            strings,
            plan: PanelPlan {
                effects: vec![],
                arithmetic: None,
                retained_definitions: None,
            },
        }
    }
    pub fn start(&mut self, font: i16) {
        self.draw(Draw::SetPort(0));
        self.draw(Draw::BackPixelPattern(0));
        self.draw(Draw::TextSize(12));
        self.draw(Draw::ForeColor(69));
        self.plan.effects.push(PanelEffect::TextMode(1));
        self.plan.effects.push(PanelEffect::TextFaceSlot(0));
        self.draw(Draw::TextFont(font));
        self.draw(Draw::GetPicture(226));
        self.draw(Draw::DrawPicture {
            handle: 226,
            bounds: BodyRect {
                top: 460,
                left: 0,
                bottom: 580,
                right: 800,
            },
        });
    }
    pub fn finish(&mut self) {
        self.draw(Draw::SetPort(0));
        self.plan.effects.push(PanelEffect::RgbBackColor {
            pointer: 0x7000d8,
            bytes: [0; 6],
        });
        self.plan.effects.push(PanelEffect::RgbForeColor {
            pointer: 0x7000de,
            bytes: [0; 6],
        });
    }
    pub fn draw(&mut self, drawing: Draw) {
        self.plan.effects.push(PanelEffect::Drawing(drawing));
    }
    pub fn move_to(&mut self, horizontal: i16, vertical: i16) {
        self.draw(Draw::MoveTo {
            horizontal,
            vertical,
        });
    }
    pub fn convert_number(&mut self, value: i16) -> Result<PascalString, RuleError> {
        let result = PascalString::from_payload(value.to_string().as_bytes())?;
        self.plan.effects.push(PanelEffect::NumberToString {
            value: i32::from(value),
            destination: 0x479ed8,
            result: result.clone(),
        });
        Ok(result)
    }
    pub fn number_at(&mut self, x: i16, y: i16, value: i16, plus: bool) -> Result<(), RuleError> {
        self.move_to(x, y);
        let text = self.convert_number(value)?;
        if value > 0 {
            if plus {
                self.draw(Draw::DrawString(PascalString::from_payload(b"+")?));
            }
            self.draw(Draw::DrawString(text));
        }
        Ok(())
    }
    pub fn indexed(&mut self, resource: i16, index: i16) -> Result<PascalString, RuleError> {
        super::super::indexed_string(self.strings, resource, index)
    }
    pub fn literal(&mut self, address: u32) -> Result<(), RuleError> {
        let length = usize::from(self.native.bytes(address, 1)?[0]);
        self.draw(Draw::DrawString(PascalString::from_raw(
            self.native.bytes(address, length + 1)?,
        )?));
        Ok(())
    }
    pub fn native_c(&self, address: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = vec![];
        for index in 0..256 {
            let byte = self.native.bytes(address + index, 1)?[0];
            bytes.push(byte);
            if byte == 0 {
                return Ok(bytes);
            }
        }
        Err("unterminated item detail text".into())
    }
    pub fn description(&mut self, text: &mut Vec<u8>) -> Result<(), RuleError> {
        let length = text.iter().position(|b| *b == 0).unwrap_or(text.len());
        let encoded = PascalString::from_payload(&text[..length])?;
        self.text_box(
            0x700101,
            encoded.payload().to_vec(),
            BodyRect {
                top: 498,
                left: 638,
                bottom: 573,
                right: 787,
            },
        );
        *text = encoded.raw().to_vec();
        text.push(0);
        Ok(())
    }
    pub fn text_box(&mut self, pointer: u32, bytes: Vec<u8>, rectangle: BodyRect) {
        self.plan.effects.push(PanelEffect::TextBox {
            pointer,
            length: bytes.len() as i32,
            bytes,
            rectangle_pointer: 0x7000f0,
            rectangle,
            alignment: 0,
        });
    }
    pub fn icon(&mut self, id: i16, left: i16) {
        self.draw(Draw::GetColorIcon(id));
        let handle = u32::from(id as u16);
        if handle != 0 {
            self.draw(Draw::PlotColorIcon {
                handle,
                rect: BodyRect {
                    top: 542,
                    left,
                    bottom: 574,
                    right: left + 32,
                },
            });
            self.draw(Draw::DisposeColorIcon(handle));
        }
    }
}
