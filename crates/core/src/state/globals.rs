//! Game values that outlast a single battle.
//!
//! The game leaves these values in place between actions, battles and screens,
//! and the next action reads whatever the last one left. The session keeps them
//! so a battle started by a story, the story itself and a reloaded game all see
//! the same values. When a battle ends, `commands::transaction` copies the
//! battle's values back here, and `begin_battle_with_group` seeds the next
//! `Combat` from them.
//!
//! The fields are flattened into `Session`, so saved games keep their keys. Save
//! schema 14 moved `party_track` and `health_context` here; schema 15 added
//! `random_battles` and `coordinates_hidden`.
use crate::{Session, ammunition, combat, story_health};
use centurions_content::{ITEM_BYTES, InventoryRecord};
use serde::{Deserialize, Serialize};

/// Game values that stay in place between actions, battles and screens, as the game leaves them.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Globals {
    /// The boat, its facing and the shore landing attempts survive map changes.
    /// The original save records only the boat and its facing.
    #[serde(default)]
    pub boat: u8,
    #[serde(default)]
    pub boat_facing: i8,
    #[serde(default)]
    pub shore_attempts: i16,
    /// Monster move attempts and `did_attack` survive script callbacks.
    #[serde(default)]
    #[serde(rename = "npc_work")]
    pub monster_turn_flags: combat::runtime::monster_turn_flags::MonsterTurnFlags,
    /// The monster record last built or picked; unknown in older saves.
    #[serde(default, skip_serializing_if = "combat::MonsterPick::is_unknown")]
    #[serde(rename = "monster_work")]
    pub monster_pick: combat::MonsterPick,
    /// The bestiary menu's selector, table and cached rows; save schema 53.
    #[serde(
        default,
        skip_serializing_if = "crate::bestiary::BestiaryMenu::is_unknown"
    )]
    pub bestiary: crate::bestiary::BestiaryMenu,
    /// Entries 6..109 of the spell target list; `party_track` holds the first six.
    #[serde(default = "track_tail_startup")]
    #[serde(rename = "spell_target_tail")]
    pub track_tail: Vec<i16>,
    /// Set when a battle ends early, so booty skips the treasure scene. Separate
    /// from `battle_rewards_suppressed`.
    #[serde(default)]
    #[serde(rename = "early_booty")]
    pub coward: i8,
    /// Makes the party walk involuntarily, whatever the warriors' conditions.
    #[serde(default)]
    #[serde(rename = "walking_confusion")]
    pub possessed: i16,
    /// The shop being visited. Fields a later shop does not set keep their values.
    #[serde(default)]
    #[serde(rename = "shop_work")]
    pub the_shop: crate::shop::TheShop,
    /// Bare-hand bonus of the character record last loaded or edited. Record
    /// loads and character screens set it apart from the selected warrior or
    /// attacker.
    #[serde(default)]
    pub character_bare_hand_bonus: i16,
    #[serde(
        default,
        skip_serializing_if = "combat::runtime::round_scripts::RoundScripts::is_unknown"
    )]
    #[serde(rename = "round_work")]
    pub round_scripts: combat::runtime::round_scripts::RoundScripts,
    /// Round number for turn messages after the battle arena closes. Only a live
    /// battle supplies it; older saves leave it unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_round: Option<i16>,
    /// The storage that holds the party's confiscated items and money.
    #[serde(default)]
    #[serde(rename = "recovery_work")]
    pub storage: crate::loot_recovery::Storage,
    /// Cleared when a battle is set up; set when booty returns from the
    /// combat-cleanup script operation.
    #[serde(default)]
    pub battle_rewards_suppressed: i8,
    /// The treasure record, kept apart from the loot list (save schema 32).
    #[serde(default = "treasure_startup")]
    #[serde(rename = "treasure_record")]
    pub treasure: Vec<u8>,
    /// The shared extra-item row; a later story can overwrite it.
    #[serde(default = "extra_startup")]
    pub extra_record: Vec<u8>,
    #[serde(default = "combat::battlefield::Battlefield::startup")]
    #[serde(rename = "tactical_work")]
    pub battlefield: combat::battlefield::Battlefield,
    #[serde(default)]
    #[serde(rename = "victory_work")]
    pub kill_counts: combat::runtime::kill_counts::KillCounts,
    /// Neighbor presence rows, body list and departure flag (save schema 28).
    #[serde(default)]
    #[serde(rename = "neighbor_work")]
    pub neighbors: combat::runtime::neighbor_scan::Neighbors,
    /// `in_spell` outside combat; a live battle keeps its own copy.
    #[serde(default)]
    #[serde(rename = "ready")]
    pub in_spell: i16,
    /// The death queue's busy and callback flags (save schema 26). Only the
    /// queue and the callback set them, never the routine that is running.
    #[serde(default)]
    pub death_processing: combat::death_scripts::DeathProcessing,
    /// The battle order and its cursor after a battle ends (save schema 27). A
    /// callback refresh reads the current order, never the saved attacker.
    #[serde(default = "combat::globals::BattleOrder::startup")]
    #[serde(rename = "initiative_work")]
    pub battle_order: combat::globals::BattleOrder,
    /// The first six entries of the 110-entry spell target list, one per party
    /// member. Choosing and clearing targets set them, the filter-party script
    /// operation filters them and pick-by-saving-throw clears them; they stay
    /// set between actions. Combat targeting, outdoor Psi, story spells, party
    /// statistics and the desktop Psi target view read them.
    #[serde(default)]
    #[serde(rename = "party_mask")]
    pub party_track: [i16; 6],
    /// Encounter headers and shared labels left by the last encounter loaded.
    #[serde(default)]
    #[serde(rename = "encounter_work")]
    pub encounter_headers: crate::story::EncounterHeaders,
    /// The monster most recently selected by getup or a warrior turn. Kept after the
    /// battle and seeded into the next one; during a battle read
    /// `Combat::monster_up`.
    #[serde(default)]
    #[serde(rename = "selected_enemy")]
    pub monster_up: usize,
    /// The monster whose script getup last ran. A monster's script death does not
    /// reset it. Seeds the next battle like `monster_up`.
    #[serde(default)]
    #[serde(rename = "script_enemy")]
    pub macro_monster: usize,
    /// Field 32 of the last item loaded for use. Exploration, equipment, shop and
    /// loot commands set it; enemy turns pass it to `enemy_fire`, where it picks
    /// the enemy's firearm ammunition (`enemy_weapons::ammo_id`).
    #[serde(default)]
    pub loaded_item_field32: i16,
    /// The item, ammunition and weapon in use. Equipment, shops, loot, refits,
    /// item estimates and spell restrictions share them.
    #[serde(default)]
    #[serde(rename = "ammunition_work")]
    pub loaded_items: ammunition::LoadedItems,
    /// The inventory record booty last copied while sorting, later changed by
    /// equipment, shop and drop transactions. Refits read it as the battery row.
    /// A new game starts with an empty record.
    #[serde(default = "inventory_record_startup")]
    pub inventory_record: InventoryRecord,
    /// The Items list scroll position. Only the Items screen changes it.
    #[serde(default)]
    pub inventory_scroll: u16,
    /// Monster records, positions and origins. Stories read inactive monsters through the
    /// battle copy seeded from here; victory, retreat and cleanup store it back.
    #[serde(default)]
    #[serde(rename = "npc_backing")]
    pub monsters: combat::globals::Monsters,
    /// The outer battle's death-script queue while a child battle replaces the
    /// roster. `finish_death_story` acknowledges it after the child returns, and
    /// the next battle resumes from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "retained_death_queue")]
    pub outer_todo_queue: Option<combat::death_scripts::TodoQueue>,
    /// Loot quality, one byte per booty entry; reserve index 200 shares the
    /// first four. Booty clears all 1000 before refitting; older saves leave it
    /// unknown.
    #[serde(default)]
    pub loot_quality: Option<Vec<u8>>,
    /// Attack count of the last prepared attack. Pick-by-saving-throw, outdoor
    /// Psi and typed encounters read it after the battle.
    #[serde(default)]
    pub attack_count: i16,
    /// `spell_info` of the last attack. Outdoor Psi, story spells, promotion scans
    /// and typed encounters use it outside battle, and story healing writes its
    /// effect. Pick-by-saving-throw falls back to it; it seeds the next battle.
    #[serde(default = "combat::spell_info_startup")]
    #[serde(rename = "attack_packet")]
    pub spell_info: Vec<u8>,
    /// Class, level and index of the last attack's spell. Outdoor Psi copies it
    /// into `define_spells`; it seeds the next battle.
    #[serde(default)]
    #[serde(rename = "attack_identity")]
    pub cast_identity: [i16; 3],
    /// The weapon in use, including the natural attack index. Ammunition changes
    /// outside combat update it; it seeds the next battle.
    #[serde(default)]
    #[serde(rename = "weapon_work")]
    pub weapon_traits: combat::firearm::weapon_traits::WeaponTraits,
    /// Psi selection memory: six 80-byte rows, loaded from each warrior's
    /// `define_spells` and written back to the warriors by the roster.
    #[serde(default = "define_spells_startup")]
    #[serde(rename = "psi_memory")]
    pub define_spells: Vec<u8>,
    /// What the last action did to the party's health: the selected warrior, the
    /// unavailable members, the effect, the affected members, whether marks were
    /// affected and the last body killbody handled. Later actions read it without
    /// resetting it: party-defeat checks, outdoor healing, clinics, time and
    /// round or death stories. A new game starts from the initial values.
    #[serde(default = "story_health::HealthContext::startup")]
    pub health_context: story_health::HealthContext,
    /// Random-rectangle battles roll only while this is nonzero; region event
    /// scripts are unaffected. A new game sets it to 1 and the set-random-battles
    /// script operation to its parameter's low byte. Saves before schema 15 never
    /// cleared it.
    #[serde(default = "random_battles_enabled")]
    pub random_battles: u8,
    /// Set by the camping-ability script operation and cleared for a new game.
    /// While set the party cannot camp, and an exhausted party may still move.
    /// The original save omits it, so a loaded game allows camping.
    #[serde(skip)]
    pub camp_disabled: bool,
    /// Set by the coordinate-display script operation and kept in the save. While
    /// nonzero the coordinate display shows the hidden text for both coordinates.
    #[serde(default, skip_serializing_if = "coordinates_shown")]
    pub coordinates_hidden: i16,
    /// Party-wide values that equipment sets by selector, as address/value pairs
    /// sorted by address. Only values that wear or removal wrote are known;
    /// difficulty and the effect 110 tail live in their own Session fields. Saves
    /// before schema 50 record none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_globals: Vec<(u32, i16)>,
    /// The map header row newland chooses when a header matches the party
    /// position; zero at launch (schema 58).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub header_row: i16,
    /// The item the fire button, Items or encounter2 leaves for combatchoice's
    /// dropitem and prepattack; getup and showspellinfo clear it. Zero at launch
    /// (schema 58).
    #[serde(default, skip_serializing_if = "is_zero")]
    #[serde(rename = "fire_item")]
    pub item_used: i16,
    /// Target selections kept outside a live battle (schema 60).
    #[serde(
        default,
        skip_serializing_if = "combat::targeting::target_picks::TargetPicks::is_startup"
    )]
    #[serde(rename = "target_work")]
    pub target_picks: combat::targeting::target_picks::TargetPicks,
    /// Warrior table records 0..13 outside the squad: the rest of the party
    /// table, the two held Upgrade records and the editor backup. Zero at launch (schema 61).
    #[serde(default, skip_serializing_if = "super::WarriorTable::is_empty")]
    pub warrior_table: super::WarriorTable,
    /// Remembered Psi power per member where a recorded menu choice does not
    /// supply it (see `Session::psi_power`). castspell, checkkeypad and combat
    /// store the cast power with the choice. Bodies 6..9 keep their under-tile
    /// values in the same place as members 0..3, and partyselect neither clears
    /// nor moves them. Zero at launch (schema 62).
    #[serde(default, skip_serializing_if = "no_held_powers")]
    #[serde(rename = "psi_powers")]
    pub old_power_level: [Option<i16>; 6],
}

