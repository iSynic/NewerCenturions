use crate::{ContentError, span, word};

pub const PANEL_PICTURES: &[i16] = &[
    153, 154, 156, 166, 174, 228, 168, 171, 172, 189, 197, 199, 220, 224, 227,
];

pub fn c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|v| *v == 0).unwrap_or(bytes.len());
    encoding_rs::MACINTOSH.decode(&bytes[..end]).0.into_owned()
}

pub fn pascal_text(bytes: &[u8]) -> Result<String, ContentError> {
    let length = usize::from(span(bytes, 0, 1)?[0]);
    Ok(encoding_rs::MACINTOSH
        .decode(span(bytes, 1, length)?)
        .0
        .into_owned())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    pub id: i16,
    pub width: i16,
    pub height: i16,
    pub definition_id: i16,
    pub reserved: u16,
    pub enable_flags: u32,
    pub title: String,
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn initially_enabled(&self) -> bool {
        self.enable_flags & 1 != 0
    }

    // MENU flags describe initial state; the original program updates them at runtime.
    pub fn initially_item_enabled(&self, item_number: usize) -> Option<bool> {
        if item_number == 0 || item_number > self.items.len() {
            return None;
        }
        Some(item_number > 31 || self.enable_flags & (1 << item_number) != 0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub text: String,
    pub icon: u8,
    pub key: u8,
    pub mark: u8,
    pub style: u8,
}

impl MenuItem {
    pub fn is_separator(&self) -> bool {
        self.text == "-"
    }

    pub fn submenu_id(&self) -> Option<i16> {
        (self.key == 0x1b).then_some(i16::from(self.mark))
    }
}

fn pascal_string(bytes: &[u8], cursor: &mut usize) -> Result<String, ContentError> {
    let length = usize::from(span(bytes, *cursor, 1)?[0]);
    *cursor += 1;
    let text = encoding_rs::MACINTOSH
        .decode(span(bytes, *cursor, length)?)
        .0
        .into_owned();
    *cursor += length;
    Ok(text)
}

pub fn menu(bytes: &[u8]) -> Result<Menu, ContentError> {
    let header = span(bytes, 0, 14)?;
    let definition_id = word(header, 6)?;
    if definition_id != 0 {
        return Err(ContentError::Invalid(format!(
            "unsupported menu definition {definition_id}"
        )));
    }
    let mut cursor = 14;
    let title = pascal_string(bytes, &mut cursor)?;
    let mut items = Vec::new();
    while span(bytes, cursor, 1)?[0] != 0 {
        let text = pascal_string(bytes, &mut cursor)?;
        let attributes = span(bytes, cursor, 4)?;
        items.push(MenuItem {
            text,
            icon: attributes[0],
            key: attributes[1],
            mark: attributes[2],
            style: attributes[3],
        });
        cursor += 4;
    }
    if cursor + 1 != bytes.len() {
        return Err(ContentError::Invalid(
            "trailing data after menu terminator".into(),
        ));
    }
    Ok(Menu {
        id: word(header, 0)?,
        width: word(header, 2)?,
        height: word(header, 4)?,
        definition_id,
        reserved: u16::from_be_bytes([header[8], header[9]]),
        enable_flags: u32::from_be_bytes([header[10], header[11], header[12], header[13]]),
        title,
        items,
    })
}

pub fn menu_bar(bytes: &[u8]) -> Result<Vec<i16>, ContentError> {
    let count = word(bytes, 0)?;
    if count < 0 {
        return Err(ContentError::Invalid("negative menu-bar count".into()));
    }
    let count = count as usize;
    if bytes.len() != 2 + count * 2 {
        return Err(ContentError::Invalid(
            "menu-bar count does not match resource size".into(),
        ));
    }
    (0..count).map(|index| word(bytes, 2 + index * 2)).collect()
}

pub fn string_list(bytes: &[u8]) -> Result<Vec<String>, ContentError> {
    let count = word(bytes, 0)?;
    if count < 0 {
        return Err(ContentError::Invalid("negative string-list count".into()));
    }
    string_prefix(bytes, count as usize)
}

#[derive(Clone, Debug)]
pub struct DialogItem {
    pub rect: [i16; 4],
    pub kind: u8,
    pub data: Vec<u8>,
}
impl DialogItem {
    pub fn text(&self) -> String {
        encoding_rs::MACINTOSH.decode(&self.data).0.into_owned()
    }
}

pub fn dialog_items(bytes: &[u8]) -> Result<Vec<DialogItem>, ContentError> {
    let count = word(bytes, 0)?;
    if count < -1 {
        return Err(ContentError::Invalid("negative dialog-item count".into()));
    }
    let mut cursor = 2;
    let mut items = Vec::new();
    for _ in 0..=count {
        let kind = span(bytes, cursor, 13)?[12] & 0x7f;
        let rect = [
            word(bytes, cursor + 4)?,
            word(bytes, cursor + 6)?,
            word(bytes, cursor + 8)?,
            word(bytes, cursor + 10)?,
        ];
        cursor += 13;
        let length = usize::from(span(bytes, cursor, 1)?[0]);
        let data = span(bytes, cursor + 1, length)?.to_vec();
        cursor += 1 + length + length % 2;
        items.push(DialogItem { rect, kind, data });
    }
    Ok(items)
}

pub fn dialog_strings(bytes: &[u8]) -> Result<Vec<String>, ContentError> {
    let mut strings = Vec::new();
    for item in dialog_items(bytes)? {
        let kind = item.kind;
        if kind == 8 || kind == 16 {
            strings.push(
                encoding_rs::MACINTOSH
                    .decode(&item.data)
                    .0
                    .replace('\r', "\n"),
            );
        } else {
            strings.push(String::new());
        }
    }
    Ok(strings)
}

pub fn string_prefix(bytes: &[u8], count: usize) -> Result<Vec<String>, ContentError> {
    if i32::from(word(bytes, 0)?) < count as i32 {
        return Err(ContentError::Invalid(
            "string-list prefix exceeds declared count".into(),
        ));
    }
    let mut cursor = 2;
    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let length = usize::from(span(bytes, cursor, 1)?[0]);
        cursor += 1;
        strings.push(
            encoding_rs::MACINTOSH
                .decode(span(bytes, cursor, length)?)
                .0
                .into_owned(),
        );
        cursor += length;
    }
    Ok(strings)
}
