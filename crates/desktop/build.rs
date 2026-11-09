//! Gives the Windows program file the game's icon, so Explorer, shortcuts and pinned
//! taskbar buttons show it as well as the open window.
use std::{env, fs, path::PathBuf};

/// The icon pictures in `src/app_icon.rs`, each as its side and RGBA pixels.
fn pictures(source: &str) -> Vec<(u32, Vec<u8>)> {
    source
        .split("pub(crate) const ")
        .skip(1)
        .map(|block| {
            let side: u32 = block
                .split(['[', ';', '*'])
                .nth(2)
                .and_then(|n| n.trim().parse().ok())
                .expect("icon side");
            let body = &block
                [block.find("= [").expect("icon pixels") + 3..block.find("];").expect("icon end")];
            let pixels: Vec<u8> = body
                .split(',')
                .filter_map(|n| n.trim().parse().ok())
                .collect();
            assert_eq!(
                pixels.len() as u32,
                side * side * 4,
                "icon {side} pixel count"
            );
            (side, pixels)
        })
        .collect()
}

/// An icon file holding each picture as a 32-bit bitmap with its transparency mask.
fn icon_file(pictures: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = pictures
        .iter()
        .map(|(side, rgba)| {
            let side = *side as usize;
            let mask_row = side.div_ceil(32) * 4;
            let mut image = Vec::new();
            for value in [40u32, side as u32, side as u32 * 2] {
                image.extend(value.to_le_bytes());
            }
            image.extend(1u16.to_le_bytes());
            image.extend(32u16.to_le_bytes());
            image.extend(0u32.to_le_bytes());
            image.extend(((side * side * 4 + mask_row * side) as u32).to_le_bytes());
            image.extend([0u8; 16]);
            for y in (0..side).rev() {
                for x in 0..side {
                    let p = &rgba[(y * side + x) * 4..][..4];
                    image.extend([p[2], p[1], p[0], p[3]]);
                }
            }
            image.extend(vec![0u8; mask_row * side]);
            image
        })
        .collect();
    let mut file = Vec::new();
    file.extend(0u16.to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend((images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len();
    for ((side, _), image) in pictures.iter().zip(&images) {
        let side = if *side >= 256 { 0 } else { *side as u8 };
        file.extend([side, side, 0, 0]);
        file.extend(1u16.to_le_bytes());
        file.extend(32u16.to_le_bytes());
        file.extend((image.len() as u32).to_le_bytes());
        file.extend((offset as u32).to_le_bytes());
        offset += image.len();
    }
    for image in images {
        file.extend(image);
    }
    file
}

fn main() {
    println!("cargo:rerun-if-changed=src/app_icon.rs");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let source = fs::read_to_string("src/app_icon.rs").expect("src/app_icon.rs");
    let mut pictures = pictures(&source);
    pictures.sort_by_key(|(side, _)| std::cmp::Reverse(*side));
    let icon = out.join("app.ico");
    fs::write(&icon, icon_file(&pictures)).expect("app.ico");
    let script = out.join("app.rc");
    let icon_path = icon.to_string_lossy().replace('\\', "\\\\");
    fs::write(&script, format!("1 ICON \"{icon_path}\"\n")).expect("app.rc");
    embed_resource::compile(&script, embed_resource::NONE)
        .manifest_optional()
        .expect("the program icon resource");
}
