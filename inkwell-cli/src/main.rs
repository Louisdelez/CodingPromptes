use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod project;
mod sdd;

#[derive(Parser)]
#[command(name = "inkwell", version, about = "Inkwell — Spec-Driven Development CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Inkwell project
    Init {
        /// Project name
        name: Option<String>,
        /// Initialize in current directory
        #[arg(long)]
        here: bool,
    },
    /// Create a project constitution
    Constitution {
        /// Description of project principles
        description: Option<String>,
    },
    /// Create a feature specification
    Specify {
        /// Feature description
        description: String,
    },
    /// Create an implementation plan
    Plan {
        /// Tech stack (e.g. "React, PostgreSQL")
        tech: Option<String>,
    },
    /// Generate task list from plan
    Tasks,
    /// Clarify specification requirements
    Clarify {
        /// Additional details
        details: Option<String>,
    },
    /// Execute tasks (autopilot)
    Implement,
    /// Generate quality checklist
    Checklist,
    /// Audit implementation plan
    Analyze,
    /// Validate all SDD phases
    Validate,
    /// List projects
    List,
    /// Show project status
    Status,
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
        Commands::Help => commands::help(),
    }
}
