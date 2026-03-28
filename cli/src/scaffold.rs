//! Project scaffold generation — combines base + plugin type + optional modules.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::config::ProjectConfig;
use crate::template::{self, TemplateContext};

fn select_template_dirs(config: &ProjectConfig) -> Vec<&'static str> {
    let mut dirs = vec!["base"];

    match config.plugin_type {
        crate::config::PluginType::Panel => dirs.push("panel"),
        crate::config::PluginType::Datasource => dirs.push("datasource"),
        crate::config::PluginType::App => dirs.push("app"),
    }

    if config.has_wasm {
        dirs.push("wasm");
    }
    if config.has_docker {
        dirs.push("docker");
    }
    if config.has_mock && config.has_docker {
        dirs.push("mock");
    }

    dirs
}

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

    let tpl_root = template::templates_root()?;
    let context = TemplateContext::from_config(config);
    let template_dirs = select_template_dirs(config);

    println!("\n  {} {}", "Creating".green().bold(), config.name.bold());
    println!("  Templates: {}", template_dirs.join(" + "));

    let files = template::collect_template_dirs(&tpl_root, &template_dirs);

    if files.is_empty() {
        anyhow::bail!("No template files found in: {}", tpl_root.display());
    }

    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create: {}", output_dir.display()))?;

    let mut count = 0;
    for (src, rel) in &files {
        let rel_str = rel
            .to_string_lossy()
            .replace("{{ crate_name }}", &context.crate_name);
        let adjusted_rel = PathBuf::from(rel_str);
        template::render_file(src, &output_dir, &adjusted_rel, &context)?;
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

    let pm = &config.package_manager;
    if config.has_wasm {
        println!("    {pm} run setup");
    } else {
        println!("    {pm} install");
    }
    println!("    {pm} run build");

    if config.has_docker {
        println!("    docker compose up -d");
    }

    println!("    {pm} run dev");
    println!();

    Ok(output_dir)
}
