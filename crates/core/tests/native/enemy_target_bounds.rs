use crate::{prelude::*, support};

use centurions_core::combat::runtime::select_enemy_target;

use support::positions;
#[test]
fn unknown_retained_target_scan_stops_without_extra_random_draws() {
    let f: serde_json::Value = support::fixture("opening-enemy-prefix.json");
    let get = |key: &str| decode(f[key].as_str().unwrap());
    let warriors = support::warriors(&get("warriors"));
    let records = support::monsters(&get("enemies"));
    let mut combat = support::battle();
    combat.battle = 3;
    combat.map.tiles = words(&get("map"));
    combat.map.shift = (-30, -36);
    combat.squad.positions = positions(&get("squad"));
    combat.enemies.records = records;
    combat.enemies.positions = positions(&get("positions"));
    combat.order = words(&get("order"));
    let mission = OpeningMission {
        tile_properties: get("properties")[64..12064].to_vec(),
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            ..support::stories()
        }),
        ..support::mission()
    };
    let mut random = RandomState::new(f.n::<u32>("initial_seed"));
    random.bound(100);
    random.bound(100);
    // This historical frame omits inactive backing; its original scan reached
    // body197. Preserve the known RNG prefix, without inventing a target record.
    assert!(matches!(
        select_enemy_target(&mut combat, &warriors, &mission, &mut random, 13),
        Err(centurions_core::RuleError::Unsupported {
            action: "combat.retained_npc",
            ..
        })
    ));
    support::assert_rng(&random, &f["seed"], &f["draws"]);
}
