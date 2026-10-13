use centurions_content::{Content, ContentError, graphics, read_resource, ui};
use macroquad::prelude::*;
use std::{collections::HashMap, path::Path};

const APPLICATION_PICTURES: &[i16] = &[
    1, 2, 4, 7, 19, 128, 130, 131, 132, 142, 149, 151, 158, 159, 160, 165, 168, 173, 179, 180, 181,
    182, 183, 187, 195, 203, 214, 215, 216, 221, 233, 234, 235,
];

pub struct Assets {
    pub terrain: HashMap<u8, Texture2D>,
    pub icons: HashMap<i16, Texture2D>,
    pub portraits: HashMap<i16, Texture2D>,
    pub panels: HashMap<i16, Texture2D>,
    pub movie_background: Texture2D,
    pub menus: HashMap<i16, ui::Menu>,
    pub menu_bar: Vec<i16>,
    pub font: Font,
    symbol_font: Font,
    pub music_ids: Vec<i16>,
    pub names: HashMap<i16, String>,
    pub spell_names: HashMap<i16, String>,
    pub warnings: Vec<String>,
    pub conditions: Vec<String>,
    pub questions: Vec<String>,
    pub squad_text: Vec<String>,
    pub upgrade_dialog: Vec<ui::DialogItem>,
    pub item_dialog: Vec<ui::DialogItem>,
    pub scan_dialog: Vec<ui::DialogItem>,
    pub upgrade_names: Vec<String>,
    pub upgrade_formulas: Vec<String>,
}
impl Assets {
    pub fn load(content: &Content, extracted: &Path) -> Result<Self, ContentError> {
        let mut terrain = HashMap::new();
        let mut icons = HashMap::new();
        let mut portraits = HashMap::new();
        let mut panels = HashMap::new();
        let mut menus = HashMap::new();
        let mut menu_bar = None;
        let mut font = None;
        let mut movie_background = None;
        let mut names = HashMap::new();
        let mut spell_names = HashMap::new();
        let mut warnings = Vec::new();
        let mut conditions = Vec::new();
        let mut questions = Vec::new();
        let mut squad_text = Vec::new();
        let mut upgrade_dialog = Vec::new();
        let mut item_dialog = Vec::new();
        let mut scan_dialog = Vec::new();
        let mut upgrade_names = Vec::new();
        let mut upgrade_formulas = Vec::new();
        for resource in &content.resources {
            let cargo = resource.original.ends_with("/Cargo Pod");
            if cargo && resource.r#type == "ppat" && resource.id == 131 {
                let image = graphics::decode_pattern(&read_resource(extracted, resource)?)?;
                let texture = Texture2D::from_rgba8(image.width, image.height, &image.rgba);
                texture.set_filter(FilterMode::Nearest);
                movie_background = Some(texture);
            }
            if cargo && resource.r#type == "MENU" {
                let menu = ui::menu(&read_resource(extracted, resource)?).map_err(|error| {
                    ContentError::Invalid(format!("MENU {}: {error}", resource.id))
                })?;
                menus.insert(resource.id, menu);
            }
            if cargo && resource.r#type == "MBAR" && resource.id == 128 {
                menu_bar = Some(ui::menu_bar(&read_resource(extracted, resource)?).map_err(
                    |error| ContentError::Invalid(format!("MBAR {}: {error}", resource.id)),
                )?);
            }
            if cargo && resource.r#type == "sfnt" && resource.id == 30600 {
                font = Some(
                    load_ttf_font_from_bytes(&read_resource(extracted, resource)?)
                        .map_err(|e| ContentError::Invalid(format!("Textile font: {e}")))?,
                );
            }
            if cargo && resource.r#type == "STR#" && resource.id == 133 {
                conditions = ui::string_list(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "STR#" && resource.id == 3 {
                warnings = ui::string_list(&read_resource(extracted, resource)?)?;
            }
            if cargo
                && resource.r#type == "STR#"
                && (1..=4).contains(&(resource.id / 1000))
                && (0..=6).contains(&(resource.id % 1000))
            {
                let base = resource.id / 1000 * 1000 + (resource.id % 1000 + 1) * 100;
                for (index, name) in ui::string_list(&read_resource(extracted, resource)?)?
                    .into_iter()
                    .enumerate()
                {
                    spell_names.insert(base + index as i16 + 1, name);
                }
            }
            if cargo && resource.r#type == "STR#" && resource.id == 135 {
                questions = ui::string_list(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "DITL" && resource.id == 145 {
                squad_text = ui::dialog_strings(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "DITL" && resource.id == 155 {
                upgrade_dialog = ui::dialog_items(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "DITL" && resource.id == 141 {
                item_dialog = ui::dialog_items(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "DITL" && resource.id == 135 {
                scan_dialog = ui::dialog_items(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "STR#" && resource.id == 143 {
                upgrade_names = ui::string_list(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "STR#" && resource.id == 144 {
                upgrade_formulas = ui::string_list(&read_resource(extracted, resource)?)?;
            }
            if resource.r#type == "STR#"
                && resource.original.ends_with("/Data ID")
                && [1, 201, 401, 601].contains(&resource.id)
            {
                for (i, name) in ui::string_prefix(&read_resource(extracted, resource)?, 200)?
                    .into_iter()
                    .enumerate()
                {
                    names.insert(resource.id - 1 + i as i16, name);
                }
            }
            let picture = cargo
                && resource.r#type == "PICT"
                && ((300..=308).contains(&resource.id)
                    || ui::PANEL_PICTURES.contains(&resource.id)
                    || APPLICATION_PICTURES.contains(&resource.id));
            let portrait = resource.original.ends_with("/Portraits") && resource.r#type == "cicn";
            let icon = (cargo
                || resource.original.ends_with("/Tacticals")
                || resource
                    .original
                    .ends_with("/The Thunder of Guns/0/Scenario"))
                && resource.r#type == "cicn";
            if !picture && !portrait && !icon {
                continue;
            }
            let bytes = read_resource(extracted, resource)?;
            let decoded = if picture {
                graphics::decode_pict(&bytes).map_err(|error| {
                    ContentError::Invalid(format!("PICT {}: {error}", resource.id))
                })?
            } else {
                graphics::decode_cicn(&bytes)?
            };
            let texture = Texture2D::from_rgba8(decoded.width, decoded.height, &decoded.rgba);
            texture.set_filter(FilterMode::Nearest);
            if picture && (300..=308).contains(&resource.id) {
                terrain.insert((resource.id - 299) as u8, texture);
            } else if picture {
                panels.insert(resource.id, texture);
            } else if portrait {
                portraits.insert(resource.id, texture);
            } else {
                icons.insert(resource.id, texture);
            }
        }
        let menu_bar = menu_bar.ok_or_else(|| ContentError::Invalid("MBAR 128 missing".into()))?;
        for id in menu_bar.iter().chain([134, 135].iter()) {
            if !menus.contains_key(id) {
                return Err(ContentError::Invalid(format!("MENU {id} missing")));
            }
        }
        for id in APPLICATION_PICTURES {
            if !panels.contains_key(id) {
                return Err(ContentError::Invalid(format!(
                    "required UI PICT {id} missing"
                )));
            }
        }
        if questions.len() < 19
            || squad_text.len() < 70
            || upgrade_dialog.len() < 102
            || item_dialog.len() < 99
            || scan_dialog.len() < 50
            || upgrade_names.len() < 33
            || upgrade_formulas.len() < 33
        {
            return Err(ContentError::Invalid(
                "incomplete original dialog text".into(),
            ));
        }
        menus.get_mut(&136).unwrap().items[7].text = "Activate Warrior Editor".into();
        let assets = Self {
            terrain,
            icons,
            portraits,
            panels,
            movie_background: movie_background
                .ok_or_else(|| ContentError::Invalid("movie pixel pattern 131 missing".into()))?,
            menus,
            menu_bar,
            symbol_font: font
                .ok_or_else(|| ContentError::Invalid("Textile font missing".into()))?,
            font: std::env::var_os("WINDIR")
                .and_then(|root| std::fs::read(Path::new(&root).join("Fonts/segoeui.ttf")).ok())
                .and_then(|bytes| load_ttf_font_from_bytes(&bytes).ok())
                .unwrap_or_else(get_default_font),
            music_ids: content
                .resources
                .iter()
                .filter(|r| r.r#type == "snd " && (20128..=20140).contains(&r.id))
                .map(|r| r.id)
                .collect(),
            names,
            spell_names,
            warnings,
            conditions,
            questions,
            squad_text,
            upgrade_dialog,
            item_dialog,
            scan_dialog,
            upgrade_names,
            upgrade_formulas,
        };
        let characters = Font::ascii_character_list();
        for size in [12, 14, 24] {
            assets.font.populate_font_cache(&characters, size);
        }
        Ok(assets)
    }
    pub fn menu_title(&self, resource_id: i16) -> Option<&str> {
        let title = self.menus.get(&resource_id)?.title.as_str();
        // Textile sfnt 30600 maps the Apple character to glyph 210 in its Unicode cmap.
        Some(if title == "\u{14}" { "\u{f8ff}" } else { title })
    }
    pub fn name(&self, id: i16) -> String {
        self.names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("Item {id}"))
    }
    pub fn panel(&self, id: i16, rect: Rect) {
        if let Some(texture) = self.panels.get(&id) {
            draw_texture_ex(
                texture,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        }
    }
    pub fn text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(if text == "\u{f8ff}" {
                    &self.symbol_font
                } else {
                    &self.font
                }),
                font_size: size,
                color,
                ..Default::default()
            },
        );
    }
    pub fn wrapped(&self, text: &str, rect: Rect, size: u16, line_height: f32) -> f32 {
        self.wrapped_color(text, rect, size, line_height, WHITE)
    }
    pub fn wrapped_color(
        &self,
        text: &str,
        rect: Rect,
        size: u16,
        line_height: f32,
        color: Color,
    ) -> f32 {
        let mut y = rect.y;
        let mut line = String::new();
        for paragraph in text.split('\n') {
            for word in paragraph.split_whitespace() {
                let proposed = if line.is_empty() {
                    word.to_owned()
                } else {
                    format!("{line} {word}")
                };
                if measure_text(&proposed, Some(&self.font), size, 1.).width > rect.w
                    && !line.is_empty()
                {
                    if y <= rect.y + rect.h {
                        self.text(&line, rect.x, y, size, color);
                    }
                    y += line_height;
                    line.clear();
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            if y <= rect.y + rect.h {
                self.text(&line, rect.x, y, size, color);
            }
            y += line_height;
            line.clear();
        }
        y - rect.y
    }
}
