use anyhow::Result;

use create_grafana_plugin::{cli, config, scaffold, updater};

fn main() -> Result<()> {
    let args = cli::parse();
    match args.command {
        Some(cli::Command::Update { dry_run }) => {
            updater::update(dry_run)?;
        }
        None => {
            let cfg = config::resolve_config(&args)?;
            scaffold::generate(&cfg)?;
        }
    }
    Ok(())
}
