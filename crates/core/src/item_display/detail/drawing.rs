//! Panel coordinates and text formatting. Addresses in the effects name the
//! original text for comparison; they are never read as game state.
use super::{BTreeMap, BodyRect, Draw, GameText, PanelEffect, PanelPlan, PascalString, RuleError};

/// The Items detail panel as it is being drawn: the drawing steps so far and the texts the panel
/// reads.
pub(super) struct Panel<'a> {
    pub plan: PanelPlan,
    game_text: &'a GameText,
    strings: &'a BTreeMap<i16, Vec<u8>>,
}

impl<'a> Panel<'a> {
    /// Starts an empty panel.
    pub fn new(game_text: &'a GameText, strings: &'a BTreeMap<i16, Vec<u8>>) -> Self {
        Self {
            game_text,
            strings,
            plan: PanelPlan {
                effects: vec![],
                arithmetic: None,
                loaded_items: None,
            },
        }
    }
    /// Starts the panel: the port and text settings are set and the background picture drawn.
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
    /// Ends the panel by putting the colors back to black.
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
    /// Adds a drawing step to the panel.
    pub fn draw(&mut self, drawing: Draw) {
        self.plan.effects.push(PanelEffect::Drawing(drawing));
    }
    /// Moves the pen to a position of the panel.
    pub fn move_to(&mut self, horizontal: i16, vertical: i16) {
        self.draw(Draw::MoveTo {
            horizontal,
            vertical,
        });
    }
    /// Turns a number into its text, noting the conversion.
    pub fn convert_number(&mut self, value: i16) -> Result<PascalString, RuleError> {
        let result = PascalString::from_payload(value.to_string().as_bytes())?;
        self.plan.effects.push(PanelEffect::NumberToString {
            value: i32::from(value),
            destination: 0x479ed8,
            result: result.clone(),
        });
        Ok(result)
    }
    /// Draws a figure at a position only when it is positive, with a plus sign in front if asked
    /// for; zero and negative figures draw nothing, though the number is still converted.
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
    /// A string of a STR# resource by its index.
    pub fn indexed(&mut self, resource: i16, index: i16) -> Result<PascalString, RuleError> {
        super::super::indexed_string(self.strings, resource, index)
    }
    /// Draws a fixed text of the game.
    pub fn literal(&mut self, address: u32) -> Result<(), RuleError> {
        let length = usize::from(self.game_text.bytes(address, 1)?[0]);
        self.draw(Draw::DrawString(PascalString::from_raw(
            self.game_text.bytes(address, length + 1)?,
        )?));
        Ok(())
    }
    /// Reads a zero-terminated text of the game, which must end within 256 bytes.
    pub fn c_string(&self, address: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = vec![];
        for index in 0..256 {
            let byte = self.game_text.bytes(address + index, 1)?[0];
            bytes.push(byte);
            if byte == 0 {
                return Ok(bytes);
            }
        }
        Err("unterminated item detail text".into())
    }
    /// Shows the item's description in its text box of the panel.
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
    /// Draws text in a box of the panel.
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
    /// Draws an icon of the panel in a 32-pixel square at the left position given.
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
