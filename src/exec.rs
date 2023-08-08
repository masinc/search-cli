use std::{
    borrow::Cow,
    cell::RefCell,
    io::{self, prelude::*},
};

use crate::{
    cli::{self, Cli},
    config::{self, find_provider, Config},
};
use anyhow::Result;
use tera::Tera;

pub trait Executable {
    type Command;
    fn exec(&self, cmd: &Self::Command, config: &Config) -> Result<()>;
}

pub struct ConfigExec<Writer> {
    stdout: RefCell<Writer>,
}

impl<Writer> Executable for ConfigExec<Writer>
where
    Writer: Write,
{
    type Command = cli::CommandConfig;

    fn exec(&self, cmd: &Self::Command, _config: &Config) -> Result<()> {
        let mut stdout = self.stdout.borrow_mut();
        if cmd.path {
            write!(stdout, "{}", config::config_path().to_str().unwrap())?;
        }

        Ok(())
    }
}

impl ConfigExec<io::Stdout> {
    pub fn new() -> Self {
        Self {
            stdout: RefCell::new(io::stdout()),
        }
    }
}

impl Default for ConfigExec<io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OpenExec;

impl Executable for OpenExec {
    type Command = cli::CommandOpen;

    fn exec(&self, cmd: &Self::Command, config: &Config) -> Result<()> {
        if config.providers.is_empty() {
            panic!("Providers is not found.")
        }

        let provider = match &cmd.provider {
            Some(name) => match find_provider(&config.providers, name) {
                Some(p) => p,
                None => {
                    eprintln!("The Provider does not exists: '{name}'");
                    std::process::exit(1);
                }
            },

            None => &config.providers[0],
        };

        let word: Cow<str> = if cmd.word == "-" {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Cow::Owned(buf.trim().to_string())
        } else {
            Cow::Borrowed(&cmd.word)
        };

        let url = Self::replace_url(&provider.url, &word)?;

        match &provider.browser {
            None => open::that(url)?,
            Some(path) => open::with(url, path)?,
        }

        Ok(())
    }
}

impl OpenExec {
    fn replace_url(url: &str, word: &str) -> Result<String> {
        let mut tera = Tera::default();
        tera.add_raw_template("url", url)?;

        let mut ctx = ::tera::Context::new();
        ctx.insert("word", word);

        Ok(tera.render("url", &ctx)?)
    }
}

pub struct CompletionExec<Writer> {
    stdout: RefCell<Writer>,
}

impl<Writer> Executable for CompletionExec<Writer>
where
    Writer: Write,
{
    type Command = cli::CommandCompletion;

    fn exec(&self, cmd: &Self::Command, _config: &Config) -> Result<()> {
        use clap::CommandFactory;

        let mut stdout = self.stdout.borrow_mut();
        clap_complete::generate(cmd.shell, &mut Cli::command(), "search", stdout.by_ref());

        Ok(())
    }
}

impl CompletionExec<io::Stdout> {
    pub fn new() -> Self {
        Self {
            stdout: RefCell::new(io::stdout()),
        }
    }
}

impl Default for CompletionExec<io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JsonschemaExec {
    stdout: RefCell<io::Stdout>,
}

impl Executable for JsonschemaExec {
    type Command = ();

    fn exec(&self, _cmd: &Self::Command, _config: &Config) -> Result<()> {
        let schema = schemars::schema_for!(config::Config);
        let mut stdout = self.stdout.borrow_mut();
        write!(stdout, "{}", serde_json::to_string_pretty(&schema).unwrap())?;
        Ok(())
    }
}

impl JsonschemaExec {
    pub fn new() -> Self {
        Self {
            stdout: RefCell::new(io::stdout()),
        }
    }
}

impl Default for JsonschemaExec {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ListExec<Writer> {
    stdout: RefCell<Writer>,
}

impl<Writer> Executable for ListExec<Writer>
where
    Writer: Write,
{
    type Command = cli::CommandList;

    fn exec(&self, cmd: &Self::Command, config: &Config) -> Result<()> {
        let mut stdout = self.stdout.borrow_mut();
        for provider in &config.providers {
            if cmd.verbose {
                let aliases = provider.aliases.clone().unwrap_or_default();
                writeln!(
                    stdout,
                    "{:20} alias: [{}]",
                    provider.name,
                    aliases.join(", ")
                )?;
            } else {
                writeln!(stdout, "{}", provider.name)?;
            }
        }

        Ok(())
    }
}

impl ListExec<io::Stdout> {
    pub fn new() -> Self {
        Self {
            stdout: RefCell::new(io::stdout()),
        }
    }
}

impl Default for ListExec<io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ExternalExec;

impl Executable for ExternalExec {
    type Command = Vec<String>;

