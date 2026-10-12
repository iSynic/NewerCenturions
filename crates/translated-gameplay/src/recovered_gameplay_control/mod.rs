//! Combat, attacks, casting, encounters, movement and area-transition control.

mod attack;
pub use attack::attack;
pub use attack::attack_body;
mod attack_secondary;
pub use attack_secondary::attack_secondary;
pub use attack_secondary::attack_secondary_body;
mod button_choice;
pub use button_choice::button_choice;
pub use button_choice::button_choice_body;
mod cast;
pub use cast::cast;
pub use cast::cast_body;
mod check_for_enemy;
pub use check_for_enemy::check_for_enemy;
pub use check_for_enemy::check_for_enemy_body;
mod collide;
pub use collide::collide;
pub use collide::collide_body;
mod combat;
pub use combat::combat;
pub use combat::combat_body;
mod combat_choice;
pub use combat_choice::combat_choice;
pub use combat_choice::combat_choice_body;
mod encounter;
pub use encounter::encounter;
pub use encounter::encounter_body;
mod move_monster;
pub use move_monster::move_monster;
pub use move_monster::move_monster_body;
mod new_land;
pub use new_land::new_land;
pub use new_land::new_land_body;
mod advance_time;
pub use advance_time::advance_time;
pub use advance_time::advance_time_body;

pub fn owns_frame(entry: u32) -> bool {
    matches!(
        entry,
        0x100af0
            | 0x101eb6
            | 0x10950e
            | 0x10d4fe
            | 0x110e96
            | 0x1140ae
            | 0x114378
            | 0x117be2
            | 0x12185a
            | 0x13e276
            | 0x1400e4
            | 0x15e148
    )
}
