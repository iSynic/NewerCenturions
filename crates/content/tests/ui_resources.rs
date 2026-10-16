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
fn native_menu_hierarchy_preserves_submenus_shortcuts_and_resource_aliases() {
    let menu = ui::menu(&raw_menu(137)).unwrap();
    assert_eq!(menu.title, "Preferences");
    assert_eq!(menu.items[0].submenu_id(), Some(135));
    assert_eq!(menu.items[1].submenu_id(), Some(134));
    assert_eq!(menu.items[2].key, b'P');
    let extracted = root().join("analysis/extracted");
    let content = Content::load(&root().join("NewCenturions"), &extracted).unwrap();
    let bar = content
        .resources
        .iter()
        .find(|resource| {
            resource.r#type == "MBAR"
                && resource.id == 128
                && resource.original == "TheNewCenturions/Data Files/Cargo Pod"
        })
        .unwrap();
    assert_eq!(
        ui::menu_bar(&read_resource(&extracted, bar).unwrap()).unwrap(),
        [128, 129, 130, 136, 132, 146, 200, 137, 145]
    );
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
