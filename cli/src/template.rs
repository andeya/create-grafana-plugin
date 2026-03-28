//! Tera helpers for scaffold generation (API reserved until scaffold is wired).
#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tera::Tera;
use walkdir::WalkDir;

use crate::config::ProjectConfig;

/// Resolve the `templates/` directory next to the binary or from the workspace (dev).
///
/// # Errors
///
/// Returns an error when no templates directory can be located.
pub fn templates_root() -> Result<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));

    if let Some(ref dir) = exe_dir {
        let candidate = dir.join("templates");
        if candidate.exists() {
            return Ok(candidate);
        }
        if let Some(c) = dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("templates"))
            && c.exists()
        {
            return Ok(c);
        }
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let candidate = workspace_root.join("templates");
    if candidate.exists() {
        return Ok(candidate);
    }

    anyhow::bail!("Could not find templates directory")
}

/// Render a template file to bytes (same rules as [`render_file`] but without writing).
///
/// # Errors
///
/// Returns an error when the template cannot be read or rendered.
pub fn render_template_to_bytes(
    src: &Path,
    rel_path: &Path,
    context: &TemplateContext,
) -> Result<Vec<u8>> {
    if is_binary_file(src) || rel_path.extension().and_then(|e| e.to_str()) != Some("tera") {
        std::fs::read(src).with_context(|| format!("Failed to read file: {}", src.display()))
    } else {
        let template_body = std::fs::read_to_string(src)
            .with_context(|| format!("Failed to read template: {}", src.display()))?;
        let rendered = render_string(&template_body, context)?;
        Ok(rendered.into_bytes())
    }
}

/// Template rendering context with user config + computed fields
#[derive(Debug, Serialize)]
pub struct TemplateContext {
    // User config
    pub plugin_name: String,
    pub plugin_description: String,
    pub author: String,
    pub org: String,
    pub plugin_type: String,
    pub has_wasm: bool,
    pub has_docker: bool,
    pub has_mock: bool,
    pub package_manager: String,

    // Computed fields
    pub plugin_id: String,
    pub crate_name: String,
    pub current_year: String,
    pub today: String,
    pub pascal_case_name: String,
}

impl TemplateContext {
    /// Build context from project config with computed fields
    pub fn from_config(config: &ProjectConfig) -> Self {
        let plugin_id = format!("{}-{}", config.org, config.name);
        let crate_name = config.name.replace('-', "_");
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let year = chrono::Utc::now().format("%Y").to_string();

        let pascal_case_name = config
            .name
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        Self {
            plugin_name: config.name.clone(),
            plugin_description: config.description.clone(),
            author: config.author.clone(),
            org: config.org.clone(),
            plugin_type: config.plugin_type.to_string(),
            has_wasm: config.has_wasm,
            has_docker: config.has_docker,
            has_mock: config.has_mock,
            package_manager: config.package_manager.to_string(),
            plugin_id,
            crate_name,
            current_year: year,
            today,
            pascal_case_name,
        }
    }

    /// Prefer `plugin.json` `info.updated` and derived year so updates do not churn dates.
    pub fn apply_dates_from_existing_plugin_json(&mut self, project_dir: &Path) {
        let plugin_path = project_dir.join("plugin.json");
        let Ok(raw) = std::fs::read_to_string(&plugin_path) else {
            return;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return;
        };
        let Some(u) = v
            .get("info")
            .and_then(|info| info.get("updated"))
            .and_then(|x| x.as_str())
        else {
            return;
        };
        self.today = u.to_string();
        if let Some(y) = u.split('-').next() {
            self.current_year = y.to_string();
        }
    }
}

/// Known binary file extensions that should be copied without template rendering
const BINARY_EXTENSIONS: &[&str] = &[
    "svg", "png", "jpg", "jpeg", "gif", "ico", "woff", "woff2", "ttf", "eot",
];

/// Check if a file should be treated as binary
fn is_binary_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| BINARY_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

/// Collect template files from specified directories under `templates_root`
pub fn collect_template_dirs(templates_root: &Path, dirs: &[&str]) -> Vec<(PathBuf, PathBuf)> {
    let mut files = Vec::new();
    for dir_name in dirs {
        let dir = templates_root.join(dir_name);
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let rel = entry.path().strip_prefix(&dir).unwrap_or(entry.path());
                files.push((entry.path().to_path_buf(), rel.to_path_buf()));
            }
        }
    }
    files
}

/// Render a template string with the given context.
///
/// # Errors
///
/// Returns an error when the template fails to parse or render.
pub fn render_string(template: &str, context: &TemplateContext) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("__inline__", template)
        .context("Failed to parse template")?;
    let tera_ctx =
        tera::Context::from_serialize(context).context("Failed to serialize template context")?;
    tera.render("__inline__", &tera_ctx)
        .context("Failed to render template")
}

/// Render a template file, stripping the `.tera` extension from output path.
///
/// # Errors
///
/// Returns an error when templates cannot be read or written.
pub fn render_file(
    src: &Path,
    output_dir: &Path,
    rel_path: &Path,
    context: &TemplateContext,
) -> Result<PathBuf> {
    // Strip .tera extension from output path
    let out_rel = if rel_path.extension().and_then(|e| e.to_str()) == Some("tera") {
        rel_path.with_extension("")
    } else {
        rel_path.to_path_buf()
    };

    let dest = output_dir.join(&out_rel);

    // Create parent directory
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    if is_binary_file(src) || (rel_path.extension().and_then(|e| e.to_str()) != Some("tera")) {
        // Copy binary files or non-tera files as-is
        std::fs::copy(src, &dest)
            .with_context(|| format!("Failed to copy file: {}", src.display()))?;
    } else {
        // Render template
        let template_body = std::fs::read_to_string(src)
            .with_context(|| format!("Failed to read template: {}", src.display()))?;
        let rendered = render_string(&template_body, context)?;
        std::fs::write(&dest, rendered)
            .with_context(|| format!("Failed to write file: {}", dest.display()))?;
    }

    Ok(dest)
}
