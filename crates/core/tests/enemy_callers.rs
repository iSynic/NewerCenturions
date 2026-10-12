use crate::prelude::*;
use crate::support::{event_sounds, monsters, positions, record_bytes, warriors};

#[derive(Clone, Copy, PartialEq)]
pub enum Caller {
    Firearm,
    Spell,
    RetainedSpell,
}

fn raw_word(bytes: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

pub fn compare(cases: &[Value], caller: Caller) {
    let content = content();
    let mission = content.map(0).unwrap();
    for (index, case) in cases.iter().enumerate() {
        let before_records = case.bytes("records_before");
        let mut party = warriors(&case["warriors_before"]);
        let mut enemy_positions = vec![(20, 20)];
        if before_records.len() > 288 {
            enemy_positions.push((24, if caller == Caller::Firearm { 20 } else { 21 }));
        }
        let mut combat = Combat {
            battle: 129,
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: positions(&case["warrior_positions"]),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: monsters(&before_records),
                positions: enemy_positions,
            },
            order: case.words("order_before"),
            ..crate::support::battle()
        };
        if caller != Caller::Firearm {
            combat.spell_fields = case.bytes("fields_before");
            combat.field_used = case.bytes("used_before");
        }
        let mut random = RandomState::new(case.n("seed_before"));
        let mut work = EnemyWork {
            item_context: Some(case.n("item_context_before")),
            attack_count: case.n("attack_count_before"),
            ..EnemyWork::default()
        };
        if caller == Caller::RetainedSpell {
            let spell: i16 = case.n("retained_spell");
            work.packet = Some(case.bytes("packet_before"));
            work.identity = Some([spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1]);
            work.spell_attempt = 0;
            work.spell_disabled = 0;
        }
        let result = enemy_action_with_work(
            &mut combat,
            &mut party,
            mission,
            &mut random,
            &content.items,
            &mut work,
        );
        if case["diagnostic"].as_bool().unwrap_or(false) {
            assert!(matches!(result, Err(RuleError::Unsupported { .. })));
            continue;
        }
        let events = result.unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        crate::support::native_expect! {
            bytes record_bytes(&party) => &case["warriors_after"], format!("case {index} party");
            bytes record_bytes(&combat.enemies.records) => &case["records_after"], format!("case {index} enemies");
            words combat.map.tiles => &case["map_after"], format!("case {index} map");
            words combat.order => &case["order_after"], format!("case {index} order");
            scalar work.attack_count => &case["attack_count_after"], format!("case {index} count");
            scalar random.seed => &case["seed_after"], format!("case {index} seed");
            scalar random.draws => &case["rng_draws"], format!("case {index} draws");
            scalar event_sounds(&events) => &case["sounds"], format!("case {index} sounds");
        }
        assert_eq!(
            combat.enemies.positions,
            positions(&case["positions_after"])
        );
        let mut item_context = case.n::<i16>("item_context_after");
        let selected_item = raw_word(&before_records, 0x4e);
        if caller == Caller::Spell
            && selected_item != 0
            && (!case.array("casts").is_empty() || !case.array("resolutions").is_empty())
        {
            // The probe skips combatupdate2; reached casts refresh the selected item.
            item_context = raw_word(
                &content.items[usize::from(selected_item.unsigned_abs())],
                32,
            );
        }
        assert_eq!(work.item_context, Some(item_context));
        if caller != Caller::Firearm {
            crate::support::native_expect! {
                bytes combat.spell_fields => &case["fields_after"], format!("case {index} fields");
                bytes combat.field_used => &case["used_after"], format!("case {index} usage");
            }
        }
        if caller == Caller::RetainedSpell {
            crate::support::native_expect! {
                bytes work.packet.as_ref().unwrap() => &case["packet_after"], format!("case {index} packet");
                words work.identity.unwrap() => &case["spell_registers_after"], format!("case {index} identity");
            }
        }
        let expected = expected_hits(case, combat.enemies.records.len(), caller);
        let actual: Vec<_> = events
            .into_iter()
            .filter(|event| match caller {
                Caller::Firearm => matches!(event, Event::AttackResolved { .. }),
                _ => matches!(event, Event::SpellResolved { .. }),
            })
            .collect();
        assert_eq!(actual, expected, "case {index} ordered resolutions");
    }
}

fn expected_hits(case: &Value, enemy_count: usize, caller: Caller) -> Vec<Event> {
    let mut party = case.bytes("warriors_before");
    let mut records = case.bytes("records_before");
    let mut expected = vec![];
    for resolution in case.array("resolutions") {
        let spell = (caller != Caller::Firearm).then(|| resolution.n::<i16>("spell_id"));
        let mask = resolution.words("mask");
        let applied = spell.map(|_| resolution.words("applied_mask"));
        let packet = spell.map(|_| resolution.bytes("packet"));
        let after_party = resolution.bytes("party_after");
        let after_records = resolution.bytes("records_after");
        for (body, selected) in mask.iter().enumerate() {
            if *selected == 0
                || spell.is_some_and(|id| {
                    packet.as_ref().unwrap()[19] == 9
                        || (id != 1703 && applied.as_ref().unwrap()[body] == 0)
                })
            {
                continue;
            }
            let (before, after, hp) = if body < 6 {
                (
                    &party[body * 3138..(body + 1) * 3138],
                    &after_party[body * 3138..(body + 1) * 3138],
                    0x50,
                )
            } else if body >= 10 && body < 10 + enemy_count {
                (
                    &records[(body - 10) * 288..(body - 9) * 288],
                    &after_records[(body - 10) * 288..(body - 9) * 288],
                    0x110,
                )
            } else {
                panic!("unexpected selected body {body}");
            };
            let before_hp = raw_word(before, hp);
            let after_hp = raw_word(after, hp);
            let damage = before_hp.wrapping_sub(after_hp);
            expected.push(match spell {
                Some(spell) => Event::SpellResolved {
                    caster: Some(10),
                    spell,
                    target: body as i16,
                    damage,
                    defeated: after_hp < 1 && before_hp != after_hp,
                },
                None => Event::AttackResolved {
                    attacker: 10,
                    target: body as i16,
                    damage,
                    defeated: after_hp < 1,
                },
            });
        }
        party = after_party;
        records = after_records;
    }
    expected
}
