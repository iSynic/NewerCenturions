//! The logo, title, introduction, About box, exchange choice and quitting.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{Command, Phase, Presentation, RuleError, Session, apply};

impl Application {
    /// Carries out the choices on the logo, introduction, About box, occupied-cell
    /// question, combat escape question and quit question.
    pub(super) fn handle_lifecycle(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let mut next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::ExchangeChoice(choice) => {
                if state.out_of_combat_walk_choice.is_some() {
                    events = apply(
                        self.command(Command::AnswerOccupiedChoice { exchange: choice }),
                        state,
                        content,
                    )?;
                } else {
                    let body = self
                        .questions
                        .exchange_offer
                        .ok_or("no occupied-cell choice is open")?;
                    if let Some(exchange) = choice {
                        events = apply(
                            if exchange {
                                Command::SwapCombatants { body }
                            } else {
                                Command::AttackFriend { body }
                            },
                            state,
                            content,
                        )?;
                    }
                }
                self.questions.exchange_offer = None;
            }
            ApplicationAction::CloseAbout => self.about = false,
            ApplicationAction::EscapeAnswer(answer) => {
                self.questions.escape = false;
                if answer {
                    events = apply(self.command(Command::EscapeCombat), state, content)?;
                }
            }
            ApplicationAction::SkipLogo => self.screen = Screen::Title,
            ApplicationAction::CloseIntro => {
                self.screen = self.intro.intro_return;
                let prologue = self.intro.prologue.take().is_some();
                if prologue
                    && matches!(
                        state
                            .exploration
                            .as_ref()
                            .and_then(|p| p.presentation.as_ref()),
                        Some(Presentation::Prologue { .. } | Presentation::Note { .. })
                    )
                {
                    next = Some(Action::Command(Command::Continue));
                }
            }
            ApplicationAction::QuitAnswer(answer) => {
                self.questions.quit = false;
                if answer {
                    self.quit_game(state, content)?;
                } else if state.phase != Phase::Combat {
                    self.quit_after_save = true;
                    next = Some(Action::Save);
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }

    /// Refuses to save or start a game when the last scenario chosen from the Mission menu (the
    /// first one when none has been chosen) holds the wrong number of owner resources for its place
    /// in the menu: warning 92 is raised and the program ends once it is read.
    ///
    /// Original: save, setupnewgame
    pub(crate) fn check_scenario_owner(&mut self, content: &Content) -> Result<(), RuleError> {
        let index = self
            .selected_scenario
            .as_ref()
            .and_then(|name| self.scenario_names.iter().position(|known| known == name))
            .unwrap_or(0);
        let item = super::menus::menu::mission::FIRST_SCENARIO + index;
        if self
            .scenario_names
            .get(index)
            .is_some_and(|name| centurions_core::scenario_owner_mismatch(content, name, item))
        {
            self.fatal_exit = true;
            return Err(RuleError::Refused {
                feedback: vec![centurions_core::feedback::GameFeedback::Warning { index: 92 }],
            });
        }
        Ok(())
    }

    /// Leaves the game. The preferences are written first, keeping the last scenario chosen. During
    /// a mission the credits are shared out and each squad member is saved to the roster, as at the
    /// end of a mission. The game is left even if the preferences or the roster cannot be written.
    ///
    /// Original: quitgame
    pub(crate) fn quit_game(
        &mut self,
        state: &Session,
        content: &Content,
    ) -> Result<(), centurions_core::RuleError> {
        self.quit = true;
        let preferences = self.help.save();
        if matches!(state.phase, Phase::Exploration | Phase::Combat) && !state.warriors.is_empty() {
            // The credits are shared out before each warrior is saved.
            let mut shared = state.clone();
            centurions_core::party::share_credits(
                &mut shared.warriors,
                &mut shared.party_currency,
            )?;
            let records = centurions_core::roster::mission_records(&shared, content)?;
            self.roster
                .as_mut()
                .ok_or("warrior roster missing")?
                .save_characters(&records)?;
        }
        preferences.map_err(Into::into)
    }
}
