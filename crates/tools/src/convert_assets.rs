//! Converts the portrait, tactical and Cargo Pod pictures into PNG files with a
//! digest manifest under `<root>/analysis/assets`.
use centurions_content::{Content, digest, graphics, read_resource};
use std::{error::Error, fs, path::Path};

/// `PICT` resource ids from the Cargo Pod file that are converted alongside the `cicn` color
/// icons of the portrait, tactical and Cargo Pod files. The list is a membership filter, so its
/// order does not affect the output.
const CARGO_POD_PICT_IDS: &[i16] = &[
    0, 7, 132, 142, 143, 144, 153, 154, 155, 156, 166, 158, 160, 161, 162, 148, 168, 170, 171, 172,
    174, 175, 183, 184, 185, 186, 189, 196, 176, 197, 202, 198, 199, 200, 201, 205, 206, 208, 212,
    220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 300, 301, 302,
    303, 304, 305, 306, 307, 308,
];

/// Resource files whose pictures are converted, as lowercased file names with
/// spaces replaced by underscores; each is also the output subdirectory.
const GROUPS: [&str; 3] = ["portraits", "tacticals", "cargo_pod"];

/// Writes the converted PNG files and `manifest.json`, returning how many
/// pictures were converted.
pub(crate) fn run(
    content: &Content,
    resource_root: &Path,
    root: &Path,
) -> Result<usize, Box<dyn Error>> {
    let output = root.join("analysis/assets");
    let mut receipts = Vec::new();
    for resource in content.resources.iter().filter(|r| {
        r.r#type == "cicn"
            || (r.r#type == "PICT"
                && CARGO_POD_PICT_IDS.contains(&r.id)
                && r.original.ends_with("/Cargo Pod"))
    }) {
        let group = Path::new(&resource.original)
            .file_name()
            .ok_or("resource name missing")?
            .to_string_lossy()
            .to_lowercase()
            .replace(' ', "_");
        if !GROUPS.contains(&group.as_str()) {
            continue;
        }
        let bytes = read_resource(resource_root, resource)?;
        let image = if resource.r#type == "PICT" {
            graphics::decode_pict(&bytes)?
        } else {
            graphics::decode_cicn(&bytes)?
        };
        let relative = format!("{group}/{}_{}.png", resource.r#type, resource.id);
        let path = output.join(&relative);
        fs::create_dir_all(path.parent().unwrap())?;
        let mut encoder = png::Encoder::new(
            fs::File::create(&path)?,
            u32::from(image.width),
            u32::from(image.height),
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&image.rgba)?;
        receipts.push(serde_json::json!({ "source":resource.original,"type":resource.r#type,"id":resource.id,"source_sha256":resource.sha256,
            "output":relative,"output_sha256":digest(&fs::read(path)?),"width":image.width,"height":image.height }));
    }
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"schema_version":1,"content_identity":content.identity,"assets":receipts}),
        )?,
    )?;
    Ok(receipts.len())
}
