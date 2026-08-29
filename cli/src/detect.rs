use std::fmt;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectKind {
    Rust,
    Python,
    TypeScript,
    Go,
    Zig,
    Cpp,
}

impl fmt::Display for ProjectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Zig => "Zig",
            Self::Cpp => "C/C++",
        })
    }
}

pub fn detect_projects(directory: &Path) -> Vec<ProjectKind> {
    let mut projects = Vec::new();

    if directory.join("Cargo.toml").is_file() {
        projects.push(ProjectKind::Rust);
    }
    if directory.join("pyproject.toml").is_file() {
        projects.push(ProjectKind::Python);
    }
    if directory.join("package.json").is_file() || directory.join("tsconfig.json").is_file() {
        projects.push(ProjectKind::TypeScript);
    }
    if directory.join("go.mod").is_file() {
        projects.push(ProjectKind::Go);
    }
    if directory.join("build.zig").is_file()
        || directory.join("build.zig.zon").is_file()
        || contains_source_with_extensions(directory, &["zig"])
    {
        projects.push(ProjectKind::Zig);
    }
    if directory.join("CMakeLists.txt").is_file()
        || directory.join("meson.build").is_file()
        || contains_source_with_extensions(directory, &["c", "cc", "cpp", "cxx"])
    {
        projects.push(ProjectKind::Cpp);
    }

    projects
}

fn contains_source_with_extensions(directory: &Path, extensions: &[&str]) -> bool {
    let Ok(entries) = directory.read_dir() else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        entry.path().is_file()
            && entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extensions.contains(&extension))
    })
}

pub fn current_system(architecture: &str, operating_system: &str) -> Result<&'static str, String> {
    match (architecture, operating_system) {
        ("x86_64", "linux") => Ok("x86_64-linux"),
        ("aarch64", "linux") => Ok("aarch64-linux"),
        ("aarch64", "macos") => Ok("aarch64-darwin"),
        _ => Err(format!(
            "Plainix: unsupported platform `{architecture}-{operating_system}`"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("plainix-{name}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn detects_multiple_project_types() {
        let directory = temporary_directory("detect");
        fs::write(directory.join("Cargo.toml"), "").unwrap();
        fs::write(directory.join("pyproject.toml"), "").unwrap();
        fs::write(directory.join("go.mod"), "module example.com/hello\n").unwrap();
        fs::write(directory.join("main.zig"), "").unwrap();
        fs::write(directory.join("main.c"), "").unwrap();

        assert_eq!(
            detect_projects(&directory),
            vec![
                ProjectKind::Rust,
                ProjectKind::Python,
                ProjectKind::Go,
                ProjectKind::Zig,
                ProjectKind::Cpp,
            ]
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn maps_supported_platforms() {
        assert_eq!(current_system("x86_64", "linux").unwrap(), "x86_64-linux");
        assert_eq!(
            current_system("aarch64", "macos").unwrap(),
            "aarch64-darwin"
        );
        assert!(current_system("riscv64", "linux").is_err());
    }
}
