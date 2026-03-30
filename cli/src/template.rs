//! Template discovery, rendering, and [`TemplateContext`] for scaffold and update flows.
//!
//! Templates are embedded into the binary at compile time via [`include_dir`].

use anyhow::{Context, Result};
use include_dir::{Dir, DirEntry, include_dir};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tera::Tera;

use crate::config::ProjectConfig;

/// All template files under `templates/`, baked into the binary at compile time.
static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Render embedded template content to bytes.
///
/// If `rel_path` ends with `.tera`, the content is rendered through Tera; otherwise it is
/// returned verbatim.  Binary files (images, fonts) are always returned as-is.
///
/// # Errors
///
/// Returns an error when the template fails to render.
pub fn render_to_bytes(
    contents: &[u8],
    rel_path: &Path,
    context: &TemplateContext,
) -> Result<Vec<u8>> {
    if is_binary_path(rel_path) || rel_path.extension().and_then(|e| e.to_str()) != Some("tera") {
        Ok(contents.to_vec())
    } else {
        let template_body =
            std::str::from_utf8(contents).context("Template file is not valid UTF-8")?;
        let rendered = render_string(template_body, context)?;
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
    pub port_offset: u16,

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
            port_offset: config.port_offset,
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

/// Known binary file extensions that should be copied without template rendering.
const BINARY_EXTENSIONS: &[&str] = &[
    "svg", "png", "jpg", "jpeg", "gif", "ico", "woff", "woff2", "ttf", "eot",
];

/// Check if a path has a binary extension.
fn is_binary_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| BINARY_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

/// Recursively collect all files from an embedded [`Dir`].
fn walk_embedded_dir(dir: &'static Dir<'static>) -> Vec<&'static include_dir::File<'static>> {
    let mut out = Vec::new();
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(d) => out.extend(walk_embedded_dir(d)),
            DirEntry::File(f) => out.push(f),
        }
    }
    out
}

/// Collect embedded template files for the given directory stack (e.g. `["base", "panel", "wasm"]`).
///
/// Returns `(file_contents, relative_path)` pairs where the relative path is within each
/// sub-directory (e.g. `src/module.ts.tera`, not `panel/src/module.ts.tera`).
pub fn collect_template_files(dirs: &[&str]) -> Vec<(&'static [u8], PathBuf)> {
    let mut files = Vec::new();
    for dir_name in dirs {
        let Some(dir) = TEMPLATES.get_dir(dir_name) else {
            continue;
        };
        for file in walk_embedded_dir(dir) {
            let rel = file.path().strip_prefix(dir_name).unwrap_or(file.path());
            files.push((file.contents(), rel.to_path_buf()));
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

/// Render embedded template content and write the result to `output_dir`.
///
/// The `.tera` suffix is stripped from `rel_path` in the output.
///
/// # Errors
///
/// Returns an error when the template fails to render or I/O fails.
pub fn write_rendered(
    contents: &[u8],
    rel_path: &Path,
    output_dir: &Path,
    context: &TemplateContext,
) -> Result<PathBuf> {
    let out_rel = if rel_path.extension().and_then(|e| e.to_str()) == Some("tera") {
        rel_path.with_extension("")
    } else {
        rel_path.to_path_buf()
    };

    let dest = output_dir.join(&out_rel);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let bytes = render_to_bytes(contents, rel_path, context)?;
    std::fs::write(&dest, bytes)
        .with_context(|| format!("Failed to write file: {}", dest.display()))?;

    Ok(dest)
}
