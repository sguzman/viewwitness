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
    assert_eq!(fs::read(&path).expect("read screenshot output"), expected_bytes);
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8"),
        format!("saved {path_arg} 2x1\n")
    );
    fs::remove_file(path).expect("remove screenshot output");
    server.join().expect("mock peer exits cleanly");
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
