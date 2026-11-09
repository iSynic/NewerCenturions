use centurions_content::mission::ScenarioContent;
use std::path::Path;

#[test]
fn level_custom_properties_survive_switching_and_cloned_map_caches() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../NewCenturions");
    let mut custom = vec![0; 16064];
    custom[1] = 155;
    custom[3] = 1;
    let level = ScenarioContent::read_level(
        |path| {
            if path.ends_with("/ BD") {
                assert_eq!(path, "TheNewCenturions/Scenarios/The Thunder of Guns/0/ BD");
                Ok(custom.clone())
            } else {
                std::fs::read(root.join(path)).map_err(Into::into)
            }
        },
        "The Thunder of Guns",
        0,
        Default::default(),
    )
    .unwrap();
    for terrain in [15, 19] {
        let land = level.land(0, terrain).unwrap();
        assert_eq!(
            (land.terrain, land.default_tile, land.minutes_per_tick),
            (terrain, 155, 1)
        );
        assert_eq!(land.tile_word(399, 38).unwrap(), 0);
    }
    let mut other = level.clone();
    std::sync::Arc::make_mut(&mut other.tile_sets)[0].default_tile = 166;
    assert_eq!(other.land(0, 15).unwrap().default_tile, 166);
    assert_eq!(level.land(0, 15).unwrap().default_tile, 155);
}
