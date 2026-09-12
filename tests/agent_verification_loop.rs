#![cfg(feature = "egui")]

use std::time::Duration;

use viewwitness::{
    EguiCorrelatedCapture, EguiFrameProbe, EguiPaintAnnotator, EguiPaintObjectDescriptor,
    correlated_capture_to_agent_text, correlated_diff_to_agent_text, diff_correlated_captures,
};

#[test]
fn mispositioned_keyed_handle_can_be_fixed_and_verified_without_body_noise() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();

    let broken = capture_scene(&ctx, &mut probe, &annotator, true, false);
    let fixed = capture_scene(&ctx, &mut probe, &annotator, false, true);

    let broken_text = correlated_capture_to_agent_text(&broken);
    assert!(broken_text.contains(
        "authored-binding object_index=0 binding_index=0 object_id=\"agent-loop:node\" authored_binding_id=\"body\""
    ));
    assert!(broken_text.contains("authored_binding_id=\"handle\""));
    assert!(broken_text.contains("bounds=[145,45,10,10]"));

    let fixed_text = correlated_capture_to_agent_text(&fixed);
    assert!(fixed_text.contains("authored_binding_id=\"handle\""));
    assert!(fixed_text.contains("bounds=[95,45,10,10]"));

    let diff = diff_correlated_captures(&broken, &fixed);
    assert!(diff.semantic.is_empty());
    assert!(diff.authored.objects_added.is_empty());
    assert!(diff.authored.objects_removed.is_empty());
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.bindings_added.is_empty());
    assert!(diff.authored.bindings_removed.is_empty());
    assert!(diff.authored.binding_ambiguities.is_empty());
    assert_eq!(diff.authored.bindings_changed.len(), 1);

    let handle_change = &diff.authored.bindings_changed[0];
    assert_eq!(handle_change.object_id, "agent-loop:node");
    assert_eq!(
        handle_change.authored_binding_id.as_deref(),
        Some("handle")
    );
    assert_eq!(handle_change.unkeyed_ordinal, None);
    assert_eq!(
        handle_change
            .fields
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["bounds"]
    );

    assert_eq!(diff.authored.execution_handle_churn.len(), 1);
    let body_churn = &diff.authored.execution_handle_churn[0];
    assert_eq!(body_churn.id, "agent-loop:node");
    assert_eq!(body_churn.authored_binding_id.as_deref(), Some("body"));
    assert_ne!(body_churn.before_shape_index, body_churn.after_shape_index);
    assert!(!diff.is_materially_empty());

    let diff_text = correlated_diff_to_agent_text(&diff);
    assert!(diff_text.contains(
        "authored-binding-change object_id=\"agent-loop:node\" authored_binding_id=\"handle\" field=\"bounds\""
    ));
    assert!(
        diff_text
            .contains("authored-handle-churn id=\"agent-loop:node\" authored_binding_id=\"body\"")
    );
    assert!(!diff_text.contains(
        "authored-binding-change object_id=\"agent-loop:node\" authored_binding_id=\"body\""
    ));
    assert!(!diff_text.contains("field=\"bindings\""));
}

fn capture_scene(
    ctx: &egui::Context,
    probe: &mut EguiFrameProbe,
    annotator: &EguiPaintAnnotator,
    broken_handle: bool,
    insert_unrelated_prefix: bool,
) -> EguiCorrelatedCapture {
    use egui::epaint::{CircleShape, RectShape};

    probe
        .request_capture()
        .expect("request agent verification capture");

    let body_bounds = egui::Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(60.0, 40.0));
    let handle_center = if broken_handle {
        egui::pos2(150.0, 50.0)
    } else {
        egui::pos2(100.0, 50.0)
    };
    let prefix_bounds = egui::Rect::from_min_size(egui::pos2(5.0, 5.0), egui::vec2(10.0, 10.0));

    let output = ctx.run_ui(test_input(), |ui| {
        let painter = ui.painter().clone();
        if insert_unrelated_prefix {
            painter.add(RectShape::filled(
                prefix_bounds,
                0.0,
                egui::Color32::DARK_GRAY,
            ));
        }

        annotator.paint_object(
            EguiPaintObjectDescriptor::new("agent-loop:node", "diagram_node")
                .with_name("Resizable node"),
            |object| {
                object.add_shape_with_id(
                    &painter,
                    "body",
                    RectShape::filled(body_bounds, 0.0, egui::Color32::WHITE),
                );
                object.add_shape_with_id(
                    &painter,
                    "handle",
                    CircleShape::filled(handle_center, 5.0, egui::Color32::GRAY),
                );
            },
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive agent verification capture");
    output.drop_without_applying_deltas();
    evidence
        .into_correlated_capture()
        .expect("convert agent verification capture")
}

fn test_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(220.0, 120.0),
        )),
        ..Default::default()
    }
}
