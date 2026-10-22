use super::*;
use crate::presentation::{Action, UiState};
use centurions_content::word;
use centurions_core::{Command, apply};

use crate::original_content::fixture as native_fixture;

use crate::fixture_hex::decode;

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
    driver.command(Command::ImportStarters);
    driver.command(Command::EnterMission {
        name: "The Thunder of Guns".into(),
    });
    driver.app.screen = Screen::Game;
    let before = driver.state.clone();
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
    assert!(
        driver
            .app
            .archives
            .iter()
            .any(|(_, name)| name == "Bore Worm")
    );
    driver.act(ApplicationAction::Menu(132, 1));
    assert!(driver.app.creature.is_some());
    driver.act(ApplicationAction::CreatureItem(3));
    driver.act(ApplicationAction::CreatureItem(2));
    driver.act(ApplicationAction::CreatureItem(4));
    assert!(driver.app.creature.is_none());
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
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    legacy["schema_version"] = 8.into();
    crate::original_content::legacy_save(&mut legacy);
    std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(centurions_core::load(&path, &driver.content).is_err());
    std::fs::remove_file(path).unwrap();
    let mut ui = crate::presentation::UiState::default();
    assert!(ui.prepare_script_pause(&driver.state));
    assert_eq!(
        ui.feedback_sound(1.),
        Some(Event::Sound { resource_id: 30005 })
    );
    assert!(ui.feedback_input(1.1, true));
    assert!(!ui.feedback.is_empty());
    assert!(ui.feedback_input(1.3, true));
    assert!(ui.feedback.is_empty());
    let events = driver.command(Command::Continue);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Feedback(centurions_core::feedback::NativeFeedback::Flash {
            address: 0x401c28,
            ..
        })
    )));
    let before = driver.state.clone();
    driver.act(ApplicationAction::Menu(200, 4));
    assert!(driver.app.note.is_some());
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
    assert!(
        driver.creation().draft.warriors[0]
            .inventory()
            .unwrap()
            .len()
            >= 2
    );
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
    assert!(
        driver.creation().draft.warriors[0]
            .knows_spell(1101)
            .unwrap()
    );
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
    assert!(
        centurions_core::roster::Roster::open(&path, &driver.content)
            .unwrap()
            .warriors
            .contains(&expected[0])
    );
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
        assert_eq!(
            (driver.app.volume, driver.app.music_volume, driver.app.sound),
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
    let before = driver.app.native_preferences.clone();
    driver.act(ApplicationAction::Menu(137, 3));
    driver.act(ApplicationAction::PreferenceItem(15));
    driver.act(ApplicationAction::PreferenceCancel);
    assert_eq!(driver.app.native_preferences, before);
    driver.act(ApplicationAction::Menu(137, 3));
    for item in [12, 13, 14, 15, 16, 17, 21, 2] {
        driver.act(ApplicationAction::PreferenceItem(item));
    }
    let mut restored = Application::at(Screen::Title, 0.);
    restored.preferences(path.clone()).unwrap();
    assert_eq!((restored.volume, restored.music_volume), (10, 5));
    assert_eq!(restored.native_preferences, driver.app.native_preferences);
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
    driver.command(Command::ImportStarters);
    driver.command(Command::EnterMission {
        name: "The Thunder of Guns".into(),
    });
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
    driver.state = serde_json::from_value(fixture["foundations"][0]["session"].clone()).unwrap();
    assert_eq!(driver.state.phase, Phase::Combat);
    let mut friendly = driver.state.clone();
    let actor = friendly.combat.as_ref().unwrap().active_body().unwrap() as usize;
    friendly.warriors[actor].bytes[0x40..0x42].copy_from_slice(&5_i16.to_be_bytes());
    friendly.warriors[actor].bytes[0xca..0xcc].fill(0);
    let target = (actor + 1) % friendly.warriors.len();
    let combat = friendly.combat.as_mut().unwrap();
    let (x, y) = combat.squad.positions[actor];
    combat.squad.positions[target] = (x + 1, y);
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
    assert!(driver.app.exchange_offer.is_none());
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
    assert!(driver.app.exchange_offer.is_none());
    driver.state.warriors[0].bytes[0x9de] = 1;
    let notices = driver.app.combat_notices(Some(-1), &driver.state);
    assert!(notices.iter().all(|e| matches!(
        e,
        Event::Feedback(centurions_core::feedback::NativeFeedback::Sound { resource_id: -139 })
    )));
    restored.native_preferences.controls[4] = false;
    restored.native_preferences.controls[5] = false;
    assert!(
        restored
            .combat_notices(driver.state.combat.as_ref().map(|c| c.round), &driver.state)
            .is_empty()
    );
    let mut ui = UiState::default();
    let mut events: Vec<_> = (0..3).map(|_| Event::Sound { resource_id: -139 }).collect();
    events.push(Event::Sound { resource_id: 138 });
    restored.prepare_combat_round(Some(-1), &driver.state, &mut events);
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, Event::Sound { .. })),
        "round audio would play twice"
    );
    for event in events {
        if let Event::Feedback(feedback) = event {
            ui.feedback.push_back(feedback);
        }
    }
    for frame in 0..60 {
        if ui.feedback_sound(f64::from(frame)).is_some() {
            ui.feedback_until = f64::from(frame) + 0.5;
        }
        ui.feedback_input(f64::from(frame), true);
    }
    assert!(
        ui.feedback.is_empty() && ui.persistent_feedback.is_none(),
        "round notice did not close"
    );
    let actor = driver.state.combat.as_ref().unwrap().active_body().unwrap() as usize;
    for (offset, value) in [(0x90, 1_i16), (0x10, 0), (0x5e, 0)] {
        driver.state.warriors[actor].bytes[offset..offset + 2]
            .copy_from_slice(&value.to_be_bytes());
    }
    let random = driver.state.random.clone();
    let turn = driver.state.combat.as_ref().unwrap().cursor;
    let events = driver.command(Command::AutomaticBandage);
    assert_eq!(driver.state.warriors[0].bytes[0x9de], 0);
    assert_eq!(word(&driver.state.warriors[actor].bytes, 0x40).unwrap(), 0);
    assert_eq!(driver.state.random, random);
    assert_eq!(driver.state.combat.as_ref().unwrap().cursor, turn);
    assert!(matches!(
        events.first(),
        Some(Event::Feedback(
            centurions_core::feedback::NativeFeedback::Flash {
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
    let content = crate::original_content::load();
    let mut state = Session::new(&content);
    for level in -2..=2 {
        let before = state.clone();
        let mut setup = Application::at(Screen::Squad, 0.);
        setup.squad = vec![0];
        setup
            .handle(ApplicationAction::Difficulty(level), &mut state, &content)
            .unwrap();
        assert_eq!(state, before);
        setup
            .handle(ApplicationAction::Done, &mut state, &content)
            .unwrap();
        assert_eq!(state.difficulty, level);
        assert_eq!(state.phase, Phase::Exploration);
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
    use centurions_core::feedback::NativeFeedback;
    let expected = [0x40361a, 0x4035fa, 0x4035be].map(|address| {
        Event::Feedback(NativeFeedback::Flash {
            address,
            x: 50,
            y: 50,
            wait: 0,
            sound: 5000,
        })
    });
    assert_eq!(events, expected);
    let text = centurions_content::native_text::NativeText::load(
        &driver.content,
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let original = std::fs::read(root.join("analysis/ghidra/startup/a5-mapped.bin")).unwrap();
    for (address, pascal) in [
        (0x403658, true),
        (0x403670, true),
        (0x40361a, false),
        (0x4035fa, false),
        (0x4035be, false),
        (0x403686, false),
    ] {
        let slice = &original[(address - 0x3fe210) as usize..];
        let bytes = if pascal {
            &slice[1..1 + usize::from(slice[0])]
        } else {
            &slice[..slice.iter().position(|b| *b == 0).unwrap()]
        };
        assert_eq!(text.text(address, pascal).unwrap().as_bytes(), bytes);
    }
    let mut ui = crate::presentation::UiState::default();
    ui.feedback.extend(expected.into_iter().map(|event| {
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
    assert!(ui.feedback.is_empty());
    let (_, events) = driver.act(ApplicationAction::Menu(136, 8));
    assert_eq!(
        events,
        [Event::Feedback(NativeFeedback::Flash {
            address: 0x403686,
            x: 50,
            y: 50,
            wait: 0,
            sound: 5000
        })]
    );
    driver.act(ApplicationAction::Menu(136, 8));
    driver.act(ApplicationAction::EditorAnswer(true));
    driver.command(Command::ImportStarters);
    driver.command(Command::EnterMission {
        name: "The Thunder of Guns".into(),
    });
    driver.app.screen = Screen::Game;
    let before = driver.state.clone();
    for item in [45, 82, 45, 83, 45, 1] {
        if driver.app.characters.editor.is_none() {
            driver.act(ApplicationAction::OpenEditor(0));
        }
        driver.act(ApplicationAction::EditorItem(item));
        assert_eq!(driver.state, before);
        if matches!(item, 82 | 83) {
            assert_eq!(driver.editor().draft, before);
        }
    }
    driver.act(ApplicationAction::OpenEditor(0));
    driver.act(ApplicationAction::EditorItem(45));
    driver.act(ApplicationAction::EditorItem(4));
    assert_eq!(
        word(&driver.state.warriors[0].bytes, 0x9e2).unwrap(),
        word(&before.warriors[0].bytes, 0x9e2).unwrap() + 1
    );
    assert_eq!(driver.state.random, before.random);
    let first_committed = driver.state.clone();
    driver.act(ApplicationAction::OpenEditor(0));
    for item in [45, 85, 45, 1] {
        driver.act(ApplicationAction::EditorItem(item));
    }
    assert_eq!(driver.state.warriors[1], first_committed.warriors[1]);
    assert_eq!(
        word(&driver.state.warriors[0].bytes, 0x9e2).unwrap(),
        word(&first_committed.warriors[0].bytes, 0x9e2).unwrap() + 1
    );
    let path = std::env::temp_dir().join(format!("centurions-editor-{}.json", std::process::id()));
    let before_levels = driver.state.clone();
    driver.act(ApplicationAction::OpenEditor(0));
    for item in [81, 81, 81, 0, 83, 4] {
        driver.act(ApplicationAction::EditorItem(item));
    }
    assert_eq!(
        word(&driver.state.warriors[0].bytes, 0x3a).unwrap(),
        word(&before_levels.warriors[0].bytes, 0x3a).unwrap() + 3
    );
    assert_eq!(
        driver.state.library_random_draws,
        before_levels.library_random_draws + 3
    );
    assert_eq!(
        &driver.state.warriors[0].bytes[0x9ce..0x9d2],
        &before_levels.warriors[0].bytes[0x9ce..0x9d2]
    );
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
    assert_eq!(
        &driver.state.native_work.psi_memory.as_ref().unwrap()[80..160],
        &driver.state.warriors[1].bytes[0xa76..0xac6]
    );
    assert_eq!(
        &driver.state.native_work.psi_memory.as_ref().unwrap()[..80],
        &before_scans.native_work.psi_memory.as_ref().unwrap()[..80]
    );
    assert_eq!(driver.state.random, before_scans.random);
    assert_eq!(
        driver.state.library_random_draws,
        before_scans.library_random_draws
    );
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
    assert_eq!(driver.app.music_request, Some(20128));
    driver.act(ApplicationAction::Playlist(0));
    assert!(!driver.app.playlist[0]);
    assert_eq!(driver.state, unchanged);
    let seed = driver.state.random.clone();
    driver.act(ApplicationAction::OpenInformation(0));
    assert_eq!(driver.state.random, seed);
    assert_eq!(driver.state, unchanged);
    let extracted = root.join("analysis/extracted");
    let native =
        centurions_content::native_text::NativeText::load(&driver.content, &extracted).unwrap();
    let strings = driver
        .content
        .resources
        .iter()
        .filter(|r| {
            r.original.ends_with("/Cargo Pod")
                && r.r#type == "STR#"
                && matches!(r.id, 129 | 131 | 132 | 133)
        })
        .map(|r| {
            (
                r.id,
                centurions_content::read_resource(&extracted, r).unwrap(),
            )
        })
        .collect();
    let fields =
        centurions_core::character_display::information(&driver.state, 0, &native, &strings)
            .unwrap();
    assert_eq!(fields[&67].0, driver.state.warriors[0].name);
    for (reference, detailed) in [
        (
            include_str!("../../../../evidence/fixtures/warrior-information.json"),
            false,
        ),
        (
            include_str!("../../../../evidence/fixtures/warrior-statistics.json"),
            true,
        ),
    ] {
        let reference: serde_json::Value = serde_json::from_str(reference).unwrap();
        for case in reference["cases"].as_array().unwrap() {
            let bytes = decode(case["before"].as_str().unwrap());
            let mut sample = Session::new(&driver.content);
            sample.warriors = vec![centurions_content::Warrior::read(&bytes).unwrap()];
            let mut fields = if detailed {
                centurions_core::character_display::statistics(&sample, 0).unwrap()
            } else {
                centurions_core::character_display::information(&sample, 0, &native, &strings)
                    .unwrap()
            };
            if detailed {
                fields.remove(&37);
            }
            assert_eq!(
                serde_json::to_value(fields).unwrap(),
                case["fields"],
                "{}",
                case["case"]
            );
        }
    }
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
    apply(Command::ImportStarters, &mut state, &content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        &content,
    )
    .unwrap();
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
    assert_eq!(state.native_work.inventory_record.as_ref(), Some(&record));
    let drag = ui.inventory.release(Some(vec2(500., 50.))).unwrap();
    let Some(Action::Command(command)) = ui
        .resolve_inventory(Some(Action::Inventory(drag)), &state, &content, 2.)
        .unwrap()
    else {
        panic!("transfer did not reach command boundary")
    };
    let events = apply(command, &mut state, &content).unwrap();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::ItemTransferred { from: 0, to: 4, .. }))
    );
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
    for case in receipt["instances"].as_array().unwrap() {
        assert_eq!(
            centurions_core::shop::preview(&content, case["item"].as_i64().unwrap() as i16)
                .unwrap()
                .0
                .to_vec(),
            decode(case["record"].as_str().unwrap())
        );
    }
    let native = centurions_content::native_text::NativeText::load(
        &content,
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut strings = std::collections::BTreeMap::new();
    for resource in content.resources.iter().filter(|r| r.r#type == "STR#") {
        strings.insert(
            resource.id,
            centurions_content::read_resource(&root.join("analysis/extracted"), resource).unwrap(),
        );
    }
    for warrior in &state.warriors {
        for item in warrior.inventory().unwrap() {
            centurions_core::item_display::row(&content, &item, &native, &strings).unwrap();
            centurions_core::item_display::detail(
                &content,
                content.map(5).unwrap(),
                &item,
                false,
                &native,
                &strings,
            )
            .unwrap();
        }
    }
    for case in receipt["instances"].as_array().unwrap() {
        let item = centurions_core::shop::preview(&content, case["item"].as_i64().unwrap() as i16)
            .unwrap();
        centurions_core::item_display::detail(
            &content,
            content.map(5).unwrap(),
            &item,
            true,
            &native,
            &strings,
        )
        .unwrap();
    }
    let panels = native_fixture("inventory-details.json");
    for case in panels["cases"].as_array().unwrap() {
        let item = centurions_content::InventoryRecord(
            decode(case["record"].as_str().unwrap()).try_into().unwrap(),
        );
        let plan = centurions_core::item_display::detail(
            &content,
            content.map(5).unwrap(),
            &item,
            case["stock"].as_bool().unwrap(),
            &native,
            &strings,
        )
        .unwrap();
        let mut position = (0, 0);
        let mut fields = vec![];
        for effect in plan.effects {
            use centurions_core::item_display::output::{Draw, PanelEffect};
            match effect {
                PanelEffect::Drawing(Draw::MoveTo {
                    horizontal,
                    vertical,
                }) => position = (horizontal, vertical),
                PanelEffect::NumberToString { value, .. } if fields.len() < 13 => {
                    fields.push(serde_json::json!([position.0, position.1, value]))
                }
                _ => {}
            }
        }
        assert_eq!(serde_json::Value::Array(fields), case["fields"]);
    }
    let source = content.map(5).unwrap().header[92 * 42..93 * 42].to_vec();
    for case in receipt["shops"].as_array().unwrap().iter().skip(1) {
        let mut probe = state.clone();
        let mut record = source.clone();
        record[26..28].copy_from_slice(&(-(case["shop"].as_i64().unwrap() as i16)).to_be_bytes());
        probe.exploration.as_mut().unwrap().script = Some(story_frame(record, 1, 92));
        probe.exploration.as_mut().unwrap().presentation =
            Some(centurions_core::Presentation::Dialogue {
                id: 919,
                text: String::new(),
            });
        apply(Command::Continue, &mut probe, &content).unwrap();
        let offset = case["shop"].as_u64().unwrap() as usize * 3002;
        assert_eq!(
            &probe.shops[&0][offset..offset + 3002],
            decode(case["stock"].as_str().unwrap())
        );
    }
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
        &state.shops[&0][..3002],
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
    let special = native_fixture("special-shop-purchase.json");
    let generated = native_fixture("special-shop.json");
    for case in special["cases"].as_array().unwrap() {
        let mut content = content.clone();
        let map_index = state.exploration.as_ref().unwrap().map as usize;
        let map = &mut content.scenario.as_mut().unwrap().maps[map_index];
        std::sync::Arc::make_mut(&mut map.shared)
            .combat
            .as_mut()
            .unwrap()
            .encounters[3650..3660]
            .copy_from_slice(&[255, 255, 0, 1, 0, 2, 0, 0, 0, 0]);
        let script = map.script(191).unwrap().to_vec();
        let mut probe = state.clone();
        probe.exploration.as_mut().unwrap().presentation =
            Some(centurions_core::Presentation::Dialogue {
                id: 740,
                text: String::new(),
            });
        probe.exploration.as_mut().unwrap().script = Some(story_frame(script, 1, 0));
        apply(Command::Continue, &mut probe, &content).unwrap();
        let id = case["item"].as_i64().unwrap() as i16;
        probe.warriors[0] =
            centurions_content::Warrior::read(&decode(case["before"].as_str().unwrap())).unwrap();
        probe.shops.get_mut(&probe.scenario_level).unwrap()[3002..6004]
            .copy_from_slice(&decode(case["stock_before"].as_str().unwrap()));
        let stock = centurions_core::shop::stock(&probe)
            .unwrap()
            .into_iter()
            .find(|(_, item, _)| *item == id)
            .unwrap()
            .0;
        assert_eq!(
            centurions_core::shop::price(&probe, &content, id, false).unwrap(),
            case["price"].as_i64().unwrap() as i32
        );
        probe.party_currency = 900000;
        probe.random = centurions_core::RandomState::new(12);
        centurions_core::save(&path, &probe, &content).unwrap();
        probe = centurions_core::load(&path, &content).unwrap();
        let before = probe.clone();
        assert!(
            matches!(apply(Command::SellItem { warrior: 0, slot: 0 }, &mut probe, &content), Err(centurions_core::RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::NativeFeedback::Warning { index: generated["refusals"][0]["warning"].as_i64().unwrap() as i16 }])
        );
        assert_eq!(probe, before);
        apply(Command::BuyItem { warrior: 0, stock }, &mut probe, &content).unwrap();
        let reference = generated["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["item"] == case["item"])
            .unwrap();
        assert_eq!(
            probe.warriors[0]
                .inventory()
                .unwrap()
                .into_iter()
                .find(|record| record.id() == id)
                .unwrap()
                .0
                .as_slice(),
            &decode(reference["record"].as_str().unwrap())
        );
        assert_eq!(
            probe.random.seed,
            reference["seed_after"].as_u64().unwrap() as u32
        );
        assert_eq!(probe.random.draws, reference["rng_draws"].as_u64().unwrap());
        assert_eq!(
            probe.warriors[0].bytes,
            decode(case["after"].as_str().unwrap())
        );
        assert_eq!(
            &probe.shops[&probe.scenario_level][3002..6004],
            &decode(case["stock_after"].as_str().unwrap())
        );
        assert_eq!(probe.random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(probe.random.draws, case["draws"].as_u64().unwrap());
        assert_eq!(
            probe.party_currency,
            case["currency"].as_i64().unwrap() as i32
        );
        assert_eq!(
            probe.shop_access.as_ref().unwrap().remaining,
            case["remaining"].as_i64().unwrap() as i16
        );
        let before = probe.clone();
        assert!(
            matches!(apply(Command::BuyItem { warrior: 0, stock }, &mut probe, &content), Err(centurions_core::RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::NativeFeedback::Warning { index: generated["refusals"][1]["warning"].as_i64().unwrap() as i16 }])
        );
        assert_eq!(probe, before);
        centurions_core::save(&path, &probe, &content).unwrap();
        assert_eq!(centurions_core::load(&path, &content).unwrap(), probe);
        apply(Command::CloseShop, &mut probe, &content).unwrap();
        assert!(matches!(
            probe.exploration.as_ref().unwrap().presentation,
            Some(centurions_core::Presentation::Dialogue { id: 741, .. })
        ));
    }
    apply(Command::CloseShop, &mut restored, &content).unwrap();
    assert!(
        restored
            .exploration
            .as_ref()
            .unwrap()
            .presentation
            .is_none()
    );
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
    let initial = restored.clone();
    for (name, index, regional) in [
        ("encounter-inert-items.json", 1),
        ("encounter-script-items.json", 0),
        ("encounter-script-items.json", 2),
        ("item-modifiers.json", 2),
        ("item-modifiers.json", 4),
        ("encounter-medical.json", 2),
        ("walking-encounter-item.json", 0),
        ("calendar-encounter-item.json", 0),
        ("outdoor-indexed-items.json", 1),
    ]
    .into_iter()
    .map(|(name, index)| (name, index, false))
    .chain(std::iter::once(("calendar-encounter-item.json", 0, true)))
    {
        let mut restored = initial.clone();
        ui.feedback.clear();
        let mut content = content.clone();
        let encounter = native_fixture(name);
        let case = &encounter["cases"][index];
        let walking = case["caller"] == "walking";
        let calendar = case["caller"] == "calendar" && !regional;
        let global = case["global"] == true;
        if let Some(definition) = case.get("definition") {
            content.items[case["item"].as_u64().unwrap() as usize] =
                decode(definition.as_str().unwrap());
        }
        if case.get("active_body").is_some() {
            let combat: serde_json::Value =
                crate::original_content::fixture("earned-combat-reload.json");
            restored = serde_json::from_value(combat["foundations"][0]["session"].clone()).unwrap();
            let battle = restored.combat.as_mut().unwrap();
            battle.order[battle.cursor] = 2;
        }
        let native_party = case["before"]
            .as_str()
            .unwrap_or_else(|| case["before"]["party"].as_str().unwrap());
        restored.warriors = decode(native_party)
            .chunks(3138)
            .enumerate()
            .map(|(i, bytes)| {
                let mut warrior = restored.warriors[i].clone();
                warrior.bytes = bytes.to_vec();
                warrior
            })
            .collect();
        if case["before"].get("seed").is_some() {
            restored.random = centurions_core::RandomState {
                seed: case["before"]["seed"].as_u64().unwrap() as u32,
                draws: case["before"]["draws"].as_u64().unwrap(),
            };
        }
        if let Some(tail) = case["before"].get("scenario_duration_tail") {
            restored.scenario_duration_tail = Some(tail.as_i64().unwrap() as i16);
        }
        let position = restored.exploration.as_mut().unwrap();
        let mut record = vec![0; 42];
        record[8..10].copy_from_slice(&5_i16.to_be_bytes());
        record[24..26].copy_from_slice(&8_i16.to_be_bytes());
        position.script = Some(centurions_core::ScriptState {
            item_return: name == "encounter-script-items.json",
            scheduled: !walking,
            origin: walking.then_some(centurions_core::ScriptOrigin {
                map: position.map,
                x: position.x,
                y: position.y,
                level: restored.scenario_level,
            }),
            ..centurions_core::ScriptState::scheduled(record, 0)
        });
        position.presentation = Some(centurions_core::Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        if calendar {
            position.day = 2;
            position.hour = 0;
            position.minute = 0;
            restored.time_continuation = Some(centurions_core::TimeContinuation {
                remaining_ticks: 0,
                minutes_per_tick: 5,
                schedule_cursor: position.schedule.len() / 40,
            });
        }
        if regional {
            position.last_move = [1, -1];
            position.floor.fill(0);
            position.event_headers.fill(0);
            restored.movement_continuation =
                Some(centurions_core::MovementContinuation::RegionalReturn { header: true });
        }
        if global {
            position.script = None;
            position.presentation = None;
        } else {
            apply(Command::Continue, &mut restored, &content).unwrap();
        }
        if case["encounter_active"] == false {
            let position = restored.exploration.as_mut().unwrap();
            position.script = None;
            position.presentation = None;
        }
        ui.inspection = Some(crate::presentation::Inspection::Items);
        centurions_core::save(&path, &restored, &content).unwrap();
        restored = centurions_core::load(&path, &content).unwrap();
        let before = restored.clone();
        ui.inventory.open(
            case.get("actor")
                .map_or(2, |v| v.as_u64().unwrap() as usize),
            restored.warriors.len(),
        );
        assert!(matches!(
            ui.resolve_inventory(
                Some(Action::Inventory(crate::inventory::InventoryAction::Done)),
                &restored,
                &content,
                30.
            )
            .unwrap(),
            Some(Action::Close)
        ));
        assert_eq!(restored, before);
        for _ in 0..2 {
            let action = crate::inventory::InventoryAction::Item {
                side: 0,
                slot: case["slot"].as_u64().unwrap() as usize,
            };
            if let Some(Action::Command(command)) = ui
                .resolve_inventory(Some(Action::Inventory(action)), &restored, &content, 31.)
                .unwrap()
            {
                let inspecting = matches!(command, Command::InspectInventory { .. });
                match apply(app.command(command), &mut restored, &content) {
                    Ok(events) => {
                        if !inspecting {
                            ui.medical_result(&restored, &events);
                        }
                        ui.events(&events);
                    }
                    Err(error) => {
                        assert!(case.get("warning").is_some());
                        ui.failure(error);
                    }
                }
            }
        }
        if name == "encounter-medical.json" {
            assert!(restored.outdoor_psi.is_some());
            assert!(ui.inspection.is_none());
            centurions_core::save(&path, &restored, &content).unwrap();
            restored = centurions_core::load(&path, &content).unwrap();
            let events = apply(
                app.command(Command::CancelOutdoorPsi),
                &mut restored,
                &content,
            )
            .unwrap();
            ui.medical_result(&restored, &events);
            ui.events(&events);
            assert!(restored.outdoor_psi.is_none());
        }
        if case.get("warning").is_some() {
            assert_eq!(
                ui.feedback.back(),
                Some(&centurions_core::feedback::NativeFeedback::Warning {
                    index: case["warning"].as_i64().unwrap() as i16
                })
            );
            assert!(ui.technical_error.is_none());
            assert!(ui.inspection == Some(crate::presentation::Inspection::Items));
            assert!(matches!(
                restored.exploration.as_ref().unwrap().presentation,
                Some(centurions_core::Presentation::Typed { .. })
            ));
        } else if case.get("dialog_open").is_some() {
            assert!(ui.inspection == Some(crate::presentation::Inspection::Items));
            assert!(ui.inventory.selected.is_none());
            assert!(
                restored
                    .exploration
                    .as_ref()
                    .unwrap()
                    .presentation
                    .is_none()
            );
        } else if walking || calendar || global || regional {
            let position = restored.exploration.as_ref().unwrap();
            assert!(position.presentation.is_none() && position.script.is_none());
            assert!(ui.inspection.is_none());
            assert!(restored.time_continuation.is_none());
            if regional {
                assert_eq!(position.last_move, [0; 2]);
                assert!(restored.movement_continuation.is_none());
            }
            if global {
                assert_eq!(
                    restored.scenario_duration_tail,
                    Some(case["after"]["scenario_duration_tail"].as_i64().unwrap() as i16)
                );
            }
        } else {
            let expected = if name == "encounter-script-items.json" {
                851
            } else {
                950
            };
            assert!(
                matches!(restored.exploration.as_ref().unwrap().presentation, Some(centurions_core::Presentation::Dialogue { id, .. }) if id == expected)
            );
        }
        assert_eq!(
            restored
                .warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            decode(
                case["after"]
                    .as_str()
                    .unwrap_or_else(|| case["after"]["party"].as_str().unwrap())
            )
        );
        assert_eq!(restored.ammunition, before.ammunition);
        let expected_random = if case["before"].get("seed").is_some() {
            centurions_core::RandomState {
                seed: case["after"]["seed"].as_u64().unwrap() as u32,
                draws: case["after"]["draws"].as_u64().unwrap(),
            }
        } else {
            before.random
        };
        assert_eq!(restored.random, expected_random);
        if global {
            centurions_core::save(&path, &restored, &content).unwrap();
            assert_eq!(centurions_core::load(&path, &content).unwrap(), restored);
        }
    }
    std::fs::remove_file(path).unwrap();
}
