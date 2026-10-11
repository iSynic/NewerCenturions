use centurions_content::{Content, read_resource, ui};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn raw_menu(id: i16) -> Vec<u8> {
    std::fs::read(root().join(format!("analysis/extracted/resources/74/4d454e55_{id}.bin")))
        .unwrap()
}

#[test]
fn supplied_menu_resources_preserve_original_titles_items_and_attributes() {
    let extracted = root().join("analysis/extracted");
    let content = Content::load(&root().join("NewCenturions"), &extracted).unwrap();
    let expected: &[(i16, i16, u32, &str, &[&str])] = &[
        (
            128,
            128,
            0xfffffff7,
            "\u{14}",
            &["About New Centurions™", "Print Order Form", "-"],
        ),
        (
            129,
            129,
            0xffffffc3,
            "Game",
            &[
                "Load A Saved Game",
                "Revert To A Previous Game",
                "Save Current Game",
                "Fast Save",
                "-",
                "Quit",
            ],
        ),
        (
            130,
            130,
            0xffff6027,
            "Mission",
            &[
                "Begin New Mission",
                "View Current Mission Briefing",
                "End This Mission",
                "-",
                "The Thunder of Guns",
                "Firefight",
                "Armagedons Tears",
                "Children of the Sun",
                "Cross of Chains",
                "Death of the Dragoons",
                "The Price of Change",
                "Humongous Fungus",
                "\0",
                "\0",
                "-",
            ],
        ),
        (131, 131, 0xffffffff, "\0", &[]),
        (132, 132, 0xfffffffe, "Archives", &[]),
        (
            133,
            133,
            0xfffffffe,
            "New Centurions™ Copyright ©2001, Tim Phillips.  Please pay if you play.",
            &["\0"],
        ),
        (
            134,
            134,
            0xffffffff,
            "Game Speed",
            &[
                "Lets Open Her Up!",
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
                "9",
                "ZZzzZZzzZzZz",
            ],
        ),
        (
            135,
            135,
            0xffff9fff,
            "Sound/Music Volume",
            &[
                ">>> Sound F/X Volume <<<",
                "Let's Hunt Wabbits.",
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
                "9",
                "Wake the Neighbors!",
                "-",
                "-",
                ">>> Music Volume <<<",
                "Let me listen to my hard drive spin.",
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
                "9",
                "Damage my eardrums, I'm an impetuous youth!",
            ],
        ),
        (
            136,
            136,
            0xffffff21,
            "Character",
            &[
                "Squad Order",
                "Change Warrior Portrait",
                "Change Warrior Icon",
                "-",
                "Generate New Warrior",
                "-",
                "Modify Squad",
                "Warrior Edtior (DEMO)",
            ],
        ),
        (
            137,
            137,
            0xffff7e3f,
            "Preferences",
            &[
                "Sound Volume",
                "Game Speed",
                "Set Preferences",
                "Refresh Screen",
                "Start Popup Help",
                "-",
                "-",
                "-",
                "Version Information",
                "New Centurions - ",
                "Items - ",
                "Scans - ",
                "Races - ",
                "M.O.S. - ",
                "-",
            ],
        ),
        (138, 128, 0xffffffff, "\0", &["\0"]),
        (
            145,
            145,
            0xfffffddb,
            "Music",
            &[
                "Stop Playing Music",
                "-",
                "Music Title Currently Playing :",
                "Music Disabled, Select 'Start Playing Music' To Resume",
                "-",
                "Playlist Control:",
                "Select from playlist to play that title now.",
                "Hold SHIFT and select title to add or remove from playlist.",
                "-",
                "New Centurions Theme",
                "Ambush",
                "Unholy Experiment",
                "Stasis Chamber",
                "Cyborg Serenade",
                "Drums Of The Apocalypse",
                "Crush Thy Enemy",
                "Space Debris",
                "Plasma",
                "Die Hard The Hunter",
                "Rebellion Of The Machines",
                "Thirst",
                "Energy Overload",
            ],
        ),
        (146, 146, 0xffe00001, "Allies", &["\0"; 20]),
        (
            200,
            200,
            0xfffffff3,
            "Maps/Notes",
            &["Mission Briefing", "-", "-"],
        ),
        (1000, 1000, 0xffffffff, "\0", &["\0"]),
    ];
    let mut reached = Vec::new();
    for resource in content.resources.iter().filter(|resource| {
        resource.r#type == "MENU" && resource.original == "TheNewCenturions/Data Files/Cargo Pod"
    }) {
        let (_, menu_id, flags, title, texts) = expected
            .iter()
            .find(|entry| entry.0 == resource.id)
            .unwrap();
        let bytes = read_resource(&extracted, resource).unwrap();
        let menu = ui::menu(&bytes).unwrap();
        assert_eq!((menu.id, menu.enable_flags), (*menu_id, *flags));
        assert_eq!(
            (menu.width, menu.height, menu.definition_id, menu.reserved),
            (0, 0, 0, 0)
        );
        assert_eq!(menu.title, *title);
        assert_eq!(
            menu.items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            *texts
        );
        assert_eq!(menu.initially_enabled(), *flags & 1 != 0);
        assert_eq!(menu.initially_item_enabled(0), None);
        assert_eq!(menu.initially_item_enabled(texts.len() + 1), None);
        for (index, item) in menu.items.iter().enumerate() {
            let number = index + 1;
            let attributes = match (resource.id, number) {
                (129, 1) => [0, b'L', 0, 0],
                (129, 2) => [0, b'R', 0, 0],
                (129, 3) => [0, b'S', 0, 0],
                (129, 4) => [0, b'F', 0, 0],
                (129, 6) => [0, b'Q', 0, 0],
                (130, 1) => [0, b'B', 0, 0],
                (130, 2) => [0, b'V', 0, 0],
                (130, 3) => [0, b'E', 0, 0],
                (133, 1) => [0, 0, 0, 1],
                (135, 2 | 16) => [124, 0, 0, 0],
                (135, 12 | 26) => [125, 0, 0, 0],
                (136, 1) => [0, b'O', 0, 0],
                (136, 2) => [0, b'C', 0, 0],
                (136, 3) => [0, b'I', 0, 0],
                (136, 5) => [0, b'G', 0, 0],
                (136, 7) => [0, b'M', 0, 0],
                (136, 8) => [122, b'W', 0, 0],
                (137, 1) => [0, 0x1b, 135, 0],
                (137, 2) => [0, 0x1b, 134, 0],
                (137, 3) => [0, b'P', 0, 0],
                (137, 4) => [0, b'K', 0, 0],
                (137, 9) => [0, 0, 0, 0x46],
                (200, 1) => [126, b'B', 0, 0],
                _ => [0; 4],
            };
            assert_eq!([item.icon, item.key, item.mark, item.style], attributes);
            assert_eq!(item.is_separator(), texts[index] == "-");
            assert_eq!(
                item.submenu_id(),
                match (resource.id, number) {
                    (137, 1) => Some(135),
                    (137, 2) => Some(134),
                    _ => None,
                }
            );
            assert_eq!(
                menu.initially_item_enabled(number),
                Some(*flags & (1 << number) != 0)
            );
        }
        reached.push(resource.id);
    }
    reached.sort();
    assert_eq!(
        reached,
        expected.iter().map(|entry| entry.0).collect::<Vec<_>>()
    );
}

