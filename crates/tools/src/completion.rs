use crate::sustained::Play;
use centurions_content::{Content, word};
use centurions_core::{Command, Session, apply, load, save};
use std::path::Path;

pub fn verify_stage_one(root: &Path, content: &Content) -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = root.join("analysis/completion");
    std::fs::create_dir_all(&artifacts)?;
    for seed in [12, 1] {
        let mut state = Session::new(content);
        state.random.seed = seed;
        let initial = state.clone();
        let mut play = Play {
            content,
            artifacts: &artifacts,
            state,
            commands: vec![],
            ordered_events: vec![],
            battles: 0,
            supplies: 0,
            reloads: 0,
            completion: true,
        };
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            play.command(Command::ImportStarters)?;
            play.command(Command::EnterMission {
                name: "The Thunder of Guns".into(),
            })?;
            play.command(Command::Move { dx: 1, dy: 1 })?;
            play.settle()?;
            play.walk_to((58, 77))?;
            for warrior in 0..6 {
                play.command(Command::Reload { warrior })?;
            }
            play.walk_to((5, 5))?;
            play.walk_to((7, 3))?;
            play.walk_to((45, 82))?;
            play.walk_to((7, 3))?;
            play.walk_to((3, 84))?;
            if play.state.mission_stage != 1 {
                return Err("first stage did not reach its authored transition".into());
            }
            Ok(())
        })();
        if let Err(error) = result {
            verify_segment(&play, initial, seed, "stage-one-prefix")?;
            save(
                &artifacts.join(format!("stage-one-failure-{seed}.json")),
                &play.state,
                content,
            )?;
            return Err(error);
        }
        verify_segment(&play, initial, seed, "stage-one")?;
        save(
            &artifacts.join(format!("stage-one-{seed}.json")),
            &play.state,
            content,
        )?;
        if load(&artifacts.join(format!("stage-one-{seed}.json")), content)? != play.state {
            return Err("post-transition map state differs after reload".into());
        }
        std::fs::write(
            artifacts.join(format!("route-{seed}.json")),
            serde_json::to_vec_pretty(&play.commands)?,
        )?;
        println!(
            "Seed {seed}: map {}, stage {}, {} commands, {} battles",
            play.state.exploration.as_ref().unwrap().map,
            play.state.mission_stage,
            play.commands.len(),
            play.battles
        );
    }
    Ok(())
}

pub fn verify_samples(
    root: &Path,
    content: &Content,
    selected_seed: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = root.join("analysis/completion");
    for seed in selected_seeds(selected_seed)? {
        let state = load(&artifacts.join(format!("stage-one-{seed}.json")), content)?;
        let initial = state.clone();
        let mut play = Play {
            content,
            artifacts: &artifacts,
            state,
            commands: vec![],
            ordered_events: vec![],
            battles: 0,
            supplies: 0,
            reloads: 0,
            completion: true,
        };
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            play.prepare_owned_firearms()?;
            for (item, target) in [
                (804, (10, 8)),
                (805, (11, 23)),
                (807, (79, 6)),
                (806, (80, 80)),
                (808, (66, 85)),
            ] {
                if matches!(item, 806..=808) {
                    safe_rest(&mut play)?;
                }
                if item == 807 {
                    if seed == 1 {
                        play.walk_to((82, 10))?;
                        play.walk_to((82, 9))?;
                    } else {
                        play.walk_to((79, 10))?;
                    }
                }
                if item == 808 {
                    play.walk_to_sample_battle()?;
                } else {
                    play.walk_to(target)?;
                }
                if item == 807 {
                    if play.state.exploration.as_ref().unwrap().map != 2 {
                        return Err("Hive entry did not reach map2".into());
                    }
                    play.walk_to((23, 8))?;
                    if centurions_core::story_inventory::query_party_item(
                        &play.state.warriors,
                        807,
                    )?
                    .is_none()
                    {
                        return Err("Hive treasure did not supply sample807".into());
                    }
                    play.walk_to((1, 2))?;
                    if play.state.exploration.as_ref().unwrap().map != 1 {
                        return Err("Hive exit did not return to map1".into());
                    }
                }
                for step in 0..2000 {
                    if centurions_core::story_inventory::query_party_item(
                        &play.state.warriors,
                        item,
                    )?
                    .is_some()
                    {
                        break;
                    }
                    play.command(Command::Move {
                        dx: if item == 805 {
                            0
                        } else if step % 2 == 0 {
                            1
                        } else {
                            -1
                        },
                        dy: if item == 805 {
                            if step % 2 == 0 { 1 } else { -1 }
                        } else {
                            0
                        },
                    })?;
                    play.settle()?;
                }
                if centurions_core::story_inventory::query_party_item(&play.state.warriors, item)?
                    .is_none()
                {
                    return Err(format!("authored sample{item} was not acquired").into());
                }
                save(
                    &artifacts.join(format!("sample-{item}-{seed}.json")),
                    &play.state,
                    content,
                )?;
                println!(
                    "Seed {seed}: sample{item} acquired after {} battles, {} commands",
                    play.battles,
                    play.commands.len()
                );
            }
            safe_rest(&mut play)?;
            play.walk_to((56, 7))?;
            for step in 0..2000 {
                if centurions_core::story_inventory::query_party_item(&play.state.warriors, 809)?
                    .is_some()
                {
                    break;
                }
                play.command(Command::Move {
                    dx: if step % 2 == 0 { 1 } else { -1 },
                    dy: 0,
                })?;
                play.settle()?;
            }
            if centurions_core::story_inventory::query_party_item(&play.state.warriors, 809)?
                .is_none()
            {
                return Err("Sample809 scene did not supply its treasure".into());
            }
            verify_segment(&play, initial.clone(), seed, "samples")?;
            save(
                &artifacts.join(format!("samples-{seed}.json")),
                &play.state,
                content,
            )?;
            println!(
                "Seed {seed}: all samples acquired in {} battles, {} commands",
                play.battles,
                play.commands.len()
            );
            Ok(())
        })();
        if let Err(error) = result {
            save(
                &artifacts.join(format!("samples-failure-{seed}.json")),
                &play.state,
                content,
            )?;
            verify_segment(&play, initial, seed, "samples-prefix")?;
            return Err(error);
        }
    }
    Ok(())
}