/// Whether no warrior is holding a power.
fn no_held_powers(powers: &[Option<i16>; 6]) -> bool {
    powers.iter().all(Option::is_none)
}

/// Whether a value is zero, so it can be left out of a save.
fn is_zero(value: &i16) -> bool {
    *value == 0
}

/// Whether the coordinates are shown, which is the case when they are not hidden.
fn coordinates_shown(hidden: &i16) -> bool {
    *hidden == 0
}

/// The item record before any item has been loaded: blank.
fn inventory_record_startup() -> InventoryRecord {
    InventoryRecord([0; 22])
}

/// Size of the treasure record.
pub(crate) const TREASURE_RECORD_BYTES: usize = 88;
/// Size of the shared extra-item row.
pub(crate) const EXTRA_RECORD_BYTES: usize = 10;
/// One loot-quality byte per booty entry.
pub(crate) const LOOT_QUALITY_BYTES: usize = 1000;

/// A treasure record before any treasure has been shown: blank.
fn treasure_startup() -> Vec<u8> {
    vec![0; TREASURE_RECORD_BYTES]
}

/// An extra record before any script has loaded one: blank.
fn extra_startup() -> Vec<u8> {
    vec![0; EXTRA_RECORD_BYTES]
}

/// A spell target list tail before any spell has chosen targets: empty ranks.
fn track_tail_startup() -> Vec<i16> {
    vec![0; combat::TARGET_TAIL_WORDS]
}

