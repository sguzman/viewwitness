#![cfg(feature = "observer")]

use std::{net::TcpListener, thread};

use egui::accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate};
use egui_inspection::{EncodedPng, Request, Response, read_message, write_message};
use viewwitness::InspectionObserver;

#[test]
fn observer_captures_a_live_protocol_tree() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock inspection peer");
    let addr = listener.local_addr().expect("listener address");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept observer");
        egui_inspection::protocol::write_handshake(&mut stream).expect("write handshake");

        let request: Request = read_message(&mut stream).expect("read request");
        assert!(matches!(request, Request::GetTree));

        write_message(
            &mut stream,
            &Response::Tree {
                step: 41,
                pixels_per_point: 1.5,
                accesskit: Some(mock_tree()),
            },
        )
        .expect("write tree response");
    });

    let mut observer = InspectionObserver::connect(&addr.to_string()).expect("connect observer");
    let witness = observer
        .capture()
        .expect("capture succeeds")
        .expect("tree exists");

    assert!(witness.validation_issues().is_empty());
    assert_eq!(witness.capture.source, "egui");
    assert_eq!(witness.capture.frame, Some(41));
    assert_eq!(witness.capture.viewport.width, 800.0);
    assert_eq!(witness.capture.viewport.height, 600.0);
    assert_eq!(witness.capture.viewport.scale_factor, 1.5);
    assert_eq!(
        witness.capture.metadata["transport"],
        serde_json::json!("egui_inspection")
    );

    let apply = witness
        .nodes
        .iter()
        .find(|node| node.name.as_deref() == Some("Apply"))
        .expect("button captured");
    let identity = apply.identity.as_ref().expect("identity evidence attached");
    assert_eq!(identity.provenance, "accesskit_node_id");
    assert_eq!(identity.stability, "structure_sensitive");
    assert_eq!(identity.author_id.as_deref(), Some("apply"));

    server.join().expect("mock peer exits cleanly");
}

#[test]
fn settle_and_capture_preserves_non_idle_evidence_and_still_captures() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock inspection peer");
    let addr = listener.local_addr().expect("listener address");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept observer");
        egui_inspection::protocol::write_handshake(&mut stream).expect("write handshake");

        let request: Request = read_message(&mut stream).expect("read settle request");
        assert!(matches!(request, Request::Settle { max_steps: 5 }));
        write_message(
            &mut stream,
            &Response::Settled {
                settled: false,
                steps: 5,
            },
        )
        .expect("write settle response");

        let request: Request = read_message(&mut stream).expect("read tree request");
        assert!(matches!(request, Request::GetTree));
        write_message(
            &mut stream,
            &Response::Tree {
                step: 46,
                pixels_per_point: 1.5,
                accesskit: Some(mock_tree()),
            },
        )
        .expect("write tree response");
    });

    let mut observer = InspectionObserver::connect(&addr.to_string()).expect("connect observer");
    let (settle, witness) = observer
        .settle_and_capture(5)
        .expect("settle and capture succeeds");

    assert!(!settle.settled);
    assert_eq!(settle.steps, 5);
    assert_eq!(
        witness.expect("tree exists").capture.frame,
        Some(46),
        "a non-idle settle result must not discard observable GUI state"
    );

    server.join().expect("mock peer exits cleanly");
}

#[test]
fn screenshot_preserves_raster_evidence_and_rejects_invalid_scale_locally() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock inspection peer");
    let addr = listener.local_addr().expect("listener address");

    let expected_bytes = vec![137, 80, 78, 71, 13, 10, 26, 10, 1, 2, 3, 4];
    let server_bytes = expected_bytes.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept observer");
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
                size: [320, 180],
                bytes: server_bytes,
            }),
        )
        .expect("write screenshot response");
    });

    let mut observer = InspectionObserver::connect(&addr.to_string()).expect("connect observer");
    let error = observer
        .screenshot(Some(0.0))
        .expect_err("zero screenshot scale must be rejected locally");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);

    let screenshot = observer.screenshot(Some(1.0)).expect("screenshot succeeds");
    assert_eq!(screenshot.size, [320, 180]);
    assert_eq!(screenshot.png_bytes, expected_bytes);

    server.join().expect("mock peer exits cleanly");
}

fn mock_tree() -> TreeUpdate {
    let root_id = NodeId(1);
    let button_id = NodeId(2);

    let mut root = Node::new(Role::Window);
    root.set_label("Mock app");
    root.set_children(vec![button_id]);
    root.set_bounds(egui::accesskit::Rect {
        x0: 0.0,
        y0: 0.0,
        x1: 800.0,
        y1: 600.0,
    });

    let mut button = Node::new(Role::Button);
    button.set_label("Apply");
    button.set_author_id("apply");
    button.set_bounds(egui::accesskit::Rect {
        x0: 20.0,
        y0: 30.0,
        x1: 100.0,
        y1: 60.0,
    });

    TreeUpdate {
        nodes: vec![(root_id, root), (button_id, button)],
        tree: Some(Tree::new(root_id)),
        tree_id: TreeId::ROOT,
        focus: root_id,
    }
}
