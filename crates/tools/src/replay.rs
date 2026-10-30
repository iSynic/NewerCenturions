//! `replay`: applies a command JSON file through the production command
//! boundary and prints each command's events, then the final session.
use centurions_content::Content;
use centurions_core::{Command, CommandPreferences, Session, apply};
use std::{error::Error, path::PathBuf};

/// Options following the command JSON path. Unrecognized arguments are ignored.
struct Options {
    /// `--trace`: enables the diagnostic trace on every command.
    trace: bool,
    /// `--save OUTPUT`: writes a versioned save of the final state, or of the
    /// state at a rejected command boundary.
    save: Option<PathBuf>,
    /// `--from SAVE`: starts from a saved session instead of a new game. The
    /// inner `None` is a missing path, reported only after the commands are read.
    from: Option<Option<PathBuf>>,
}

impl Options {
    /// Reads the replay options, rejecting a repeated --save and a --save with no
    /// path.
    fn parse(options: &[String]) -> Result<Self, Box<dyn Error>> {
        let trace = options.iter().any(|arg| arg == "--trace");
        let saves: Vec<usize> = options
            .iter()
            .enumerate()
            .filter(|(_, arg)| *arg == "--save")
            .map(|(index, _)| index)
            .collect();
        if saves.len() > 1 {
            return Err("--save may be specified only once".into());
        }
        let save = saves
            .first()
            .map(|index| {
                options
                    .get(index + 1)
                    .filter(|path| !path.starts_with("--"))
                    .map(PathBuf::from)
                    .ok_or("--save needs an output path")
            })
            .transpose()?;
        let from = options
            .iter()
            .position(|arg| arg == "--from")
            .map(|index| options.get(index + 1).map(PathBuf::from));
        Ok(Self { trace, save, from })
    }
}

/// Applies the command JSON file one command at a time to a new session or to
/// the one in `--from`, printing the trace, events and final session. A rejected
/// command stops the replay, after saving the state at that point when --save
/// was given.
pub(crate) fn run(
    mut args: impl Iterator<Item = String>,
    content: &Content,
) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(args.next().ok_or("replay needs command JSON path")?);
    let options = Options::parse(&args.collect::<Vec<_>>())?;
    let commands: Vec<Command> = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut session = match options.from {
        Some(from) => centurions_core::load(&from.ok_or("--from needs a save path")?, content)?,
        None => Session::new(content),
    };
    for command in commands {
        let command = if options.trace {
            traced(command)
        } else {
            command
        };
        let events = apply(command, &mut session, content);
        for entry in centurions_core::trace::take() {
            println!("{}", serde_json::to_string(&entry)?);
        }
        match events {
            Ok(events) => println!("{}", serde_json::to_string(&events)?),
            Err(error) => {
                // Preserve the actual command boundary, including deliberate partial effects.
                if let Some(output) = &options.save {
                    centurions_core::save(output, &session, content)?;
                    println!("{}", serde_json::to_string(&session)?);
                }
                return Err(error.into());
            }
        }
    }
    if let Some(output) = &options.save {
        centurions_core::save(output, &session, content)?;
    }
    println!("{}", serde_json::to_string(&session)?);
    Ok(())
}

/// Enables the diagnostic trace through the command's preferences. A bare
/// command already runs with default preferences, so wrapping it is neutral.
pub(crate) fn traced(command: Command) -> Command {
    let (command, mut preferences) = match command {
        Command::WithPreferences {
            command,
            preferences,
        } => (command, preferences),
        command => (Box::new(command), CommandPreferences::default()),
    };
    preferences.trace = true;
    Command::WithPreferences {
        command,
        preferences,
    }
}
