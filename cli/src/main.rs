mod detect;
mod generate;
mod preset;

use generate::{EnvrcResult, GitResult, GitignoreResult};
use std::process::ExitCode;

const USAGE: &str = "Plainix scaffolds reproducible project environments with Nix.\n\nUsage:\n  plainix init\n  plainix --help\n  plainix --version\n\nPlainix generates .envrc for nix-direnv but never runs direnv allow.\n";

enum Action {
    Init,
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
        "init" => {
            if let Some(argument) = arguments.next() {
                if argument == "-h" || argument == "--help" {
                    Ok(Action::Help)
                } else {
                    Err(format!("Plainix: unknown option `{argument}`\n\n{USAGE}"))
                }
            } else {
                Ok(Action::Init)
            }
        }
        _ => Err(format!("Plainix: unknown command `{command}`\n\n{USAGE}")),
    }
}

fn run() -> Result<(), String> {
    match parse_args(std::env::args().skip(1))? {
        Action::Help => print!("{USAGE}"),
        Action::Version => println!("plainix {}", env!("CARGO_PKG_VERSION")),
        Action::Init => {
            let directory = std::env::current_dir()
                .map_err(|error| format!("Plainix: could not read current directory: {error}"))?;
            let result = generate::initialize(&directory)?;
            let presets = if result.projects.is_empty() {
                "none".to_owned()
            } else {
                result
                    .projects
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let packages = if result.packages.is_empty() {
                "none".to_owned()
            } else {
                result.packages.join(", ")
            };

            println!("Presets: {presets}");
            println!("Platform: {}", result.system);
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
    fn parses_init() {
        assert!(matches!(
            parse_args(["init"].map(str::to_owned)).unwrap(),
            Action::Init
        ));
    }

    #[test]
    fn rejects_unknown_options() {
        assert!(parse_args(["init", "--unknown"].map(str::to_owned)).is_err());
        assert!(parse_args(["init", "--direnv"].map(str::to_owned)).is_err());
    }
}
