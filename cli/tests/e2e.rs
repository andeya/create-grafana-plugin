//! End-to-end: full option scaffold and spot checks (9.6).

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

fn count_files_excluding_git(dir: &Path) -> usize {
    let mut n = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        let Ok(entries) = fs::read_dir(&p) else {
            continue;
        };
        for e in entries.flatten() {
            let path = e.path();
            if path.file_name().and_then(|s| s.to_str()) == Some(".git") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn full_scaffold_has_many_files_and_expected_markers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let name = "it-e2e-full";
    let status = Command::new(common::create_grafana_plugin_bin())
        .args([
            "--name",
            name,
            "--type",
            "panel",
            "--author",
            "E2E Author",
            "--org",
            "e2eorg",
            "--wasm",
            "--docker",
            "--mock",
        ])
        .current_dir(tmp.path())
        .status()
        .expect("spawn create-grafana-plugin");
    assert!(status.success(), "e2e scaffold should succeed");

    let root = tmp.path().join(name);
    let n = count_files_excluding_git(&root);
    assert!(n > 25, "expected a rich scaffold (>25 files), got {n}");

    let marker = fs::read_to_string(root.join(".grafana-plugin-version"))
        .expect("read .grafana-plugin-version")
        .trim()
        .to_string();
    assert_eq!(marker, env!("CARGO_PKG_VERSION"));

    let compose = fs::read_to_string(root.join("docker-compose.yml")).expect("compose");
    assert!(compose.to_lowercase().contains("grafana"));

    assert!(root.join("otel-mock").is_dir(), "mock module should exist");

    let wasm_bridge = fs::read_to_string(root.join("src/services/wasm-bridge.ts")).expect("wasm");
    assert!(wasm_bridge.contains("wasm"));

    let plugin = fs::read_to_string(root.join("plugin.json")).expect("plugin.json");
    assert!(plugin.contains("\"type\": \"panel\""));
    assert!(plugin.contains("e2eorg-it-e2e-full"));
}
