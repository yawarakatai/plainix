use crate::generate::write_atomic;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Eq, PartialEq)]
pub enum UpdateResult {
    Updated,
    AlreadyCurrent,
}

pub fn update(directory: &Path) -> Result<UpdateResult, String> {
    update_with(directory, update_lock, verify_systems)
}

fn update_with<U, V>(directory: &Path, update: U, verify: V) -> Result<UpdateResult, String>
where
    U: FnOnce(&Path) -> Result<(), String>,
    V: FnOnce(&Path) -> Result<(), String>,
{
    for file in ["plainix.nix", "flake.nix", "flake.lock"] {
        if !directory.join(file).is_file() {
            return Err(format!(
                "Plainix: `{file}` not found; run this from a directory set up with `plainix init`"
            ));
        }
    }

    let lock_path = directory.join("flake.lock");
    let previous = read_lock(&lock_path)?;

    if let Err(error) = update(directory).and_then(|()| verify(directory)) {
        return Err(match write_atomic(&lock_path, &previous) {
            Ok(()) => format!("{error}\nPlainix: restored the previous `flake.lock`"),
            Err(restore_error) => format!("{error}\n{restore_error}"),
        });
    }

    if read_lock(&lock_path)? == previous {
        Ok(UpdateResult::AlreadyCurrent)
    } else {
        Ok(UpdateResult::Updated)
    }
}

fn read_lock(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("Plainix: could not read `{}`: {error}", path.display()))
}

fn update_lock(directory: &Path) -> Result<(), String> {
    let status = Command::new("nix")
        .args(["flake", "update"])
        .current_dir(directory)
        .status()
        .map_err(|error| format!("Plainix: could not run `nix flake update`: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Plainix: `nix flake update` failed with {status}. See the Nix output above."
        ))
    }
}

fn verify_systems(directory: &Path) -> Result<(), String> {
    eprintln!("Checking that every declared system still evaluates...");
    let status = Command::new("nix")
        .args([
            "eval",
            "--json",
            ".#devShells",
            "--apply",
            "shells: builtins.mapAttrs (_: shell: shell.default.drvPath) shells",
        ])
        .current_dir(directory)
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|error| format!("Plainix: could not run `nix eval`: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(
            "Plainix: the updated environment does not evaluate. See the Nix output above."
                .to_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn project(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("plainix-update-{name}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        for file in ["plainix.nix", "flake.nix"] {
            fs::write(path.join(file), "{ }\n").unwrap();
        }
        fs::write(path.join("flake.lock"), "old\n").unwrap();
        path
    }

    fn write_new_lock(path: &Path) -> Result<(), String> {
        fs::write(path.join("flake.lock"), "new\n").map_err(|error| error.to_string())
    }

    fn lock_contents(path: &Path) -> String {
        fs::read_to_string(path.join("flake.lock")).unwrap()
    }

    #[test]
    fn keeps_new_lock_when_verification_passes() {
        let directory = project("success");

        let result = update_with(&directory, write_new_lock, |_| Ok(())).unwrap();

        assert_eq!(result, UpdateResult::Updated);
        assert_eq!(lock_contents(&directory), "new\n");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn restores_lock_when_verification_fails() {
        let directory = project("rollback");

        let error =
            update_with(&directory, write_new_lock, |_| Err("broken".to_owned())).unwrap_err();

        assert!(error.starts_with("broken\n"));
        assert!(error.contains("restored"));
        assert_eq!(lock_contents(&directory), "old\n");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn restores_lock_when_update_fails() {
        let directory = project("update-failure");

        let error = update_with(
            &directory,
            |path| {
                write_new_lock(path)?;
                Err("offline".to_owned())
            },
            |_| panic!("verification must not run after a failed update"),
        )
        .unwrap_err();

        assert!(error.starts_with("offline\n"));
        assert_eq!(lock_contents(&directory), "old\n");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reports_when_lock_is_unchanged() {
        let directory = project("current");

        let result = update_with(&directory, |_| Ok(()), |_| Ok(())).unwrap();

        assert_eq!(result, UpdateResult::AlreadyCurrent);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn refuses_directories_without_a_project() {
        let directory = project("missing");
        fs::remove_file(directory.join("plainix.nix")).unwrap();

        let error = update_with(&directory, |_| panic!("must not update"), |_| Ok(())).unwrap_err();

        assert!(error.contains("plainix.nix"));
        fs::remove_dir_all(directory).unwrap();
    }
}
