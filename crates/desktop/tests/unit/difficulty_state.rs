use super::*;
use crate::presentation::{Action, UiState};
use centurions_content::word;
use centurions_core::{Command, apply};

use crate::original_content::fixture as native_fixture;

use crate::fixture_hex::decode;

#[path = "../native/inventory_reference.rs"]
mod inventory_reference;

fn story_frame(record: Vec<u8>, cursor: u8, map_slot: usize) -> centurions_core::ScriptState {
    centurions_core::ScriptState {
        map_slot,
        ..centurions_core::ScriptState::scheduled(record, cursor)
    }
}

struct DesktopReplay {
    app: Application,
    state: Session,
    content: Content,
}

impl DesktopReplay {
    fn new() -> Self {
        let content = crate::original_content::load();
        Self {
            app: Application::at(Screen::Title, 0.),
            state: Session::new(&content),
            content,
        }
    }

    fn editor(&self) -> &crate::editor::WarriorEditor {
        self.app.characters.editor.as_ref().unwrap()
    }

    fn creation(&self) -> &crate::creation::Creation {
        self.app.characters.creation.as_ref().unwrap()
    }

    fn information(&self) -> &crate::information::WarriorInformation {
        self.app.characters.information.as_ref().unwrap()
    }

    fn appearance(&self) -> &crate::appearance::Appearance {
        self.app.characters.appearance.as_ref().unwrap()
    }

    #[track_caller]
    fn act(&mut self, action: ApplicationAction) -> (Option<Action>, Vec<Event>) {
        self.handle(action).unwrap()
    }

    #[track_caller]
    fn command(&mut self, command: Command) -> Vec<Event> {
        apply(command, &mut self.state, &self.content).unwrap()
    }

    fn handle(
        &mut self,
        action: ApplicationAction,
    ) -> Result<(Option<Action>, Vec<Event>), centurions_core::RuleError> {
        self.app.handle(action, &mut self.state, &self.content)
    }
}

// Protects viewing and navigation from accidentally advancing gameplay or consuming RNG.
#[test]
fn scenario_selection_and_archive_children_use_the_application_handlers() {
    let mut driver = DesktopReplay::new();
    driver.app.help.enabled = false;
    driver.act(ApplicationAction::Menu(130, 5));
    assert_eq!(
        driver.app.selected_scenario.as_deref(),
        Some("The Thunder of Guns")
    );
    crate::original_content::enter_mission(&mut driver.state, &driver.content);
    driver.app.screen = Screen::Game;
    let mut before = driver.state.clone();
    let (_, about) = driver.act(ApplicationAction::Menu(128, 1));
    assert_eq!(
        about,
        [
            Event::Sound { resource_id: 10107 },
            Event::Sound { resource_id: 30000 },
            Event::Delay { ticks: 20 },
            Event::Sound { resource_id: 10141 },
        ]
    );
    assert!(driver.app.about && driver.app.blocks_game());
    driver.act(ApplicationAction::CloseAbout);
    assert!(!driver.app.about);
    assert_eq!(driver.state, before);
    let archives = &driver.app.archives;
    assert!(archives.iter().any(|(_, name)| name == "Bore Worm"));
    driver.act(ApplicationAction::Menu(132, 1));
    assert!(driver.app.creature.is_some());
    driver.act(ApplicationAction::CreatureItem(3));
    driver.act(ApplicationAction::CreatureItem(2));
    driver.act(ApplicationAction::CreatureItem(4));
    assert!(driver.app.creature.is_none());
    // beast103ebe copies the viewed record into MONSTER work.
    before.globals.monster_pick = driver.state.globals.monster_pick.clone();
    assert_eq!(driver.state, before);
    driver.state.exploration.as_mut().unwrap().presentation =
        Some(Presentation::Prologue { resource_id: 1000 });
    let mut record = vec![0; 42];
    record[8..10].copy_from_slice(&26_i16.to_be_bytes());
    record[10..12].copy_from_slice(&29_i16.to_be_bytes());
    driver.state.exploration.as_mut().unwrap().script = Some(story_frame(record, 0, 0));
    assert_eq!(
        driver.command(Command::Continue),
        [Event::Feedback(centurions_core::feedback::script_pause())]
    );
    let path = std::env::temp_dir().join(format!("centurions-pause-{}.json", std::process::id()));
    centurions_core::save(&path, &driver.state, &driver.content).unwrap();
    driver.state = centurions_core::load(&path, &driver.content).unwrap();
    std::fs::remove_file(path).unwrap();
    let mut ui = crate::presentation::UiState::default();
    assert!(ui.prepare_script_pause(&driver.state));
    assert_eq!(
        ui.feedback_sound(1.),
        Some(Event::Sound { resource_id: 30005 })
    );
    assert!(ui.feedback_input(1.1, true));
    assert!(!ui.feedback.queue.is_empty());
    assert!(ui.feedback_input(1.3, true));
    assert!(ui.feedback.queue.is_empty());
    let events = driver.command(Command::Continue);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Feedback(centurions_core::feedback::GameFeedback::Flash {
            address: 0x401c28,
            ..
        })
    )));
    let before = driver.state.clone();
    driver.act(ApplicationAction::Menu(200, 4));
    assert!(driver.app.notes.open.is_some());
    driver.act(ApplicationAction::CloseNote);
    assert_eq!(driver.state, before);
}

