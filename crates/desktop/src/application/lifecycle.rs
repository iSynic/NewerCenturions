//! The logo, title, introduction, About box, exchange choice and quitting.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{Command, Phase, Session, apply};

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
                if self.intro.prologue.take().is_some() {
                    next = Some(Action::Command(Command::Continue));
                }
            }
            ApplicationAction::QuitAnswer(answer) => {
                self.questions.quit = false;
                if answer {
                    self.quit = true;
                } else if state.phase != Phase::Combat {
                    self.quit_after_save = true;
                    next = Some(Action::Save);
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }
}
