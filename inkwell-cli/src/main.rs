use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

mod commands;
mod project;
mod sdd;
mod chat;

#[derive(Parser)]
#[command(name = "inkwell", version, about = "Inkwell — Spec-Driven Development CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Inkwell project
    Init { name: Option<String>, #[arg(long)] here: bool },
    /// Create a project constitution
    Constitution { description: Option<String> },
    /// Create a feature specification
    Specify { description: String },
    /// Create an implementation plan
    Plan { tech: Option<String> },
    /// Generate task list from plan
    Tasks,
    /// Clarify specification requirements
    Clarify { details: Option<String> },
    /// Execute tasks (autopilot)
    Implement,
    /// Generate quality checklist
    Checklist,
    /// Audit implementation plan
    Analyze,
    /// Validate all SDD phases (offline)
    Validate,
    /// List projects
    List,
    /// Show project status
    Status,
    /// Configure API keys and model
    Config { action: Option<String>, key: Option<String>, value: Option<String> },
    /// Install MCP server for Claude Code
    McpInstall,
    /// Install DevTools MCP for live GPUI app control
    DevtoolsInstall,
    /// Interactive chat with AI
    Chat,
    /// Generate shell completions (bash, zsh, fish)
    Completions { shell: String },
    /// Show available commands
    Help,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, here } => commands::init(name, here),
        Commands::Constitution { description } => sdd::constitution(description).await,
        Commands::Specify { description } => sdd::specify(&description).await,
        Commands::Plan { tech } => sdd::plan(tech).await,
        Commands::Tasks => sdd::tasks().await,
        Commands::Clarify { details } => sdd::clarify(details).await,
        Commands::Implement => sdd::implement().await,
        Commands::Checklist => sdd::checklist().await,
        Commands::Analyze => sdd::analyze().await,
        Commands::Validate => sdd::validate(),
        Commands::List => commands::list(),
        Commands::Status => commands::status(),
        Commands::Config { action, key, value } => commands::config(action, key, value),
        Commands::McpInstall => commands::mcp_install(),
        Commands::DevtoolsInstall => commands::devtools_install(),
        Commands::Chat => chat::run().await,
        Commands::Help => commands::help(),
        Commands::Completions { shell } => {
            let shell: Shell = shell.parse().unwrap_or_else(|_| {
                eprintln!("Shell invalide: {}. Options: bash, zsh, fish, elvish, powershell", shell);
                std::process::exit(1);
            });
            let mut cmd = Cli::command().disable_help_subcommand(true);
            generate(shell, &mut cmd, "inkwell", &mut std::io::stdout());
        }
    }
}
