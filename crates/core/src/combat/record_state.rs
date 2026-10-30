use super::{
    Combat, CombatField, EnemySetup, SquadPlacement, empty_collide_check, empty_spell_queue,
};

impl Combat {
    /// A battle built from records alone; battle setup supplies the rest.
    pub fn from_records(map: CombatField, squad: SquadPlacement, roster: EnemySetup) -> Self {
        Self {
            kill_counts: super::runtime::kill_counts::KillCounts::startup(),
            neighbors: super::runtime::neighbor_scan::Neighbors::startup(),
            in_spell: 0,
            difficulty: 0,
            battle: 0,
            field: map,
            squad,
            roster,
            monsters: Default::default(),
            direct_deaths: false,
            fatigue_reset: false,
            alias_writes: vec![],
            item_condition_caller: None,
            monster_pick_written: false,
            monster_template: None,
            did_attack_write: None,
            direct_death: None,
            battle_order: vec![-1; super::BODY_SLOTS],
            battle_order_index: 1,
            combat_round: 0,
            blank_round: 0,
            selection_pending: false,
            macro_monster: 0,
            monster_up: 0,
            round_macros: Default::default(),
            todo_queue: Default::default(),
            spell_queue: empty_spell_queue(),
            collide_check: empty_collide_check(),
            spell_info: super::spell_info_startup(),
            cast_identity: [0; 3],
            weapon_traits: super::firearm::weapon_traits::WeaponTraits::zero(),
            kill_party: 0,
            pending_last_defeated: None,
            firearm_selection: None,
            capture_actor: None,
            do_auto: [false; 6],
            turn_origin: None,
            track_tail: super::track_tail_startup(),
            target_picks: Default::default(),
        }
    }
}
