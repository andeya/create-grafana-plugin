use clap::Parser;

/// CLI tool to scaffold production-ready Grafana plugin projects.
#[derive(Parser, Debug)]
#[command(name = "create-grafana-plugin", version, about)]
pub struct Args {
    /// Plugin name (kebab-case)
    #[arg(long)]
    pub name: Option<String>,

    /// Plugin description
    #[arg(long)]
    pub description: Option<String>,

    /// Author name
    #[arg(long)]
    pub author: Option<String>,

    /// Grafana organization name
    #[arg(long)]
    pub org: Option<String>,

    /// Plugin type: panel, datasource, app
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,

    /// Include Rust WASM engine
    #[arg(long)]
    pub wasm: bool,

    /// Include Docker dev environment
    #[arg(long)]
    pub docker: bool,

    /// Include mock data generator (requires --docker)
    #[arg(long)]
    pub mock: bool,

    /// Package manager: bun, npm, pnpm, yarn
    #[arg(long, default_value = "bun")]
    pub pm: String,

    /// Read config from file
    #[arg(long, value_name = "FILE")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Subcommands
#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Update an existing project to the latest template
    Update {
        /// Show changes without applying
        #[arg(long)]
        dry_run: bool,
    },
}

/// Parse CLI arguments.
pub fn parse() -> Args {
    Args::parse()
}