/// The party's psi memory before any spell is selected: blank.
fn define_spells_startup() -> Vec<u8> {
    vec![0; crate::DEFINE_SPELLS_BYTES]
}

/// Random battles are on at the start of a game.
fn random_battles_enabled() -> u8 {
    1
}

impl Globals {
    /// The values of a new game.
    pub(crate) fn startup() -> Self {
        Self {
            boat: 0,
            equipment_globals: Default::default(),
            boat_facing: 0,
            shore_attempts: 0,
            monster_turn_flags: combat::runtime::monster_turn_flags::MonsterTurnFlags::startup(),
            monster_pick: combat::MonsterPick::startup(),
            bestiary: crate::bestiary::BestiaryMenu::startup(),
            track_tail: track_tail_startup(),
            coward: 0,
            possessed: 0,
            the_shop: crate::shop::TheShop::startup(),
            character_bare_hand_bonus: 0,
            round_scripts: Default::default(),
            turn_round: None,
            storage: crate::loot_recovery::Storage::startup(),
            battle_rewards_suppressed: 0,
            treasure: treasure_startup(),
            extra_record: extra_startup(),
            battlefield: combat::battlefield::Battlefield::startup(),
            kill_counts: combat::runtime::kill_counts::KillCounts::startup(),
            neighbors: combat::runtime::neighbor_scan::Neighbors::startup(),
            in_spell: 0,
            death_processing: combat::death_scripts::DeathProcessing {
                doing_queue: 0,
                callback: 0,
            },
            battle_order: combat::globals::BattleOrder::startup(),
            party_track: [0; 6],
            encounter_headers: crate::story::EncounterHeaders::startup(),
            monster_up: 0,
            macro_monster: 0,
            loaded_item_field32: 0,
            loaded_items: ammunition::LoadedItems {
                item: Some(vec![0; ITEM_BYTES]),
                ammo: Some(vec![0; ITEM_BYTES]),
                weapon: Some(vec![0; ITEM_BYTES]),
            },
            inventory_record: inventory_record_startup(),
            inventory_scroll: 0,
            monsters: combat::globals::Monsters::startup(),
            outer_todo_queue: None,
            loot_quality: Some(vec![0; LOOT_QUALITY_BYTES]),
            attack_count: 0,
            spell_info: combat::spell_info_startup(),
            cast_identity: [0; 3],
            weapon_traits: combat::firearm::weapon_traits::WeaponTraits::zero(),
            define_spells: define_spells_startup(),
            health_context: story_health::HealthContext::startup(),
            random_battles: 1,
            camp_disabled: false,
            coordinates_hidden: 0,
            header_row: 0,
            item_used: 0,
            target_picks: Default::default(),
            warrior_table: Default::default(),
            old_power_level: [None; 6],
        }
    }
}

