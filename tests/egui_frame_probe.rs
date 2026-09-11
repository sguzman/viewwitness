#![cfg(feature = "egui")]

use std::{io::ErrorKind, time::Duration};

use viewwitness::{
    EguiCaptureContext, EguiFrameProbe, EguiPaintKind, Rect, Viewport,
    paint_observations_from_egui_output, witness_from_egui_tree_update,
};

#[test]
fn requested_probe_captures_semantics_and_paint_from_one_exact_output() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let request_id = probe.request_capture().expect("request correlated capture");

    let output = ctx.run_ui(test_input(), |ui| {
        ui.button("Apply");
        ui.painter().rect_filled(
            egui::Rect::from_min_size(egui::pos2(30.0, 70.0), egui::vec2(50.0, 20.0)),
            0.0,
            egui::Color32::WHITE,
        );
    });
    let expected_paint = paint_observations_from_egui_output(&output);
    let expected_tree = output
        .platform_output
        .accesskit_update
        .as_ref()
        .expect("probe installation enables AccessKit");
    let expected_button_id = expected_tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Apply"))
        .map(|(id, _)| *id)
        .expect("button exists in frame AccessKit output");

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive exact same-pass evidence");

    assert_eq!(evidence.request_id, request_id);
    assert_eq!(evidence.pass_nr, ctx.cumulative_pass_nr());
    assert_eq!(evidence.viewport_id, ctx.viewport_id().0.value());
    assert_eq!(evidence.pixels_per_point, output.pixels_per_point);
    assert_eq!(evidence.paint, expected_paint);
    assert!(evidence.paint.iter().any(|observation| {
        observation.kind == EguiPaintKind::Rect
            && observation.bounds
                == Rect {
                    x: 30.0,
                    y: 70.0,
                    width: 50.0,
                    height: 20.0,
                }
    }));

    let captured_tree = evidence.accesskit.as_ref().expect("semantic evidence copied");
    assert_eq!(captured_tree.nodes.len(), expected_tree.nodes.len());
    assert!(
        captured_tree
            .nodes
            .iter()
            .any(|(id, node)| *id == expected_button_id && node.label() == Some("Apply")),
        "the copied semantic tree contains the same frame's button identity"
    );

    let witness = witness_from_egui_tree_update(
        captured_tree,
        EguiCaptureContext::new(Viewport {
            width: 200.0,
            height: 120.0,
            scale_factor: evidence.pixels_per_point,
        })
        .with_frame(evidence.pass_nr),
    );
    assert!(
        witness
            .nodes
            .iter()
            .any(|node| node.name.as_deref() == Some("Apply")),
        "raw same-pass semantic evidence remains convertible off the GUI hook"
    );

    output.drop_without_applying_deltas();
}

#[test]
fn full_probe_response_queue_drops_request_instead_of_blocking_gui_pass() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);

    probe.request_capture().expect("request first capture");
    let timeout = probe
        .recv_timeout(Duration::from_millis(1))
        .expect_err("first request intentionally times out before a GUI pass");
    assert_eq!(timeout.kind(), ErrorKind::TimedOut);

    // The timed-out request is still observed by the next output hook, so its
    // now-stale response occupies the one-slot queue.
    let stale_output = ctx.run_ui(test_input(), |ui| {
        ui.label("stale response");
    });
    stale_output.drop_without_applying_deltas();

    let second_request = probe.request_capture().expect("request second capture");
    let live_output = ctx.run_ui(test_input(), |ui| {
        ui.label("rendering must still complete");
    });
    live_output.drop_without_applying_deltas();

    let dropped = probe
        .recv_timeout(Duration::from_millis(100))
        .expect_err("full response queue must drop instead of backpressuring egui");
    assert_eq!(dropped.kind(), ErrorKind::WouldBlock);
    assert!(
        dropped.to_string().contains(&second_request.to_string()),
        "drop evidence identifies the affected request"
    );
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
