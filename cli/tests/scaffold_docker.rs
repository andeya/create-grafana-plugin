//! Integration test: panel + Docker provisioning files (9.3).

mod common;

use std::process::Command;

#[test]
fn panel_docker_includes_compose_and_provisioning() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let name = "it-docker-panel";
    let status = Command::new(common::create_grafana_plugin_bin())
        .args([
            "--name",
            name,
            "--type",
            "panel",
            "--author",
            "Docker Author",
            "--org",
            "dockorg",
            "--docker",
        ])
        .current_dir(tmp.path())
        .status()
        .expect("spawn create-grafana-plugin");
    assert!(status.success(), "scaffold should succeed");

    let root = tmp.path().join(name);
    assert!(root.join("docker-compose.yml").is_file());
    assert!(
        root.join("provisioning/datasources/datasources.yml")
            .is_file()
    );
    assert!(root.join("provisioning/plugins/plugins.yml").is_file());

    let compose =
        std::fs::read_to_string(root.join("docker-compose.yml")).expect("read docker-compose.yml");
    assert!(
        compose.to_lowercase().contains("grafana"),
        "docker-compose should reference Grafana"
    );
}
