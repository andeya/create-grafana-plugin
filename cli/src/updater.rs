//! Incremental template updates for existing Grafana plugin projects.

use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{self, ProjectConfig};
use crate::template::{self, TemplateContext};

const VERSION_MARKER_FILE: &str = ".grafana-plugin-version";

/// Marker for JS/TS/TSX/RS/MJS files (must match templates).
pub const MANAGED_MARKER_JS: &str = "// @managed by create-grafana-plugin — do not edit";
/// Marker for YAML, `.gitignore`, Dockerfiles, etc.
pub const MANAGED_MARKER_HASH: &str = "# @managed by create-grafana-plugin — do not edit";
/// Marker for Markdown files.
pub const MANAGED_MARKER_HTML: &str = "<!-- @managed by create-grafana-plugin — do not edit -->";

/// JSON outputs that cannot carry comments — matched by path relative to project root.
const KNOWN_MANAGED_JSON_RELS: &[&str] = &[
    "plugin.json",
    "package.json",
    "tsconfig.json",
    "tsconfig.test.json",
    "biome.json",
];

/// Run update in the current directory (expects a scaffolded project).
///
/// # Errors
///
/// Returns an error when the project layout is invalid, templates are missing, or I/O fails.
pub fn update(dry_run: bool) -> Result<()> {
    let project_dir = std::env::current_dir().context("Failed to get current directory")?;
    let stored_version = read_version_marker(&project_dir);
    let cfg = discover_project_config(&project_dir)?;
    let mut context = TemplateContext::from_config(&cfg);
    context.apply_dates_from_existing_plugin_json(&project_dir);

    let dirs = config::template_directory_stack(&cfg);
    println!(
        "\n  {} {} (scaffold version: {})",
        "Updating".green().bold(),
        project_dir.display(),
        stored_version.as_deref().unwrap_or("unknown")
    );
    println!("  Templates: {}", dirs.join(" + "));
    println!("  Tool version: {}", env!("CARGO_PKG_VERSION"));

    let files = template::collect_template_files(&dirs);
    if files.is_empty() {
        anyhow::bail!("No embedded template files found for: {}", dirs.join(", "));
    }

    let mut by_output: HashMap<PathBuf, (&[u8], PathBuf)> = HashMap::new();
    for (contents, rel) in files {
        let rel_str = rel
            .to_string_lossy()
            .replace("{{ crate_name }}", &context.crate_name);
        let adjusted_rel = PathBuf::from(rel_str);
        let out_rel = if adjusted_rel.extension().and_then(|e| e.to_str()) == Some("tera") {
            adjusted_rel.with_extension("")
        } else {
            adjusted_rel
        };
        by_output.insert(out_rel, (contents, rel));
    }

    let mut sorted: Vec<_> = by_output.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut updated = 0usize;
    let mut created = 0usize;
    let mut skipped = 0usize;

    for (out_rel, (contents, template_rel)) in sorted {
        let new_bytes = template::render_to_bytes(contents, &template_rel, &context)?;
        let dest = project_dir.join(&out_rel);

        if dest.exists() {
            let old_bytes =
                fs::read(&dest).with_context(|| format!("Failed to read {}", dest.display()))?;
            if old_bytes == new_bytes {
                continue;
            }
            let old_text = String::from_utf8_lossy(&old_bytes);
            if !is_managed_existing_file(&out_rel, old_text.as_ref()) {
                skipped += 1;
                continue;
            }
            let new_text = std::str::from_utf8(&new_bytes)
                .with_context(|| format!("Rendered template is not UTF-8: {}", dest.display()))?;
            if dry_run {
                print_colored_diff(&dest, old_text.as_ref(), new_text);
            } else {
                write_file_atomic(&dest, &new_bytes)?;
            }
            updated += 1;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {}", parent.display()))?;
            }
            if dry_run {
                println!(
                    "\n{} (new file)\n{}",
                    dest.display().to_string().bold().cyan(),
                    String::from_utf8_lossy(&new_bytes).to_string().green()
                );
            } else {
                fs::write(&dest, &new_bytes)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;
            }
            created += 1;
        }
    }

    if !dry_run {
        write_version_marker(&project_dir)?;
    }

    println!(
        "\n  {} updated {} file(s), created {} file(s), skipped {} unmanaged file(s){}",
        "✓".green().bold(),
        updated,
        created,
        skipped,
        if dry_run {
            " (dry run — no files written)"
        } else {
            ""
        }
    );
    println!();
    Ok(())
}

