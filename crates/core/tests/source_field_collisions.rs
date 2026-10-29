//! Source-derived field transitions; fixture inputs do not qualify these cases.
use crate::{prelude::*, support};
use centurions_content::records::{AttackPacket, AttackPacketMut, Condition};

#[test]
fn fields_check_resistance_before_shared_usage_and_do_not_repeat_effects() {
    let case = &support::fixture("spell-fields.json")["cases"][0];
    let warriors = support::warriors(&case["warriors_before"]);
    let enemies = support::monsters(&case["enemies_before"]);
    for (spell, excluded, count, revive) in [
        (2206, false, 0, 0),
        (2206, false, 9, 0),
        (2206, false, -1, 0),
        (1703, false, 2, 0),
        (2206, true, 2, 0),
        (1703, false, 1, -26_i16),
        (2603, false, 3, 0),
    ] {
        let resisted = spell == 2603;
        let mut enemies = enemies.clone();
        if resisted {
            for enemy in &mut enemies {
                enemy.record_mut().set_magic_immunity(0);
                enemy.record_mut().set_movement_points(6);
            }
        }
        let mut packet = spells::load_packet(&data("Data S"), spell).unwrap();
        AttackPacketMut(&mut packet).prepare_field_collision();
        if !resisted {
            packet[9] = 3;
        }
        let mut warriors = warriors.clone();
        if revive != 0 {
            warriors[0].record_mut().set_stamina(-10);
            warriors[0].bytes[0x2c..0x2e].fill(0);
            warriors[0].record_mut().set_shield_points(0);
            packet[32..36].copy_from_slice(&[0, 3, 0, 3]);
            packet[40..42].copy_from_slice(&revive.to_be_bytes());
        }
        if excluded {
            packet[50..52].copy_from_slice(&(-1_i16).to_be_bytes());
        }
        let context = DirectSpell {
            packet: &packet,
            level: spell % 1000 / 100 - 1,
            count,
            armor_filter: !resisted,
            weapon_sound: 0,
        };
        let mut mask = vec![0; 110];
        mask[if resisted { 10..12 } else { 0..2 }].fill(1);
        let mut usage = FieldUsage {
            slot: 7,
            used: vec![0; 128],
        };
        let mut random = RandomState::new(12);
        let cast = |party, used: &mut FieldUsage, random: &mut RandomState| {
            resolve_collision(party, &enemies, &mask, &context, used, random).unwrap()
        };
        let first = cast(&warriors, &mut usage, &mut random);
        let targets: Vec<_> = first.targets.iter().map(|(body, _)| *body).collect();
        let target = if resisted { 10 } else { 0 };
        assert_eq!(targets, if excluded { vec![] } else { vec![target] });
        if resisted {
            assert_eq!(first.enemies[0].record().condition(Condition::Helpless), 3);
            assert_eq!(first.enemies[0].record().movement_points(), 0);
            assert_eq!(first.enemies[1], enemies[1]);
            assert_eq!(random.draws, 6);
        } else if excluded {
            assert_eq!(first.warriors, warriors);
        } else if spell == 2206 {
            let condition = Condition::Poisoned;
            let before = warriors[0].record().condition(condition).unwrap();
            let after = first.warriors[0].record().condition(condition).unwrap();
            assert_eq!(after > before, count > 0);
        } else {
            assert_ne!(first.warriors[0], warriors[0]);
        }
        if revive != 0 {
            assert_eq!(
                first.warriors[0].stamina().unwrap(),
                warriors[0].record().maximum_stamina().unwrap() / 4 - first.targets[0].1.damage
            );
            assert_eq!(first.warriors[0].record().automatic_mode().unwrap(), -1);
        }
        assert_eq!(first.warriors[1], warriors[1]);
        assert_eq!(usage.used[8], u8::from(!excluded));
        let mut expected = random.clone();
        let (minimum, maximum) = AttackPacket(&packet).duration_bounds();
        expected.range(minimum, maximum);
        let (minimum, maximum) = AttackPacket(&packet).additional_duration_bounds();
        if minimum != 0 {
            for _ in 0..count {
                expected.range(minimum, maximum);
            }
        }
        if resisted {
            for _ in 0..2 {
                expected.bound(100);
            }
        }
        let second = cast(&first.warriors, &mut usage, &mut random);
        assert!(second.targets.is_empty());
        assert_eq!(second.warriors, first.warriors);
        assert_eq!(second.enemies, enemies);
        assert_eq!(random, expected);
    }
}
