use centurions_core::Session;

/// Compare a command against the same command resumed from a strict save checkpoint.
pub fn saved_command(
    command: centurions_core::Command,
    state: &mut Session,
    content: &centurions_content::Content,
) -> Vec<centurions_core::Event> {
    let mut resumed = super::checkpoint(state, content, "command");
    super::replay_command(command, state, &mut resumed, content)
}

/// The active combat state shared by native command comparisons.
pub fn current_combat(state: &Session) -> &centurions_core::combat::Combat {
    state.combat.as_ref().unwrap()
}
