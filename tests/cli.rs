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

#[cfg(feature = "observer")]
#[test]
fn screenshot_command_writes_exact_peer_raster_bytes() {
    use std::{fs, net::TcpListener, thread};

    use egui_inspection::{EncodedPng, Request, Response, read_message, write_message};

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock inspection peer");
    let addr = listener.local_addr().expect("listener address");
    let expected_bytes = vec![137, 80, 78, 71, 13, 10, 26, 10, 9, 8, 7, 6];
    let server_bytes = expected_bytes.clone();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept CLI observer");
        egui_inspection::protocol::write_handshake(&mut stream).expect("write handshake");
        let request: Request = read_message(&mut stream).expect("read screenshot request");
        assert!(matches!(
            request,
            Request::GetScreenshot {
                pixels_per_point: Some(scale)
            } if scale == 1.0
        ));
        write_message(
            &mut stream,
            &Response::Screenshot(EncodedPng {
                size: [2, 1],
                bytes: server_bytes,
            }),
        )
        .expect("write screenshot response");
    });

    let path = std::env::temp_dir().join(format!(
        "viewwitness-cli-screenshot-{}.png",
        std::process::id()
    ));
    let path_arg = path.to_string_lossy().into_owned();
    let output = cli()
        .args([
            "screenshot",
            &path_arg,
            &format!("--address={addr}"),
            "--scale=1",
        ])
        .output()
        .expect("run screenshot CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        fs::read(&path).expect("read screenshot output"),
        expected_bytes
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8"),
        format!("saved {path_arg} 2x1\n")
    );
    fs::remove_file(path).expect("remove screenshot output");
    server.join().expect("mock peer exits cleanly");
}

#[cfg(feature = "egui")]
#[test]
fn capture_exact_defaults_to_correlated_agent_text() {
    let (addr, server) = spawn_exact_capture_peer(sample_correlated_capture());
    let output = cli()
        .args(["capture-exact", &addr.to_string()])
        .output()
        .expect("run exact capture CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.starts_with(
            "egui-correlated request=7 viewport_id=2 pass=9 viewport_rect=[0,0,100,50]"
        )
    );
    assert!(stdout.contains("node id=\"ak:1\" role=\"button\" name=\"Apply\""));
    assert!(stdout.contains("paint order=1 kind=\"rect\" bounds=[10,10,20,10]"));
    assert!(stdout.contains("correlation=same_full_output"));
    server.join().expect("mock exact peer exits cleanly");
}

#[cfg(feature = "egui")]
#[test]
fn capture_exact_yaml_preserves_full_correlated_envelope() {
    let (addr, server) = spawn_exact_capture_peer(sample_correlated_capture());
    let output = cli()
        .args(["capture-exact", &addr.to_string(), "--yaml"])
        .output()
        .expect("run exact capture YAML CLI");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("request_id: 7"));
    assert!(stdout.contains("viewport_id: 2"));
    assert!(stdout.contains("pass_nr: 9"));
    assert!(stdout.contains("viewport_rect:"));
    assert!(stdout.contains("witness:"));
    assert!(stdout.contains("paint:"));
    assert!(stdout.contains("kind: rect"));
    server.join().expect("mock exact peer exits cleanly");
}

#[cfg(feature = "egui")]
fn spawn_exact_capture_peer(
    capture: viewwitness::EguiCorrelatedCapture,
) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread,
    };

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock exact capture peer");
    let addr = listener.local_addr().expect("exact peer address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept exact CLI observer");
        writeln!(
            stream,
            "{} {}",
            viewwitness::EGUI_CAPTURE_PROTOCOL_MAGIC,
            viewwitness::EGUI_CAPTURE_PROTOCOL_VERSION
        )
        .expect("write exact capture handshake");

        let mut request = String::new();
        BufReader::new(stream.try_clone().expect("clone exact peer stream"))
            .read_line(&mut request)
            .expect("read exact capture request");
        assert_eq!(request.trim_end(), "CAPTURE");

        let response = serde_json::json!({
            "status": "ok",
            "capture": capture,
        });
        writeln!(stream, "{response}").expect("write exact capture response");
    });
    (addr, server)
}

#[cfg(feature = "egui")]
fn sample_correlated_capture() -> viewwitness::EguiCorrelatedCapture {
    use viewwitness::{
        EguiCorrelatedCapture, EguiPaintKind, EguiPaintObservation, Rect, from_yaml,
    };

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
    .expect("parse exact CLI semantic witness");

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
    }
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