pub fn verify_laboratory(
    root: &Path,
    content: &Content,
    selected_seed: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = root.join("analysis/completion");
    for seed in selected_seeds(selected_seed)? {
        let state = load(&artifacts.join(format!("samples-{seed}.json")), content)?;
        let initial = state.clone();
        let mut play = Play {
            content,
            artifacts: &artifacts,
            state,
            commands: vec![],
            ordered_events: vec![],
            battles: 0,
            supplies: 0,
            reloads: 0,
            completion: true,
        };
        if play.state.mission_stage != 1 || play.state.exploration.as_ref().unwrap().map != 1 {
            return Err(
                format!("samples seed {seed} is not in the authored laboratory stage").into(),
            );
        }
        for item in 804..=809 {
            if centurions_core::story_inventory::query_party_item(&play.state.warriors, item)?
                .is_none()
            {
                return Err(format!("laboratory seed {seed} is missing sample{item}").into());
            }
        }
        if let Err(error) = play.enter_laboratory() {
            verify_segment(&play, initial, seed, "laboratory-prefix")?;
            save(
                &artifacts.join(format!("laboratory-failure-{seed}.json")),
                &play.state,
                content,
            )?;
            return Err(error);
        }
        if play.state.mission_stage != 2 || play.state.exploration.as_ref().unwrap().map != 3 {
            verify_segment(&play, initial, seed, "laboratory-postcondition")?;
            save(
                &artifacts.join(format!("laboratory-postcondition-{seed}.json")),
                &play.state,
                content,
            )?;
            return Err(
                format!("laboratory seed {seed} did not reach authored stage2/map3").into(),
            );
        }
        verify_segment(&play, initial, seed, "laboratory")?;
        let destination = artifacts.join(format!("laboratory-{seed}.json"));
        save(&destination, &play.state, content)?;
        if load(&destination, content)? != play.state {
            return Err(format!("laboratory seed {seed} state differs after reload").into());
        }
        println!(
            "Seed {seed}: laboratory reached stage2/map3 in {} battles, {} commands",
            play.battles,
            play.commands.len()
        );
    }
    Ok(())
}

