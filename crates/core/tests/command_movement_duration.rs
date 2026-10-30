//! Source-derived authored spell 1106 routes: movement duration draws and effect58 summons.
use crate::{prelude::*, support};
use centurions_content::records::SpellInfoRecord;

/// Spell 1106 as authored: movement scaling, then effect58 copying MONSTER work.
#[test]
fn authored_1106_casts_keep_original_order_through_saved_commands() {
    let original = content();
    // Additional durations [2, 5]; or range 20, area mode 3, shape 1, no field, effect58.
    let movement: &[(usize, u8)] = &[(15, 2), (16, 5)];
    let summon: &[(usize, u8)] = &[(2, 20), (4, 0), (9, 3), (31, 1), (40, 0), (41, 58)];
    for authored in [movement, summon] {
        let mut content = original.clone();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        let tables = tables.combat.as_mut().unwrap();
        for &(offset, value) in authored {
            tables.attacks[5 * 58 + offset] = value;
        }
        let (low, high) = SpellInfoRecord(&tables.attacks[290..]).duration_range();
        // Thunder's cached loop ends on WORD937, which seeks with its own count, 81.
        let mut template = MonsterRecord::read(&tables.monsters, 81).unwrap();
        let mut state = support::mission_session(&content);
        // Automatic WORD 0x90 outside 442739: getbattleorder124ede also inserts body 1.
        state.warriors[3].record_mut().set_computer_control(1);
        begin_battle(&mut state, &content, 14, 0).unwrap();
        state.globals.define_spells = vec![0; 480];
        state.bandage_ready = 0;
        let w = &mut state.warriors[1].bytes;
        w[0x9f0..0xa21].fill(1);
        for (offset, value) in [(0x36, 1_i16), (0x40, 6), (0x58, 30000), (0xac6, 100)] {
            w[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        let combat = state.combat.as_mut().unwrap();
        assert_eq!(combat.battle_order.iter().filter(|b| **b == 1).count(), 2);
        combat.battle_order[combat.battle_order_index] = 1;
        let ((x, y), (dx, dy)) = (combat.squad.pos[1], combat.field.map_shift);
        let free = |&(x, y): &(i16, i16)| {
            let cell = OpeningMission::index(i32::from(x + dx + 5), i32::from(y + dy + 5));
            combat.field.tiles[cell.unwrap()] >= 0x6f
        };
        let target = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
            .into_iter()
            .find(free);
        let body = combat.roster.records.len() as i16 + 10;
        let power = if authored == movement { 2 } else { 1 };
        let command = Command::CastPsi {
            rotation: 0,
            spell: 1106,
            power,
            target,
        };
        let psi = state.warriors[1].psi_points().unwrap();
        let mut expected = state.random.clone();
        if authored == movement {
            // Movement draws its scaled duration before the interaction-marker warning.
            let mut base = state.clone();
            apply(command.clone(), &mut base, &original).unwrap();
            expected = base.random;
            (0..2).for_each(|_| _ = expected.range(2, 5));
            let events = support::saved_command(command, &mut state, &content);
            let warning = centurions_core::feedback::GameFeedback::Warning { index: 87 };
            assert!(events.contains(&Event::Feedback(warning)));
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, Event::TacticalMoved { .. }))
            );
            assert!(state.warriors[1].psi_points().unwrap() < psi);
            let in_spell = (
                state.combat.as_ref().unwrap().in_spell,
                state.globals.in_spell,
            );
            assert_eq!((in_spell, state.random), ((1, 1), expected));
            continue;
        }
        // A beast view rewrites MONSTER work, which the summon then refuses atomically.
        let mut viewed = state.clone();
        apply(Command::ViewCombatBody { body: 10 }, &mut viewed, &content).unwrap();
        let before = viewed.clone();
        let result = apply(command.clone(), &mut viewed, &content);
        assert!(result.is_err_and(|e| format!("{e:?}").contains("unrecorded MONSTER work")));
        assert_eq!(viewed, before);
        expected.range(low, high);
        template.record_mut().set_target(-1);
        let built = centurions_core::combat::monster::summon(template, -1, 0, &mut expected);
        let events = support::saved_command(command, &mut state, &content);
        assert!(events.contains(&Event::CombatantSummoned { body }));
        let combat = state.combat.as_ref().unwrap();
        let summoned = &combat.roster.records[body as usize - 10];
        let faction = state.warriors[1].record().faction().unwrap();
        assert_eq!(summoned.record().faction().unwrap(), faction);
        let stamina = built.unwrap().stamina().unwrap();
        assert_eq!(summoned.stamina().unwrap(), stamina);
        assert_eq!(
            (combat.roster.mon_pos.last(), state.random),
            (target.as_ref(), expected)
        );
    }
}
