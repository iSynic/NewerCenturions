use crate::prelude::*;

use centurions_core::travel::*;

fn fixture() -> serde_json::Value {
    crate::support::fixture("map-actions.json")
}

#[test]
fn scripted_map_mutations_match_original_complete_map_layers() {
    for case in fixture()["cases"].as_array().unwrap() {
        let extra = case.bytes("extra");
        let target = i16::from_be_bytes(extra[..2].try_into().unwrap()) as usize;
        for map in 0..2 {
            let before = &case[format!("map{map}_before")];
            let after = &case[format!("map{map}_after")];
            let mut floor = words(before.text("floor"));
            let mut objects = words(before.text("objects"));
            let mut headers = before.bytes("headers");
            if map == target {
                match case.n::<u64>("opcode") {
                    12 => {
                        let mutation = tile_mutation(&extra).unwrap();
                        match mutation.layer {
                            TileLayer::Floor => floor = apply_tile(&floor, &mutation).unwrap(),
                            TileLayer::Object => objects = apply_tile(&objects, &mutation).unwrap(),
                        }
                    }
                    13 => headers = set_event_chances(&headers, &extra).unwrap(),
                    7 => {
                        let replacement = event_replacement(&extra).unwrap();
                        headers = replace_event_script(
                            &headers,
                            replacement.slot,
                            &case.bytes("replacement"),
                        )
                        .unwrap();
                    }
                    opcode => panic!("unexpected fixture opcode {opcode}"),
                }
            }
            crate::support::native_expect! {
            words &(floor) => &after["floor"], "binary_travel";
            words &(objects) => &after["objects"], "binary_travel";
            bytes headers => &after["headers"], "binary_travel";
            }
        }
    }
}

#[test]
fn normal_secret_search_matches_original_full_map_and_rng() {
    for case in fixture()["secret_cases"].as_array().unwrap() {
        let floor = words(case.text("before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = discover_secrets(
            &floor,
            (case.n::<i16>("x"), case.n::<i16>("y")),
            case.n::<i16>("chance"),
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        words &(result.floor) => &case["after"], "binary_travel";
        scalar random.seed => &case["seed_after"], "binary_travel";
        scalar random.draws => &case["rng_draws"], "binary_travel";
        scalar result.center_revealed => &case["center_revealed"], "binary_travel";
        scalar result.discoveries.len() => &case["flashes"], "binary_travel";
        }
    }
}

#[test]
fn secret_search_skill_matches_original_signed_word_average() {
    let mut compared = 0;
    for case in fixture()["secret_cases"].as_array().unwrap() {
        if case["forced_skill"].as_bool() != Some(false) {
            continue;
        }
        let records = case.bytes("warriors");
        let warriors: Vec<centurions_content::Warrior> = records
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| centurions_content::Warrior {
                name: String::new(),
                bytes: record.to_vec(),
                original_sha256: String::new(),
            })
            .collect();
        crate::support::expect(secret_search_skill(&warriors).unwrap(), &case["chance"]);
        compared += 1;
    }
    assert!(compared > 0, "missing original skill fixtures");
}