pub fn verify_stage_two(
    root: &Path,
    content: &Content,
    selected_seed: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = root.join("analysis/completion");
    for seed in selected_seeds(selected_seed)? {
        let state = load(&artifacts.join(format!("laboratory-{seed}.json")), content)?;
        let initial = state.clone();
        let mut play = Play {
            content,
            artifacts: &artifacts,
            state,
            commands: vec![],
            ordered_events: vec![],
            battles: 0,
            supplies: 0,
            reloads: 0,
            completion: true,
        };
        if play.state.mission_stage != 2 || play.state.exploration.as_ref().unwrap().map != 3 {
            return Err(format!("laboratory seed {seed} is not at authored stage2/map3").into());
        }
        if let Err(error) = stage_two_route(&mut play) {
            verify_segment(&play, initial, seed, "stage-two-prefix")?;
            save(
                &artifacts.join(format!("stage-two-failure-{seed}.json")),
                &play.state,
                content,
            )?;
            return Err(error);
        }
        verify_segment(&play, initial, seed, "stage-two")?;
        let destination = artifacts.join(format!("stage-two-{seed}.json"));
        save(&destination, &play.state, content)?;
        if load(&destination, content)? != play.state {
            return Err(format!("stage2 seed {seed} state differs after reload").into());
        }
        println!(
            "Seed {seed}: five parts/charges, bridge and password reached stage3 in {} battles, {} commands",
            play.battles,
            play.commands.len()
        );
    }
    Ok(())
}

fn require_item(play: &Play<'_>, item: i16) -> Result<(), Box<dyn std::error::Error>> {
    if centurions_core::story_inventory::query_party_item(&play.state.warriors, item)?.is_none() {
        return Err(format!("authored route did not acquire item{item}").into());
    }
    Ok(())
}

pub fn verify_stage_three(
    root: &Path,
    content: &Content,
    selected_seed: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = root.join("analysis/completion");
    for seed in selected_seeds(selected_seed)? {
        let initial = load(&artifacts.join(format!("stage-two-{seed}.json")), content)?;
        let position = initial
            .exploration
            .as_ref()
            .ok_or("stage-three map missing")?;
        if initial.mission_stage != 3
            || initial.phase != centurions_core::Phase::Exploration
            || initial.combat.is_some()
            || initial.medical_continuation.is_some()
            || (position.map, position.x, position.y) != (8, 5, 50)
            || position.presentation.is_some()
            || position.script.is_some()
        {
            return Err(format!("seed{seed} is not at the settled stage-three entrance").into());
        }
        let mut play = Play {
            content,
            artifacts: &artifacts,
            state: initial.clone(),
            commands: vec![],
            ordered_events: vec![],
            battles: 0,
            supplies: 0,
            reloads: 0,
            completion: false,
        };
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            play.command(Command::Move { dx: 1, dy: 0 })?;
            if !matches!(
                play.state.exploration.as_ref().unwrap().presentation,
                Some(centurions_core::Presentation::Dialogue { id: -900, .. })
            ) {
                return Err(
                    "stage-three move did not open the authored registration dialogue".into(),
                );
            }
            let before_transition = play.state.clone();
            let mut expected_warriors = before_transition.warriors.clone();
            for member in expected_warriors.iter_mut().take(5) {
                for (target, maximum) in [(0x50, 0x52), (0x58, 0x5a)] {
                    let value = word(&member.bytes, maximum)?;
                    member.bytes[target..target + 2].copy_from_slice(&value.to_be_bytes());
                }
            }
            play.command(Command::Continue)?;
            if play.state.warriors != expected_warriors
                || play.state.flags != before_transition.flags
                || play.state.ammunition != before_transition.ammunition
                || play.state.crate_items != before_transition.crate_items
                || play.state.booty_items != before_transition.booty_items
                || play.state.party_currency != before_transition.party_currency
            {
                return Err(
                    "stage-three transition changed preserved records or restoration bounds".into(),
                );
            }
            let position = play.state.exploration.as_ref().unwrap();
            if play.state.mission_stage != 4
                || (position.map, position.x, position.y) != (4, 5, 5)
                || !matches!(
                    position.presentation,
                    Some(centurions_core::Presentation::Prologue { resource_id: 1004 })
                )
            {
                return Err(
                    "registration did not directly enter the authored stage-four prologue".into(),
                );
            }
            play.command(Command::Continue)?;
            let position = play.state.exploration.as_ref().unwrap();
            if play.state.mission_stage != 4
                || play.state.phase != centurions_core::Phase::Exploration
                || play.state.combat.is_some()
                || play.state.medical_continuation.is_some()
                || (position.map, position.x, position.y) != (4, 5, 5)
                || position.presentation.is_some()
                || position.script.is_some()
                || !play.ordered_events.iter().flatten().any(|event| {
                    matches!(event, centurions_core::Event::MissionEntered { title, x: 5, y: 5 }
                        if title == "Junk Yard Dogs")
                })
                || play.ordered_events.iter().flatten().any(|event| {
                    matches!(
                        event,
                        centurions_core::Event::CombatStarted { .. }
                            | centurions_core::Event::BattleEnded { .. }
                    )
                })
            {
                return Err(
                    "registration route did not reach the ordinary stage-four entrance".into(),
                );
            }
            Ok(())
        })();
        if let Err(error) = result {
            verify_segment(&play, initial, seed, "stage-three-prefix")?;
            save(
                &artifacts.join(format!("stage-three-failure-{seed}.json")),
                &play.state,
                content,
            )?;
            return Err(error);
        }
        verify_segment(&play, initial, seed, "stage-three")?;
        let destination = artifacts.join(format!("stage-three-{seed}.json"));
        save(&destination, &play.state, content)?;
        if load(&destination, content)? != play.state {
            return Err(format!("stage-three seed{seed} state differs after reload").into());
        }
        println!(
            "Seed {seed}: registered stage-three exit reached stage4/map4 in {} commands",
            play.commands.len()
        );
    }
    Ok(())
}

