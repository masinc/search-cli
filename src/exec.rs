use std::io;

use crate::{
    cli::{self, Cli},
    config::{self, find_provider, Config},
};
use anyhow::Result;
use tera::Tera;

fn replace_url(url: &str, word: &str) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("url", url)?;

    let mut ctx = ::tera::Context::new();
    ctx.insert("word", word);

    Ok(tera.render("url", &ctx)?)
}

pub trait Executable {
    type Command;
    fn exec(cmd: &Self::Command, config: &Config) -> Result<()>;
}

pub struct ConfigExec;

impl Executable for ConfigExec {
    type Command = cli::CommandConfig;

    fn exec(cmd: &Self::Command, _config: &Config) -> Result<()> {
        if cmd.path {
            println!("{}", config::config_path().to_str().unwrap());
        }

        Ok(())
    }
}

pub struct OpenExec;

impl Executable for OpenExec {
    type Command = cli::CommandOpen;

    fn exec(cmd: &Self::Command, config: &Config) -> Result<()> {
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

        let url = replace_url(&provider.url, &cmd.word)?;

        match &provider.browser {
            None => open::that(url)?,
            Some(path) => open::with(url, path)?,
        }

        Ok(())
    }
}

pub struct CompletionExec;

impl Executable for CompletionExec {
    type Command = cli::CommandCompletion;

    fn exec(cmd: &Self::Command, _config: &Config) -> Result<()> {
        use clap::CommandFactory;

        clap_complete::generate(cmd.shell, &mut Cli::command(), "search", &mut io::stdout());

        Ok(())
    }
}

pub struct JsonschemaExec;

impl Executable for JsonschemaExec {
    type Command = ();

    fn exec(_cmd: &Self::Command, _config: &Config) -> Result<()> {
        let schema = schemars::schema_for!(config::Config);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }
}

pub struct ListExec;

impl Executable for ListExec {
    type Command = cli::CommandList;

    fn exec(cmd: &Self::Command, config: &Config) -> Result<()> {
        for provider in &config.providers {
            if cmd.verbose {
                let aliases = provider.aliases.clone().unwrap_or_default();
                println!("{:20} alias: [{}]", provider.name, aliases.join(", "));
            } else {
                println!("{}", provider.name);
            }
        }

        Ok(())
    }
}

pub struct ExternalExec;

impl Executable for ExternalExec {
    type Command = Vec<String>;

    fn exec(cmd: &Self::Command, _config: &Config) -> Result<()> {
        if cmd.is_empty() || cmd.len() > 2 {
            eprintln!("Usage: search [PROVIDER] WORD");
            std::process::exit(1);
        }

        if cmd.len() == 1 {
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_url() {
        let search_url = "https://google.com/search?q={{ word | urlencode }}";

        assert_eq!(
            replace_url(search_url, "aaa").unwrap(),
            "https://google.com/search?q=aaa".to_string()
        );

        assert_eq!(
            replace_url(search_url, "aaa bbb").unwrap(),
            "https://google.com/search?q=aaa%20bbb".to_string()
        )
    }
}
