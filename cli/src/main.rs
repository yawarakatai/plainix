mod generate;
mod platform;
mod preset;

use generate::{EnvrcResult, GitResult, GitignoreResult};
use inquire::{MultiSelect, error::InquireError};
use preset::Preset;
use std::process::ExitCode;

const USAGE: &str = "Plainix scaffolds reproducible project environments with Nix.\n\nUsage:\n  plainix init [OPTIONS]\n  plainix --help\n  plainix --version\n\nOptions for init:\n  --with <TOOL>  Include a tool without prompting; may be repeated\n  --empty        Create an empty environment without prompting\n\nWithout init options, Plainix opens an interactive tool selection.\nPlainix generates .envrc for nix-direnv but never runs direnv allow.\n";

enum SelectionMode {
    Interactive,
    Explicit(Vec<String>),
    Empty,
}

enum Action {
    Init(SelectionMode),
    Help,
    Version,
}

fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Action, String> {
    let mut arguments = arguments.into_iter();
    let Some(command) = arguments.next() else {
        return Ok(Action::Help);
    };

    match command.as_str() {
        "-h" | "--help" => Ok(Action::Help),
        "-V" | "--version" => Ok(Action::Version),
        "init" => parse_init_args(arguments),
        _ => Err(format!("Plainix: unknown command `{command}`\n\n{USAGE}")),
    }
}

fn parse_init_args(arguments: impl IntoIterator<Item = String>) -> Result<Action, String> {
    let mut arguments = arguments.into_iter();
    let mut tools = Vec::new();
    let mut empty = false;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "--empty" => empty = true,
            "--with" => {
                let Some(tool) = arguments.next().filter(|value| !value.starts_with('-')) else {
                    return Err(format!("Plainix: `--with` requires a tool name\n\n{USAGE}"));
                };
                tools.push(tool);
            }
            _ => {
                if let Some(tool) = argument.strip_prefix("--with=")
                    && !tool.is_empty()
                {
                    tools.push(tool.to_owned());
                } else {
                    return Err(format!("Plainix: unknown option `{argument}`\n\n{USAGE}"));
                }
            }
        }
    }

    if empty && !tools.is_empty() {
        return Err("Plainix: `--empty` cannot be combined with `--with`".to_owned());
    }
    if empty {
        Ok(Action::Init(SelectionMode::Empty))
    } else if tools.is_empty() {
        Ok(Action::Init(SelectionMode::Interactive))
    } else {
        Ok(Action::Init(SelectionMode::Explicit(tools)))
    }
}

fn choose_presets(mode: SelectionMode, system: &str) -> Result<Vec<Preset>, String> {
    match mode {
        SelectionMode::Empty => Ok(Vec::new()),
        SelectionMode::Explicit(ids) => preset::resolve(&ids, system),
        SelectionMode::Interactive => MultiSelect::new(
            "Select tools for this project:",
            preset::available_for(system),
        )
        .with_help_message("Space to select, Enter to continue, type to filter")
        .prompt()
        .map_err(|error| match error {
            InquireError::NotTTY => {
                "Plainix: interactive selection requires a terminal; use `plainix init --with <tool>` or `plainix init --empty`"
                    .to_owned()
            }
            InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                "Plainix: initialization cancelled".to_owned()
            }
            error => format!("Plainix: could not read tool selection: {error}"),
        }),
    }
}

fn run() -> Result<(), String> {
    match parse_args(std::env::args().skip(1))? {
        Action::Help => print!("{USAGE}"),
        Action::Version => println!("plainix {}", env!("CARGO_PKG_VERSION")),
        Action::Init(mode) => {
            let directory = std::env::current_dir()
                .map_err(|error| format!("Plainix: could not read current directory: {error}"))?;
            generate::check_directory(&directory)?;
            let system = platform::current_system(std::env::consts::ARCH, std::env::consts::OS)?;
            let presets = choose_presets(mode, system)?;
            let result = generate::initialize(&directory, &presets)?;
            let tools = if result.presets.is_empty() {
                "none".to_owned()
            } else {
                result
                    .presets
                    .iter()
                    .map(|preset| preset.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let packages = if result.packages.is_empty() {
                "none".to_owned()
            } else {
                result.packages.join(", ")
            };

            println!("Tools: {tools}");
            println!("Platforms: {}", result.systems.join(", "));
            println!("Packages: {packages}");
            println!();
            println!("Created plainix.nix");
            println!("Created flake.nix");
            println!("Created flake.lock");

            match result.envrc {
                EnvrcResult::Created => {
                    println!("Created .envrc");
                }
                EnvrcResult::KeptExisting => {
                    println!("Kept existing .envrc; add `use flake` manually if needed");
                }
            }

            match result.gitignore {
                GitignoreResult::Created => println!("Created .gitignore with .direnv/"),
                GitignoreResult::Updated => println!("Added .direnv/ to .gitignore"),
                GitignoreResult::Failed(error) => {
                    println!("Could not add .direnv/ to .gitignore: {error}");
                }
                GitignoreResult::NotApplicable | GitignoreResult::AlreadyPresent => {}
            }

            match result.git {
                GitResult::NotRepository => {}
                GitResult::Prepared { files } => {
                    println!();
                    println!(
                        "Prepared generated files for Git so Nix can see them; contents remain unstaged."
                    );
                    println!("To commit them later:");
                    println!("  git add {}", files.join(" "));
                }
                GitResult::Manual { files, error } => {
                    println!();
                    println!("Could not prepare generated files for Git: {error}");
                    println!("Before running nix develop, stage them manually:");
                    println!("  git add {}", files.join(" "));
                }
            }

            if result.envrc == EnvrcResult::Created {
                println!();
                println!("If you use direnv, review the files and run:");
                println!("  direnv allow");
            }

            println!();
            println!("Enter the environment with:");
            println!("  nix develop");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plainix_without_arguments_shows_help() {
        assert!(matches!(parse_args(Vec::new()).unwrap(), Action::Help));
    }

    #[test]
    fn parses_interactive_init() {
        assert!(matches!(
            parse_args(["init"].map(str::to_owned)).unwrap(),
            Action::Init(SelectionMode::Interactive)
        ));
    }

    #[test]
    fn parses_explicit_tools() {
        let action =
            parse_args(["init", "--with", "godot", "--with=blender"].map(str::to_owned)).unwrap();
        let Action::Init(SelectionMode::Explicit(tools)) = action else {
            panic!("expected explicit tool selection");
        };
        assert_eq!(tools, vec!["godot", "blender"]);
    }

    #[test]
    fn parses_empty_environment() {
        assert!(matches!(
            parse_args(["init", "--empty"].map(str::to_owned)).unwrap(),
            Action::Init(SelectionMode::Empty)
        ));
    }

    #[test]
    fn rejects_invalid_init_options() {
        assert!(parse_args(["init", "--with"].map(str::to_owned)).is_err());
        assert!(parse_args(["init", "--empty", "--with", "rust"].map(str::to_owned)).is_err());
        assert!(parse_args(["init", "--direnv"].map(str::to_owned)).is_err());
    }
}
