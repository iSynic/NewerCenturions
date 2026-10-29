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
use crate::{Session, ammunition, combat, story_health};
use centurions_content::InventoryRecord;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeWorkContext {
    /// BYTE442748, signed BYTE442740 and signed WORD437a86 survive map changes.
    /// Native saves record the first two only.
    #[serde(default)]
    pub boat: u8,
    #[serde(default)]
    pub boat_facing: i8,
    #[serde(default)]
    pub shore_attempts: i16,
    /// WORD438780 attempts and WORD43877e attack inhibit survive callback returns.
    #[serde(default)]
    pub npc_work: combat::npc_work::NpcWork,
    /// Current288-byte MONSTER scratch at440dbc; absent history stays unknown.
    #[serde(default, skip_serializing_if = "combat::MonsterWork::is_unknown")]
    pub monster_work: combat::MonsterWork,
    /// updatemonstermenu's selector, table and modelled caches; schema 53.
    #[serde(
        default,
        skip_serializing_if = "crate::bestiary::BestiaryWork::is_unknown"
    )]
    pub bestiary: crate::bestiary::BestiaryWork,
    /// WORDs4434ec..4435ba complete the110-word spell mask after party_mask.
    #[serde(default = "spell_target_tail_startup")]
    pub spell_target_tail: Vec<i16>,
    /// BYTE4386fc selects booty's early return; distinct from BYTE435658.
    #[serde(default)]
    pub early_booty: i8,
    /// WORD438764 selects involuntary walking independently of record conditions.
    #[serde(default)]
    pub walking_confusion: i16,
    /// loadshop/moveicon record at 0x478288; unwritten tails survive later loads.
    #[serde(default)]
    pub shop_work: crate::shop::ShopWork,
    /// Signed WORD447d96 in the retained character work record at44721c.
    /// Record loads and character workflows write it independently of the
    /// selected warrior or attacker.
    #[serde(default)]
    pub character_bare_hand_bonus: i16,
    #[serde(
        default,
        skip_serializing_if = "combat::round_work::RoundWork::is_unknown"
    )]
    pub round_work: combat::round_work::RoundWork,
    /// Reconstruction round number for turn notifications after the arena is detached.
    /// Only a represented live Combat supplies this value; legacy history stays unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_round: Option<i16>,
    #[serde(default)]
    pub recovery_work: crate::loot_recovery::RecoveryWork,
    /// BYTE435658 is reset by combatsetup and set after newland100 booty returns.
    #[serde(default)]
    pub battle_rewards_suppressed: i8,
    /// Schema32 retains the88-byte treasure work at44331c, independently of loot.
    #[serde(default = "treasure_startup")]
    pub treasure_record: Vec<u8>,
    /// Shared10-byte EXTRA work at44330a; a later story can overwrite the row.
    #[serde(default = "extra_startup")]
    pub extra_record: Vec<u8>,
    #[serde(default = "combat::tactical_work::TacticalWork::startup")]
    pub tactical_work: combat::tactical_work::TacticalWork,
    #[serde(default)]
    pub victory_work: combat::victory_work::VictoryWork,
    /// Schema28 retains the neighbor presence rows, body list and departure WORD.
    #[serde(default)]
    pub neighbor_work: combat::neighbor_work::NeighborWork,
    /// Signed WORD438760 retained outside combat. The live Combat owns its copy.
    #[serde(default)]
    pub ready: i16,
    /// Schema26 records independent queue/callback bytes; an active caller is
    /// not a writer of either byte.
    #[serde(default)]
    pub death_processing: combat::death_scripts::DeathProcessing,
    /// Schema27 retains the initiative WORD array/cursor after combat teardown.
    /// Callback refresh reads the current global row, never the saved attacker.
    #[serde(default = "combat::retained::InitiativeWork::startup")]
    pub initiative_work: combat::retained::InitiativeWork,
    /// The first six WORDs of the 110-entry target mask at 0x4434e0, one per
    /// party member. getchoice and cleartarget write it, newland30 filters it
    /// and newland52 clears it; it stays set between actions for the next
    /// reader. Combat targeting, outdoor Psi, story spells, party statistics and
    /// the desktop Psi target view read it.
    #[serde(default)]
    pub party_mask: [i16; 6],
    /// Separate encounter headers and shared labels left by native loaders.
    #[serde(default)]
    pub encounter_work: crate::story::EncounterWork,
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
    pub loaded_item_field32: i16,
    /// Item, ammunition and weapon work records following the editor backup at
    /// 0x45363c. Shared by equipment, shops, loot, refits, item estimates and
    /// spell restrictions, which read the item record's WORD 0x32 alias.
    #[serde(default)]
    pub ammunition_work: ammunition::WorkBuffers,
    /// The record booty1061fc..10630c last copied while sorting, later updated by
    /// equipment, shop and drop transactions. Refits read it as the retained
    /// battery row. The startup image holds a zero record.
    #[serde(default = "inventory_record_startup")]
    pub inventory_record: InventoryRecord,
    /// The Items scroll WORD44719c. Only Items writes it.
    #[serde(default)]
    pub inventory_scroll: u16,
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
    /// Loot-quality bytes42df04..42e2eb; reserve index200 aliases the first four.
    /// Booty resets all1000 bytes before refit; absent legacy history stays unknown.
    #[serde(default)]
    pub loot_quality: Option<Vec<u8>>,
    /// Attack count of the last prepared attack, retained after combat for
    /// newland52 party saves, outdoor Psi and typed encounters.
    #[serde(default)]
    pub attack_count: i16,
    /// The retained 58-byte attack packet. Outdoor Psi, story spells (heal1287dc
    /// writes its effect byte here), promotion scans and typed encounters use it
    /// without a battle; newland52 falls back to it and it seeds the next battle.
    #[serde(default = "combat::attack_packet_startup")]
    pub attack_packet: Vec<u8>,
    /// Class, level and index of the last attack's spell. Outdoor Psi copies it
    /// into `psi_memory`; it seeds the next battle.
    #[serde(default)]
    pub attack_identity: [i16; 3],
    /// Retained weapon work words, including WORD4435d4's natural attack index.
    /// Ammunition work outside combat projects into it; it seeds the next battle.
    #[serde(default)]
    pub weapon_work: combat::weapon_work::WeaponWork,
    /// Psi selection memory at 0x43788a: six 80-byte rows, initialized from each
    /// warrior's 0xa76 bytes and written back to the warriors by the roster.
    #[serde(default = "psi_memory_startup")]
    pub psi_memory: Vec<u8>,
    /// Health work left by the last action: selected warrior, unavailable
    /// members, effect BYTE 0x443760, affected WORDs at 0x442798, marks-affected
    /// BYTE 0x442750 and killbody's last body WORD 0x408574. Later actions read
    /// it without reinitializing: party-defeat checks, outdoor healing, clinics,
    /// time and round/death stories. Fresh sessions copy initialized DATA.
    #[serde(default = "story_health::HealthContext::startup")]
    pub health_context: story_health::HealthContext,
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
    /// Party-wide WORDs that equipment wrote at 0x442562 + 2 * (selector - 30),
    /// as address/value pairs sorted by address. Only values that wear or
    /// removal wrote are known; difficulty and the effect110 tail live in their
    /// own Session fields. Schema 50 added them; older saves record none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_globals: Vec<(u32, i16)>,
    /// WORD4427c2, the map header row newland 0x1402de writes when a header matches
    /// the party position; zero at launch (schema 58).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub header_row: i16,
    /// WORD44719a, the item the fire button, Items or encounter2 leaves for
    /// combatchoice's dropitem and prepattack; getup and showspellinfo clear it.
    /// Zero at launch (schema 58).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fire_item: i16,
    /// WORDs 443722/443724, 4435c0 and 442bd4 outside live combat; schema 60.
    #[serde(
        default,
        skip_serializing_if = "combat::target_work::TargetWork::is_startup"
    )]
    pub target_work: combat::target_work::TargetWork,
    /// Warrior table records 0..13 outside the squad: the party table tail, the
    /// Upgrade records 0x44d42c/0x44e06e and the editor backup 0x44ecb0. Zero at
    /// launch (schema 61).
    #[serde(default, skip_serializing_if = "super::WarriorTable::is_empty")]
    pub warrior_table: super::WarriorTable,
    /// Remembered Psi power WORDs 0x437a7a + 2 * member where a recorded menu choice
    /// does not supply them (see `Session::psi_power`). castspell (0x10d328),
    /// checkkeypad (0x111e0e) and combat (0x1169e4) store the cast power with the
    /// choice; under-tile WORDs 0x437a6e + 2 * body for bodies 6..9 alias members
    /// 0..3, and partyselect neither clears nor moves them. Zero at launch (schema 62).
    #[serde(default, skip_serializing_if = "no_held_powers")]
    pub psi_powers: [Option<i16>; 6],
}

