//! Project scaffold generation — combines base + plugin type + optional modules.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::config::{ProjectConfig, template_directory_stack};
use crate::template::{self, TemplateContext};

/// Generate the complete project scaffold.
///
/// # Errors
///
/// Returns an error when the output directory exists, templates are missing, or I/O fails.
#[allow(clippy::too_many_lines)]
pub fn generate(config: &ProjectConfig) -> Result<PathBuf> {
    let output_dir = std::env::current_dir()?.join(&config.name);

    if output_dir.exists() {
        anyhow::bail!(
            "Directory '{}' already exists. Choose a different name or remove it first.",
            output_dir.display()
        );
    }

    let context = TemplateContext::from_config(config);
    let template_dirs = template_directory_stack(config);

    println!("\n  {} {}", "Creating".green().bold(), config.name.bold());
    println!("  Templates: {}", template_dirs.join(" + "));

    let files = template::collect_template_files(&template_dirs);

    if files.is_empty() {
        anyhow::bail!(
            "No embedded template files found for: {}",
            template_dirs.join(", ")
        );
    }

    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create: {}", output_dir.display()))?;

    let mut count = 0;
    for (contents, rel) in &files {
        let rel_str = rel
            .to_string_lossy()
            .replace("{{ crate_name }}", &context.crate_name);
        let adjusted_rel = PathBuf::from(rel_str);
        template::write_rendered(contents, &adjusted_rel, &output_dir, &context)?;
        count += 1;
    }

    let version_marker = output_dir.join(".grafana-plugin-version");
    std::fs::write(&version_marker, env!("CARGO_PKG_VERSION"))
        .context("Failed to write version marker")?;

    if let Ok(output) = Command::new("git")
        .arg("init")
        .current_dir(&output_dir)
        .output()
        && output.status.success()
    {
        let _ = Command::new("git")
            .args(["add", "."])
            .current_dir(&output_dir)
            .output();
        let _ = Command::new("git")
            .args([
                "commit",
                "-m",
                "Initial scaffold from create-grafana-plugin",
            ])
            .current_dir(&output_dir)
            .output();
    }

    println!("  {} Generated {} files", "✓".green().bold(), count);
    println!("  {} Initialized git repository", "✓".green().bold());

    println!("\n  {}\n", "Next steps:".bold());
    println!("    cd {}", config.name);

    if config.has_wasm {
        println!("    bun run setup");
    } else {
        println!("    bun install");
    }
    println!("    bun run build");

    if config.has_docker {
        println!("    docker compose up -d");
    }

    println!("    bun run dev");
    println!();

    Ok(output_dir)
}
