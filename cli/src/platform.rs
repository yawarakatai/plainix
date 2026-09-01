pub const SUPPORTED_SYSTEMS: &[&str] = &["x86_64-linux", "aarch64-linux", "aarch64-darwin"];

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
