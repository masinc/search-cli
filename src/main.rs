mod cli;
mod config;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, SubCommand};
use config::Config;
use std::fs::{self, DirBuilder};
use std::io::{self, BufReader, BufWriter};
use std::process::exit;
use tera::Tera;

use crate::config::find_provider;

fn create_default_config_file() -> Result<()> {
    let f = fs::File::create(config::config_path())?;
    let f = BufWriter::new(f);
    serde_yaml::to_writer(f, &config::default_config_file()?)?;

    Ok(())
}

fn load_config_file() -> Result<Config> {
    let f = fs::File::open(config::config_path())?;
    let f = BufReader::new(f);

    Ok(serde_yaml::from_reader(f)?)
}

fn init_config() -> Result<()> {
    // if not exists config_dir then create config_dir
    let config_dir = config::config_dir();
    if !config_dir.is_dir() {
        DirBuilder::new().recursive(true).create(&config_dir)?
    }

    let config_path = config::config_path();

    // if not exists config_path then create default config file
    if !config_path.is_file() {
        create_default_config_file()?;
    };

    Ok(())
}

fn replace_url(url: &str, word: &str) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("url", url)?;

    let mut ctx = ::tera::Context::new();
    ctx.insert("word", word);

    Ok(tera.render("url", &ctx)?)
}

fn exec_config(cmd: cli::CommandConfig, _config: Config) -> anyhow::Result<()> {
    if cmd.path {
        println!("{}", config::config_path().to_str().unwrap());
    }

    Ok(())
}

fn exec_open(cmd: cli::CommandOpen, config: Config) -> anyhow::Result<()> {
    if config.providers.is_empty() {
        panic!("Providers is not found.")
    }

    let provider = match cmd.provider {
        Some(name) => match find_provider(&config.providers, name.clone()) {
            Some(p) => p,
            None => {
                eprintln!("The Provider does not exists: '{name}'");
                exit(1);
            }
        },

        None => &config.providers[0],
    };

    let url = replace_url(&provider.url, &cmd.word)?;
    open::that(url)?;

    Ok(())
}

fn exec_list(cmd: cli::CommandList, config: Config) -> anyhow::Result<()> {
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

fn exec_complition(cmd: cli::CommandCompletion, _config: Config) -> anyhow::Result<()> {
    use clap::CommandFactory;

    clap_complete::generate(cmd.shell, &mut Cli::command(), "search", &mut io::stdout());

    Ok(())
}

fn exec_external(cmd: Vec<String>, config: Config) -> anyhow::Result<()> {
    if cmd.is_empty() || cmd.len() > 2 {
        eprintln!("Usage: search [PROVIDER] WORD");
        exit(1);
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

    exec_open(cmd, config)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_config()?;
    let config = load_config_file()?;

    match cli.subcommand {
        SubCommand::Config(cmd) => exec_config(cmd, config)?,
        SubCommand::Open(cmd) => exec_open(cmd, config)?,
        SubCommand::Complition(cmd) => exec_complition(cmd, config)?,
        SubCommand::List(cmd) => exec_list(cmd, config)?,
        SubCommand::External(v) => exec_external(v, config)?,
    };

    Ok(())
}