fn selected_seeds(selected_seed: Option<u32>) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    match selected_seed {
        None => Ok(vec![12, 1]),
        Some(seed @ (12 | 1)) => Ok(vec![seed]),
        Some(_) => Err("completion seed must be 12 or 1".into()),
    }
}

fn safe_rest(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let full = |play: &Play<'_>| {
        play.state.warriors.iter().all(|warrior| {
            centurions_content::word(&warrior.bytes, 0x50).unwrap_or(-10)
                >= centurions_content::word(&warrior.bytes, 0x52).unwrap_or(i16::MAX)
        })
    };
    if full(play) {
        return Ok(());
    }
    if play.state.exploration.as_ref().unwrap().map != 1 {
        return Err("safe recovery route requires map1".into());
    }
    if play
        .state
        .warriors
        .iter()
        .any(|warrior| centurions_content::word(&warrior.bytes, 0x50).unwrap_or(-10) <= -10)
    {
        return Err("safe recovery cannot revive a dead warrior".into());
    }
    play.walk_to((19, 3))?;
    for step in 0..20000 {
        if full(play) {
            return Ok(());
        }
        play.command(Command::Move {
            dx: if step % 2 == 0 { 1 } else { -1 },
            dy: 0,
        })?;
        play.settle()?;
    }
    Err("safe recovery did not restore the complete squad".into())
}

fn walk_leg(
    play: &mut Play<'_>,
    map: i32,
    target: (i32, i32),
    destination: (i32, i32, i32),
) -> Result<(), Box<dyn std::error::Error>> {
    if play.state.exploration.as_ref().unwrap().map != map {
        return Err(format!("route leg to {target:?} requires map{map}").into());
    }
    play.walk_to(target)?;
    let p = play.state.exploration.as_ref().unwrap();
    if (p.map, p.x, p.y) != destination {
        return Err(format!(
            "route leg map{map} {target:?} reached {:?}, expected {destination:?}",
            (p.map, p.x, p.y)
        )
        .into());
    }
    Ok(())
}

