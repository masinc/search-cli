use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, 
    propagate_version = true, arg_required_else_help = true,
    after_help = "\
EXAMPLES:
    search google <word> // search word
    search google - // search word from stdin
    search g <word> // alias
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
    Completion(CommandCompletion),
    #[clap(about = "Show config.yaml schema")]
    Jsonschema,
    #[clap(external_subcommand)]
    External(Vec<String>),
}

#[derive(Parser, Debug)]
#[clap(arg_required_else_help = true, about = "Configuration")]
pub struct CommandConfig {
    /// If specified, outputs the config file path.
    #[clap(long, short, help = "Print config file path")]
    pub path: bool,
}

#[derive(Parser, Debug)]
#[clap(about = "List search providers")]
pub struct CommandList {
    /// Verbose mode
    #[clap(long, short)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
#[clap(arg_required_else_help = true, about = "Search words")]
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
    about = "Generate completion scripts",
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
