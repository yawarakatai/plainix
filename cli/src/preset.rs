use crate::platform::SUPPORTED_SYSTEMS;
use std::fmt;

const LINUX_SYSTEMS: &[&str] = &["x86_64-linux", "aarch64-linux"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub packages: &'static [&'static str],
    pub systems: &'static [&'static str],
}

impl Preset {
    pub fn supports(self, system: &str) -> bool {
        self.systems.contains(&system)
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} — {}", self.name, self.description)
    }
}

pub const PRESETS: &[Preset] = &[
    Preset {
        id: "blender",
        name: "Blender",
        description: "3D creation suite",
        packages: &["blender"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "cpp",
        name: "C/C++",
        description: "compiler, CMake, Make, Ninja, and pkg-config",
        packages: &["cmake", "gnumake", "ninja", "pkg-config", "stdenv.cc"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "freecad",
        name: "FreeCAD",
        description: "parametric 3D CAD",
        packages: &["freecad"],
        systems: LINUX_SYSTEMS,
    },
    Preset {
        id: "go",
        name: "Go",
        description: "compiler and language server",
        packages: &["go", "gopls"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "godot",
        name: "Godot",
        description: "Godot 4 game engine",
        packages: &["godot_4"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "kicad",
        name: "KiCad",
        description: "electronics design automation suite",
        packages: &["kicad"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "python",
        name: "Python",
        description: "Python, Ruff, and uv",
        packages: &["python3", "ruff", "uv"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "rust",
        name: "Rust",
        description: "Cargo, compiler, formatter, and language tools",
        packages: &["cargo", "clippy", "rust-analyzer", "rustc", "rustfmt"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "typescript",
        name: "TypeScript",
        description: "Node.js, TypeScript, and language server",
        packages: &["nodejs_22", "typescript", "typescript-language-server"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "zig",
        name: "Zig",
        description: "compiler and language server",
        packages: &["zig", "zls"],
        systems: SUPPORTED_SYSTEMS,
    },
];

pub fn available_for(system: &str) -> Vec<Preset> {
    PRESETS
        .iter()
        .copied()
        .filter(|preset| preset.supports(system))
        .collect()
}

pub fn resolve(ids: &[String], system: &str) -> Result<Vec<Preset>, String> {
    let mut selected = Vec::new();

    for id in ids {
        let Some(preset) = PRESETS.iter().find(|preset| preset.id == id) else {
            let available = available_for(system)
                .iter()
                .map(|preset| preset.id)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "Plainix: unknown tool `{id}`. Available tools: {available}"
            ));
        };
        if !preset.supports(system) {
            return Err(format!(
                "Plainix: tool `{id}` is not available on `{system}`"
            ));
        }
        if !selected.contains(preset) {
            selected.push(*preset);
        }
    }

    Ok(selected)
}

pub fn packages_for(presets: &[Preset]) -> Vec<&'static str> {
    let mut packages = Vec::new();

    for preset in presets {
        for package in preset.packages {
            if !packages.contains(package) {
                packages.push(package);
            }
        }
    }

    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combines_application_presets() {
        let presets = resolve(&["godot".to_owned(), "blender".to_owned()], "x86_64-linux").unwrap();
        assert_eq!(packages_for(&presets), vec!["godot_4", "blender"]);
    }

    #[test]
    fn combines_presets_without_duplicate_packages() {
        let rust = *PRESETS.iter().find(|preset| preset.id == "rust").unwrap();
        let packages = packages_for(&[rust, rust]);
        assert_eq!(packages.iter().filter(|name| **name == "rustc").count(), 1);
    }

    #[test]
    fn rejects_unknown_and_unsupported_presets() {
        assert!(resolve(&["unknown".to_owned()], "x86_64-linux").is_err());
        assert!(resolve(&["freecad".to_owned()], "aarch64-darwin").is_err());
        assert!(
            !available_for("aarch64-darwin")
                .iter()
                .any(|preset| preset.id == "freecad")
        );
    }

    #[test]
    fn empty_selection_has_no_packages() {
        assert!(packages_for(&[]).is_empty());
    }
}
