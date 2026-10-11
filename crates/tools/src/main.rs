use centurions_content::Content;
use centurions_core::{Command, Session, apply, load, save};
use std::path::PathBuf;

mod completion;
mod sustained;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let action = args.next().unwrap_or_else(|| "inspect".into());
    let root = args
        .next()
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("CENTURIONS_ASSET_ROOT").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )?;
    match action.as_str() {
        "verify-stage-one" => completion::verify_stage_one(&root, &content)?,
        "verify-samples" => completion::verify_samples(
            &root,
            &content,
            args.next().map(|s| s.parse()).transpose()?,
        )?,
        "verify-laboratory" => completion::verify_laboratory(
            &root,
            &content,
            args.next().map(|s| s.parse()).transpose()?,
        )?,
        "verify-stage-two" => completion::verify_stage_two(
            &root,
            &content,
            args.next().map(|s| s.parse()).transpose()?,
        )?,
        "verify-stage-three" => completion::verify_stage_three(
            &root,
            &content,
            args.next().map(|s| s.parse()).transpose()?,
        )?,
        "verify-sustained" => sustained::verify(&root, &content)?,
        "verify-day-two" => sustained::verify_day_two(&root, &content)?,
        "convert-assets" => {
            let output = root.join("analysis/assets");
            let mut receipts = Vec::new();
            for resource in content.resources.iter().filter(|r| {
                r.r#type == "cicn"
                    || (r.r#type == "PICT"
                        && [
                            153, 154, 155, 156, 166, 158, 160, 161, 162, 168, 171, 172, 174, 183,
                            184, 185, 186, 189, 196, 197, 198, 199, 200, 201, 205, 206, 208, 212,
                            220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233,
                            234, 235, 300, 301, 302, 303, 304, 305, 306, 307, 308,
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
                let bytes =
                    centurions_content::read_resource(&root.join("analysis/extracted"), resource)?;
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
        "verify-reference" => {
            let reference: serde_json::Value = serde_json::from_slice(&std::fs::read(
                root.join("analysis/native/exploration-start.json"),
            )?)?;
            let opening = content.map(0).ok_or("opening content missing")?;
            fn decode(value: &serde_json::Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                let hex = value["hex"].as_str().ok_or("native hex missing")?;
                Ok((0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                    .collect::<Result<Vec<_>, _>>()?)
            }
            let floor: Vec<u8> = opening.floor.iter().flat_map(|v| v.to_be_bytes()).collect();
            let objects: Vec<u8> = opening
                .objects
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect();
            let events: Vec<u8> = opening
                .events
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect();
            for (key, bytes) in [
                ("map_header", &opening.header),
                ("map_regions", &opening.regions),
                ("floor", &floor),
                ("objects", &objects),
                ("events", &events),
                ("tile_properties", &opening.tile_properties),
            ] {
                if *bytes != decode(&reference[key])? {
                    return Err(format!("native {key} differs").into());
                }
            }
            for (i, warrior) in content.warriors.iter().enumerate() {
                if warrior.bytes != decode(&reference["warriors"][i])? {
                    return Err(format!("native warrior {} differs", warrior.name).into());
                }
            }
            println!(
                "Six warriors and all six opening map data regions match the original game byte for byte"
            );
        }
        "replay" => {
            let path = PathBuf::from(args.next().ok_or("replay needs command JSON path")?);
            let commands: Vec<Command> = serde_json::from_slice(&std::fs::read(path)?)?;
            let mut session = Session::new(&content);
            for command in commands {
                println!(
                    "{}",
                    serde_json::to_string(&apply(command, &mut session, &content)?)?
                );
            }
            println!("{}", serde_json::to_string(&session)?);
        }
        "verify-movement" => {
            let before: serde_json::Value = serde_json::from_slice(&std::fs::read(
                root.join("analysis/native/before-prologue3.json"),
            )?)?;
            let after: serde_json::Value = serde_json::from_slice(&std::fs::read(
                root.join("analysis/native/at-prologue3.json"),
            )?)?;
            let mut state = Session::new(&content);
            apply(Command::ImportStarters, &mut state, &content)?;
            apply(
                Command::EnterMission {
                    name: "The Thunder of Guns".into(),
                },
                &mut state,
                &content,
            )?;
            state.random.seed = before["random_seed"]
                .as_u64()
                .ok_or("native seed missing")? as u32;
            for (i, warrior) in state.warriors.iter_mut().enumerate() {
                let hex = before["warriors"][i]["hex"]
                    .as_str()
                    .ok_or("native warrior missing")?;
                warrior.bytes = (0..hex.len())
                    .step_by(2)
                    .map(|j| u8::from_str_radix(&hex[j..j + 2], 16))
                    .collect::<Result<_, _>>()?;
            }
            let effects = apply(Command::Move { dx: 1, dy: 1 }, &mut state, &content)?;
            if state.random.seed
                != after["random_seed"].as_u64().ok_or("native seed missing")? as u32
                || state.random.draws != 3
            {
                return Err("first-step random sequence differs".into());
            }
            let position = state.exploration.as_ref().unwrap();
            if (position.x, position.y, position.minute) != (3, 3, 10) {
                return Err("first-step position or clock differs".into());
            }
            for (i, warrior) in state.warriors.iter().enumerate() {
                let hex = after["warriors"][i]["hex"]
                    .as_str()
                    .ok_or("native warrior missing")?;
                let native: Vec<u8> = (0..hex.len())
                    .step_by(2)
                    .map(|j| u8::from_str_radix(&hex[j..j + 2], 16))
                    .collect::<Result<_, _>>()?;
                if warrior.bytes != native {
                    return Err(format!("first-step warrior {} differs", warrior.name).into());
                }
            }
            println!(
                "First step: position 3,3, time 12:10, all six warrior records, and three ordered random draws match reference gameplay; presentation was bypassed in this capture. Effects: {effects:?}"
            );
        }
        "verify-opening" => {
            let mut state = Session::new(&content);
            state.random.seed = args.next().as_deref().unwrap_or("12").parse()?;
            apply(Command::ImportStarters, &mut state, &content)?;
            apply(
                Command::EnterMission {
                    name: "The Thunder of Guns".into(),
                },
                &mut state,
                &content,
            )?;
            apply(Command::Move { dx: 1, dy: 1 }, &mut state, &content)?;
            for _ in 0..32 {
                if state.exploration.as_ref().unwrap().presentation.is_none() {
                    break;
                }
                if matches!(
                    state.exploration.as_ref().unwrap().presentation,
                    Some(centurions_core::Presentation::Loot)
                ) {
                    while let Some(slot) = state.booty_items.iter().position(|item| item.id() != 0)
                    {
                        apply(Command::TakeLoot { slot, warrior: 0 }, &mut state, &content)?;
                    }
                }
                apply(Command::Continue, &mut state, &content)?;
            }
            for _ in 0..6 {
                apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content)?;
            }
            apply(Command::Move { dx: 1, dy: -1 }, &mut state, &content)?;
            apply(Command::Move { dx: -1, dy: 1 }, &mut state, &content)?;
            for _ in 0..3 {
                apply(Command::Move { dx: -1, dy: 0 }, &mut state, &content)?;
            }
            for step in 0..2000 {
                if state.phase == centurions_core::Phase::Combat {
                    break;
                }
                apply(
                    Command::Move {
                        dx: if step % 2 == 0 { -1 } else { 1 },
                        dy: 0,
                    },
                    &mut state,
                    &content,
                )?;
            }
            let battle = state
                .combat
                .as_ref()
                .ok_or("opening route did not enter combat")?;
            if !(2..=4).contains(&battle.battle)
                || ![5, 6, 8].contains(&battle.enemies.records.len())
            {
                return Err("opening encounter selection differs".into());
            }
            let path = root.join("analysis/test-saves/opening-combat.json");
            save(&path, &state, &content)?;
            if state != load(&path, &content)? {
                return Err("combat save roundtrip differs".into());
            }
            println!(
                "Opening story, equipment grant, exploration, regional encounter, and combat save/reload passed. Active body {:?}; random draws {}",
                battle.active_body(),
                state.random.draws
            );
        }
        "verify-fight" => {
            let mut state = load(
                &root.join("analysis/test-saves/opening-combat.json"),
                &content,
            )?;
            let mut commands = vec![];
            let mut shots = 0;
            let initial = state.clone();
            for step in 0..2000 {
                let Some(combat) = state.combat.as_ref() else {
                    break;
                };
                let body = combat.active_body().ok_or("no active combatant")?;
                let command = if body >= 10 {
                    Command::RunEnemyTurn
                } else {
                    let mut shot = None;
                    for target in 10..10 + combat.enemies.records.len() as i16 {
                        let mut candidate = state.clone();
                        if apply(
                            Command::ShootEnemy { body: target },
                            &mut candidate,
                            &content,
                        )
                        .is_ok()
                        {
                            shot = Some(Command::ShootEnemy { body: target });
                            break;
                        }
                    }
                    let neighbors =
                        centurions_core::combat::runtime::neighbors(combat, &state.warriors, body)?;
                    if let Some(command) = shot {
                        shots += 1;
                        command
                    } else if let Some(target) = neighbors.first() {
                        Command::AttackEnemy { body: *target }
                    } else {
                        Command::EndTurn
                    }
                };
                commands.push(command.clone());
                let events = apply(command, &mut state, &content)
                    .map_err(|e| format!("step {step}, body {body}: {e}"))?;
                println!("step {step} body {body}: {events:?}");
            }
            if state.phase != centurions_core::Phase::Exploration {
                return Err("fight did not return to exploration".into());
            }
            if shots == 0 {
                return Err("workflow did not exercise firearms".into());
            }
            let checkpoint = root.join("analysis/test-saves/fight-checkpoint.json");
            let mut uninterrupted = initial;
            let split = commands.len() / 2;
            for command in &commands[..split] {
                apply(command.clone(), &mut uninterrupted, &content)?;
            }
            save(&checkpoint, &uninterrupted, &content)?;
            let mut resumed = load(&checkpoint, &content)?;
            for (index, command) in commands[split..].iter().enumerate() {
                if split + index + 1 == commands.len() {
                    save(
                        &root.join("analysis/test-saves/before-victory.json"),
                        &uninterrupted,
                        &content,
                    )?;
                }
                let a = apply(command.clone(), &mut uninterrupted, &content)?;
                let b = apply(command.clone(), &mut resumed, &content)?;
                if a != b || uninterrupted != resumed {
                    return Err("restarted replay diverged".into());
                }
            }
            if uninterrupted != state {
                return Err("full replay diverged".into());
            }
            std::fs::write(
                root.join("analysis/test-saves/fight-replay.json"),
                serde_json::to_vec_pretty(&commands)?,
            )?;
            let path = root.join("analysis/test-saves/after-fight.json");
            save(&path, &state, &content)?;
            if state != load(&path, &content)? {
                return Err("postcombat save differs".into());
            }
            println!(
                "Opening battle victory with {shots} firearm commands; rewards, exploration return, save/reload and resumed deterministic replay passed"
            );
        }
        "verify-save" => {
            let mut session = Session::new(&content);
            apply(Command::ImportStarters, &mut session, &content)?;
            apply(
                Command::EnterMission {
                    name: "The Thunder of Guns".into(),
                },
                &mut session,
                &content,
            )?;
            let path = root.join("analysis/test-saves/roundtrip.json");
            save(&path, &session, &content)?;
            if session != load(&path, &content)? {
                return Err("save roundtrip differs".into());
            }
            save(&path, &session, &content)?;
            println!("Imported all six warriors; save, reload, and replacement passed");
        }
        _ => return Err(format!("unknown command {action}").into()),
    }
    Ok(())
}
