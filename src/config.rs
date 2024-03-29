use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CONFIG_FILE: &str = "config.yaml";

pub fn config_dir() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".config");
    dir.push("search");

    dir
}

pub fn config_path() -> PathBuf {
    let mut path = config_dir();
    path.push(CONFIG_FILE);
    path
}

pub fn default_config_file() -> Result<Config, serde_yaml::Error> {
    let yaml = r#"
version: "v1.0"
providers:
  - name: google
    aliases:
      - g
    url: "https://google.com/search?q={{ word | urlencode }}"
  - name: bing
    url: "https://www.bing.com/search?q={{ word | urlencode }}"
  - name: duckduckgo
    aliases:
      - d
    url: "https://duckduckgo.com/?q={{ word | urlencode }}"
"#;

    serde_yaml::from_str(yaml)
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Config {
    pub version: String,
    pub providers: Vec<Provider>,
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Provider {
    /// The name of the provider
    pub name: String,
    /// The name aliases
    pub aliases: Option<Vec<String>>,
    /// The URL of the provider
    pub url: String,
    /// Open browser for the provider
    pub browser: Option<String>,
}

pub fn find_provider<'a>(providers: &'a [Provider], name: &str) -> Option<&'a Provider> {
    for provider in providers.iter() {
        if provider.name == name {
            return Some(provider);
        }

        if let Some(aliases) = &provider.aliases {
            if aliases.iter().any(|alias| alias == name) {
                return Some(provider);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_provider() {
        let name = "google";
        assert_eq!(find_provider(&[], name), None);

        let providers = [Provider {
            name: "google".into(),
            ..Default::default()
        }];

        assert_eq!(
            find_provider(&providers, name),
            Some(&Provider {
                name: "google".into(),
                ..Default::default()
            })
        );
    }
}
