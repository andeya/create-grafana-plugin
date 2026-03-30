//! Integration test: panel scaffold file layout (9.1).

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

fn run_scaffold(temp: &Path, name: &str) {
    let status = Command::new(common::create_grafana_plugin_bin())
        .args([
            "--name",
            name,
            "--type",
            "panel",
            "--author",
            "Test Author",
            "--org",
            "acmecorp",
        ])
        .current_dir(temp)
        .status()
        .expect("spawn create-grafana-plugin");
    assert!(status.success(), "scaffold should succeed");
}

#[test]
fn panel_plugin_has_expected_files_and_metadata() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let name = "it-panel-basic";
    run_scaffold(tmp.path(), name);

    let root = tmp.path().join(name);
    let expected = [
        "package.json",
        "plugin.json",
        "tsconfig.json",
        "rspack.config.js",
        "tsconfig.test.json",
        "bunfig.toml",
        "tests/unit/smoke.test.ts",
        "src/module.ts",
        "src/components/MainPanel.tsx",
        "src/types/index.ts",
        ".gitignore",
        ".eslintrc.json",
        ".prettierrc",
        "AGENTS.md",
        "README.md",
        ".github/workflows/ci.yml",
        "scripts/bump-version.ts",
        "scripts/clean-plugin-dist.ts",
        "tests/setup/happydom.ts",
        ".grafana-plugin-version",
    ];
    for rel in expected {
        let p = root.join(rel);
        assert!(p.is_file(), "expected file missing: {}", p.display());
    }

    let pkg = fs::read_to_string(root.join("package.json")).expect("read package.json");
    assert!(
        pkg.contains(&format!("\"name\": \"acmecorp-{name}\"")),
        "package.json should contain scoped project name"
    );

    let plugin = fs::read_to_string(root.join("plugin.json")).expect("read plugin.json");
    assert!(
        plugin.contains("\"type\": \"panel\""),
        "plugin.json should declare panel type"
    );
    assert!(
        plugin.contains("\"id\": \"acmecorp-it-panel-basic\""),
        "plugin.json should contain org-prefixed id"
    );
}