    fn exec(&self, cmd: &Self::Command, config: &Config) -> Result<()> {
        if cmd.is_empty() || cmd.len() > 2 {
            eprintln!("Usage: search [PROVIDER] WORD");
            std::process::exit(1);
        }

        let open_cmd = if cmd.len() == 1 {
            cli::CommandOpen {
                provider: None,
                word: cmd[0].clone(),
            }
        } else if cmd.len() == 2 {
            cli::CommandOpen {
                provider: Some(cmd[0].clone()),
                word: cmd[1].clone(),
            }
        } else {
            unreachable!()
        };

        OpenExec.exec(&open_cmd, config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_config_exec() -> Result<()> {
        let cmd = cli::CommandConfig { path: true };
        let config = Config {
            version: "v1.0".to_string(),
            providers: vec![],
        };

        let stdout = Cursor::new(vec![]);
        let config_exec = ConfigExec {
            stdout: RefCell::new(stdout),
        };

        config_exec.exec(&cmd, &config)?;

        let stdout = config_exec.stdout.into_inner();
        assert_eq!(
            String::from_utf8(stdout.into_inner()).unwrap(),
            config::config_path().to_str().unwrap()
        );

        Ok(())
    }
    #[test]
    fn test_open_exec_replace_url() {
        let search_url = "https://google.com/search?q={{ word | urlencode }}";

        assert_eq!(
            OpenExec::replace_url(search_url, "aaa").unwrap(),
            "https://google.com/search?q=aaa".to_string()
        );

        assert_eq!(
            OpenExec::replace_url(search_url, "aaa bbb").unwrap(),
            "https://google.com/search?q=aaa%20bbb".to_string()
        )
    }

    #[test]
    fn test_jsonschema_exec() -> Result<()> {
        JsonschemaExec::default().exec(&(), &config::default_config_file()?)?;
        Ok(())
    }

    #[test]
    fn test_list_exec() -> Result<()> {
        let cmd = cli::CommandList { verbose: false };
        let config = Config {
            version: "v1.0".to_string(),
            providers: vec![
                config::Provider {
                    name: "google".to_string(),
                    aliases: Some(vec!["g".to_string()]),
                    url: "https://google.com/search?q={{ word | urlencode }}".to_string(),
                    browser: None,
                },
                config::Provider {
                    name: "bing".to_string(),
                    aliases: None,
                    url: "https://www.bing.com/search?q={{ word | urlencode }}".to_string(),
                    browser: None,
                },
                config::Provider {
                    name: "duckduckgo".to_string(),
                    aliases: Some(vec!["d".to_string()]),
                    url: "https://duckduckgo.com/?q={{ word | urlencode }}".to_string(),
                    browser: None,
                },
            ],
        };

        let stdout = Cursor::new(vec![]);
        let list_exec = ListExec {
            stdout: RefCell::new(stdout),
        };

        list_exec.exec(&cmd, &config)?;

        let stdout = list_exec.stdout.into_inner();
        assert_eq!(
            String::from_utf8(stdout.into_inner()).unwrap(),
            "google\nbing\nduckduckgo\n"
        );

        Ok(())
    }

    #[test]
    fn test_list_exec_verbose() -> Result<()> {
        let cmd = cli::CommandList { verbose: true };
        let config = Config {
            version: "v1.0".to_string(),
            providers: vec![
                config::Provider {
                    name: "google".to_string(),
                    aliases: Some(vec!["g".to_string()]),
                    url: "https://google.com/search?q={{ word | urlencode }}".to_string(),
                    browser: None,
                },
                config::Provider {
                    name: "bing".to_string(),
                    aliases: None,
                    url: "https://www.bing.com/search?q={{ word | urlencode }}".to_string(),
                    browser: None,
                },
                config::Provider {
                    name: "duckduckgo".to_string(),
                    aliases: Some(vec!["d".to_string()]),
                    url: "https://duckduckgo.com/?q={{ word | urlencode }}".to_string(),
                    browser: None,
                },
            ],
        };

        let stdout = Cursor::new(vec![]);
        let list_exec = ListExec {
            stdout: RefCell::new(stdout),
        };

        list_exec.exec(&cmd, &config)?;

        let stdout = list_exec.stdout.into_inner();

        let str = String::from_utf8(stdout.into_inner()).unwrap();
        let lines = str.lines().collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert!(regex::Regex::new(r"google\s+alias: \[g\]")
            .unwrap()
            .is_match(lines[0]));
        assert!(regex::Regex::new(r"bing\s+alias: \[\]")
            .unwrap()
            .is_match(lines[1]));
        assert!(regex::Regex::new(r"duckduckgo\s+alias: \[d\]")
            .unwrap()
            .is_match(lines[2]));

        Ok(())
    }
}
