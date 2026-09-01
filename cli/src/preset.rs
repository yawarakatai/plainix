use crate::detect::ProjectKind;

pub fn packages_for(kinds: &[ProjectKind]) -> Vec<&'static str> {
    let mut packages = Vec::new();

    for kind in kinds {
        let preset: &[&str] = match kind {
            ProjectKind::Rust => &["cargo", "clippy", "rust-analyzer", "rustc", "rustfmt"],
            ProjectKind::Python => &["python3", "ruff", "uv"],
            ProjectKind::TypeScript => &["nodejs_22", "typescript", "typescript-language-server"],
            ProjectKind::Go => &["go", "gopls"],
            ProjectKind::Zig => &["zig", "zls"],
            ProjectKind::Cpp => &["cmake", "gnumake", "ninja", "pkg-config", "stdenv.cc"],
        };

        for package in preset {
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
    fn combines_presets_without_duplicates() {
        let packages = packages_for(&[ProjectKind::Rust, ProjectKind::TypeScript]);
        assert!(packages.contains(&"rustc"));
        assert!(packages.contains(&"nodejs_22"));
        assert_eq!(packages.iter().filter(|name| **name == "rustc").count(), 1);
    }

    #[test]
    fn additional_languages_get_their_tools() {
        let packages = packages_for(&[ProjectKind::Go, ProjectKind::Zig, ProjectKind::Cpp]);
        assert!(packages.contains(&"go"));
        assert!(packages.contains(&"gopls"));
        assert!(packages.contains(&"zig"));
        assert!(packages.contains(&"zls"));
        assert!(packages.contains(&"stdenv.cc"));
    }

    #[test]
    fn project_without_presets_gets_no_packages() {
        assert!(packages_for(&[]).is_empty());
    }
}
