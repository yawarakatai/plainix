mod detect;
mod generate;
mod preset;

use generate::{EnvrcResult, InitOptions};
use std::process::ExitCode;

const USAGE: &str = "Plainix scaffolds reproducible project environments with Nix.\n\nUsage:\n  plainix init [--direnv]\n  plainix --help\n  plainix --version\n\nOptions:\n  --direnv   Generate .envrc for nix-direnv (never runs direnv allow)\n";

enum Action {
    Init(InitOptions),
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
            let mut options = InitOptions::default();
            for argument in arguments {
                match argument.as_str() {
                    "--direnv" => options.direnv = true,
                    "-h" | "--help" => return Ok(Action::Help),
                    _ => return Err(format!("Plainix: unknown option `{argument}`\n\n{USAGE}")),
                }
            }
            Ok(Action::Init(options))
        }
        _ => Err(format!("Plainix: unknown command `{command}`\n\n{USAGE}")),
    }
}

fn run() -> Result<(), String> {
    match parse_args(std::env::args().skip(1))? {
        Action::Help => print!("{USAGE}"),
        Action::Version => println!("plainix {}", env!("CARGO_PKG_VERSION")),
        Action::Init(options) => {
            let directory = std::env::current_dir()
                .map_err(|error| format!("Plainix: could not read current directory: {error}"))?;
            let result = generate::initialize(&directory, options)?;
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
                EnvrcResult::NotRequested => {}
                EnvrcResult::Created => {
                    println!("Created .envrc");
                    println!();
                    println!("To activate it automatically, review the files and run:");
                    println!("  direnv allow");
                }
                EnvrcResult::KeptExisting => {
                    println!("Kept existing .envrc; add `use flake` manually if needed");
                }
            }

            if !result.untracked_files.is_empty() {
                println!();
                println!(
                    "Nix only sees tracked files in a Git repository. Stage the generated files:"
                );
                println!("  git add {}", result.untracked_files.join(" "));
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
    fn parses_init_options() {
        let action = parse_args(["init", "--direnv"].map(str::to_owned)).unwrap();
        let Action::Init(options) = action else {
            panic!("expected init action");
        };
        assert!(options.direnv);
    }

    #[test]
    fn rejects_unknown_options() {
        assert!(parse_args(["init", "--unknown"].map(str::to_owned)).is_err());
    }
}
