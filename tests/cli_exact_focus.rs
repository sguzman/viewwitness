#![cfg(feature = "egui")]

use std::{fs, process::Command};

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiLayerOrder,
    EguiPaintKind, EguiPaintObservation, Rect, from_yaml,
};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_viewwitness"))
}

#[test]
fn capture_exact_can_focus_one_authored_binding() {
    let (addr, server) = spawn_exact_capture_peer(sample_capture());
    let output = cli()
        .args([
            "capture-exact",
            &addr.to_string(),
            "--object=canvas:node",
            "--binding=handle",
        ])
        .output()
        .expect("run focused exact capture");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.starts_with(
        "egui-correlated-focus request=7 viewport_id=2 pass=9 viewport_rect=[0,0,100,50] object_id=\"canvas:node\" binding_id=\"handle\" object_match_count=1 binding_match_count=1"
    ));
    assert!(stdout.contains("projection=authored_focus"));
    assert!(stdout.contains("omitted=canonical_semantics,generic_paint"));
    assert!(stdout.contains("authored_binding_id=\"handle\""));
    assert!(!stdout.contains("authored_binding_id=\"outline\""));
    assert!(!stdout.contains("node id=\"ak:1\""));
    assert!(!stdout.contains("paint order="));
    server.join().expect("mock exact peer exits cleanly");
}

#[test]
fn inspect_exact_focus_reads_saved_full_envelope() {
    let path = write_capture(&sample_capture(), "inspect-focus");
    let output = cli()
        .args([
            "inspect-exact",
            &path.to_string_lossy(),
            "--object=canvas:node",
        ])
        .output()
        .expect("run focused exact inspection");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("object_match_count=1 binding_match_count=2"));
    assert!(stdout.contains("authored_binding_id=\"outline\""));
    assert!(stdout.contains("authored_binding_id=\"handle\""));
    assert!(!stdout.contains("node id=\"ak:1\""));
    assert!(!stdout.contains("paint order="));

    fs::remove_file(path).expect("remove saved exact envelope");
}

#[test]
fn inspect_exact_without_focus_can_emit_full_yaml() {
    let path = write_capture(&sample_capture(), "inspect-yaml");
    let output = cli()
        .args(["inspect-exact", &path.to_string_lossy(), "--yaml"])
        .output()
        .expect("run exact YAML inspection");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("request_id: 7"));
    assert!(stdout.contains("witness:"));
    assert!(stdout.contains("paint:"));
    assert!(stdout.contains("authored_objects:"));
    assert!(stdout.contains("authored_binding_id: outline"));
    assert!(stdout.contains("authored_binding_id: handle"));

    fs::remove_file(path).expect("remove saved exact envelope");
}

#[test]
fn exact_focus_rejects_binding_without_object() {
    let output = cli()
        .args(["capture-exact", "--binding=handle"])
        .output()
        .expect("run invalid focused exact capture");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--binding requires --object"));
}

#[test]
fn exact_focus_rejects_yaml_because_projection_is_not_full_envelope() {
    let output = cli()
        .args(["capture-exact", "--object=canvas:node", "--yaml"])
        .output()
        .expect("run invalid focused YAML capture");

    assert!(!output.status.success());
    assert!(stderr(&output).contains(
        "authored focus is an agent-text projection; --object/--binding cannot be combined with --yaml"
    ));
}

#[test]
fn exact_focus_rejects_derive_because_semantics_are_omitted() {
    let output = cli()
        .args(["capture-exact", "--object=canvas:node", "--derive"])
        .output()
        .expect("run invalid focused derived capture");

    assert!(!output.status.success());
    assert!(stderr(&output).contains(
        "authored focus omits canonical semantics; --derive cannot be combined with --object/--binding"
    ));
}

fn spawn_exact_capture_peer(
    capture: EguiCorrelatedCapture,
) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread,
    };

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock focused exact peer");
    let addr = listener.local_addr().expect("focused exact peer address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept focused exact observer");
        writeln!(
            stream,
            "{} {}",
            viewwitness::EGUI_CAPTURE_PROTOCOL_MAGIC,
            viewwitness::EGUI_CAPTURE_PROTOCOL_VERSION
        )
        .expect("write focused exact handshake");

        let mut request = String::new();
        BufReader::new(stream.try_clone().expect("clone focused peer stream"))
            .read_line(&mut request)
            .expect("read focused capture request");
        assert_eq!(request.trim_end(), "CAPTURE");

        let response = serde_json::json!({
            "status": "ok",
            "capture": capture,
        });
        writeln!(stream, "{response}").expect("write focused exact response");
    });
    (addr, server)
}

fn write_capture(capture: &EguiCorrelatedCapture, suffix: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "viewwitness-exact-focus-{}-{suffix}.yaml",
        std::process::id()
    ));
    fs::write(
        &path,
        serde_yaml_ng::to_string(capture).expect("serialize focused exact capture"),
    )
    .expect("write focused exact capture");
    path
}

fn sample_capture() -> EguiCorrelatedCapture {
    let witness = from_yaml(
        r#"
viewwitness_version: "0.1"
capture:
  source: egui
  frame: 9
  viewport:
    width: 100.0
    height: 50.0
    scale_factor: 1.0
nodes:
  - id: "ak:1"
    role: button
    name: Apply
    bounds:
      x: 10.0
      y: 10.0
      width: 20.0
      height: 10.0
relations: []
"#,
    )
    .expect("parse focused CLI semantic witness");

    let binding = |id: &str, shape_index, kind| EguiAuthoredPaintBinding {
        authored_binding_id: Some(id.into()),
        binding_evidence: "observed".into(),
        layer_order: EguiLayerOrder::Background,
        layer_id: 42,
        shape_index,
        verified_at_end_pass: true,
        kind: Some(kind),
        bounds: Some(Rect {
            x: 10.0 + shape_index as f32,
            y: 10.0,
            width: 20.0,
            height: 10.0,
        }),
        clip_rect: None,
    };

    EguiCorrelatedCapture {
        request_id: 7,
        viewport_id: 2,
        pass_nr: 9,
        viewport_rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        witness,
        paint: vec![EguiPaintObservation {
            order: 1,
            kind: EguiPaintKind::Rect,
            bounds: Rect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 10.0,
            },
            clip_rect: None,
        }],
        authored_objects: vec![EguiAuthoredPaintObject {
            id: "canvas:node".into(),
            role: "diagram_node".into(),
            name: Some("Canvas node".into()),
            semantic_evidence: "intended".into(),
            bindings: vec![
                binding("outline", 3, EguiPaintKind::Rect),
                binding("handle", 4, EguiPaintKind::Circle),
            ],
        }],
    }
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
