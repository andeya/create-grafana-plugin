//! Shared helpers for integration tests (binary path resolution).

use std::path::{Path, PathBuf};

/// Path to the `create-grafana-plugin` binary.
///
/// Cargo normally sets `CARGO_BIN_EXE_create_grafana_plugin` when running integration tests.
/// Some workspace invocations omit it; we fall back to the workspace `target/` layout.
pub fn create_grafana_plugin_bin() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_create-grafana-plugin") {
        return PathBuf::from(p);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let candidate = manifest_dir
        .join("../target")
        .join(profile)
        .join("create-grafana-plugin");
    if candidate.is_file() {
        return candidate;
    }

    panic!(
        "could not resolve create-grafana-plugin binary: set CARGO_BIN_EXE_create_grafana_plugin or build so that {} exists",
        candidate.display()
    );
}
