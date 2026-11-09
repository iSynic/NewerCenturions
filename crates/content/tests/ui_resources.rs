use centurions_content::{Content, Resource, read_resource, ui};
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
    let menu_bar =
        |r: &&Resource| r.r#type == "MBAR" && r.id == 128 && r.original.ends_with("/Cargo Pod");
    let bar = content.resources.iter().find(menu_bar).unwrap();
    assert_eq!(
        ui::menu_bar(&read_resource(&extracted, bar).unwrap()).unwrap(),
        [128, 129, 130, 136, 132, 146, 200, 137, 145]
    );
    assert_eq!(ui::menu(&raw_menu(138)).unwrap().id, 128);
}