fn no_held_powers(powers: &[Option<i16>; 6]) -> bool {
    powers.iter().all(Option::is_none)
}

fn is_zero(value: &i16) -> bool {
    *value == 0
}

fn coordinates_shown(hidden: &i16) -> bool {
    *hidden == 0
}

fn inventory_record_startup() -> InventoryRecord {
    InventoryRecord([0; 22])
}

fn treasure_startup() -> Vec<u8> {
    vec![0; 88]
}

fn extra_startup() -> Vec<u8> {
    vec![0; 10]
}

fn spell_target_tail_startup() -> Vec<i16> {
    vec![0; 104]
}

fn psi_memory_startup() -> Vec<u8> {
    vec![0; 480]
}

fn random_battles_enabled() -> u8 {
    1
}

impl NativeWorkContext {
    pub(crate) fn startup() -> Self {
        Self {
            boat: 0,
            equipment_globals: Default::default(),
            boat_facing: 0,
            shore_attempts: 0,
            npc_work: combat::npc_work::NpcWork::startup(),
            monster_work: combat::MonsterWork::startup(),
            bestiary: crate::bestiary::BestiaryWork::startup(),
            spell_target_tail: spell_target_tail_startup(),
            early_booty: 0,
            walking_confusion: 0,
            shop_work: crate::shop::ShopWork::startup(),
            character_bare_hand_bonus: 0,
            round_work: Default::default(),
            turn_round: None,
            recovery_work: crate::loot_recovery::RecoveryWork::startup(),
            battle_rewards_suppressed: 0,
            treasure_record: treasure_startup(),
            extra_record: extra_startup(),
            tactical_work: combat::tactical_work::TacticalWork::startup(),
            victory_work: combat::victory_work::VictoryWork::startup(),
            neighbor_work: combat::neighbor_work::NeighborWork::startup(),
            ready: 0,
            death_processing: combat::death_scripts::DeathProcessing {
                queue: 0,
                callback: 0,
            },
            initiative_work: combat::retained::InitiativeWork::startup(),
            party_mask: [0; 6],
            encounter_work: crate::story::EncounterWork::startup(),
            selected_enemy: 0,
            script_enemy: 0,
            // The cold item work record is zero, so its field32 reads zero.
            loaded_item_field32: 0,
            // ITEM 45363c, AMMO 4536d2 and WEAPON 453768 are zero in the startup image.
            ammunition_work: ammunition::WorkBuffers {
                item: Some(vec![0; 150]),
                ammo: Some(vec![0; 150]),
                weapon: Some(vec![0; 150]),
            },
            inventory_record: inventory_record_startup(),
            inventory_scroll: 0,
            npc_backing: combat::retained::NpcBacking::startup(),
            retained_death_queue: None,
            loot_quality: Some(vec![0; 1000]),
            attack_count: 0,
            attack_packet: combat::attack_packet_startup(),
            attack_identity: [0; 3],
            // CODE 1 startup initializes retained work words at 453776/4537f0/45375a to zero.
            weapon_work: combat::weapon_work::WeaponWork::zero(),
            psi_memory: psi_memory_startup(),
            health_context: story_health::HealthContext::startup(),
            random_battles: 1,
            camp_disabled: false,
            coordinates_hidden: 0,
            header_row: 0,
            fire_item: 0,
            target_work: Default::default(),
            warrior_table: Default::default(),
            psi_powers: [None; 6],
        }
    }
}

