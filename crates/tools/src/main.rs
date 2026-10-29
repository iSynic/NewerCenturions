use centurions_content::installation::Installation;
use centurions_core::{Command, CommandPreferences, Session, apply};
use std::path::PathBuf;
mod acceptance;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let action = args.next().unwrap_or_else(|| "inspect".into());
    let root = args
        .next()
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("CENTURIONS_ASSET_ROOT").map(PathBuf::from))
        .unwrap_or_else(Installation::default_root);
    let installation = Installation::open(&root)?;
    let content = installation.content;
    let resource_root = installation.resource_root;
    match action.as_str() {
        "audit-resources" => {
            println!("{}", acceptance::resources(&content, &resource_root));
        }
        "audit-import" => {
            let original = PathBuf::from(
                args.next()
                    .ok_or("audit-import needs the original Data I1 path")?,
            );
            let commands = PathBuf::from(args.next().ok_or("audit-import needs command JSON")?);
            let work = PathBuf::from(
                args.next()
                    .ok_or("audit-import needs a new analysis work directory")?,
            );
            println!(
                "{}",
                acceptance::import(&content, &original, &commands, &work)?
            );
        }
        "convert-assets" => {
            let output = root.join("analysis/assets");
            let mut receipts = Vec::new();
            for resource in content.resources.iter().filter(|r| {
                r.r#type == "cicn"
                    || (r.r#type == "PICT"
                        && [
                            0, 7, 132, 142, 143, 144, 153, 154, 155, 156, 166, 158, 160, 161, 162,
                            148, 153, 168, 170, 171, 172, 174, 175, 183, 184, 185, 186, 189, 196,
                            176, 197, 202, 198, 199, 200, 201, 205, 206, 208, 212, 220, 221, 222,
                            223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 300,
                            301, 302, 303, 304, 305, 306, 307, 308,
                        ]
                        .contains(&r.id)
                        && r.original.ends_with("/Cargo Pod"))
            }) {
                let group = std::path::Path::new(&resource.original)
                    .file_name()
                    .ok_or("resource name missing")?
                    .to_string_lossy()
                    .to_lowercase()
                    .replace(' ', "_");
                if !["portraits", "tacticals", "cargo_pod"].contains(&group.as_str()) {
                    continue;
                }
                let bytes = centurions_content::read_resource(&resource_root, resource)?;
                let image = if resource.r#type == "PICT" {
                    centurions_content::graphics::decode_pict(&bytes)?
                } else {
                    centurions_content::graphics::decode_cicn(&bytes)?
                };
                let relative = format!("{group}/{}_{}.png", resource.r#type, resource.id);
                let path = output.join(&relative);
                std::fs::create_dir_all(path.parent().unwrap())?;
                let mut encoder = png::Encoder::new(
                    std::fs::File::create(&path)?,
                    u32::from(image.width),
                    u32::from(image.height),
                );
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.write_header()?.write_image_data(&image.rgba)?;
                receipts.push(serde_json::json!({ "source":resource.original,"type":resource.r#type,"id":resource.id,"source_sha256":resource.sha256,
                    "output":relative,"output_sha256":centurions_content::digest(&std::fs::read(path)?),"width":image.width,"height":image.height }));
            }
            std::fs::write(
                output.join("manifest.json"),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"schema_version":1,"content_identity":content.identity,"assets":receipts}),
                )?,
            )?;
            println!("Converted {} original color icons", receipts.len());
        }
        "inspect" => {
            println!(
                "Content identity: {}\nItems: {}",
                content.identity,
                content.items.len()
            );
            for warrior in &content.warriors {
                println!(
                    "{}: stamina {}, inventory {}",
                    warrior.name,
                    warrior.stamina()?,
                    warrior.inventory()?.len()
                );
            }
            if let Some(opening) = content.map(0) {
                println!(
                    "Opening mission: {} at {:?}; {} map cells",
                    opening.name,
                    opening.start,
                    opening.floor.len()
                );
            }
        }
        "replay" => {
            let path = PathBuf::from(args.next().ok_or("replay needs command JSON path")?);
            let options: Vec<String> = args.collect();
            let trace = options.iter().any(|arg| arg == "--trace");
            let outputs: Vec<_> = options
                .iter()
                .enumerate()
                .filter(|(_, arg)| *arg == "--save")
                .collect();
            if outputs.len() > 1 {
                return Err("--save may be specified only once".into());
            }
            let output = outputs
                .first()
                .map(|(index, _)| {
                    options
                        .get(index + 1)
                        .filter(|path| !path.starts_with("--"))
                        .map(PathBuf::from)
                        .ok_or("--save needs an output path")
                })
                .transpose()?;
            let commands: Vec<Command> = serde_json::from_slice(&std::fs::read(path)?)?;
            // `--from <save>` starts from a saved session instead of a new game.
            let mut session = match options.iter().position(|arg| arg == "--from") {
                Some(index) => centurions_core::load(
                    &PathBuf::from(options.get(index + 1).ok_or("--from needs a save path")?),
                    &content,
                )?,
                None => Session::new(&content),
            };
            for command in commands {
                let command = if trace { traced(command) } else { command };
                let events = apply(command, &mut session, &content);
                for entry in centurions_core::trace::take() {
                    println!("{}", serde_json::to_string(&entry)?);
                }
                match events {
                    Ok(events) => println!("{}", serde_json::to_string(&events)?),
                    Err(error) => {
                        // Preserve the actual command boundary, including deliberate partial effects.
                        if let Some(output) = &output {
                            centurions_core::save(output, &session, &content)?;
                            println!("{}", serde_json::to_string(&session)?);
                        }
                        return Err(error.into());
                    }
                }
            }
            if let Some(output) = &output {
                centurions_core::save(output, &session, &content)?;
            }
            println!("{}", serde_json::to_string(&session)?);
        }
        _ => return Err(format!("unknown command {action}").into()),
    }
    Ok(())
}

/// Enables the diagnostic trace through the command's preferences. A bare
/// command already runs with default preferences, so wrapping it is neutral.
fn traced(command: Command) -> Command {
    let (command, mut preferences) = match command {
        Command::WithPreferences {
            command,
            preferences,
        } => (command, preferences),
        command => (Box::new(command), CommandPreferences::default()),
    };
    preferences.trace = true;
    Command::WithPreferences {
        command,
        preferences,
    }
}