#[test]
fn supplied_menu_bars_preserve_resource_order_and_distinct_menu_ids() {
    let extracted = root().join("analysis/extracted");
    let content = Content::load(&root().join("NewCenturions"), &extracted).unwrap();
    let mut reached = Vec::new();
    for resource in content.resources.iter().filter(|resource| {
        resource.r#type == "MBAR" && resource.original == "TheNewCenturions/Data Files/Cargo Pod"
    }) {
        let expected: &[i16] = match resource.id {
            128 => &[128, 129, 130, 136, 132, 146, 200, 137, 145],
            129 => &[138, 133],
            _ => panic!("unexpected menu bar {}", resource.id),
        };
        assert_eq!(
            ui::menu_bar(&read_resource(&extracted, resource).unwrap()).unwrap(),
            expected
        );
        reached.push(resource.id);
    }
    reached.sort();
    assert_eq!(reached, [128, 129]);
    assert_eq!(ui::menu(&raw_menu(138)).unwrap().id, 128);
}

#[test]
fn malformed_menu_resources_return_diagnostics() {
    for id in [128, 129, 130, 131, 137, 146] {
        let bytes = raw_menu(id);
        for end in 0..bytes.len() {
            assert!(ui::menu(&bytes[..end]).is_err(), "MENU {id} prefix {end}");
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(ui::menu(&trailing).is_err());
    }
    let mut custom = raw_menu(129);
    custom[6..8].copy_from_slice(&1_i16.to_be_bytes());
    assert!(
        ui::menu(&custom)
            .unwrap_err()
            .to_string()
            .contains("unsupported menu definition 1")
    );
    let mut long_title = raw_menu(131);
    long_title[14] = 255;
    assert!(ui::menu(&long_title).is_err());
    for bytes in [
        &[][..],
        &[0],
        &[0xff, 0xff],
        &[0, 1],
        &[0, 1, 0],
        &[0, 0, 0, 0],
    ] {
        assert!(ui::menu_bar(bytes).is_err());
    }
}

#[test]
fn standard_menu_flags_and_mac_roman_keep_control_and_extended_bytes() {
    let mut bytes = vec![0; 14];
    bytes[..2].copy_from_slice(&(-12_i16).to_be_bytes());
    bytes[10..14].copy_from_slice(&0x80000000_u32.to_be_bytes());
    bytes.extend_from_slice(&[2, 0x80, 0xa9]);
    for _ in 0..33 {
        bytes.extend_from_slice(&[1, 0, 255, b'Z', 0xa5, 0x7f]);
    }
    bytes.push(0);
    let menu = ui::menu(&bytes).unwrap();
    assert_eq!(menu.id, -12);
    assert_eq!(menu.title, "Ä©");
    assert!(!menu.initially_enabled());
    assert_eq!(menu.initially_item_enabled(1), Some(false));
    assert_eq!(menu.initially_item_enabled(31), Some(true));
    assert_eq!(menu.initially_item_enabled(32), Some(true));
    assert_eq!(menu.initially_item_enabled(33), Some(true));
    assert_eq!(menu.initially_item_enabled(34), None);
    assert_eq!(menu.items[0].text, "\0");
    assert_eq!(
        [
            menu.items[0].icon,
            menu.items[0].key,
            menu.items[0].mark,
            menu.items[0].style
        ],
        [255, b'Z', 0xa5, 0x7f]
    );
    assert!(ui::menu_bar(&[0, 0]).unwrap().is_empty());
}