fn stage_two_route(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    prepare_stage_two_shield(play)?;
    recover_stage_two_psi(play)?;
    prepare_stage_two_rifle(play)?;
    prepare_stage_two_medical(play)?;
    prepare_stage_two_loaded_weapon(play)?;
    if word(&play.state.party_buffs, 4)? < 20 {
        play.command(Command::CastPartyPsi {
            warrior: 5,
            spell: 1103,
            power: 1,
        })?;
    }
    walk_leg(play, 3, (5, 3), (3, 5, 3))?;
    walk_leg(play, 3, (9, 77), (5, 68, 61))?;
    walk_leg(play, 5, (85, 62), (5, 85, 62))?;
    require_item(play, 837)?;
    walk_leg(play, 5, (68, 60), (3, 10, 77))?;
    walk_leg(play, 3, (8, 76), (5, 53, 88))?;
    walk_leg(play, 5, (54, 88), (5, 54, 88))?;
    walk_leg(play, 5, (87, 86), (5, 87, 86))?;
    require_item(play, 838)?;
    walk_leg(play, 5, (53, 88), (3, 8, 77))?;
    walk_leg(play, 3, (5, 79), (7, 0, 82))?;
    walk_leg(play, 7, (1, 82), (7, 1, 82))?;
    walk_leg(play, 7, (15, 76), (7, 15, 76))?;
    require_item(play, 834)?;
    walk_leg(play, 7, (20, 71), (7, 20, 71))?;
    require_item(play, 840)?;
    walk_leg(play, 7, (43, 75), (7, 43, 75))?;
    walk_leg(play, 7, (53, 67), (7, 53, 67))?;
    require_item(play, 836)?;
    walk_leg(play, 7, (85, 69), (7, 85, 69))?;
    walk_leg(play, 7, (88, 66), (7, 88, 66))?;
    require_item(play, 835)?;
    for (item, x) in [(834, 2), (835, 5), (836, 8), (837, 11), (838, 14)] {
        require_item(play, item)?;
        walk_leg(play, 7, (x, 86), (7, x, 86))?;
        if centurions_core::story_inventory::query_party_item(&play.state.warriors, item)?.is_some()
        {
            return Err(format!("generator repair did not consume part{item}").into());
        }
    }
    for (portal, right, left, returning) in [
        (25, 68, 66, Some((67, 25))),
        (27, 73, 71, Some((72, 27))),
        (29, 78, 76, Some((77, 29))),
        (31, 83, 81, Some((82, 31))),
        (33, 88, 86, None),
    ] {
        walk_leg(play, 7, (portal, 85), (7, right - 1, 88))?;
        walk_leg(play, 7, (right, 86), (7, right, 86))?;
        walk_leg(play, 7, (left, 86), (7, left, 86))?;
        if let Some((x, destination)) = returning {
            require_item(play, 839)?;
            walk_leg(play, 7, (x, 87), (7, destination, 84))?;
        } else {
            require_item(play, 841)?;
            require_item(play, 840)?;
            play.command(Command::Encounter)?;
            if !matches!(
                play.state.exploration.as_ref().unwrap().presentation,
                Some(centurions_core::Presentation::Typed { id: 8, .. })
            ) {
                return Err("generator encounter did not open authored typed encounter8".into());
            }
            play.command(Command::SubmitText {
                text: "rescue 911".into(),
            })?;
            play.settle()?;
            if centurions_core::story_inventory::query_party_item(&play.state.warriors, 840)?
                .is_some()
            {
                return Err("generator encounter did not consume wall cutter840".into());
            }
            walk_leg(play, 7, (82, 87), (7, 31, 84))?;
        }
    }
    walk_leg(play, 7, (0, 82), (3, 4, 79))?;
    walk_leg(play, 3, (66, 36), (3, 66, 36))?;
    let p = play.state.exploration.as_ref().unwrap();
    if p.floor[65 * 90 + 36] != 115 {
        return Err("power crystal did not open the authored drawbridge".into());
    }
    walk_leg(play, 3, (59, 39), (3, 59, 39))?;
    play.walk_to((58, 41))?;
    if play.state.mission_stage != 3 || play.state.exploration.as_ref().unwrap().map != 8 {
        return Err("power/bridge/password route did not reach authored stage3".into());
    }
    Ok(())
}

fn prepare_stage_two_rifle(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let inventory = play.state.warriors[0].inventory()?;
    let slot = inventory
        .iter()
        .position(|item| item.id() == 38 && item.0[9] == 1)
        .ok_or("stage-two rifle has not been collected and equipped")?;
    let item = &inventory[slot];
    if word(&item.0, 2)? == 0 && word(&item.0, 4)? == 0 {
        play.command(Command::FitAmmunition {
            warrior: 0,
            slot,
            ammunition: 203,
        })?;
    }
    Ok(())
}

fn prepare_stage_two_medical(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    for _ in 0..100 {
        let Some(slot) = play
            .state
            .crate_items
            .iter()
            .position(|item| item.id() == 700 && word(&item.0, 2).unwrap_or(0) > 0)
        else {
            return Ok(());
        };
        let before = play
            .state
            .crate_items
            .iter()
            .filter(|item| item.id() == 700)
            .count();
        play.command(Command::TakeCrate { slot, warrior: 4 })?;
        if play
            .state
            .crate_items
            .iter()
            .filter(|item| item.id() == 700)
            .count()
            != before - 1
        {
            return Err("earned medical crate collection did not clear its source".into());
        }
    }
    Err("earned medical crate preparation exceeded its inventory bound".into())
}