fn write_version_marker(project_dir: &Path) -> Result<()> {
    let path = project_dir.join(VERSION_MARKER_FILE);
    fs::write(&path, env!("CARGO_PKG_VERSION"))
        .with_context(|| format!("Failed to write version marker {}", path.display()))
}

/// Read the version stored when the project was scaffolded or last updated.
fn read_version_marker(project_dir: &Path) -> Option<String> {
    let path = project_dir.join(VERSION_MARKER_FILE);
    fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn normalized_rel(rel: &Path) -> String {
    rel.to_string_lossy().replace('\\', "/")
}

fn is_known_json_managed_path(rel: &Path) -> bool {
    let n = normalized_rel(rel);
    KNOWN_MANAGED_JSON_RELS.iter().any(|&p| p == n)
}

fn has_js_style_marker(content: &str) -> bool {
    for line in content.lines().take(40) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("#!") {
            continue;
        }
        return t == MANAGED_MARKER_JS;
    }
    false
}

fn has_hash_style_marker(content: &str) -> bool {
    for line in content.lines().take(40) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return t == MANAGED_MARKER_HASH;
    }
    false
}

fn has_html_style_marker(content: &str) -> bool {
    for line in content.lines().take(40) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return t == MANAGED_MARKER_HTML;
    }
    false
}

fn is_managed_existing_file(rel: &Path, content: &str) -> bool {
    if is_known_json_managed_path(rel) {
        return true;
    }

    let ext = rel.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "ts" | "tsx" | "js" | "mjs" | "rs" => has_js_style_marker(content),
        "yml" | "yaml" => has_hash_style_marker(content),
        "md" => has_html_style_marker(content),
        _ => {
            let name = rel.file_name();
            if name == Some(std::ffi::OsStr::new("Dockerfile"))
                || name == Some(std::ffi::OsStr::new(".gitignore"))
                || normalized_rel(rel).ends_with("/ci.yml")
            {
                has_hash_style_marker(content)
            } else {
                has_js_style_marker(content)
                    || has_hash_style_marker(content)
                    || has_html_style_marker(content)
            }
        }
    }
}

fn print_colored_diff(path: &Path, old: &str, new: &str) {
    println!("\n{}", path.display().to_string().bold().cyan());
    let diff = TextDiff::from_lines(old, new);
    for change in diff.iter_all_changes() {
        let styled = match change.tag() {
            ChangeTag::Delete => format!("-{}", change.value()).red(),
            ChangeTag::Insert => format!("+{}", change.value()).green(),
            ChangeTag::Equal => format!(" {}", change.value()).normal(),
        };
        print!("{styled}");
    }
}

fn write_file_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().context("path has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents).with_context(|| format!("Failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Split plugin id into (org, name).
///
/// Uses the author URL from `plugin.json` (`https://github.com/{org}`) for robust
/// extraction — handles org names that contain hyphens (e.g. `acme-corp-my-plugin`
/// with `org = acme-corp`).  Falls back to simple `split_once('-')` when the URL
/// is unavailable.
fn split_plugin_id<'a>(id: &'a str, author_url: Option<&str>) -> Result<(&'a str, &'a str)> {
    if let Some(url) = author_url {
        let org_from_url = url.rsplit('/').next().unwrap_or("");
        if !org_from_url.is_empty()
            && let Some(name) = id
                .strip_prefix(org_from_url)
                .and_then(|s| s.strip_prefix('-'))
        {
            return Ok((&id[..org_from_url.len()], name));
        }
    }
    id.split_once('-')
        .context("plugin id must be in the form org-name")
}

