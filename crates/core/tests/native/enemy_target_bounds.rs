use crate::prelude::*;

use centurions_core::combat::runtime::select_enemy_target;

use crate::support::positions;
#[test]
fn invisible_target_scan_stays_in_bounds_without_extra_random_draws() {
    let f: serde_json::Value = crate::support::fixture("opening-enemy-prefix.json");
    let get = |key: &str| decode(f[key].as_str().unwrap());
    let warriors = crate::support::warriors(&get("warriors"));
    let records = crate::support::monsters(&get("enemies"));
    let mut combat = Combat {
        battle: 3,
        map: TacticalMap {
            tiles: words(&get("map")),
            shift: (-30, -36),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(&get("squad")),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records,
            positions: positions(&get("positions")),
        },
        order: words(&get("order")),
        ..crate::support::battle()
    };
    let mission = OpeningMission {
        tile_properties: get("properties")[64..12064].to_vec(),
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            ..crate::support::stories()
        }),
        ..crate::support::mission()
    };
    let mut random = RandomState::new(f.n::<u32>("initial_seed"));
    random.bound(100);
    random.bound(100);
    let target = select_enemy_target(&mut combat, &warriors, &mission, &mut random, 13).unwrap();
    crate::support::native_expect! {
    scalar target => &f["selected_target"], "enemy_target_bounds";
    scalar random.seed => &f["seed"], "enemy_target_bounds";
    scalar random.draws => &f["draws"], "enemy_target_bounds";
    }
    assert!(target < 6 || (10..16).contains(&target));
}
