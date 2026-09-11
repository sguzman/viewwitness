#![cfg(feature = "egui")]

use std::{fs, process::Command};

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiLayerOrder,
    EguiPaintKind, Rect, from_yaml,
};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_viewwitness"))
}

#[test]
fn diff_exact_reports_shape_index_churn_as_non_material() {
    let before = capture(1, 9, 3);
    let after = capture(2, 10, 4);
    let (before_path, after_path) = write_pair(&before, &after, "agent");

    let output = cli()
        .args([
            "diff-exact",
            &before_path.to_string_lossy(),
            &after_path.to_string_lossy(),
        ])
        .output()
        .expect("run diff-exact CLI");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.starts_with(
        "egui-diff before_request=1 after_request=2 before_pass=9 after_pass=10 materially_empty=true\n"
    ));
    assert!(stdout.contains(
        "authored-handle-churn id=\"canvas:node\" binding_ordinal=0 before_shape_index=3 after_shape_index=4 material=false continuity=frame_local_structure_sensitive"
    ));
    assert!(!stdout.contains("authored-change"));

    cleanup(&before_path, &after_path);
}

#[test]
fn diff_exact_yaml_preserves_correlated_diff_structure() {
    let before = capture(11, 20, 5);
    let after = capture(12, 21, 6);
    let (before_path, after_path) = write_pair(&before, &after, "yaml");

    let output = cli()
        .args([
            "diff-exact",
            &before_path.to_string_lossy(),
            &after_path.to_string_lossy(),
            "--yaml",
        ])
        .output()
        .expect("run diff-exact YAML CLI");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("before_request_id: 11"));
    assert!(stdout.contains("after_request_id: 12"));
    assert!(stdout.contains("semantic:"));
    assert!(stdout.contains("authored:"));
    assert!(stdout.contains("execution_handle_churn:"));
    assert!(stdout.contains("before_shape_index: 5"));
    assert!(stdout.contains("after_shape_index: 6"));

    cleanup(&before_path, &after_path);
}

fn capture(request_id: u64, pass_nr: u64, shape_index: usize) -> EguiCorrelatedCapture {
    let witness = from_yaml(&format!(
        r#"
viewwitness_version: "0.1"
capture:
  source: egui
  frame: {pass_nr}
  viewport:
    width: 100.0
    height: 50.0
    scale_factor: 1.0
nodes: []
relations: []
"#
    ))
    .expect("parse synthetic correlated semantic witness");

    EguiCorrelatedCapture {
        request_id,
        viewport_id: 1,
        pass_nr,
        viewport_rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        witness,
        paint: Vec::new(),
        authored_objects: vec![EguiAuthoredPaintObject {
            id: "canvas:node".into(),
            role: "diagram_node".into(),
            name: Some("Node".into()),
            semantic_evidence: "intended".into(),
            bindings: vec![EguiAuthoredPaintBinding {
                binding_evidence: "observed".into(),
                layer_order: EguiLayerOrder::Background,
                layer_id: 42,
                shape_index,
                verified_at_end_pass: true,
                kind: Some(EguiPaintKind::Rect),
                bounds: Some(Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 20.0,
                    height: 20.0,
                }),
                clip_rect: None,
            }],
        }],
    }
}

fn write_pair(
    before: &EguiCorrelatedCapture,
    after: &EguiCorrelatedCapture,
    suffix: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let base = format!("viewwitness-diff-exact-{}-{suffix}", std::process::id());
    let before_path = std::env::temp_dir().join(format!("{base}-before.yaml"));
    let after_path = std::env::temp_dir().join(format!("{base}-after.yaml"));
    fs::write(
        &before_path,
        serde_yaml_ng::to_string(before).expect("serialize before correlated capture"),
    )
    .expect("write before correlated capture");
    fs::write(
        &after_path,
        serde_yaml_ng::to_string(after).expect("serialize after correlated capture"),
    )
    .expect("write after correlated capture");
    (before_path, after_path)
}

fn cleanup(before: &std::path::Path, after: &std::path::Path) {
    fs::remove_file(before).expect("remove before correlated capture");
    fs::remove_file(after).expect("remove after correlated capture");
}
