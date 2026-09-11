#![cfg(feature = "egui")]

use std::{io::ErrorKind, time::Duration};

use viewwitness::{
    EguiFrameProbe, EguiLayerOrder, EguiPaintKind, EguiPaintObjectDescriptor, Rect,
    paint_observations_from_egui_output,
};

#[test]
fn requested_probe_captures_semantics_paint_and_viewport_from_one_exact_output() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let request_id = probe.request_capture().expect("request correlated capture");

    let output = ctx.run_ui(test_input(), |ui| {
        let _ = ui.button("Apply");
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
    let expected_tree_len = expected_tree.nodes.len();
    let expected_button_id = expected_tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Apply"))
        .map(|(id, _)| *id)
        .expect("button exists in frame AccessKit output");

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive exact same-pass evidence");

    output.drop_without_applying_deltas();

    assert_eq!(evidence.request_id, request_id);
    assert_eq!(evidence.pass_nr, ctx.cumulative_pass_nr());
    assert_eq!(evidence.viewport_id, ctx.viewport_id().0.value());
    assert_eq!(evidence.pixels_per_point, 1.0);
    assert_eq!(
        evidence.viewport_rect,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        }
    );
    assert_eq!(evidence.paint, expected_paint);
    assert!(evidence.authored_objects.is_empty());
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

    let captured_tree = evidence
        .accesskit
        .as_ref()
        .expect("semantic evidence copied");
    assert_eq!(captured_tree.nodes.len(), expected_tree_len);
    assert!(
        captured_tree
            .nodes
            .iter()
            .any(|(id, node)| *id == expected_button_id && node.label() == Some("Apply")),
        "the copied semantic tree contains the same frame's button identity"
    );

    let mut invalid = evidence.clone();
    invalid.viewport_rect.width = f32::NAN;
    let invalid_error = invalid
        .into_correlated_capture()
        .expect_err("invalid observed viewport geometry must not be guessed around");
    assert_eq!(invalid_error.kind(), ErrorKind::InvalidData);

    let capture = evidence
        .into_correlated_capture()
        .expect("convert same-pass evidence off the GUI hook");
    assert_eq!(capture.request_id, request_id);
    assert_eq!(capture.pass_nr, ctx.cumulative_pass_nr());
    assert_eq!(
        capture.viewport_rect,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        }
    );
    assert_eq!(capture.paint, expected_paint);
    assert!(capture.authored_objects.is_empty());
    assert_eq!(capture.witness.capture.frame, Some(capture.pass_nr));
    assert_eq!(capture.witness.capture.viewport.width, 200.0);
    assert_eq!(capture.witness.capture.viewport.height, 120.0);
    assert_eq!(
        capture.witness.capture.metadata["frame_clock"],
        serde_json::json!("egui_cumulative_pass_nr")
    );
    assert_eq!(
        capture.witness.capture.metadata["semantic_paint_correlation"],
        serde_json::json!("same_full_output")
    );
    assert_eq!(
        capture.witness.capture.metadata["viewport_evidence_source"],
        serde_json::json!("egui_input_state_viewport_rect")
    );
    assert_eq!(
        capture.witness.capture.metadata["egui_viewport_rect"],
        serde_json::json!({
            "x": 0.0,
            "y": 0.0,
            "width": 200.0,
            "height": 120.0,
        })
    );
    assert!(
        capture
            .witness
            .nodes
            .iter()
            .any(|node| node.name.as_deref() == Some("Apply")),
        "same-pass semantic evidence becomes a canonical witness"
    );
}

