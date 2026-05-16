use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "0".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: default_version(),
        }
    }
}

pub fn default_config_contents() -> String {
    let mut contents = serde_json::to_string_pretty(&Config::default())
        .expect("default Config should always serialize");
    contents.push('\n');
    contents
}

#[cfg(test)]
mod tests {
    use super::{Config, default_config_contents, default_version};

    #[test]
    fn default_has_empty_name_and_version_zero() {
        let config = Config::default();
        assert_eq!(config.name, "");
        assert_eq!(config.version, "0");
    }

    #[test]
    fn default_contents_round_trip() {
        let contents = default_config_contents();
        let parsed: Config = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed, Config::default());
    }

    #[test]
    fn empty_object_uses_defaults() {
        let parsed: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.name, "");
        assert_eq!(parsed.version, default_version());
    }
}
