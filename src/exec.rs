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

pub fn config(cmd: cli::CommandConfig, _config: Config) -> anyhow::Result<()> {
    if cmd.path {
        println!("{}", config::config_path().to_str().unwrap());
    }

    Ok(())
}

pub fn open(cmd: cli::CommandOpen, config: Config) -> anyhow::Result<()> {
    if config.providers.is_empty() {
        panic!("Providers is not found.")
    }

    let provider = match cmd.provider {
        Some(name) => match find_provider(&config.providers, &name) {
            Some(p) => p,
            None => {
                eprintln!("The Provider does not exists: '{name}'");
                std::process::exit(1);
            }
        },

        None => &config.providers[0],
    };

    let url = replace_url(&provider.url, &cmd.word)?;

    let browser = {
        use config::Browser;
        match &provider.browser {
            Browser::DefaultConfig => match &config.default {
                Some(default) => match &default.browser {
                    Some(default_browser) => Browser::Browser(default_browser.clone()),
                    None => Browser::Default,
                },
                None => Browser::Default,
            },
            b => b.clone(),
        }
    };

    match browser {
        config::Browser::Default => open::that(url)?,
        config::Browser::DefaultConfig => {
            match config.default.and_then(|default| default.browser) {
                Some(browser) => open::with(url, browser)?,
                None => open::that(url)?,
            }
        }
        config::Browser::Browser(browser) => open::with(url, browser)?,
    }

    Ok(())
}

pub fn list(cmd: cli::CommandList, config: Config) -> anyhow::Result<()> {
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

pub fn completion(cmd: cli::CommandCompletion, _config: Config) -> anyhow::Result<()> {
    use clap::CommandFactory;

    clap_complete::generate(cmd.shell, &mut Cli::command(), "search", &mut io::stdout());

    Ok(())
}

pub fn external(cmd: Vec<String>, config: Config) -> anyhow::Result<()> {
    if cmd.is_empty() || cmd.len() > 2 {
        eprintln!("Usage: search [PROVIDER] WORD");
        std::process::exit(1);
    }

    let cmd = if cmd.len() == 1 {
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

    open(cmd, config)
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