fn discover_project_config(project_dir: &Path) -> Result<ProjectConfig> {
    let plugin_path = project_dir.join("plugin.json");
    let raw = fs::read_to_string(&plugin_path)
        .with_context(|| format!("Expected plugin.json at {}", plugin_path.display()))?;
    let v: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in {}", plugin_path.display()))?;
    let id = v
        .get("id")
        .and_then(Value::as_str)
        .context("plugin.json: missing id")?;
    let author_url = v
        .get("info")
        .and_then(|i| i.get("author"))
        .and_then(|a| a.get("url"))
        .and_then(Value::as_str);
    let (org, name) = split_plugin_id(id, author_url)?;
    let plugin_type = config::parse_plugin_type(
        v.get("type")
            .and_then(Value::as_str)
            .context("plugin.json: missing type")?,
    )?;
    let description = v
        .get("info")
        .and_then(|i| i.get("description"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let author = v
        .get("info")
        .and_then(|i| i.get("author"))
        .and_then(|a| a.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let has_wasm = project_dir.join("Cargo.toml").exists();
    let has_docker = project_dir.join("docker-compose.yml").exists()
        || project_dir.join("docker-compose.yaml").exists();
    let has_mock = has_docker && project_dir.join("otel-mock").is_dir();
    let port_offset = if has_docker {
        infer_port_offset(project_dir)
    } else {
        0
    };

    Ok(ProjectConfig {
        name: config::to_kebab_case(name),
        description,
        author,
        org: org.to_string(),
        plugin_type,
        has_wasm,
        has_docker,
        has_mock,
        port_offset,
    })
}

/// Infer `port_offset` from the Grafana host port in `docker-compose.yml`.
///
/// Looks for a host:container port pair where container port is 3000 (Grafana default),
/// then derives offset = `host_port` - 3000.  Returns 0 on any parse failure.
fn infer_port_offset(project_dir: &Path) -> u16 {
    let raw = fs::read_to_string(project_dir.join("docker-compose.yml"))
        .or_else(|_| fs::read_to_string(project_dir.join("docker-compose.yaml")));
    let Ok(raw) = raw else {
        return 0;
    };
    // Simple regex-free scan: find `"<host>:3000"` pattern after `grafana:` service.
    let mut in_grafana = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("grafana:") {
            in_grafana = true;
            continue;
        }
        if in_grafana
            && !trimmed.is_empty()
            && !trimmed.starts_with('-')
            && !trimmed.starts_with('#')
            && !line.starts_with(' ')
            && !line.starts_with('\t')
        {
            break;
        }
        if in_grafana {
            // Match `- "XXXX:3000"` or `- 'XXXX:3000'`
            let stripped = trimmed
                .trim_start_matches('-')
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some(host_str) = stripped.strip_suffix(":3000")
                && let Ok(host_port) = host_str.parse::<u16>()
            {
                return host_port.saturating_sub(3000);
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_json_paths() {
        assert!(is_known_json_managed_path(Path::new("plugin.json")));
        assert!(is_known_json_managed_path(Path::new("biome.json")));
        assert!(!is_known_json_managed_path(Path::new("src/foo.json")));
    }

    #[test]
    fn detects_js_marker() {
        let s = format!("{MANAGED_MARKER_JS}\nexport const x = 1;\n");
        assert!(has_js_style_marker(&s));
        assert!(!has_js_style_marker("export const x = 1;\n"));
    }

    #[test]
    fn detects_hash_marker() {
        let s = format!("{MANAGED_MARKER_HASH}\nfoo: bar\n");
        assert!(has_hash_style_marker(&s));
    }

    #[test]
    fn shebang_then_marker() {
        let s = format!("#!/usr/bin/env node\n{MANAGED_MARKER_JS}\nconsole.log(1);\n");
        assert!(has_js_style_marker(&s));
    }

    #[test]
    fn split_plugin_id_simple() {
        let (org, name) = split_plugin_id("myorg-my-plugin", None).unwrap();
        assert_eq!(org, "myorg");
        assert_eq!(name, "my-plugin");
    }

    #[test]
    fn split_plugin_id_with_hyphenated_org() {
        let (org, name) =
            split_plugin_id("acme-corp-my-plugin", Some("https://github.com/acme-corp")).unwrap();
        assert_eq!(org, "acme-corp");
        assert_eq!(name, "my-plugin");
    }

    #[test]
    fn split_plugin_id_fallback_without_url() {
        let (org, name) = split_plugin_id("acme-corp-my-plugin", None).unwrap();
        assert_eq!(org, "acme");
        assert_eq!(name, "corp-my-plugin");
    }
}
