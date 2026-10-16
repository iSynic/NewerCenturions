use centurions_content::{Content, ContentError, graphics, read_resource, ui};
use macroquad::prelude::*;
use std::{collections::HashMap, path::Path};

const APPLICATION_PICTURES: &[i16] = &[
    1, 2, 4, 7, 19, 128, 130, 131, 132, 142, 149, 151, 154, 158, 159, 160, 162, 165, 168, 173, 179,
    180, 181, 182, 183, 187, 195, 203, 204, 214, 215, 216, 221, 222, 233, 234, 235, 15, 16, 133,
    134, 136, 137, 138, 144, 147, 148, 150, 152, 153, 156, 167, 170, 175, 190, 191, 202, 193, 194,
    212, 213, 226, 227,
];

pub struct Assets {
    pub reference_strings: HashMap<i16, Vec<String>>,
    pub native_text: centurions_content::native_text::NativeText,
    pub terrain: HashMap<u8, Texture2D>,
    pub icons: HashMap<i16, Texture2D>,
    target_patterns: HashMap<i16, Texture2D>,
    target_material: Material,
    pub portraits: HashMap<i16, Texture2D>,
    pub panels: HashMap<i16, Texture2D>,
    base_panels: HashMap<i16, Texture2D>,
    base_icons: HashMap<i16, Texture2D>,
    pub movie_background: Texture2D,
    pub menus: HashMap<i16, ui::Menu>,
    pub menu_bar: Vec<i16>,
    pub font: Font,
    original_font: Font,
    pub menu_font: Font,
    symbol_font: Font,
    pub music_ids: Vec<i16>,
    pub names: HashMap<i16, String>,
    pub spell_names: HashMap<i16, String>,
    pub warnings: Vec<String>,
    pub conditions: Vec<String>,
    pub questions: Vec<String>,
    pub squad_text: Vec<String>,
    pub upgrade_dialog: Vec<ui::DialogItem>,
    pub information_dialog: Vec<ui::DialogItem>,
    pub appearance_dialog: Vec<ui::DialogItem>,
    pub character_strings: std::collections::BTreeMap<i16, Vec<u8>>,
    pub item_dialog: Vec<ui::DialogItem>,
    pub scan_dialog: Vec<ui::DialogItem>,
    pub upgrade_names: Vec<String>,
    pub upgrade_formulas: Vec<String>,
    pub main_controls: HashMap<i16, Rect>,
    pub dialogs: HashMap<i16, Vec<ui::DialogItem>>,
    pub dialog_sizes: HashMap<i16, Vec2>,
    pub preference_descriptions: Vec<String>,
}
impl Assets {
    pub fn target_pattern(&self, shape: i16, point: Vec2, clip: Rect) {
        let Some(texture) = self.target_patterns.get(&(9999 + shape)) else {
            return;
        };
        let left = point.x.max(clip.x);
        let top = point.y.max(clip.y);
        let right = (point.x + 224.).min(clip.x + clip.w);
        let bottom = (point.y + 224.).min(clip.y + clip.h);
        if right <= left || bottom <= top {
            return;
        }
        let scale = vec2(texture.width() / 224., texture.height() / 224.);
        gl_use_material(&self.target_material);
        draw_texture_ex(
            texture,
            left,
            top,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(right - left, bottom - top)),
                source: Some(Rect::new(
                    (left - point.x) * scale.x,
                    (top - point.y) * scale.y,
                    (right - left) * scale.x,
                    (bottom - top) * scale.y,
                )),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    pub fn load(content: &Content, extracted: &Path) -> Result<Self, ContentError> {
        let mut terrain = HashMap::new();
        let mut icons = HashMap::new();
        let mut target_patterns = HashMap::new();
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
        let mut information_dialog = Vec::new();
        let mut appearance_dialog = Vec::new();
        let mut character_strings = std::collections::BTreeMap::new();
        let mut item_dialog = Vec::new();
        let mut scan_dialog = Vec::new();
        let mut upgrade_names = Vec::new();
        let mut upgrade_formulas = Vec::new();
        let mut main_controls = HashMap::new();
        let mut dialogs = HashMap::new();
        let mut dialog_sizes = HashMap::new();
        let mut reference_strings = HashMap::new();
        let mut preference_descriptions = Vec::new();
        let mut dialog_pictures = std::collections::HashSet::new();
        for resource in &content.resources {
            if resource.original.ends_with("/Cargo Pod")
                && resource.r#type == "STR#"
                && matches!(resource.id, 128 | 137 | 138 | 1201)
            {
                reference_strings.insert(
                    resource.id,
                    ui::string_list(&read_resource(extracted, resource)?)?,
                );
            }
            if resource.original.ends_with("/Cargo Pod")
                && resource.r#type == "DITL"
                && matches!(
                    resource.id,
                    128 | 131 | 132 | 133 | 136 | 148 | 153 | 158 | 159 | 161 | 167 | 177 | 190
                )
            {
                let items = ui::dialog_items(&read_resource(extracted, resource)?)?;
                for item in &items {
                    if item.kind == 64 && item.data.len() == 2 {
                        let id = i16::from_be_bytes(item.data[..2].try_into().unwrap());
                        if id > 0 {
                            dialog_pictures.insert(id);
                        }
                    }
                }
                dialogs.insert(resource.id, items);
            }
            if resource.original.ends_with("/Cargo Pod")
                && resource.r#type == "DLOG"
                && matches!(
                    resource.id,
                    128 | 131 | 132 | 133 | 136 | 148 | 153 | 158 | 159 | 161 | 167 | 177 | 190
                )
            {
                let bytes = read_resource(extracted, resource)?;
                dialog_sizes.insert(
                    resource.id,
                    vec2(
                        (centurions_content::word(&bytes, 6)?
                            - centurions_content::word(&bytes, 2)?) as f32,
                        (centurions_content::word(&bytes, 4)?
                            - centurions_content::word(&bytes, 0)?) as f32,
                    ),
                );
            }
        }
        for resource in &content.resources {
            let cargo = resource.original.ends_with("/Cargo Pod");
            if cargo && resource.r#type == "STR#" && resource.id == 6001 {
                preference_descriptions = ui::string_list(&read_resource(extracted, resource)?)?;
            }
            if resource.r#type == "STR#" && !resource.original.contains("/Scenarios/") {
                character_strings.insert(resource.id, read_resource(extracted, resource)?);
            }
            if cargo && resource.r#type == "DITL" && resource.id == 138 {
                information_dialog = ui::dialog_items(&read_resource(extracted, resource)?)?;
            }
            if cargo && resource.r#type == "DITL" && resource.id == 170 {
                appearance_dialog = ui::dialog_items(&read_resource(extracted, resource)?)?;
            }
            if cargo
                && resource.r#type == "CNTL"
                && ((300..=309).contains(&resource.id) || matches!(resource.id, 168 | 169))
            {
                let bytes = read_resource(extracted, resource)?;
                let top = f32::from(centurions_content::word(&bytes, 0)?);
                let left = f32::from(centurions_content::word(&bytes, 2)?);
                let bottom = f32::from(centurions_content::word(&bytes, 4)?);
                let right = f32::from(centurions_content::word(&bytes, 6)?);
                // ToolBoxInit131978 and mainscreeninit: large mode offsets160/140 minus32, plus our menu20.
                main_controls.insert(
                    resource.id,
                    Rect::new(left + 160., top + 128., right - left, bottom - top),
                );
            }
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
                    || APPLICATION_PICTURES.contains(&resource.id)
                    || dialog_pictures.contains(&resource.id));
            let portrait = resource.original.ends_with("/Portraits") && resource.r#type == "cicn";
            let icon =
                (cargo || resource.original.ends_with("/Tacticals")) && resource.r#type == "cicn";
            if !picture && !portrait && !icon {
                continue;
            }
            let bytes = read_resource(extracted, resource)?;
            if cargo && resource.r#type == "cicn" && (10000..=10050).contains(&resource.id) {
                let bitmap = graphics::decode_cicn_bitmap(&bytes)?;
                let texture = Texture2D::from_rgba8(bitmap.width, bitmap.height, &bitmap.rgba);
                texture.set_filter(FilterMode::Nearest);
                target_patterns.insert(resource.id, texture);
            }
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
            || (300..=309)
                .chain([168, 169])
                .any(|id| !main_controls.contains_key(&id))
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
        let native_text = centurions_content::native_text::NativeText::load(content, extracted)?;
        menus.get_mut(&136).unwrap().items[7].text = native_text.text(0x403670, true)?;
        let font = font.ok_or_else(|| ContentError::Invalid("Textile font missing".into()))?;
        let target_material = load_material(
            macroquad::miniquad::ShaderSource::Glsl {
                vertex: r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1); uv = texcoord; }