fn prepare_stage_two_loaded_weapon(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let warrior = 4;
    if !play.state.warriors[warrior]
        .inventory()?
        .iter()
        .any(|item| item.id() == 165)
    {
        let slot = play
            .state
            .crate_items
            .iter()
            .position(|item| item.id() == 165)
            .ok_or("earned loaded firearm is missing from the crate")?;
        play.command(Command::TakeCrate { slot, warrior })?;
        if play.state.crate_items.iter().any(|item| item.id() == 165) {
            return Err("earned loaded firearm collection did not clear its source".into());
        }
    }
    let inventory = play.state.warriors[warrior].inventory()?;
    let slot = inventory
        .iter()
        .position(|item| item.id() == 165)
        .ok_or("collected loaded firearm is missing")?;
    if inventory[slot].0[9] == 0 {
        play.command(Command::EquipItem { warrior, slot })?;
    }
    Ok(())
}

fn prepare_stage_two_shield(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let position = play.state.exploration.as_ref().unwrap();
    if (position.map, position.x, position.y) != (3, 4, 2) {
        return Err("stage-two shield preparation requires its authored starting position".into());
    }
    let warrior = 3;
    for id in [600, 637] {
        let slot = play
            .state
            .crate_items
            .iter()
            .position(|item| item.id() == id)
            .ok_or("earned stage-two shield equipment is missing from the crate")?;
        play.command(Command::TakeCrate { slot, warrior })?;
    }
    let slot = play.state.warriors[warrior]
        .inventory()?
        .iter()
        .position(|item| item.id() == 637)
        .ok_or("collected shield battery is missing")?;
    play.command(Command::EquipItem { warrior, slot })?;
    let slot = play.state.warriors[warrior]
        .inventory()?
        .iter()
        .position(|item| item.id() == 637)
        .ok_or("equipped shield battery is missing")?;
    play.command(Command::FitAmmunition {
        warrior,
        slot,
        ammunition: 307,
    })?;
    let slot = play.state.warriors[warrior]
        .inventory()?
        .iter()
        .position(|item| item.id() == 600)
        .ok_or("collected personal shield is missing")?;
    play.command(Command::EquipItem { warrior, slot })?;
    Ok(())
}

fn recover_stage_two_psi(play: &mut Play<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let position = play.state.exploration.as_ref().unwrap();
    if (position.map, position.x, position.y) != (3, 4, 2) {
        return Err("stage-two recovery requires its authored starting position".into());
    }
    for _ in 0..200 {
        let caster = &play.state.warriors[5];
        if caster.psi_points()? >= word(&caster.bytes, 0x5a)? {
            return Ok(());
        }
        if caster.stamina()? <= 0 || word(&caster.bytes, 0x90)? != 0 {
            return Err("stage-two caster cannot recover psi".into());
        }
        for (dx, dy) in [(-1, -1), (1, 1)] {
            play.command(Command::Move { dx, dy })?;
            play.settle()?;
        }
        let position = play.state.exploration.as_ref().unwrap();
        if (position.map, position.x, position.y) != (3, 4, 2) {
            return Err("stage-two recovery left its authored starting cells".into());
        }
    }
    Err("stage-two psi recovery exceeded its movement bound".into())
}

fn verify_segment(
    play: &Play<'_>,
    initial: Session,
    seed: u32,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut replay = initial;
    let mut resumed = None;
    for (index, command) in play.commands.iter().enumerate() {
        if index == play.commands.len() / 2 {
            let checkpoint = play
                .artifacts
                .join(format!("{name}-checkpoint-{seed}.json"));
            save(&checkpoint, &replay, play.content)?;
            resumed = Some(load(&checkpoint, play.content)?);
        }
        let events = apply(command.clone(), &mut replay, play.content)?;
        if events != play.ordered_events[index] {
            return Err(
                format!("{name} seed {seed} event replay differs at command {index}").into(),
            );
        }
        if let Some(restored) = resumed.as_mut()
            && (apply(command.clone(), restored, play.content)? != events || restored != &replay)
        {
            return Err(format!(
                "{name} seed {seed} resumed state/events differ at command {index}"
            )
            .into());
        }
    }
    if replay != play.state {
        return Err(format!("{name} seed {seed} final replay state differs").into());
    }
    std::fs::write(
        play.artifacts.join(format!("{name}-commands-{seed}.json")),
        serde_json::to_vec_pretty(&play.commands)?,
    )?;
    std::fs::write(
        play.artifacts.join(format!("{name}-events-{seed}.json")),
        serde_json::to_vec_pretty(&play.ordered_events)?,
    )?;
    Ok(())
}
