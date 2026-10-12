use crate::prelude::*;

use centurions_core::ammunition;
use centurions_core::ammunition::RefitFlags;
use centurions_core::ammunition::WorkBuffers;
use centurions_core::items;

fn party(snapshot: &Value) -> Vec<Warrior> {
    crate::support::warriors(&snapshot["party"])
}
fn reserves(snapshot: &Value) -> Vec<i32> {
    crate::support::longs(&snapshot.bytes("ammunition"))
}
fn data() -> Value {
    crate::support::fixture("empty-rifle-ammunition.json")
}

use crate::support::assert_bytes;

fn assert_party(warriors: &[Warrior], snapshot: &Value, label: &str) {
    assert_bytes(
        &crate::support::record_bytes(warriors),
        &snapshot.bytes("party"),
        label,
    );
}
fn sounds(snapshot: &Value) -> Vec<i16> {
    snapshot.n::<Vec<i16>>("sounds")
}
#[test]
fn actual_empty_rifle_collection_wear_and_compaction_match_four_native_cases() {
    let f = data();
    let content = content();
    let map = f["foundations"][0]["session"]["exploration"]["map"]
        .as_u64()
        .unwrap() as i32;
    let mission = content.map(map).unwrap();

    for (i, case) in f.array("wear_cases").iter().enumerate() {
        let mut w = party(&case["before"]);
        let count = w[0].inventory().unwrap().len();
        let original = case["before"]
            .bytes("crate")
            .as_chunks::<22>()
            .0
            .iter()
            .find(|r| word(*r, 0).unwrap() == 38)
            .map(|r| InventoryRecord(*r))
            .unwrap();
        assert_eq!(word(&original.0, 2).unwrap(), 0);
        assert_eq!(word(&original.0, 4).unwrap(), 0);
        let collected = equipment::collect_item(&w[0], &original, &content, mission).unwrap();
        assert!(collected.collected);
        assert_eq!(collected.sounds, sounds(&case["picked"]));
        w[0] = collected.warrior;
        assert_party(&w, &case["picked"], &format!("pickup{i}"));
        assert_eq!(collected.item.id(), 0);
        let result = equipment::wear(
            &w[0],
            &content,
            mission,
            count,
            case.n::<u64>("combat") != 0,
        )
        .unwrap();
        assert!(result.changed);
        assert!(result.warnings.is_empty());
        assert_eq!(
            result.sounds,
            &sounds(&case["worn"])[sounds(&case["picked"]).len()..]
        );
        w[0] = result.warrior;
        assert_party(&w, &case["worn"], &format!("wear{i}"));
        items::compact(&mut w[0]).unwrap();
        assert_party(&w, &case["compacted"], &format!("compact{i}"));
        assert_eq!(word(&w[0].bytes, 0x126).unwrap(), 0);
    }
}
#[test]
fn selected_203_ammunition_matches_24_native_callers_complete_party_stock_rng_and_audio() {
    let f = data();
    let content = content();

    for (i, case) in f.array("manual_cases").iter().enumerate() {
        let before = &case["operation_before"];
        let after = &case["after"];
        let mut warriors = party(before);
        let input = warriors.clone();
        let stock = reserves(before);
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = firearm::fit_ammunition(
            &warriors[0],
            case.n::<usize>("inventory_slot"),
            &content,
            &stock,
            case.n::<u64>("combat") != 0,
            &mut random,
        )
        .unwrap_or_else(|e| panic!("manual{i}:{e:?}"));
        assert_eq!(warriors, input, "manual{i} immutable input");
        warriors[0] = result.warrior;
        assert_party(&warriors, after, &format!("manual{i}"));
        assert_eq!(
            result.ammunition,
            reserves(after),
            "manual{i} all200 reserves"
        );
        crate::support::expect(result.loaded_item_field32, &after["item_context"]);
        assert_eq!(
            result.loaded,
            case.n::<i64>("reserve203") > 0,
            "manual{i} accepted"
        );
        assert_eq!(
            result.sounds,
            &sounds(after)[sounds(before).len()..],
            "manual{i} audio"
        );
        crate::support::native_expect! {
        scalar random.seed => &after["rng_seed"], "binary_empty_rifle_ammunition";
        scalar random.draws => &after["rng_draws"], "binary_empty_rifle_ammunition";
        }

        if result.loaded {
            assert_eq!(
                &after.array("draws")[before.array("draws").len()..],
                &[serde_json::json!({"bound":2,"result":result.sounds.last().unwrap()-403})],
                "manual{i} native ordered draw"
            );
        } else {
            assert_eq!(after["draws"], before["draws"], "manual{i} cancel draws");
        }
    }
}
#[test]
fn automatic_empty_refit_matches_twelve_native_direct_helper_cases() {
    let f = data();
    let content = content();

    for (i, case) in f.array("refit_cases").iter().enumerate() {
        let before = &case["operation_before"];
        let after = &case["after"];
        let native = party(after);
        let stock = case.n::<i64>("reserve203");
        assert_eq!(
            word(&native[0].bytes, 0x126).unwrap(),
            0,
            "refit{i} native cache"
        );
        assert_eq!(
            word(&native[0].bytes, 0x136).unwrap(),
            if stock == 12 {
                203
            } else if stock == 2 {
                200
            } else {
                0
            },
            "refit{i} native instance ID"
        );
        assert_eq!(
            reserves(after)[3],
            if stock == 12 { 9 } else { stock as i32 },
            "refit{i} native stock"
        );
        let mut warriors = party(before);
        let work = WorkBuffers {
            item: Some(before.bytes("item_work")),
            weapon: Some(before.bytes("weapon")),
            ammo: Some(before.bytes("ammo")),
        };
        let stock = reserves(before);
        let mut random = RandomState {
            seed: before.n::<u32>("rng_seed"),
            draws: before.n::<u64>("rng_draws"),
        };
        let original = (warriors.clone(), stock.clone(), work.clone());
        // VerifyEmptyRifleAmmunition invokes refit(0,0,1,1): audible,
        // forced, with warning49 suppressed. It never calls DoReload111.
        let result = ammunition::refit(
            &warriors[0],
            &stock,
            &content,
            RefitFlags {
                in_combat: case.n::<u64>("combat") != 0,
                force: true,
                silent: false,
                warnings: false,
            },
            &work,
            &mut random,
        )
        .unwrap();
        assert_eq!((warriors.clone(), stock, work), original);
        warriors[0] = result.warrior;
        assert_party(&warriors, after, &format!("refit{i} all six warriors"));
        assert_eq!(result.reserves, reserves(after), "refit{i} all200 reserves");
        crate::support::expect(result.native_return, &case["refit_return"]);
        for (actual, key) in [
            (result.work.item, "item_work"),
            (result.work.weapon, "weapon"),
            (result.work.ammo, "ammo"),
        ] {
            assert_eq!(
                actual,
                Some(bytes(after[key].as_str().unwrap())),
                "refit{i} full150 {key}"
            );
        }
        assert_eq!(result.sounds, sounds(after)[sounds(before).len()..]);
        assert!(result.warnings.is_empty());
        let mut independent = RandomState {
            seed: before.n::<u32>("rng_seed"),
            draws: before.n::<u64>("rng_draws"),
        };
        for draw in &after.array("draws")[before.array("draws").len()..] {
            crate::support::expect(independent.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
        assert_eq!(random, independent);
        crate::support::native_expect! {
        scalar random.seed => &after["rng_seed"], "binary_empty_rifle_ammunition";
        scalar random.draws => &after["rng_draws"], "binary_empty_rifle_ammunition";
        }
    }
}
