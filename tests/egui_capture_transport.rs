#![cfg(feature = "egui")]

use std::{
    io::Write,
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

use viewwitness::{
    EguiCaptureObserver, EguiFrameProbe, EguiPaintKind, Rect, run_egui_capture_server,
};

#[test]
fn external_observer_receives_exact_correlated_capture() {
    let ctx = egui::Context::default();
    let probe = EguiFrameProbe::install(&ctx, 1);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind exact capture server");
    let addr = listener.local_addr().expect("capture server address");

    let _server = thread::spawn(move || {
        run_egui_capture_server(listener, probe, Duration::from_millis(500))
            .expect("exact capture server stays healthy");
    });

    let client = thread::spawn(move || {
        let mut observer = EguiCaptureObserver::connect(&addr.to_string())?;
        observer.capture()
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    while !client.is_finished() && Instant::now() < deadline {
        let output = ctx.run_ui(test_input(), |ui| {
            let _ = ui.button("Network Apply");
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(40.0, 70.0), egui::vec2(60.0, 20.0)),
                0.0,
                egui::Color32::WHITE,
            );
        });
        output.drop_without_applying_deltas();
        thread::sleep(Duration::from_millis(1));
    }

    assert!(
        client.is_finished(),
        "network capture must complete while egui is serviced"
    );
    let capture = client
        .join()
        .expect("capture client thread does not panic")
        .expect("external exact capture succeeds");

    assert_eq!(
        capture.viewport_rect,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        }
    );
    assert_eq!(capture.witness.capture.viewport.width, 200.0);
    assert_eq!(capture.witness.capture.viewport.height, 120.0);
    assert_eq!(capture.witness.capture.frame, Some(capture.pass_nr));
    assert_eq!(
        capture.witness.capture.metadata["semantic_paint_correlation"],
        serde_json::json!("same_full_output")
    );
    assert!(
        capture
            .witness
            .nodes
            .iter()
            .any(|node| node.name.as_deref() == Some("Network Apply")),
        "network response preserves canonical semantic evidence"
    );
    assert!(
        capture.paint.iter().any(|observation| {
            observation.kind == EguiPaintKind::Rect
                && observation.bounds
                    == Rect {
                        x: 40.0,
                        y: 70.0,
                        width: 60.0,
                        height: 20.0,
                    }
        }),
        "network response preserves same-pass renderer evidence"
    );
}

#[test]
fn server_capture_timeout_is_reported_to_observer() {
    let ctx = egui::Context::default();
    let probe = EguiFrameProbe::install(&ctx, 1);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind timeout server");
    let addr = listener.local_addr().expect("timeout server address");

    let _server = thread::spawn(move || {
        run_egui_capture_server(listener, probe, Duration::from_millis(20))
            .expect("timeout is a client-visible capture result, not a server failure");
    });

    let mut observer =
        EguiCaptureObserver::connect(&addr.to_string()).expect("connect timeout observer");
    let error = observer
        .capture()
        .expect_err("without an egui pass the exact request must time out");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
}

#[test]
fn observer_rejects_unrelated_protocol_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake server");
    let addr = listener.local_addr().expect("fake server address");

    let fake = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fake client");
        writeln!(stream, "SOMETHING-ELSE 1").expect("write fake handshake");
    });

    let error = EguiCaptureObserver::connect(&addr.to_string())
        .expect_err("observer must reject an alien capture protocol");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    fake.join().expect("fake server thread exits");
}

#[test]
fn observer_rejects_previous_capture_protocol_version() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind old-version server");
    let addr = listener.local_addr().expect("old-version server address");

    let fake = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept old-version client");
        writeln!(stream, "VIEWWITNESS-EGUI-CAPTURE 1").expect("write v1 handshake");
    });

    let error = EguiCaptureObserver::connect(&addr.to_string())
        .expect_err("v2 observer must reject the incompatible v1 envelope");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("VIEWWITNESS-EGUI-CAPTURE 2"));
    fake.join().expect("old-version server thread exits");
}

fn test_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(200.0, 120.0),
        )),
        ..Default::default()
    }
}