#[test]
fn authored_custom_paint_binds_identity_to_real_shape_slots_not_geometry() {
    use egui::epaint::{CircleShape, RectShape};

    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();
    probe.request_capture().expect("request annotated capture");

    let shared_rect = egui::Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(40.0, 40.0));
    let mut first_shape_index = None;
    let mut second_shape_index = None;

    let output = ctx.run_ui(test_input(), |ui| {
        let painter = ui.painter().clone();

        let first = annotator.add_shape(
            &painter,
            EguiPaintObjectDescriptor::new("canvas:first", "diagram_node").with_name("First"),
            RectShape::filled(shared_rect, 0.0, egui::Color32::DARK_GRAY),
        );
        first_shape_index = Some(first.0);

        painter.set(
            first,
            CircleShape {
                center: shared_rect.center(),
                radius: 20.0,
                fill: egui::Color32::WHITE,
                stroke: egui::Stroke::NONE,
            },
        );

        let second = annotator.add_shape(
            &painter,
            EguiPaintObjectDescriptor::new("canvas:second", "diagram_node").with_name("Second"),
            RectShape::filled(shared_rect, 0.0, egui::Color32::GRAY),
        );
        second_shape_index = Some(second.0);
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive authored paint evidence");
    output.drop_without_applying_deltas();

    assert_eq!(evidence.authored_objects.len(), 2);
    let first = evidence
        .authored_objects
        .iter()
        .find(|object| object.id == "canvas:first")
        .expect("first authored object");
    let second = evidence
        .authored_objects
        .iter()
        .find(|object| object.id == "canvas:second")
        .expect("second authored object");
    let first_binding = first.bindings.first().expect("first binding");
    let second_binding = second.bindings.first().expect("second binding");

    assert_eq!(first.semantic_evidence, "intended");
    assert_eq!(first.bindings.len(), 1);
    assert_eq!(first_binding.binding_evidence, "observed");
    assert_eq!(first_binding.layer_order, EguiLayerOrder::Background);
    assert!(first_binding.verified_at_end_pass);
    assert_eq!(
        first_binding.shape_index,
        first_shape_index.expect("first shape index")
    );
    assert_eq!(first_binding.kind, Some(EguiPaintKind::Circle));

    assert_eq!(second.semantic_evidence, "intended");
    assert_eq!(second.bindings.len(), 1);
    assert_eq!(second_binding.binding_evidence, "observed");
    assert_eq!(second_binding.layer_order, EguiLayerOrder::Background);
    assert!(second_binding.verified_at_end_pass);
    assert_eq!(
        second_binding.shape_index,
        second_shape_index.expect("second shape index")
    );
    assert_eq!(second_binding.kind, Some(EguiPaintKind::Rect));

    assert_ne!(
        first_binding.shape_index, second_binding.shape_index,
        "overlapping identical bounds must remain distinct through exact paint handles"
    );
    assert_eq!(first_binding.bounds, second_binding.bounds);

    let capture = evidence
        .into_correlated_capture()
        .expect("convert authored capture");
    assert_eq!(capture.authored_objects.len(), 2);
    assert_eq!(
        capture.witness.capture.metadata["authored_paint_evidence"],
        serde_json::json!("application_semantics_plus_verified_egui_paint_handles")
    );
}

#[test]
fn explicit_multi_shape_object_keeps_one_identity_and_per_binding_clip_evidence() {
    use egui::epaint::{CircleShape, RectShape};

    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();
    probe.request_capture().expect("request composite capture");

    let bounds = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(40.0, 40.0));
    let half_clip = egui::Rect::from_min_size(egui::pos2(30.0, 20.0), egui::vec2(20.0, 40.0));
    let mut expected_indices = Vec::new();

    let output = ctx.run_ui(test_input(), |ui| {
        let painter = ui.painter().clone();
        let clipped_painter = painter.with_clip_rect(half_clip);

        annotator.paint_object(
            EguiPaintObjectDescriptor::new("canvas:composite", "diagram_node")
                .with_name("Composite node"),
            |object| {
                let body = object.add_shape(
                    &painter,
                    RectShape::filled(bounds, 0.0, egui::Color32::DARK_GRAY),
                );
                expected_indices.push(body.0);

                let accent = object.add_shape(
                    &clipped_painter,
                    CircleShape {
                        center: bounds.center(),
                        radius: 20.0,
                        fill: egui::Color32::WHITE,
                        stroke: egui::Stroke::NONE,
                    },
                );
                expected_indices.push(accent.0);
            },
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive composite authored evidence");
    output.drop_without_applying_deltas();

    assert_eq!(evidence.authored_objects.len(), 1);
    let object = evidence
        .authored_objects
        .first()
        .expect("one explicit composite object");
    assert_eq!(object.id, "canvas:composite");
    assert_eq!(object.role, "diagram_node");
    assert_eq!(object.name.as_deref(), Some("Composite node"));
    assert_eq!(object.semantic_evidence, "intended");
    assert_eq!(object.bindings.len(), 2);

    let body = &object.bindings[0];
    let accent = &object.bindings[1];
    assert_eq!(body.shape_index, expected_indices[0]);
    assert_eq!(accent.shape_index, expected_indices[1]);
    assert_ne!(body.shape_index, accent.shape_index);
    assert!(body.verified_at_end_pass);
    assert!(accent.verified_at_end_pass);
    assert_eq!(body.kind, Some(EguiPaintKind::Rect));
    assert_eq!(accent.kind, Some(EguiPaintKind::Circle));
    assert_eq!(body.bounds, accent.bounds);

    assert_eq!(body.visible_fraction(), 1.0);
    assert_eq!(
        accent.clip_rect,
        Some(Rect {
            x: 30.0,
            y: 20.0,
            width: 20.0,
            height: 40.0,
        })
    );
    assert_eq!(
        accent.visible_bounds(),
        Some(Rect {
            x: 30.0,
            y: 20.0,
            width: 20.0,
            height: 40.0,
        })
    );
    assert_eq!(accent.visible_fraction(), 0.5);
}

#[test]
fn duplicate_ids_are_not_implicitly_grouped_across_independent_annotations() {
    use egui::epaint::RectShape;

    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();
    probe
        .request_capture()
        .expect("request duplicate-id capture");

    let first_rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(20.0, 20.0));
    let second_rect = egui::Rect::from_min_size(egui::pos2(40.0, 10.0), egui::vec2(20.0, 20.0));

    let output = ctx.run_ui(test_input(), |ui| {
        annotator.add_shape(
            ui.painter(),
            EguiPaintObjectDescriptor::new("duplicate", "first"),
            RectShape::filled(first_rect, 0.0, egui::Color32::WHITE),
        );
        annotator.add_shape(
            ui.painter(),
            EguiPaintObjectDescriptor::new("duplicate", "second"),
            RectShape::filled(second_rect, 0.0, egui::Color32::GRAY),
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive duplicate-id evidence");
    output.drop_without_applying_deltas();

    assert_eq!(evidence.authored_objects.len(), 2);
    assert!(
        evidence
            .authored_objects
            .iter()
            .all(|object| object.id == "duplicate" && object.bindings.len() == 1),
        "repeated authored IDs must remain two explicit object records unless the application grouped them"
    );
}

#[test]
fn annotations_are_not_collected_on_unrequested_passes() {
    use egui::epaint::RectShape;

    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(20.0, 20.0));

    let ordinary = ctx.run_ui(test_input(), |ui| {
        annotator.add_shape(
            ui.painter(),
            EguiPaintObjectDescriptor::new("ordinary", "shape"),
            RectShape::filled(rect, 0.0, egui::Color32::WHITE),
        );
    });
    ordinary.drop_without_applying_deltas();

    probe.request_capture().expect("request later capture");
    let requested = ctx.run_ui(test_input(), |ui| {
        ui.label("requested pass contains no authored custom object");
    });
    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive requested pass");
    requested.drop_without_applying_deltas();

    assert!(
        evidence.authored_objects.is_empty(),
        "ordinary-pass annotations must not leak forward into an exact request"
    );
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