impl Session {
    /// The Psi power a member's menu offers: the remembered power, else the
    /// recorded choice's power.
    ///
    /// Original: castspell
    pub(crate) fn psi_power(&self, member: usize) -> i16 {
        self.globals.old_power_level[member].unwrap_or_else(|| {
            self.psi_menu_choices[member]
                .as_ref()
                .map_or(0, |choice| choice.power)
        })
    }

    /// Remembers the powers a menu-choice change left unchanged.
    pub(crate) fn keep_psi_powers(&mut self, words: [i16; 6]) {
        for (member, word) in words.into_iter().enumerate() {
            self.globals.old_power_level[member] = None;
            if self.psi_power(member) != word {
                self.globals.old_power_level[member] = Some(word);
            }
        }
    }

    /// Clears the spell target list. A live battle owns entries 6..109 until it
    /// ends and copies them back.
    ///
    /// Original: cleartarget
    pub(crate) fn clear_spell_targets(&mut self) {
        self.globals.party_track.fill(0);
        *self.track_tail_mut() = vec![0; combat::TARGET_TAIL_WORDS];
    }

    /// Stores the spell target list: its first six entries belong to the party, the rest to the
    /// tail.
    pub(crate) fn publish_spell_targets(&mut self, mask: &[i16; 110]) {
        self.globals.party_track.copy_from_slice(&mask[..6]);
        *self.track_tail_mut() = mask[6..].to_vec();
    }

