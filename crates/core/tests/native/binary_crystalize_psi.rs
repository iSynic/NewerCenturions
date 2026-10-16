use crate::prelude::*;
use crate::support::positions;

use crate::native_common::snapshot_party as party;

fn combat(s: &Value) -> Combat {
    Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(s.text("map")),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(s.text("party_positions")),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(&s["records"]),
            positions: positions(s.text("monster_positions")),
        },
        order: words(s.text("order")),
        cursor: s.n::<usize>("cursor"),
        round: s.n::<i16>("round"),
        spell_fields: s.bytes("fields"),
        field_used: s.bytes("used"),
        attack_packet: Some(s.bytes("packet")),
        attack_identity: Some(words(s.text("identity")).try_into().unwrap()),
        unavailable: s.n::<i8>("unavailable"),
        ..crate::support::battle()
    }
}
#[test]
fn learned_crystalize_matches_complete_native_cast_field_creation_cost_rng_and_audio() {
    let f = crate::support::fixture("crystalize-psi.json");
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
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&w) => &cast["party"], &format!("{label} complete party");
        bytes crate::support::record_bytes(&c.enemies.records) => &cast["records"], &format!("{label} enemies");
        words &(c.map.tiles) => &cast["map"], "binary_crystalize_psi";
        }
        assert_eq!(
            c.squad.positions,
            positions(cast.text("party_positions")),
            "{label} party positions"
        );
        assert_eq!(
            c.enemies.positions,
            positions(cast.text("monster_positions")),
            "{label} monster positions"
        );
        crate::support::native_expect! {
        bytes c.spell_fields => &cast["fields"], &format!("{label} all60fields");
        bytes c.field_used => &cast["used"], &format!("{label} all128usage");
        bytes c.attack_packet.as_ref().unwrap() => &cast["packet"], &format!("{label} packet");
        words &(c.attack_identity.unwrap().to_vec()) => &cast["identity"], "binary_crystalize_psi";
        bytes memory => &cast["memory"], &format!("{label} memory");
        scalar c.unavailable => &cast["unavailable"], "binary_crystalize_psi";
        words &(c.order) => &cast["order"], "binary_crystalize_psi";
        scalar c.cursor => &cast["cursor"], "binary_crystalize_psi";
        scalar c.round => &cast["round"], "binary_crystalize_psi";
        }
        assert_eq!(c.death_queue, original.death_queue);
        assert_eq!(c.round_scripts, original.round_scripts);
        crate::support::native_expect! {
        scalar random.seed => &cast["seed"], "binary_crystalize_psi";
        scalar random.draws => &cast["draws"], "binary_crystalize_psi";
        }
        let sounds: Vec<_> = crate::support::event_sounds(&events);
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
                } = e
                {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap();
        crate::support::expect(result, &cast["last_amount"]);

        let mut receipt_rng = RandomState::new(case.n::<u32>("seed_before"));
        crate::support::replay_rng(&mut receipt_rng, cast.array("rng_records"));
        assert_eq!(receipt_rng, random);
    }
}
