const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

pub fn version() -> String {
    format!("{NAME}: {VERSION}")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn includes_package_name_and_version() {
        assert_eq!(
            version(),
            format!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        );
    }
}