    /// Clears every target choice to -1 and empties the spell target list.
    ///
    /// Original: cleartarget
    pub(crate) fn clear_targets(&mut self) {
        self.clear_spell_targets();
        self.target_picks_mut().clear();
    }

    /// The target picks of the battle, or of the field when no battle is under way.
    pub(crate) fn target_picks_mut(&mut self) -> &mut combat::targeting::target_picks::TargetPicks {
        match self.combat.as_mut() {
            Some(combat) => &mut combat.target_picks,
            None => &mut self.globals.target_picks,
        }
    }

    /// The tail of the spell target list, in the battle or outside it.
    fn track_tail_mut(&mut self) -> &mut Vec<i16> {
        match self.combat.as_mut() {
            Some(combat) => &mut combat.track_tail,
            None => &mut self.globals.track_tail,
        }
    }

    /// The whole spell target list: the six party entries followed by the tail.
    pub(crate) fn spell_targets(&self) -> Result<[i16; 110], crate::RuleError> {
        let tail = match self.combat.as_ref() {
            Some(combat) => &combat.track_tail,
            None => &self.globals.track_tail,
        };
        if tail.len() != 104 {
            return Err("invalid retained spell target mask".into());
        }
        let mut mask = [0; 110];
        mask[..6].copy_from_slice(&self.globals.party_track);
        mask[6..].copy_from_slice(tail);
        Ok(mask)
    }

    /// Sets whether a spell is in effect, in the session and in the battle.
    pub(crate) fn set_in_spell(&mut self, in_spell: i16) {
        self.globals.in_spell = in_spell;
        if let Some(combat) = self.combat.as_mut() {
            combat.in_spell = in_spell;
        }
    }

    /// `char_select_new` is separate from the warrior whose turn it is (`char_up`).
    pub(crate) fn set_warrior_selector(&mut self, warrior: usize) {
        let health = &mut self.globals.health_context;
        health.char_select_new = warrior;
    }

    /// Copies the battle's kill counts, neighbours, spell state and monster values into the
    /// party-wide globals.
    pub(crate) fn store_combat_globals(&mut self) {
        self.store_monster_turn_flags();
        self.store_monster_pick();
        if let Some(combat) = self.combat.as_ref() {
            self.globals.kill_counts = combat.kill_counts;
            self.globals.neighbors = combat.neighbors.clone();
            self.globals.in_spell = combat.in_spell;
            self.globals.monster_up = combat.monster_up;
            self.globals.macro_monster = combat.macro_monster;
            self.globals.spell_info = combat.spell_info.clone();
            self.globals.cast_identity = combat.cast_identity;
            self.globals.weapon_traits = combat.weapon_traits;
            let health = &mut self.globals.health_context;
            health.kill_party = combat.kill_party;
            let packet = &combat.spell_info;
            health.effect = centurions_content::records::SpellInfoRecord(packet).health_effect();
        }
    }
}
