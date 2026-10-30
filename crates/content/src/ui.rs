use crate::{ContentError, span, word};

/// The `PICT` ids of the panel pictures the game draws its controls on, among them the combat
/// panel (153) and the exploration panel (176).
pub const PANEL_PICTURES: &[i16] = &[
    153, 154, 156, 176, 200, 208, 225, 166, 174, 228, 168, 171, 172, 189, 197, 199, 220, 224, 227,
];

/// Reads a zero-terminated string in the game's character set; without a zero the
/// whole slice is the text.
pub fn c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|v| *v == 0).unwrap_or(bytes.len());
    encoding_rs::MACINTOSH.decode(&bytes[..end]).0.into_owned()
}

/// Reads a length-prefixed string in the game's character set.
pub fn pascal_text(bytes: &[u8]) -> Result<String, ContentError> {
    let length = usize::from(span(bytes, 0, 1)?[0]);
    Ok(encoding_rs::MACINTOSH
        .decode(span(bytes, 1, length)?)
        .0
        .into_owned())
}

/// A `MENU` resource: a pull-down menu with its title and items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    /// The menu's resource id.
    pub id: i16,
    /// Width in pixels the menu was authored at.
    pub width: i16,
    /// Height in pixels the menu was authored at.
    pub height: i16,
    /// The menu definition procedure; only 0, the standard menu, is accepted.
    pub definition_id: i16,
    /// A field of the resource that is kept as stored.
    pub reserved: u16,
    /// Which items start enabled: bit 0 is the whole menu and bit n is item n.
    pub enable_flags: u32,
    /// The title shown in the menu bar.
    pub title: String,
    /// The menu's items, in order.
    pub items: Vec<MenuItem>,
}

impl Menu {
    /// Whether the menu starts enabled.
    pub fn initially_enabled(&self) -> bool {
        self.enable_flags & 1 != 0
    }

    // MENU flags give the initial state; the game changes them as it runs.
    /// Whether item `item_number` (counted from one) starts enabled; items past the
    /// 31st always do. `None` for an item the menu lacks.
    pub fn initially_item_enabled(&self, item_number: usize) -> Option<bool> {
        if item_number == 0 || item_number > self.items.len() {
            return None;
        }
        Some(item_number > 31 || self.enable_flags & (1 << item_number) != 0)
    }
}

/// One line of a menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    /// The item's text; a lone hyphen draws a separator line.
    pub text: String,
    /// Icon number shown beside the item; zero is none.
    pub icon: u8,
    /// Command-key equivalent; 0x1b marks an item that opens a submenu.
    pub key: u8,
    /// Mark character drawn beside the item; for a submenu item, the submenu's menu id.
    pub mark: u8,
    /// Text style bits of the item.
    pub style: u8,
}

impl MenuItem {
    /// Whether the item is a separator line rather than a choice.
    pub fn is_separator(&self) -> bool {
        self.text == "-"
    }

    /// The menu id of the submenu this item opens, if it opens one.
    pub fn submenu_id(&self) -> Option<i16> {
        (self.key == 0x1b).then_some(i16::from(self.mark))
    }
}

/// Reads a length-prefixed string at `cursor` and moves the cursor past it.
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

/// Parses a `MENU` resource.
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

/// Parses an `MBAR` resource into the ids of its menus, left to right.
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

/// Parses a `STR#` resource into all of its strings.
pub fn string_list(bytes: &[u8]) -> Result<Vec<String>, ContentError> {
    let count = word(bytes, 0)?;
    if count < 0 {
        return Err(ContentError::Invalid("negative string-list count".into()));
    }
    string_prefix(bytes, count as usize)
}

/// One item of a `DITL` resource.
#[derive(Clone, Debug)]
pub struct DialogItem {
    /// The item's bounds in the dialog: top, left, bottom, right.
    pub rect: [i16; 4],
    /// The item type with its disabled bit removed: 8 is static text and 16 edit text.
    pub kind: u8,
    /// The item's text, or the resource it names, as stored.
    pub data: Vec<u8>,
}
impl DialogItem {
    /// The item's data as text in the game's character set.
    pub fn text(&self) -> String {
        encoding_rs::MACINTOSH.decode(&self.data).0.into_owned()
    }
}

/// Parses a `DITL` resource into its items.
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

/// The text of each item of a `DITL`, with line ends as newlines; items that
/// carry no text give an empty string.
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

/// Parses the first `count` strings of a `STR#` resource; the resource must hold
/// at least that many.
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
