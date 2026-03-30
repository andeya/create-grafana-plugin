//! Integration test: panel + WASM scaffold and Cargo workspace (9.2).

mod common;

use std::fs;
use std::process::Command;

#[test]
fn panel_wasm_includes_workspace_and_crate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let name = "it-wasm-panel";
    let status = Command::new(common::create_grafana_plugin_bin())
        .args([
            "--name",
            name,
            "--type",
            "panel",
            "--author",
            "Wasm Author",
            "--org",
            "wasmorg",
            "--wasm",
        ])
        .current_dir(tmp.path())
        .status()
        .expect("spawn create-grafana-plugin");
    assert!(status.success(), "scaffold should succeed");

    let root = tmp.path().join(name);
    let crate_name = name.replace('-', "_");

    let cargo_root = fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo.toml");
    assert!(
        cargo_root.contains("[workspace]"),
        "root Cargo.toml should declare a workspace"
    );

    let crate_manifest = root.join(&crate_name).join("Cargo.toml");
    assert!(
        crate_manifest.is_file(),
        "crate Cargo.toml should exist at {}",
        crate_manifest.display()
    );

    let lib_rs = root.join(&crate_name).join("src/lib.rs");
    assert!(lib_rs.is_file(), "lib.rs should exist");

    let wasm_bridge = fs::read_to_string(root.join("src/services/wasm-bridge.ts")).expect("wasm");
    assert!(
        wasm_bridge.contains("initSync") && wasm_bridge.contains("_bg.wasm"),
        "wasm bridge should use initSync + inlined wasm import"
    );

    let rspack = fs::read_to_string(root.join("rspack.config.ts")).expect("rspack");
    assert!(
        rspack.contains("asset/inline") && rspack.contains(".wasm"),
        "rspack should inline .wasm"
    );

    assert!(
        root.join("src/types/wasm.d.ts").is_file(),
        "wasm.d.ts should exist"
    );
    assert!(
        root.join("scripts/bump-version.ts").is_file(),
        "bump-version.ts should exist"
    );
}
