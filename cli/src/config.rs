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

/// Package manager variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Bun,
    Npm,
    Pnpm,
    Yarn,
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bun => write!(f, "bun"),
            Self::Npm => write!(f, "npm"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Yarn => write!(f, "yarn"),
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
    pub package_manager: PackageManager,
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
    pm: Option<String>,
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

fn parse_pm(s: &str) -> Result<PackageManager> {
    match s.to_lowercase().as_str() {
        "bun" => Ok(PackageManager::Bun),
        "npm" => Ok(PackageManager::Npm),
        "pnpm" => Ok(PackageManager::Pnpm),
        "yarn" => Ok(PackageManager::Yarn),
        _ => anyhow::bail!("Invalid package manager: {s}. Use: bun, npm, pnpm, or yarn"),
    }
}

/// Parse package manager from a `package.json` `packageManager` value (e.g. `bun@1.2.0`).
///
/// # Errors
///
/// Returns an error when the package manager name is not supported.
pub fn parse_package_manager_value(value: &str) -> Result<PackageManager> {
    let name = value.split('@').next().unwrap_or("bun").trim();
    parse_pm(name)
}

/// Build config from CLI args, falling back to TOML file, then interactive prompts.
///
/// # Errors
///
/// Returns an error when config values are invalid or files cannot be read.
///
/// # Panics
///
/// Panics if internal option combinations are inconsistent (should not occur in normal use).
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
    let pm_str = if args.pm == "bun" {
        toml_cfg.as_ref().and_then(|c| c.pm.clone())
    } else {
        Some(args.pm.clone())
    };

    // Check if we have enough info for non-interactive mode
    let all_provided =
        name.is_some() && plugin_type_str.is_some() && author.is_some() && org.is_some();

    if all_provided {
        return Ok(ProjectConfig {
            name: to_kebab_case(&name.unwrap()),
            description: description.unwrap_or_default(),
            author: author.unwrap(),
            org: org.unwrap(),
            plugin_type: parse_plugin_type(&plugin_type_str.unwrap())?,
            has_wasm: has_wasm.unwrap_or(false),
            has_docker: has_docker.unwrap_or(false),
            has_mock: has_mock.unwrap_or(false),
            package_manager: parse_pm(&pm_str.unwrap_or_else(|| "bun".into()))?,
        });
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

    let package_manager = if let Some(ref pm) = pm_str {
        parse_pm(pm)?
    } else {
        let pms = ["bun", "npm", "pnpm", "yarn"];
        let idx = Select::new()
            .with_prompt("  Package manager")
            .items(&pms)
            .default(0)
            .interact()?;
        match idx {
            0 => PackageManager::Bun,
            1 => PackageManager::Npm,
            2 => PackageManager::Pnpm,
            _ => PackageManager::Yarn,
        }
    };

    Ok(ProjectConfig {
        name,
        description,
        author,
        org,
        plugin_type,
        has_wasm,
        has_docker,
        has_mock,
        package_manager,
    })
}