// Protects the creation-to-mission lifecycle and atomic roster persistence, including failed exit.
#[test]
fn creation_roster_squad_and_mission_exit_use_the_application_handlers() {
    let mut driver = DesktopReplay::new();
    let directory = std::env::temp_dir().join(format!("centurions-roster-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("warriors.json");
    let _ = std::fs::remove_file(&path);
    driver.app.help.enabled = false;
    driver.app.load_roster(&path, &driver.content).unwrap();
    let supplied = driver.app.roster.as_ref().unwrap().warriors.len();
    driver.act(ApplicationAction::Menu(136, 5));
    driver.act(ApplicationAction::CreationCancel);
    assert_eq!(driver.app.roster.as_ref().unwrap().warriors.len(), supplied);
    driver.act(ApplicationAction::Menu(136, 5));
    driver
        .app
        .characters
        .creation
        .as_mut()
        .unwrap()
        .request
        .name = "New Warrior".into();
    for item in [4, 3, 29] {
        driver.act(ApplicationAction::CreationItem(item));
    }
    driver.act(ApplicationAction::CreationChoice(1));
    driver.act(ApplicationAction::CreationItem(56));
    driver.act(ApplicationAction::CreationChoice(1));
    for item in [56, 2, 6, 2, 2, 66] {
        driver.act(ApplicationAction::CreationItem(item));
    }
    assert!(driver.app.characters.editor.is_none());
    assert_eq!(
        word(&driver.creation().draft.warriors[0].bytes, 0xe4).unwrap(),
        1
    );
    let created = &driver.creation().draft.warriors[0];
    assert!(created.inventory().unwrap().len() >= 2);
    assert!(driver.state.warriors.is_empty());
    driver.act(ApplicationAction::CreationAnswer(true));
    let roster = centurions_core::roster::Roster::open(&path, &driver.content).unwrap();
    assert_eq!(roster.warriors.len(), supplied + 1);
    assert_eq!(roster.warriors.last().unwrap().name, "New Warrior");
    driver.act(ApplicationAction::Menu(136, 5));
    driver
        .app
        .characters
        .creation
        .as_mut()
        .unwrap()
        .request
        .name = "New Psion".into();
    for item in [4, 3, 29] {
        driver.act(ApplicationAction::CreationItem(item));
    }
    driver.act(ApplicationAction::CreationChoice(8));
    driver.act(ApplicationAction::CreationItem(56));
    driver.act(ApplicationAction::CreationChoice(1));
    for item in [56, 2, 6, 2, 2, 66] {
        driver.act(ApplicationAction::CreationItem(item));
    }
    assert!(driver.editor().scans.is_some());
    driver.act(ApplicationAction::EditorItem(10));
    driver.act(ApplicationAction::EditorItem(32));
    driver.act(ApplicationAction::ScanAnswer(true));
    let created = &driver.creation().draft.warriors[0];
    assert!(created.knows_spell(1101).unwrap());
    assert!(driver.app.characters.editor.is_none());
    driver.act(ApplicationAction::CreationAnswer(false));
    assert_eq!(
        driver.app.roster.as_ref().unwrap().warriors.len(),
        supplied + 1
    );
    driver.act(ApplicationAction::Menu(130, 1));
    driver.act(ApplicationAction::Warrior(supplied));
    driver.act(ApplicationAction::Add);
    driver.act(ApplicationAction::Warrior(0));
    driver.act(ApplicationAction::Add);
    driver.act(ApplicationAction::Done);
    let original = driver.state.clone();
    driver.act(ApplicationAction::Menu(136, 7));
    driver.act(ApplicationAction::Member(1));
    driver.act(ApplicationAction::Remove);
    driver.act(ApplicationAction::SquadCancel);
    assert_eq!(driver.state, original);
    driver.act(ApplicationAction::Menu(136, 1));
    for member in [0, 1] {
        driver.act(ApplicationAction::OrderMember(member));
    }
    assert_eq!(
        driver.state.warriors,
        [original.warriors[1].clone(), original.warriors[0].clone()]
    );
    assert_eq!(driver.state.random, original.random);
    driver.act(ApplicationAction::Menu(136, 7));
    driver.act(ApplicationAction::Member(0));
    driver.act(ApplicationAction::Remove);
    driver.act(ApplicationAction::Done);
    assert_eq!(driver.state.warriors.len(), 1);
    assert_eq!(driver.state.exploration, original.exploration);
    driver.act(ApplicationAction::Menu(130, 3));
    let (next, _) = driver.act(ApplicationAction::EndAnswer(false));
    assert!(matches!(next, Some(Action::Save)));
    let saved = directory.join("session.json");
    centurions_core::save(&saved, &driver.state, &driver.content).unwrap();
    let before_exit = driver.state.clone();
    let expected =
        centurions_core::roster::mission_records(&driver.state, &driver.content).unwrap();
    driver.act(ApplicationAction::EndSaved);
    assert_eq!(driver.app.screen, Screen::Title);
    assert_eq!(driver.state.phase, Phase::SquadSelection);
    assert_eq!(driver.state.random, before_exit.random);
    assert_eq!(
        centurions_core::load(&saved, &driver.content).unwrap(),
        before_exit
    );
    let roster = centurions_core::roster::Roster::open(&path, &driver.content).unwrap();
    assert!(roster.warriors.contains(&expected[0]));
    driver.state = before_exit;
    driver.app.screen = Screen::Game;
    let blocked = directory.join("blocked");
    std::fs::write(&blocked, b"file").unwrap();
    driver
        .app
        .load_roster(&blocked.join("warriors.json"), &driver.content)
        .unwrap();
    let before = driver.state.clone();
    let roster_before = driver.app.roster.as_ref().unwrap().warriors.clone();
    assert!(driver.handle(ApplicationAction::EndSaved).is_err());
    assert_eq!(driver.state, before);
    assert_eq!(driver.app.roster.as_ref().unwrap().warriors, roster_before);
    std::fs::remove_dir_all(&directory).unwrap();
}

// Protects preference commit/cancel/I/O failure and their command and refusal consumers.
#[test]
fn preference_dialog_commits_only_persisted_choices_and_drives_gameplay_policy() {
    let mut driver = DesktopReplay::new();
    driver.app.help.enabled = false;
    let path = std::env::temp_dir().join(format!(
        "centurions-preferences-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    driver.app.preferences(path.clone()).unwrap();
    // HandleMenuChoice 0x127f9a..0x128076: independent FX/music rows, preview and persistence.
    let untouched = driver.state.clone();
    for (item, effects, music) in [
        (2, 0, 5),
        (26, 0, 10),
        (12, 10, 10),
        (16, 10, 0),
        (21, 10, 5),
    ] {
        let (_, events) = driver.act(ApplicationAction::Menu(135, item));
        let audio = &driver.app.audio;
        assert_eq!(
            (audio.volume, audio.music_volume, audio.sound),
            (effects, music, effects != 0)
        );
        assert_eq!(
            events,
            if item <= 12 {
                vec![Event::Sound { resource_id: 642 }]
            } else {
                vec![]
            }
        );
    }
    assert_eq!(driver.state, untouched);
    let before = driver.app.preferences.clone();
    driver.act(ApplicationAction::Menu(137, 3));
    driver.act(ApplicationAction::PreferenceItem(15));
    driver.act(ApplicationAction::PreferenceCancel);
    assert_eq!(driver.app.preferences, before);
    driver.act(ApplicationAction::Menu(137, 3));
    for item in [12, 13, 14, 15, 16, 17, 21, 2] {
        driver.act(ApplicationAction::PreferenceItem(item));
    }
    let mut restored = Application::at(Screen::Title, 0.);
    restored.preferences(path.clone()).unwrap();
    let audio = &restored.audio;
    assert_eq!((audio.volume, audio.music_volume), (10, 5));
    assert_eq!(restored.preferences, driver.app.preferences);
    assert!(matches!(
        driver.app.command(Command::Continue),
        Command::WithPreferences {
            preferences: centurions_core::CommandPreferences {
                auto_refit: false,
                ..
            },
            ..
        }
    ));
    crate::original_content::enter_mission(&mut driver.state, &driver.content);
    let before = driver.state.clone();
    assert!(
        apply(
            driver.app.command(Command::DropItem {
                warrior: 0,
                slot: 100
            }),
            &mut driver.state,
            &driver.content
        )
        .is_err()
    );
    assert_eq!(driver.state, before);
    let fixture: serde_json::Value = crate::original_content::fixture("earned-combat-reload.json");
    driver.state =
        crate::original_content::source_frame(fixture["foundations"][0]["session"].clone());
    assert_eq!(driver.state.phase, Phase::Combat);
    let mut friendly = driver.state.clone();
    let actor = friendly.combat.as_ref().unwrap().active_body().unwrap() as usize;
    friendly.warriors[actor].bytes[0x40..0x42].copy_from_slice(&5_i16.to_be_bytes());
    friendly.warriors[actor].bytes[0xca..0xcc].fill(0);
    let target = (actor + 1) % friendly.warriors.len();
    let combat = friendly.combat.as_mut().unwrap();
    let (x, y) = combat.squad.pos[actor];
    combat.squad.pos[target] = (x + 1, y);
    let faction = friendly.warriors[actor].bytes[0x10..0x12].to_vec();
    friendly.warriors[target].bytes[0x10..0x12].copy_from_slice(&faction);
    driver.app.observe(&[Event::FriendAttackOffered {
        body: target as i16,
    }]);
    driver
        .app
        .handle(
            ApplicationAction::ExchangeChoice(None),
            &mut friendly,
            &driver.content,
        )
        .unwrap();
    assert!(driver.app.questions.exchange_offer.is_none());
    let mut expected = friendly.clone();
    let expected_events = apply(
        Command::AttackFriend {
            body: target as i16,
        },
        &mut expected,
        &driver.content,
    )
    .unwrap();
    driver.app.observe(&[Event::FriendAttackOffered {
        body: target as i16,
    }]);
    let (_, events) = driver
        .app
        .handle(
            ApplicationAction::ExchangeChoice(Some(false)),
            &mut friendly,
            &driver.content,
        )
        .unwrap();
    assert_eq!(friendly, expected);
    assert_eq!(events, expected_events);
    assert!(driver.app.questions.exchange_offer.is_none());
    driver.state.warriors[0].bytes[0x9de] = 1;
    let notices = driver.app.combat_notices(Some(-1), &driver.state);
    assert!(notices.iter().all(|e| matches!(
        e,
        Event::Feedback(centurions_core::feedback::GameFeedback::Sound { resource_id: -139 })
    )));
    restored.preferences.controls[4] = false;
    restored.preferences.controls[5] = false;
    let round = driver.state.combat.as_ref().map(|c| c.combat_round);
    assert!(restored.combat_notices(round, &driver.state).is_empty());
    let mut ui = UiState::default();
    let mut events: Vec<_> = (0..3).map(|_| Event::Sound { resource_id: -139 }).collect();
    events.push(Event::Sound { resource_id: 138 });
    restored.prepare_combat_round(Some(-1), &driver.state, &mut events);
    let sound = |event: &Event| matches!(event, Event::Sound { .. });
    assert!(!events.iter().any(sound), "round audio would play twice");
    for event in events {
        if let Event::Feedback(feedback) = event {
            ui.feedback.queue.push_back(feedback);
        }
    }
    for frame in 0..60 {
        if ui.feedback_sound(f64::from(frame)).is_some() {
            ui.feedback.until = f64::from(frame) + 0.5;
        }
        ui.feedback_input(f64::from(frame), true);
    }
    assert!(
        ui.feedback.queue.is_empty() && ui.feedback.persistent.is_none(),
        "round notice did not close"
    );
    let actor = driver.state.combat.as_ref().unwrap().active_body().unwrap() as usize;
    for (offset, value) in [(0x90, 1_i16), (0x10, 0), (0x5e, 0)] {
        driver.state.warriors[actor].bytes[offset..offset + 2]
            .copy_from_slice(&value.to_be_bytes());
    }
    let random = driver.state.random.clone();
    let index = |state: &Session| state.combat.as_ref().unwrap().battle_order_index;
    let turn = index(&driver.state);
    let events = driver.command(Command::AutomaticBandage);
    assert_eq!(driver.state.warriors[0].bytes[0x9de], 0);
    assert_eq!(word(&driver.state.warriors[actor].bytes, 0x40).unwrap(), 0);
    assert_eq!(driver.state.random, random);
    assert_eq!(index(&driver.state), turn);
    assert!(matches!(
        events.first(),
        Some(Event::Feedback(
            centurions_core::feedback::GameFeedback::Flash {
                address: 0x403cd6,
                ..
            }
        ))
    ));
    driver.app.screen = Screen::Game;
    let before = driver.state.clone();
    driver.act(ApplicationAction::Menu(130, 3));
    driver.act(ApplicationAction::EndAnswer(false));
    assert_eq!(driver.state, before);
    driver.act(ApplicationAction::Menu(130, 3));
    driver.act(ApplicationAction::EndAnswer(true));
    assert_eq!(driver.state.phase, Phase::SquadSelection);
    std::fs::remove_file(path).unwrap();
}

// Protects committing the selected level while an existing mission remains untouched during setup.
#[test]
fn difficulty_controls_commit_all_original_levels_at_mission_start() {
    let mut driver = DesktopReplay::new();
    for level in -2..=2 {
        let before = driver.state.clone();
        driver.app = Application::at(Screen::Squad, 0.);
        driver.app.squad = vec![0];
        driver.act(ApplicationAction::Difficulty(level));
        assert_eq!(driver.state, before);
        driver.act(ApplicationAction::Done);
        assert_eq!(driver.state.difficulty, level);
        assert_eq!(driver.state.phase, Phase::Exploration);
    }
}

// Protects original editor activation, cancellation, undo, commit and saved continuity.
#[test]
fn editor_cancel_undo_member_navigation_and_saved_commit_preserve_native_scope() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut driver = DesktopReplay::new();
    driver.act(ApplicationAction::Menu(136, 8));
    driver.act(ApplicationAction::EditorAnswer(false));
    assert!(!driver.app.editing);
    driver.act(ApplicationAction::Menu(136, 8));
    let (_, events) = driver.act(ApplicationAction::EditorAnswer(true));
    use centurions_core::feedback::GameFeedback;
    let expected = [0x40361a, 0x4035fa, 0x4035be].map(|address| {
        Event::Feedback(GameFeedback::Flash {
            address,
            x: 50,
            y: 50,
            wait: 0,
            sound: 5000,
        })
    });
    assert_eq!(events, expected);
    inventory_reference::compare_editor_data(&driver.content, &root);
    let mut ui = crate::presentation::UiState::default();
    ui.feedback.queue.extend(expected.into_iter().map(|event| {
        let Event::Feedback(feedback) = event else {
            unreachable!()
        };
        feedback
    }));
    for time in [0., 1., 2.] {
        assert_eq!(
            ui.feedback_sound(time),
            Some(Event::Sound { resource_id: 5000 })
        );
        assert!(ui.feedback_input(time + 0.1, true));
        assert!(ui.feedback_input(time + 0.3, true));
    }
    assert!(ui.feedback.queue.is_empty());
    let (_, events) = driver.act(ApplicationAction::Menu(136, 8));
    assert_eq!(
        events,
        [Event::Feedback(GameFeedback::Flash {
            address: 0x403686,
            x: 50,
            y: 50,
            wait: 0,
            sound: 5000
        })]
    );
    driver.act(ApplicationAction::Menu(136, 8));
    driver.act(ApplicationAction::EditorAnswer(true));
    crate::original_content::enter_mission(&mut driver.state, &driver.content);
    driver.app.screen = Screen::Game;
    let before = driver.state.clone();
    for item in [45, 82, 45, 83, 45, 1] {
        if driver.app.characters.editor.is_none() {
            driver.act(ApplicationAction::OpenEditor(0));
        }
        driver.act(ApplicationAction::EditorItem(item));
        let mut scope = driver.state.clone();
        scope.globals = before.globals.clone();
        assert_eq!(scope, before);
        if matches!(item, 82 | 83) {
            assert_eq!(driver.editor().draft.warriors, before.warriors);
        }
    }
    driver.act(ApplicationAction::OpenEditor(0));
    driver.act(ApplicationAction::EditorItem(45));
    driver.act(ApplicationAction::EditorItem(4));
    let lead = |state: &Session, offset| word(&state.warriors[0].bytes, offset).unwrap();
    assert_eq!(lead(&driver.state, 0x9e2), lead(&before, 0x9e2) + 1);
    assert_eq!(driver.state.random, before.random);
    let first_committed = driver.state.clone();
    driver.act(ApplicationAction::OpenEditor(0));
    for item in [45, 85, 45, 1] {
        driver.act(ApplicationAction::EditorItem(item));
    }
    assert_eq!(driver.state.warriors[1], first_committed.warriors[1]);
    assert_eq!(
        lead(&driver.state, 0x9e2),
        lead(&first_committed, 0x9e2) + 1
    );
    let path = std::env::temp_dir().join(format!("centurions-editor-{}.json", std::process::id()));
    let before_levels = driver.state.clone();
    driver.act(ApplicationAction::OpenEditor(0));
    for item in [81, 81, 81, 0, 83, 4] {
        driver.act(ApplicationAction::EditorItem(item));
    }
    assert_eq!(lead(&driver.state, 0x3a), lead(&before_levels, 0x3a) + 3);
    let draws = |state: &Session| state.library_random_draws;
    assert_eq!(draws(&driver.state), draws(&before_levels) + 3);
    let costs = |state: &Session| state.warriors[0].bytes[0x9ce..0x9d2].to_vec();
    assert_eq!(costs(&driver.state), costs(&before_levels));
    let before_scans = driver.state.clone();
    driver.act(ApplicationAction::OpenEditor(1));
    for item in [39, 2, 10, 11, 32] {
        driver.act(ApplicationAction::EditorItem(item));
    }
    assert_eq!(driver.state, before_scans);
    assert!(driver.editor().scans.as_ref().unwrap().question);
    driver.act(ApplicationAction::ScanAnswer(false));
    assert!(driver.editor().scans.is_some());
    driver.act(ApplicationAction::EditorItem(32));
    driver.act(ApplicationAction::ScanAnswer(true));
    assert_eq!(driver.state, before_scans);
    driver.act(ApplicationAction::EditorItem(4));
    assert_eq!(&driver.state.warriors[1].bytes[0x9f0..0x9f2], &[0, 1]);
    let memory = |state: &Session| state.globals.define_spells.clone();
    assert_eq!(
        memory(&driver.state)[80..160],
        driver.state.warriors[1].bytes[0xa76..0xac6]
    );
    assert_eq!(memory(&driver.state)[..80], memory(&before_scans)[..80]);
    assert_eq!(driver.state.random, before_scans.random);
    assert_eq!(draws(&driver.state), draws(&before_scans));
    driver.act(ApplicationAction::Menu(136, 8));
    driver.act(ApplicationAction::OpenEditor(0));
    assert!(!driver.editor().free);
    let declined = driver.editor().draft.clone();
    assert!(matches!(
        driver.handle(ApplicationAction::EditorItem(45)),
        Err(centurions_core::RuleError::Refused { .. })
    ));
    assert_eq!(driver.editor().draft, declined);
    driver.state.warriors[0].bytes[0x9d2..0x9d6].copy_from_slice(&5000_i32.to_be_bytes());
    driver.app.characters.editor.as_mut().unwrap().draft = driver.state.clone();
    let before_payment = driver.state.clone();
    let actor = &driver.state.warriors[0];
    let mos = word(&actor.bytes, 0x34).unwrap() as usize;
    let balance = centurions_core::character::upgrade_payment(
        actor,
        0,
        &driver.content.map(0).unwrap().profiles[(mos - 1) * 576..mos * 576],
    )
    .unwrap()
    .unwrap();
    driver.act(ApplicationAction::EditorItem(45));
    driver.act(ApplicationAction::EditorItem(4));
    assert_eq!(
        i32::from_be_bytes(
            driver.state.warriors[0].bytes[0x9d2..0x9d6]
                .try_into()
                .unwrap()
        ),
        balance
    );
    assert_eq!(driver.state.random, before_payment.random);
    let before_appearance = driver.state.clone();
    driver.state.exploration.as_mut().unwrap().presentation =
        Some(Presentation::Prologue { resource_id: 1 });
    let modal = driver.state.clone();
    assert!(matches!(
        driver.handle(ApplicationAction::Menu(136, 2)),
        Err(centurions_core::RuleError::Refused { .. })
    ));
    assert!(matches!(
        apply(
            Command::SetWarriorAppearance {
                warrior: 0,
                portrait: true,
                resource: 258,
            },
            &mut driver.state,
            &driver.content
        ),
        Err(centurions_core::RuleError::Refused { .. })
    ));
    assert_eq!(driver.state, modal);
    driver.state = before_appearance.clone();
    driver.act(ApplicationAction::Menu(136, 2));
    driver.act(ApplicationAction::AppearanceItem(10));
    assert_eq!(driver.state, before_appearance);
    let draft = driver.appearance().draft.clone();
    assert!(
        centurions_core::apply(
            Command::SetWarriorAppearance {
                warrior: 0,
                portrait: true,
                resource: 1,
            },
            &mut driver.app.characters.appearance.as_mut().unwrap().draft,
            &driver.content
        )
        .is_err()
    );
    assert_eq!(driver.appearance().draft, draft);
    driver.act(ApplicationAction::AppearanceItem(34));
    driver.act(ApplicationAction::AppearanceItem(11));
    driver.act(ApplicationAction::AppearanceItem(2));
    assert_eq!(word(&driver.state.warriors[0].bytes, 0x54).unwrap(), 258);
    assert_eq!(word(&driver.state.warriors[1].bytes, 0x54).unwrap(), 259);
    assert_eq!(driver.state.random, before_appearance.random);
    driver.act(ApplicationAction::Menu(136, 3));
    driver.act(ApplicationAction::AppearanceItem(10));
    driver.act(ApplicationAction::AppearanceItem(2));
    assert_eq!(word(&driver.state.warriors[1].bytes, 0x56).unwrap(), 9258);
    let unchanged = driver.state.clone();
    driver.act(ApplicationAction::Menu(145, 10));
    assert_eq!(driver.app.audio.music_request, Some(20128));
    driver.act(ApplicationAction::Playlist(0));
    assert!(!driver.app.audio.playlist[0]);
    assert_eq!(driver.state, unchanged);
    let seed = driver.state.random.clone();
    driver.act(ApplicationAction::OpenInformation(0));
    assert_eq!(driver.state.random, seed);
    inventory_reference::information_entry(&driver.state, &unchanged, 0);
    inventory_reference::compare_character_fields(&driver.content, &driver.state, &root);
    let viewed = driver.state.clone();
    driver.act(ApplicationAction::InformationItem(100));
    assert!(driver.information().detailed);
    driver.act(ApplicationAction::InformationItem(73));
    assert_eq!(driver.information().member, 1);
    driver.act(ApplicationAction::InformationItem(58));
    assert!(!driver.information().detailed);
    assert_eq!(driver.state, viewed);
    driver.app.restore_information(0);
    for item in 2..=5 {
        let (child, _) = driver.act(ApplicationAction::InformationItem(item));
        assert!(matches!(child, Some(Action::InformationInspection(0, _))));
        driver.app.restore_information(0);
        assert_eq!(driver.state, viewed);
    }
    for item in [96, 97] {
        driver.act(ApplicationAction::InformationItem(item));
        assert!(driver.creation().read_only);
        driver.act(ApplicationAction::CreationChoice(1));
        driver.act(ApplicationAction::CreationItem(56));
        assert!(driver.app.characters.information.is_some());
        assert_eq!(driver.state, viewed);
    }
    driver.act(ApplicationAction::InformationItem(65));
    assert!(!driver.editor().free);
    driver.act(ApplicationAction::EditorItem(1));
    driver.act(ApplicationAction::InformationItem(72));
    assert!(!driver.app.blocks_game());
    centurions_core::save(&path, &driver.state, &driver.content).unwrap();
    assert_eq!(
        centurions_core::load(&path, &driver.content).unwrap(),
        driver.state
    );
    std::fs::remove_file(path).unwrap();
}

// Protects item-dialog commit scope, equipped modifiers and saved warrior provenance.
#[test]
fn item_editor_commits_equipped_changes_and_preserves_saved_provenance() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = crate::original_content::load();
    let mut state = Session::new(&content);
    crate::original_content::enter_mission(&mut state, &content);
    let mut app = Application::at(Screen::Game, 0.);
    app.editing = true;
    let before = state.clone();
    app.handle(
        ApplicationAction::OpenItemEditor(0, 0),
        &mut state,
        &content,
    )
    .unwrap();
    app.handle(ApplicationAction::ItemEditorItem(54), &mut state, &content)
        .unwrap();
    assert_eq!(state, before);
    let draft = app.characters.item_editor.as_ref().unwrap().draft.clone();
    assert!(
        app.handle(ApplicationAction::ItemEditorItem(39), &mut state, &content)
            .is_err()
    );
    assert_eq!(app.characters.item_editor.as_ref().unwrap().draft, draft);
    app.handle(ApplicationAction::ItemEditorItem(2), &mut state, &content)
        .unwrap();
    assert_eq!(state, draft);
    assert_eq!(
        state.warriors[0].original_sha256,
        before.warriors[0].original_sha256
    );
    let path = std::env::temp_dir().join(format!(
        "centurions-item-editor-{}.json",
        std::process::id()
    ));
    centurions_core::save(&path, &state, &content).unwrap();
    assert_eq!(centurions_core::load(&path, &content).unwrap(), state);
    // Paired inventory must route a loaded-item transfer without losing charges or unrelated actions.
    state = before;
    state.exploration.as_mut().unwrap().presentation = None;
    let mut ui = UiState::default();
    ui.inventory.open(0, state.warriors.len());
    let record = state.warriors[0].inventory().unwrap()[0].clone();
    let recipient_slot = state.warriors[4].inventory().unwrap().len();
    let source_count = state.warriors[0].inventory().unwrap().len();
    let random = state.random.clone();
    for action in [
        crate::inventory::InventoryAction::Warrior { side: 1, member: 4 },
        crate::inventory::InventoryAction::Item { side: 0, slot: 0 },
    ] {
        let Some(Action::Command(command)) = ui
            .resolve_inventory(Some(Action::Inventory(action)), &state, &content, 1.)
            .unwrap()
        else {
            panic!("inventory inspection did not reach command boundary")
        };
        apply(command, &mut state, &content).unwrap();
    }
    assert_eq!(state.globals.inventory_record, record);
    let drag = ui.inventory.release(Some(vec2(500., 50.))).unwrap();
    let Some(Action::Command(command)) = ui
        .resolve_inventory(Some(Action::Inventory(drag)), &state, &content, 2.)
        .unwrap()
    else {
        panic!("transfer did not reach command boundary")
    };
    let events = apply(command, &mut state, &content).unwrap();
    let moved = |event: &Event| matches!(event, Event::ItemTransferred { from: 0, to: 4, .. });
    assert!(events.iter().any(moved));
    let transferred = state.warriors[4].inventory().unwrap()[recipient_slot].clone();
    assert_eq!(
        state.warriors[0].inventory().unwrap().len(),
        source_count - 1
    );
    assert_eq!(&transferred.0[..8], &record.0[..8]);
    assert_eq!(transferred.0[9], 0);
    assert_eq!(state.random, random);
    assert!(matches!(
        ui.resolve_inventory(
            Some(Action::Command(Command::Continue)),
            &state,
            &content,
            3.
        )
        .unwrap(),
        Some(Action::Command(Command::Continue))
    ));
    centurions_core::save(&path, &state, &content).unwrap();
    assert_eq!(centurions_core::load(&path, &content).unwrap(), state);
    // Shop entry, native record decoding, refused purchase, buy/resale, and reload share this item flow.
    let receipt = native_fixture("shop-records.json");
    inventory_reference::compare_previews(&content, &state, &root, &receipt);
    let source = content.map(5).unwrap().header[92 * 42..93 * 42].to_vec();
    inventory_reference::compare_shop_stock(&content, &mut state, &receipt, &source);
    state.exploration.as_mut().unwrap().script = Some(story_frame(source, 1, 92));
    state.exploration.as_mut().unwrap().presentation =
        Some(centurions_core::Presentation::Dialogue {
            id: 919,
            text: String::new(),
        });
    apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(
        state.exploration.as_ref().unwrap().presentation,
        Some(centurions_core::Presentation::Shop)
    );
    assert_eq!(
        inventory_reference::stock(&state),
        decode(receipt["shops"][0]["stock"].as_str().unwrap())
    );
    let (stock, id, _) = centurions_core::shop::stock(&state)
        .unwrap()
        .into_iter()
        .find(|(_, id, _)| *id == 200)
        .unwrap();
    let cost = centurions_core::shop::price(&state, &content, id, false).unwrap();
    state.party_currency = 0;
    let refused = state.clone();
    assert!(matches!(
        apply(Command::BuyItem { warrior: 0, stock }, &mut state, &content),
        Err(centurions_core::RuleError::Refused { .. })
    ));
    assert_eq!(state, refused);
    state.party_currency = cost + 100;
    let ammo = state.ammunition[0];
    for action in [
        crate::inventory::InventoryAction::Category(1),
        crate::inventory::InventoryAction::Item {
            side: 1,
            slot: stock,
        },
        crate::inventory::InventoryAction::Transfer,
    ] {
        if let Some(Action::Command(command)) = ui
            .resolve_inventory(Some(Action::Inventory(action)), &state, &content, 10.)
            .unwrap()
        {
            apply(command, &mut state, &content).unwrap();
        }
    }
    assert!(state.ammunition[0] > ammo);
    assert_eq!(state.party_currency, 100);
    let item = state.warriors[0].inventory().unwrap()[0].clone();
    let sale = centurions_core::shop::price(&state, &content, item.id(), true).unwrap();
    apply(
        Command::SellItem {
            warrior: 0,
            slot: 0,
        },
        &mut state,
        &content,
    )
    .unwrap();
    assert_eq!(state.party_currency, 100 + sale);
    assert!(
        centurions_core::shop::stock(&state)
            .unwrap()
            .iter()
            .any(|(_, id, _)| *id == item.id())
    );
    centurions_core::save(&path, &state, &content).unwrap();
    let mut restored = centurions_core::load(&path, &content).unwrap();
    assert_eq!(restored, state);
    inventory_reference::compare_special_purchases(&content, &state, &path);
    apply(Command::CloseShop, &mut restored, &content).unwrap();
    let position = restored.exploration.as_ref().unwrap();
    assert!(position.presentation.is_none());
    let (actor, needle) = restored
        .warriors
        .iter()
        .enumerate()
        .find_map(|(actor, warrior)| {
            warrior
                .inventory()
                .unwrap()
                .iter()
                .position(|record| record.id() == 700)
                .map(|slot| (actor, slot))
        })
        .unwrap();
    ui.inventory.open(actor, restored.warriors.len());
    for action in [
        crate::inventory::InventoryAction::Item {
            side: 0,
            slot: needle,
        },
        crate::inventory::InventoryAction::Activate,
    ] {
        if let Some(Action::Command(command)) = ui
            .resolve_inventory(Some(Action::Inventory(action)), &restored, &content, 20.)
            .unwrap()
        {
            apply(command, &mut restored, &content).unwrap();
        }
    }
    assert_eq!(restored.outdoor_psi.as_ref().unwrap().item, Some(700));
    centurions_core::save(&path, &restored, &content).unwrap();
    let mut resumed = centurions_core::load(&path, &content).unwrap();
    assert_eq!(
        apply(Command::CancelOutdoorPsi, &mut restored, &content).unwrap(),
        apply(Command::CancelOutdoorPsi, &mut resumed, &content).unwrap()
    );
    assert_eq!(restored, resumed);
    assert!(restored.outdoor_psi.is_none());
    inventory_reference::compare_encounter_returns(&mut app, &mut ui, &restored, &content, &path);
    std::fs::remove_file(path).unwrap();
}
