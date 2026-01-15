use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skillset")]
#[command(about = "A package manager for coding agent skills")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new skill from a source
    Add {
        /// Skill reference (name, git URL, OCI reference, or local path)
        /// Examples: file-analyzer@1.0.0, @user/skill@2.0.0, git:https://github.com/user/repo
        reference: String,
        /// Override convention detection
        #[arg(long, short)]
        convention: Option<String>,
        /// Override version (for use with explicit source references)
        #[arg(long, short)]
        version: Option<String>,
    },
    /// Remove an installed skill
    Remove {
        /// Skill name to remove
        name: String,
    },
    /// List installed skills
    List {
        /// Show detailed information
        #[arg(long, short)]
        verbose: bool,
    },
    /// Update skills to latest versions
    Update {
        /// Specific skill to update (optional)
        name: Option<String>,
    },
    /// Get information about a skill
    Info {
        /// Skill name
        name: String,
    },
    /// Manage agent conventions
    Convention {
        #[command(subcommand)]
        command: ConventionCommands,
    },
    /// OCI registry operations
    Publish {
        /// Local skill path to publish
        path: String,
        /// Target OCI reference
        reference: String,
        /// Registry to publish to
        #[arg(long, short)]
        registry: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConventionCommands {
    /// List available conventions
    List,
    /// Enable a convention
    Enable { name: String },
    /// Disable a convention
    Disable { name: String },
    /// Configure a convention
    Configure { name: String },
}

// Export enums for use in other modules

mod args;
mod commands;

pub async fn handle_command(cli: Cli) -> crate::error::Result<()> {
    match cli.command {
        Commands::Add {
            reference,
            convention,
            version,
        } => commands::handle_add(reference, convention, version).await,
        Commands::Remove { name } => commands::handle_remove(name).await,
        Commands::List { verbose } => commands::handle_list(verbose).await,
        Commands::Update { name } => commands::handle_update(name).await,
        Commands::Info { name } => commands::handle_info(name).await,
        Commands::Convention { command } => commands::handle_convention(command).await,
        Commands::Publish {
            path,
            reference,
            registry,
        } => commands::handle_publish(path, reference, registry).await,
    }
}
