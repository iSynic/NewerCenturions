use super::{
    Combat, EnemySetup, SquadPlacement, TacticalMap, empty_field_used, empty_spell_fields,
};

impl Combat {
    /// Assemble record-only work. combatsetup supplies its actual retained globals.
    pub fn from_records(map: TacticalMap, squad: SquadPlacement, enemies: EnemySetup) -> Self {
        Self {
            victory_work: super::victory_work::VictoryWork::startup(),
            neighbor_work: super::neighbor_work::NeighborWork::startup(),
            ready: 0,
            difficulty: 0,
            battle: 0,
            map,
            squad,
            enemies,
            retained_npcs: Default::default(),
            direct_deaths: false,
            fatigue_reset: false,
            alias_writes: vec![],
            item_condition_caller: None,
            monster_work_written: false,
            monster_template: None,
            npc_inhibit_write: None,
            direct_death: None,
            order: vec![-1; 110],
            cursor: 1,
            round: 0,
            inactive_rounds: 0,
            selection_pending: false,
            script_enemy: 0,
            selected_enemy: 0,
            round_scripts: Default::default(),
            death_queue: Default::default(),
            spell_fields: empty_spell_fields(),
            field_used: empty_field_used(),
            attack_packet: super::attack_packet_startup(),
            attack_identity: [0; 3],
            weapon_work: super::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            pending_last_defeated: None,
            firearm_selection: None,
            capture_actor: None,
            automatic: [false; 6],
            turn_origin: None,
            target_tail: super::target_tail_startup(),
            target_work: Default::default(),
        }
    }
}
