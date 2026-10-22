//! Session copies of native work globals that outlive one combat roster.
//!
//! The original routines leave these globals in place between actions, battles
//! and screens; later callers read whatever the previous writer left. The
//! session keeps them so a child battle, a story or a reload sees the same
//! values. Combat-owned twins are copied back by `commands::transaction`
//! `finish_action`, `confirm_survivors`, retreat and newland100 cleanup, and seed
//! the next `Combat` in `begin_battle_with_group`.
//!
//! The fields are flattened into `Session`, so the saved JSON keeps its original
//! keys. Save schema 14 moved `party_mask` and `health_context` here; their keys
//! now serialize with this group instead of at their former `Session` positions.
//! Schema 15 added `random_battles` and `coordinates_hidden`.
use crate::{ammunition, combat, story_health};
use centurions_content::InventoryRecord;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeWorkContext {
    /// The first six WORDs of the 110-entry target mask at 0x4434e0, one per
    /// party member. getchoice and cleartarget write it, newland30 filters it
    /// and newland52 clears it; it stays set between actions for the next
    /// reader. Combat targeting, outdoor Psi, story spells, party statistics and
    /// the desktop Psi target view read it.
    #[serde(default)]
    pub party_mask: [i16; 6],
    /// WORD447196, the NPC most recently selected by getup or a warrior turn.
    /// Kept across combat teardown and seeded into the next battle; in-combat
    /// readers use `Combat::selected_enemy`.
    #[serde(default)]
    pub selected_enemy: usize,
    /// getup's retained script origin WORD43800e; an NPC script death skips its
    /// reset. Seeds the next battle like `selected_enemy`.
    #[serde(default)]
    pub script_enemy: usize,
    /// `retained_field32` of the last item loaditem placed in the item work
    /// record (field32 at 0x20). Exploration, equipment, shop and loot commands
    /// write it; enemy turns pass it to `enemy_fire` as the work-item context
    /// that selects enemy firearm ammunition (`enemy_weapons::ammo_id`).
    #[serde(default)]
    pub loaded_item_field32: Option<i16>,
    /// Item, ammunition and weapon work records following the editor backup at
    /// 0x45363c. Shared by equipment, shops, loot, refits, item estimates and
    /// spell restrictions, which read the item record's WORD 0x32 alias.
    #[serde(default)]
    pub ammunition_work: ammunition::WorkBuffers,
    /// The record booty1061fc..10630c last copied while sorting, later updated by
    /// equipment, shop and drop transactions. Refits read it as the retained
    /// battery row. `None` comes from saves older than version 6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory_record: Option<InventoryRecord>,
    /// NPC record, position and origin arrays (records439d3c, positions446f0a,
    /// origin442780). Stories read inactive NPCs through the combat copy that
    /// this seeds; victory, retreat and cleanup store it back.
    #[serde(default)]
    pub npc_backing: combat::retained::NpcBacking,
    /// The outer battle's death-script queue while a child battle replaces the
    /// roster. `finish_death_story` acknowledges it after the child returns, and
    /// the next battle resumes from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_death_queue: Option<combat::death_scripts::DeathQueue>,
    /// The 1000-byte table that the ammunition reserves alias past their end;
    /// reserve index 200 reads its first four bytes. Ammunition fitting reads it,
    /// and only saves supply it.
    #[serde(default)]
    pub loot_quality: Option<Vec<u8>>,
    /// Attack count of the last prepared attack, retained after combat for
    /// newland52 party saves, outdoor Psi and typed encounters.
    #[serde(default)]
    pub attack_count: i16,
    /// The retained 58-byte attack packet. Outdoor Psi, story spells (heal1287dc
    /// writes its effect byte here), promotion scans and typed encounters use it
    /// without a battle; newland52 falls back to it and it seeds the next battle.
    #[serde(default)]
    pub attack_packet: Option<Vec<u8>>,
    /// Class, level and index of the last attack's spell. Outdoor Psi copies it
    /// into `psi_memory`; it seeds the next battle.
    #[serde(default)]
    pub attack_identity: Option<[i16; 3]>,
    /// Retained weapon work words, including WORD4435d4's natural attack index.
    /// Ammunition work outside combat projects into it; it seeds the next battle.
    #[serde(default)]
    pub weapon_work: combat::weapon_work::WeaponWork,
    /// Psi selection memory at 0x43788a: six 80-byte rows, initialized from each
    /// warrior's 0xa76 bytes and written back to the warriors by the roster.
    #[serde(default)]
    pub psi_memory: Option<Vec<u8>>,
    /// Health work left by the last action: selected warrior, unavailable
    /// members, effect BYTE 0x443760, affected WORDs at 0x442798, marks-affected
    /// BYTE 0x442750 and killbody's last body WORD 0x408574. Later actions read
    /// it without reinitializing: party-defeat checks, outdoor healing, clinics,
    /// time and round/death stories. `None` until the first health action.
    #[serde(default)]
    pub health_context: Option<story_health::HealthContext>,
    /// BYTE402f42: setupnewgame sets it to 1 and newland104 to its parameter's
    /// low byte. Random-rectangle battles roll only while it is nonzero; region
    /// event scripts are unaffected. Saves before schema 15 never cleared it.
    #[serde(default = "random_battles_enabled")]
    pub random_battles: u8,
    /// WORD4394b4, set by newland66 and cleared by setupnewgame. While set,
    /// buttonchoice refuses to camp and lets an exhausted party move. The
    /// original save omits it, so a loaded session starts with camping allowed.
    #[serde(skip)]
    pub camp_disabled: bool,
    /// WORD4354fe, set by newland71 and kept in the original save. While nonzero
    /// xy138a40 draws the literal at 0x40226a for both coordinates.
    #[serde(default, skip_serializing_if = "coordinates_shown")]
    pub coordinates_hidden: i16,
}

fn coordinates_shown(hidden: &i16) -> bool {
    *hidden == 0
}

fn random_battles_enabled() -> u8 {
    1
}

impl NativeWorkContext {
    pub(crate) fn startup() -> Self {
        Self {
            party_mask: [0; 6],
            selected_enemy: 0,
            script_enemy: 0,
            loaded_item_field32: None,
            ammunition_work: ammunition::WorkBuffers::default(),
            inventory_record: Some(InventoryRecord([0; 22])),
            npc_backing: combat::retained::NpcBacking::startup(),
            retained_death_queue: None,
            loot_quality: None,
            attack_count: 0,
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            // CODE 1 startup initializes retained work words at 453776/4537f0/45375a to zero.
            weapon_work: combat::weapon_work::WeaponWork::zero(),
            psi_memory: Some(vec![0; 480]),
            health_context: None,
            random_battles: 1,
            camp_disabled: false,
            coordinates_hidden: 0,
        }
    }
}
