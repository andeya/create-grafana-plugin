//! Integration test: `update --dry-run` does not mutate files (9.4).

mod common;

use std::fs;
use std::process::Command;

#[test]
fn update_dry_run_leaves_managed_edits_intact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let name = "it-update-dry";
    let status = Command::new(common::create_grafana_plugin_bin())
        .args([
            "--name", name, "--type", "panel", "--author", "U Author", "--org", "updorg",
        ])
        .current_dir(tmp.path())
        .status()
        .expect("spawn scaffold");
    assert!(status.success());

    let project = tmp.path().join(name);
    let pkg_path = project.join("package.json");
    let original = fs::read_to_string(&pkg_path).expect("read package.json");
    let trimmed = original.trim();
    let modified = trimmed.strip_suffix('}').map_or_else(
        || panic!("package.json should end with a closing brace"),
        |body| format!("{body},\n  \"x-test-dry-run-marker\": \"stay\"\n}}\n"),
    );

    fs::write(&pkg_path, &modified).expect("write package.json");

    let status = Command::new(common::create_grafana_plugin_bin())
        .args(["update", "--dry-run"])
        .current_dir(&project)
        .status()
        .expect("spawn update");
    assert!(status.success(), "dry-run update should exit 0");

    let after = fs::read_to_string(&pkg_path).expect("read package.json after dry-run");
    assert_eq!(
        after, modified,
        "dry-run must not rewrite managed files on disk"
    );
}
