use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_viewwitness"))
}

#[test]
fn validate_accepts_known_good_fixture() {
    let output = cli()
        .args(["validate", "examples/snapshots/01-minimal-button.yaml"])
        .output()
        .expect("run CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8"),
        "ok examples/snapshots/01-minimal-button.yaml\n"
    );
}

#[test]
fn inspect_defaults_to_agent_text() {
    let output = cli()
        .args(["inspect", "examples/snapshots/01-minimal-button.yaml"])
        .output()
        .expect("run CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.starts_with("view version=\"0.1\" source=\"synthetic\""));
    assert!(stdout.contains("node id=\"greet\" role=\"button\""));
}

#[test]
fn diff_defaults_to_material_agent_text() {
    let output = cli()
        .args([
            "diff",
            "examples/transitions/03-busy-state/before.yaml",
            "examples/transitions/03-busy-state/after.yaml",
        ])
        .output()
        .expect("run CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.starts_with("diff before_frame="));
    assert!(stdout.contains("change node=\"export\" field=\"enabled\""));
    assert!(stdout.contains("+node id=\"export-progress\""));
}

#[test]
fn derive_can_emit_enriched_yaml() {
    let output = cli()
        .args([
            "derive",
            "examples/snapshots/10-pathological-overlap.yaml",
            "--yaml",
        ])
        .output()
        .expect("run CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("evidence: derived"));
    assert!(stdout.contains("kind: overlaps"));
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