impl Session {
    /// Remembered Psi power WORD 0x437a7a + 2 * member, which castspell 0x10bc0a takes
    /// back as the menu power: the held WORD, else the recorded choice's power.
    pub(crate) fn psi_power(&self, member: usize) -> i16 {
        self.native_work.psi_powers[member].unwrap_or_else(|| {
            self.psi_menu_choices[member]
                .as_ref()
                .map_or(0, |choice| choice.power)
        })
    }

    /// Holds the power WORDs a menu-choice change left unchanged.
    pub(crate) fn keep_psi_powers(&mut self, words: [i16; 6]) {
        for (member, word) in words.into_iter().enumerate() {
            self.native_work.psi_powers[member] = None;
            if self.psi_power(member) != word {
                self.native_work.psi_powers[member] = Some(word);
            }
        }
    }

    /// cleartarget13822a. Live combat owns words 6..109 until teardown copies them back.
    pub(crate) fn clear_spell_targets(&mut self) {
        self.native_work.party_mask.fill(0);
        *self.target_tail_mut() = vec![0; 104];
    }

    pub(crate) fn publish_spell_targets(&mut self, mask: &[i16; 110]) {
        self.native_work.party_mask.copy_from_slice(&mask[..6]);
        *self.target_tail_mut() = mask[6..].to_vec();
    }

