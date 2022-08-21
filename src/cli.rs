use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, 
    propagate_version = true, arg_required_else_help = true,
    after_help = "\
EXAMPLES:
    search google searchword
    search g searchword // alias
    search config -p // print config file path
    search list // list providers    
"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Config(CommandConfig),
    List(CommandList),
    Open(CommandOpen),
    Complition(CommandCompletion),
    #[clap(external_subcommand)]
    External(Vec<String>),
}

#[derive(Parser, Debug)]
#[clap(arg_required_else_help = true)]
pub struct CommandConfig {
    /// If specified, outputs the config file path.
    #[clap(long, short)]
    pub path: bool,
}

#[derive(Parser, Debug)]
pub struct CommandList {
    #[clap(long, short)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
#[clap(arg_required_else_help = true)]
pub struct CommandOpen {
    /// Specify a search provider.
    #[clap(long, short)]
    pub provider: Option<String>,

    /// Specify a search words
    pub word: String,
}

#[derive(Parser, Debug)]
#[clap(
    arg_required_else_help = true,
    after_help = "\
EXAMPLES:
    search completion bash
    search completion powershell
    search completion fish
    search completion zsh
"
)]
pub struct CommandCompletion {
    /// Specify a shell value    
    pub shell: clap_complete::Shell,
}
