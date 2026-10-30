use centurions_content::installation::Installation;
use std::path::PathBuf;
mod acceptance;
mod convert_assets;
mod replay;

/// The text printed for --help and after an unknown action.
const USAGE: &str = "\
Usage: centurions-tools [ACTION] [ROOT] [ARGUMENTS...]

ROOT is the New Centurions installation folder. Without it, CENTURIONS_ASSET_ROOT
is used, then the executable's own folder.

Actions:
  inspect ROOT                      Print content identity, warriors and the
                                    opening mission (the default action)
  replay ROOT COMMANDS [OPTIONS]    Apply a command JSON file, printing events and
                                    the final session
      --trace                       Enable the diagnostic trace
      --save OUTPUT                 Write a versioned save of the final state, or
                                    of the state at a rejected command
      --from SAVE                   Start from a saved session, not a new game
  audit-resources ROOT              Print a JSON decoder report for every resource
  audit-import ROOT DATA_I1 COMMANDS WORK
                                    Import an original save, apply commands and
                                    round-trip it; WORK must be inside the
                                    checkout's analysis directory
  convert-assets ROOT               Write portrait, tactical and Cargo Pod PNG
                                    files to ROOT/analysis/assets

Options:
  -h, --help                        Print this message";

/// The actions the tool understands.
const ACTIONS: [&str; 5] = [
    "inspect",
    "replay",
    "audit-resources",
    "audit-import",
    "convert-assets",
];

/// Runs the action named first on the command line (inspect when none is given)
/// on the installation found from ROOT, CENTURIONS_ASSET_ROOT or the
/// executable's folder.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let action = args.next().unwrap_or_else(|| "inspect".into());
    if action == "--help" || action == "-h" {
        println!("{USAGE}");
        return Ok(());
    }
    if !ACTIONS.contains(&action.as_str()) {
        // Same first line as an error returned from `main`, followed by the usage.
        eprintln!(
            "Error: {:?}\n\n{USAGE}",
            format!("unknown command {action}")
        );
        std::process::exit(1);
    }
    let root = args
        .next()
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("CENTURIONS_ASSET_ROOT").map(PathBuf::from))
        .unwrap_or_else(Installation::default_root);
    let installation = Installation::open(&root)?;
    let content = installation.content;
    let resource_root = installation.resource_root;
    match action.as_str() {
        "audit-resources" => {
            println!("{}", acceptance::resources(&content, &resource_root));
        }
        "audit-import" => {
            let original = PathBuf::from(
                args.next()
                    .ok_or("audit-import needs the original Data I1 path")?,
            );
            let commands = PathBuf::from(args.next().ok_or("audit-import needs command JSON")?);
            let work = PathBuf::from(
                args.next()
                    .ok_or("audit-import needs a new analysis work directory")?,
            );
            println!(
                "{}",
                acceptance::import(&content, &original, &commands, &work)?
            );
        }
        "convert-assets" => {
            let converted = convert_assets::run(&content, &resource_root, &root)?;
            println!("Converted {converted} original color icons and pictures");
        }
        "inspect" => {
            println!(
                "Content identity: {}\nItems: {}",
                content.identity,
                content.items.len()
            );
            for warrior in &content.warriors {
                println!(
                    "{}: stamina {}, inventory {}",
                    warrior.name,
                    warrior.stamina()?,
                    warrior.inventory()?.len()
                );
            }
            if let Some(opening) = content.map(0) {
                println!(
                    "Opening mission: {} at {:?}; {} map cells",
                    opening.name,
                    opening.start,
                    opening.floor.len()
                );
            }
        }
        "replay" => replay::run(args, &content)?,
        _ => unreachable!("actions are validated before the installation opens"),
    }
    Ok(())
}
