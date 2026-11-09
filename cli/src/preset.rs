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
        description: "compiler, build tools, language server, formatter, and linter",
        packages: &[
            "clang-tools",
            "cmake",
            "gnumake",
            "ninja",
            "pkg-config",
            "stdenv.cc",
        ],
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
        description: "compiler, language server, formatter, and linter",
        packages: &["go", "golangci-lint", "gopls"],
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
        id: "haskell",
        name: "Haskell",
        description: "GHC, Cabal, language server, formatter, and linter",
        packages: &[
            "cabal-install",
            "ghc",
            "haskell-language-server",
            "hlint",
            "ormolu",
        ],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "kicad",
        name: "KiCad",
        description: "electronics design automation suite",
        packages: &["kicad"],
        systems: LINUX_SYSTEMS,
    },
    Preset {
        id: "lua",
        name: "Lua",
        description: "interpreter, language server, formatter, and linter",
        packages: &[
            "lua",
            "lua-language-server",
            "luaPackages.luacheck",
            "stylua",
        ],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "nix",
        name: "Nix",
        description: "language server, formatter, and static analysis tools",
        packages: &["deadnix", "nixd", "nixfmt", "statix"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "ocaml",
        name: "OCaml",
        description: "compiler, Dune, language server, and formatter",
        packages: &["dune_3", "ocaml", "ocamlPackages.ocaml-lsp", "ocamlformat"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "python",
        name: "Python",
        description: "Python, language server, Ruff, and uv",
        packages: &["basedpyright", "python3", "ruff", "uv"],
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
        id: "shell",
        name: "Shell",
        description: "Bash, language server, formatter, and linter",
        packages: &[
            "bashInteractive",
            "bash-language-server",
            "shellcheck",
            "shfmt",
        ],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "toml",
        name: "TOML",
        description: "language server, formatter, and validator",
        packages: &["taplo"],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "typescript",
        name: "JavaScript / TypeScript",
        description: "Node.js, TypeScript, language server, formatter, and linter",
        packages: &[
            "biome",
            "nodejs_22",
            "typescript",
            "typescript-language-server",
        ],
        systems: SUPPORTED_SYSTEMS,
    },
    Preset {
        id: "yaml",
        name: "YAML",
        description: "language server, formatter, and linter",
        packages: &["prettier", "yaml-language-server", "yamllint"],
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
        let first = Preset {
            id: "first",
            name: "First",
            description: "test preset",
            packages: &["shared", "first"],
            systems: SUPPORTED_SYSTEMS,
        };
        let second = Preset {
            id: "second",
            name: "Second",
            description: "test preset",
            packages: &["shared", "second"],
            systems: SUPPORTED_SYSTEMS,
        };

        assert_eq!(
            packages_for(&[first, second, first]),
            vec!["shared", "first", "second"]
        );
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

    fn json_strings(values: &[&str]) -> String {
        let quoted = values
            .iter()
            .map(|value| format!("\"{value}\""))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{quoted}]")
    }

    // `nix flake check` reads this fixture to evaluate presets, so it must not drift.
    #[test]
    fn evaluation_fixture_matches_presets() {
        let entries = PRESETS
            .iter()
            .map(|preset| {
                format!(
                    "{{\"id\":\"{}\",\"packages\":{},\"systems\":{}}}",
                    preset.id,
                    json_strings(preset.packages),
                    json_strings(preset.systems)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");

        assert_eq!(
            include_str!("../../tests/presets.json"),
            format!("[\n{entries}\n]\n"),
            "tests/presets.json must mirror PRESETS"
        );
    }

    #[test]
    fn empty_selection_has_no_packages() {
        assert!(packages_for(&[]).is_empty());
    }
}