    /// cleartarget13822a: every triple WORD -1 and the 110-word mask clear.
    pub(crate) fn clear_targets(&mut self) {
        self.clear_spell_targets();
        self.target_work_mut().clear();
    }

    pub(crate) fn target_work_mut(&mut self) -> &mut combat::target_work::TargetWork {
        match self.combat.as_mut() {
            Some(combat) => &mut combat.target_work,
            None => &mut self.native_work.target_work,
        }
    }

    fn target_tail_mut(&mut self) -> &mut Vec<i16> {
        match self.combat.as_mut() {
            Some(combat) => &mut combat.target_tail,
            None => &mut self.native_work.spell_target_tail,
        }
    }

    pub(crate) fn spell_targets(&self) -> Result<[i16; 110], crate::RuleError> {
        let tail = match self.combat.as_ref() {
            Some(combat) => &combat.target_tail,
            None => &self.native_work.spell_target_tail,
        };
        if tail.len() != 104 {
            return Err("invalid retained spell target mask".into());
        }
        let mut mask = [0; 110];
        mask[..6].copy_from_slice(&self.native_work.party_mask);
        mask[6..].copy_from_slice(tail);
        Ok(mask)
    }

    pub(crate) fn set_ready(&mut self, ready: i16) {
        self.native_work.ready = ready;
        if let Some(combat) = self.combat.as_mut() {
            combat.ready = ready;
        }
    }

    /// BYTE44273f is separate from the selected-warrior WORD4435d2.
    pub(crate) fn set_warrior_selector(&mut self, warrior: usize) {
        let health = &mut self.native_work.health_context;
        health.selected_warrior = warrior;
    }

    pub(crate) fn retain_combat_work(&mut self) {
        self.retain_npc_work();
        self.retain_monster_work();
        if let Some(combat) = self.combat.as_ref() {
            self.native_work.victory_work = combat.victory_work;
            self.native_work.neighbor_work = combat.neighbor_work.clone();
            self.native_work.ready = combat.ready;
            self.native_work.selected_enemy = combat.selected_enemy;
            self.native_work.script_enemy = combat.script_enemy;
            self.native_work.attack_packet = combat.attack_packet.clone();
            self.native_work.attack_identity = combat.attack_identity;
            self.native_work.weapon_work = combat.weapon_work;
            let health = &mut self.native_work.health_context;
            health.unavailable = combat.unavailable;
            let packet = &combat.attack_packet;
            health.effect = centurions_content::records::AttackPacket(packet).health_effect();
        }
    }
}