"#,
                fragment: r#"#version 100
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() { gl_FragColor = texture2D(Texture, uv); }
"#,
            },
            MaterialParams {
                pipeline_params: macroquad::miniquad::PipelineParams {
                    color_blend: Some(macroquad::miniquad::BlendState::new(
                        macroquad::miniquad::Equation::Add,
                        macroquad::miniquad::BlendFactor::OneMinusValue(
                            macroquad::miniquad::BlendValue::DestinationColor,
                        ),
                        macroquad::miniquad::BlendFactor::OneMinusValue(
                            macroquad::miniquad::BlendValue::SourceColor,
                        ),
                    )),
                    alpha_blend: Some(macroquad::miniquad::BlendState::new(
                        macroquad::miniquad::Equation::Add,
                        macroquad::miniquad::BlendFactor::Zero,
                        macroquad::miniquad::BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|error| ContentError::Invalid(format!("target overlay: {error}")))?;
        let mut assets = Self {
            base_panels: panels.clone(),
            base_icons: icons.clone(),
            reference_strings,
            native_text,
            terrain,
            icons,
            target_patterns,
            target_material,
            portraits,
            panels,
            movie_background: movie_background
                .ok_or_else(|| ContentError::Invalid("movie pixel pattern 131 missing".into()))?,
            menus,
            menu_bar,
            symbol_font: font.clone(),
            original_font: font.clone(),
            menu_font: std::env::var_os("WINDIR")
                .and_then(|root| std::fs::read(Path::new(&root).join("Fonts/segoeui.ttf")).ok())
                .and_then(|bytes| load_ttf_font_from_bytes(&bytes).ok())
                .unwrap_or_else(get_default_font),
            font,
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
            information_dialog,
            appearance_dialog,
            character_strings,
            item_dialog,
            scan_dialog,
            upgrade_names,
            upgrade_formulas,
            main_controls,
            dialogs,
            dialog_sizes,
            preference_descriptions,
        };
        let characters = Font::ascii_character_list();
        for size in [12, 14, 24] {
            assets.font.populate_font_cache(&characters, size);
        }
        assets.select_content(content, extracted)?;
        Ok(assets)
    }
    pub fn set_artistic_font(&mut self, enabled: bool) {
        self.font = if enabled {
            self.original_font.clone()
        } else {
            self.menu_font.clone()
        };
    }
    pub fn select_content(&mut self, content: &Content, root: &Path) -> Result<(), ContentError> {
        let mut names = self.names.clone();
        names.retain(|id, _| *id < 800);
        let mut strings = self.character_strings.clone();
        strings.remove(&801);
        let mut panels = self.base_panels.clone();
        let mut icons = self.base_icons.clone();
        if let Some(scenario) = &content.scenario {
            let owner = format!(
                "TheNewCenturions/Scenarios/{}/{}/Scenario",
                scenario.name, scenario.level
            );
            for resource in content
                .resources
                .iter()
                .filter(|r| r.r#type == "STR#" && r.id == 801 && r.original == owner)
            {
                let bytes = read_resource(root, resource)?;
                for (index, name) in ui::string_prefix(&bytes, 200)?.into_iter().enumerate() {
                    names.insert(800 + index as i16, name);
                }
                strings.insert(801, bytes);
            }
            let owner = format!(
                "TheNewCenturions/Scenarios/{}/{}/Scenario",
                scenario.name, scenario.level
            );
            for resource in content.resources.iter().filter(|resource| {
                resource.original == owner && matches!(resource.r#type.as_str(), "PICT" | "cicn")
            }) {
                let bytes = read_resource(root, resource)?;
                let decoded = if resource.r#type == "PICT" {
                    graphics::decode_pict(&bytes)?
                } else {
                    graphics::decode_cicn(&bytes)?
                };
                let texture = Texture2D::from_rgba8(decoded.width, decoded.height, &decoded.rgba);
                texture.set_filter(FilterMode::Nearest);
                if resource.r#type == "PICT" {
                    panels.insert(resource.id, texture);
                } else {
                    icons.insert(resource.id, texture);
                }
            }
        }
        self.names = names;
        self.character_strings = strings;
        self.panels = panels;
        self.icons = icons;
        Ok(())
    }
    pub fn menu_title(&self, resource_id: i16) -> Option<&str> {
        let title = self.menus.get(&resource_id)?.title.as_str();
        // Textile sfnt 30600 maps the Apple character to glyph 210 in its Unicode cmap.
        Some(if title == "\u{14}" { "\u{f8ff}" } else { title })
    }
    pub fn name(&self, id: i16) -> String {
        self.names.get(&id).cloned().unwrap_or_default()
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
    pub fn dialog_origin(&self, id: i16) -> Vec2 {
        let size = self
            .dialog_sizes
            .get(&id)
            .copied()
            .unwrap_or(vec2(640., 460.));
        vec2(
            ((800. - size.x) / 2.).max(0.),
            20. + ((580. - size.y) / 2.).max(0.),
        )
    }
    pub fn dialog_rect(&self, id: i16, item: usize) -> Rect {
        let origin = self.dialog_origin(id);
        let [top, left, bottom, right] = self.dialogs[&id][item - 1].rect;
        Rect::new(
            origin.x + left as f32,
            origin.y + top as f32,
            (right - left) as f32,
            (bottom - top) as f32,
        )
    }
    pub fn draw_dialog(&self, id: i16) {
        let origin = self.dialog_origin(id);
        let size = self.dialog_sizes[&id];
        draw_rectangle(origin.x, origin.y, size.x, size.y, BLACK);
        let port = Rect::new(origin.x, origin.y, size.x, size.y);
        for (i, item) in self.dialogs[&id].iter().enumerate() {
            let rect = self.dialog_rect(id, i + 1);
            let Some(clip) = rect.intersect(port) else {
                continue;
            };
            if item.kind & 0x7f == 64 && item.data.len() == 2 {
                let picture = i16::from_be_bytes(item.data[..2].try_into().unwrap());
                if let Some(image) = self.panels.get(&picture) {
                    draw_texture_ex(
                        image,
                        clip.x,
                        clip.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(clip.size()),
                            source: Some(Rect::new(
                                (clip.x - rect.x) / rect.w * image.width(),
                                (clip.y - rect.y) / rect.h * image.height(),
                                clip.w / rect.w * image.width(),
                                clip.h / rect.h * image.height(),
                            )),
                            ..Default::default()
                        },
                    );
                }
            }
        }
        for (i, item) in self.dialogs[&id].iter().enumerate() {
            if item.kind & 0x7f == 8
                && !item.text().contains('^')
                && let Some(clip) = self.dialog_rect(id, i + 1).intersect(port)
            {
                self.wrapped_color(&item.text(), clip, 12, 14., YELLOW);
            }
        }
    }
    pub fn text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
        self.draw_text(text, x, y, size, color, &self.font);
    }
    pub fn menu_text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
        self.draw_text(text, x, y, size, color, &self.menu_font);
    }
    fn draw_text(&self, text: &str, x: f32, y: f32, size: u16, color: Color, font: &Font) {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(if text == "\u{f8ff}" {
                    &self.symbol_font
                } else {
                    font
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
        let mut y = rect.y + f32::from(size);
        let mut line = String::new();
        let paragraphs = text.replace("\r\n", "\n").replace('\r', "\n");
        for paragraph in paragraphs.split('\n') {
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
