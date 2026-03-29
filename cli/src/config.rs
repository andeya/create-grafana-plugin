use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use serde::{Deserialize, Serialize};

/// Plugin type variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Panel,
    Datasource,
    App,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panel => write!(f, "panel"),
            Self::Datasource => write!(f, "datasource"),
            Self::App => write!(f, "app"),
        }
    }
}

/// Resolved project configuration after merging all input sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
    pub author: String,
    pub org: String,
    pub plugin_type: PluginType,
    pub has_wasm: bool,
    pub has_docker: bool,
    pub has_mock: bool,
}

/// Template layer directories in merge order — single source of truth for scaffold and [`crate::updater::update`].
pub fn template_directory_stack(config: &ProjectConfig) -> Vec<&'static str> {
    let mut dirs = vec!["base"];

    match config.plugin_type {
        PluginType::Panel => dirs.push("panel"),
        PluginType::Datasource => dirs.push("datasource"),
        PluginType::App => dirs.push("app"),
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

/// Ensures flag combinations match what the scaffold can emit.
///
/// # Errors
///
/// Returns an error when options are inconsistent (e.g. mock without Docker).
pub fn validate_project_config(config: &ProjectConfig) -> Result<()> {
    if config.has_mock && !config.has_docker {
        anyhow::bail!(
            "Mock data generator requires Docker: pass --docker with --mock, or set docker = true in .grafana-plugin.toml"
        );
    }
    Ok(())
}

/// TOML config file structure
#[derive(Debug, Deserialize)]
struct TomlConfig {
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    org: Option<String>,
    r#type: Option<String>,
    wasm: Option<bool>,
    docker: Option<bool>,
    mock: Option<bool>,
}

/// Convert plugin name to valid kebab-case
pub fn to_kebab_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Parse plugin type string from `plugin.json` or CLI.
///
/// # Errors
///
/// Returns an error when `s` is not a supported plugin type string.
pub fn parse_plugin_type(s: &str) -> Result<PluginType> {
    match s.to_lowercase().as_str() {
        "panel" => Ok(PluginType::Panel),
        "datasource" | "data-source" => Ok(PluginType::Datasource),
        "app" => Ok(PluginType::App),
        _ => anyhow::bail!("Invalid plugin type: {s}. Use: panel, datasource, or app"),
    }
}

/// Build config from CLI args, falling back to TOML file, then interactive prompts.
///
/// # Errors
///
/// Returns an error when config values are invalid or files cannot be read.
///
#[allow(clippy::too_many_lines)]
pub fn resolve_config(args: &crate::cli::Args) -> Result<ProjectConfig> {
    // Load TOML config if specified
    let toml_cfg = if let Some(ref path) = args.config {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {path}"))?;
        Some(
            toml::from_str::<TomlConfig>(&content)
                .with_context(|| format!("Failed to parse config file: {path}"))?,
        )
    } else {
        None
    };

    let name = args
        .name
        .clone()
        .or_else(|| toml_cfg.as_ref().and_then(|c| c.name.clone()));
    let description = args
        .description
        .clone()
        .or_else(|| toml_cfg.as_ref().and_then(|c| c.description.clone()));
    let author = args
        .author
        .clone()
        .or_else(|| toml_cfg.as_ref().and_then(|c| c.author.clone()));
    let org = args
        .org
        .clone()
        .or_else(|| toml_cfg.as_ref().and_then(|c| c.org.clone()));
    let plugin_type_str = args
        .r#type
        .clone()
        .or_else(|| toml_cfg.as_ref().and_then(|c| c.r#type.clone()));
    let has_wasm = if args.wasm {
        Some(true)
    } else {
        toml_cfg.as_ref().and_then(|c| c.wasm)
    };
    let has_docker = if args.docker {
        Some(true)
    } else {
        toml_cfg.as_ref().and_then(|c| c.docker)
    };
    let has_mock = if args.mock {
        Some(true)
    } else {
        toml_cfg.as_ref().and_then(|c| c.mock)
    };
    if let (Some(name_val), Some(ptype_val), Some(author_val), Some(org_val)) = (
        name.as_deref(),
        plugin_type_str.as_deref(),
        author.as_deref(),
        org.as_deref(),
    ) {
        let cfg = ProjectConfig {
            name: to_kebab_case(name_val),
            description: description.clone().unwrap_or_default(),
            author: author_val.to_string(),
            org: org_val.to_string(),
            plugin_type: parse_plugin_type(ptype_val)?,
            has_wasm: has_wasm.unwrap_or(false),
            has_docker: has_docker.unwrap_or(false),
            has_mock: has_mock.unwrap_or(false),
        };
        validate_project_config(&cfg)?;
        return Ok(cfg);
    }

    // Interactive mode
    println!("\n  🔧 Grafana Plugin Creator\n");

    let name = name.map_or_else(
        || {
            Input::<String>::new()
                .with_prompt("  Plugin name")
                .interact_text()
                .map(|s| to_kebab_case(&s))
        },
        |n| Ok(to_kebab_case(&n)),
    )?;

    let description = description.map_or_else(
        || {
            Input::<String>::new()
                .with_prompt("  Description")
                .default("A Grafana plugin".to_string())
                .interact_text()
        },
        Ok,
    )?;

    let author = author.map_or_else(
        || {
            Input::<String>::new()
                .with_prompt("  Author")
                .interact_text()
        },
        Ok,
    )?;

    let org = org.map_or_else(
        || {
            Input::<String>::new()
                .with_prompt("  Organization")
                .interact_text()
        },
        Ok,
    )?;

    let plugin_type = if let Some(ref t) = plugin_type_str {
        parse_plugin_type(t)?
    } else {
        let types = ["Panel", "Datasource", "App"];
        let idx = Select::new()
            .with_prompt("  Plugin type")
            .items(&types)
            .default(0)
            .interact()?;
        match idx {
            0 => PluginType::Panel,
            1 => PluginType::Datasource,
            _ => PluginType::App,
        }
    };

    let has_wasm = has_wasm.map_or_else(
        || {
            Confirm::new()
                .with_prompt("  Include Rust WASM engine?")
                .default(false)
                .interact()
        },
        Ok,
    )?;

    let has_docker = has_docker.map_or_else(
        || {
            Confirm::new()
                .with_prompt("  Include Docker dev environment?")
                .default(true)
                .interact()
        },
        Ok,
    )?;

    let has_mock = if has_docker {
        has_mock.map_or_else(
            || {
                Confirm::new()
                    .with_prompt("  Include mock data generator?")
                    .default(true)
                    .interact()
            },
            Ok,
        )?
    } else {
        false
    };

    let cfg = ProjectConfig {
        name,
        description,
        author,
        org,
        plugin_type,
        has_wasm,
        has_docker,
        has_mock,
    };
    validate_project_config(&cfg)?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cfg(has_docker: bool, has_mock: bool) -> ProjectConfig {
        ProjectConfig {
            name: "x".to_string(),
            description: String::new(),
            author: String::new(),
            org: String::new(),
            plugin_type: PluginType::Panel,
            has_wasm: false,
            has_docker,
            has_mock,
        }
    }

    #[test]
    fn validate_rejects_mock_without_docker() {
        let err = validate_project_config(&sample_cfg(false, true)).unwrap_err();
        assert!(
            err.to_string().contains("Mock"),
            "unexpected message: {err}"
        );
    }

    #[test]
    fn validate_accepts_mock_with_docker() {
        validate_project_config(&sample_cfg(true, true)).unwrap();
    }

    #[test]
    fn template_stack_includes_mock_only_with_docker() {
        let with = template_directory_stack(&sample_cfg(true, true));
        assert!(with.contains(&"mock"));

        let without = template_directory_stack(&sample_cfg(true, false));
        assert!(!without.contains(&"mock"));

        let mock_but_no_docker = template_directory_stack(&sample_cfg(false, true));
        assert!(!mock_but_no_docker.contains(&"mock"));
    }
}
