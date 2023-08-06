mod cli;
mod config;
mod exec;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, SubCommand};
use config::Config;
use exec::{
    CompletionExec, ConfigExec, Executable, ExternalExec, JsonschemaExec, ListExec, OpenExec,
};
use std::fs::{self, DirBuilder};
use std::io::{BufReader, BufWriter};

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_config()?;
    let config = load_config_file()?;

    match cli.subcommand {
        SubCommand::Config(cmd) => ConfigExec::default().exec(&cmd, &config)?,
        SubCommand::Open(cmd) => OpenExec.exec(&cmd, &config)?,
        SubCommand::Completion(cmd) => CompletionExec::default().exec(&cmd, &config)?,
        SubCommand::Jsonschema => JsonschemaExec::default().exec(&(), &config)?,
        SubCommand::List(cmd) => ListExec::default().exec(&cmd, &config)?,
        SubCommand::External(v) => ExternalExec.exec(&v, &config)?,
    };

    Ok(())
}
