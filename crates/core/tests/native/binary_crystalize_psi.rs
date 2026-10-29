use crate::{native_common, prelude::*, support};

use native_common::snapshot_party as party;

use support::psi_frame::combat;
#[test]
fn learned_crystalize_matches_complete_native_cast_field_creation_cost_rng_and_audio() {
    let f = support::fixture("crystalize-psi.json");
    let content = content();
    let mission = content.map(0).unwrap();

    for (i, case) in f.array("cases").iter().enumerate() {
        let label = format!("crystalize{i}");
        let before = &case["before"];
        let cast = &case["cast"];
        let power = case.n::<i16>("power");
        let mut c = combat(before);
        let original = c.clone();
        let mut w = party(before);
        let mut memory = before.bytes("memory");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let point = words(case["casts"][0]["targets"].as_str().unwrap());
        let target = (point[0] - 5, point[1] - 5);
        let events = player_psi::cast(
            &mut c,
            &mut w,
            mission,
            &mut random,
            &mut memory,
            0,
            CastRequest {
                rotation: 0,
                spell: 2101,
                power,
                target: Some(target),
            },
        )
        .unwrap_or_else(|e| panic!("{label}:{e:?}"));
        support::psi_frame::compare(&c, &w, cast, &label);
        support::native_expect! {
            bytes memory => &cast["memory"], &label;
            scalar c.unavailable => &cast["unavailable"], &label;
            words c.order => &cast["order"], &label;
            scalar c.cursor => &cast["cursor"], &label;
            scalar c.round => &cast["round"], &label;
        }
        assert_eq!(c.death_queue, original.death_queue);
        assert_eq!(c.round_scripts, original.round_scripts);
        support::assert_rng(&random, &cast["seed"], &cast["draws"]);
        let sounds: Vec<_> = support::event_sounds(&events);
        assert_eq!(
            serde_json::json!(sounds),
            cast["sounds"],
            "{label} ordered raw audio"
        );
        let damage = word(&party(before)[1].bytes, 0x50).unwrap();
        assert_eq!(
            word(&w[1].bytes, 0x50).unwrap(),
            damage,
            "{label} caster health"
        );
        let result = events
            .iter()
            .find_map(|e| {
                if let Event::SpellResolved {
                    caster: Some(1),
                    spell: 2101,
                    target: 10,
                    damage,
                    defeated: false,
                    ..
                } = e
                {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap();
        support::expect(result, &cast["last_amount"]);

        let mut receipt_rng = RandomState::new(case.n::<u32>("seed_before"));
        support::replay_rng(&mut receipt_rng, cast.array("rng_records"));
        assert_eq!(receipt_rng, random);
    }
}
