use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Config {
    pub version: String,
    pub providers: Vec<Provider>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Provider {
    /// The name of the provider
    pub name: String,
    /// The name aliases
    pub aliases: Option<Vec<String>>,
    /// The URL of the provider
    pub url: String,
}

pub fn find_provider(providers: &[Provider], name: String) -> Option<&Provider> {
    for provider in providers.iter() {
        if provider.name == name {
            return Some(provider);
        }

        if let Some(aliases) = &provider.aliases {
            if aliases.iter().any(|alias| alias == &name) {
                return Some(provider);
            }
        }
    }

    None
}
