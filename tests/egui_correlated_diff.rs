#![cfg(feature = "egui")]

use std::time::Duration;

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiFrameProbe,
    EguiLayerOrder, EguiPaintAnnotator, EguiPaintKind, EguiPaintObjectDescriptor, Rect,
    diff_correlated_captures, from_yaml,
};

#[test]
fn shape_index_churn_is_diagnostic_not_material_change() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();

    let before = capture_object(&ctx, &mut probe, &annotator, false);
    let after = capture_object(&ctx, &mut probe, &annotator, true);
    let diff = diff_correlated_captures(&before, &after);

    assert!(diff.semantic.is_empty());
    assert!(diff.authored.objects_added.is_empty());
    assert!(diff.authored.objects_removed.is_empty());
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.ambiguous_ids.is_empty());
    assert_eq!(diff.authored.execution_handle_churn.len(), 1);

    let churn = &diff.authored.execution_handle_churn[0];
    assert_eq!(churn.id, "canvas:persistent");
    assert_eq!(churn.binding_ordinal, 0);
    assert_ne!(churn.before_shape_index, churn.after_shape_index);
    assert!(
        diff.is_materially_empty(),
        "renderer slot churn alone must not become a material object change"
    );
}

#[test]
fn duplicate_authored_ids_are_ambiguity_not_implicit_matching() {
    let before = synthetic_capture(
        1,
        vec![
            authored_object("duplicate", "node_a", EguiPaintKind::Rect, 0),
            authored_object("duplicate", "node_b", EguiPaintKind::Circle, 1),
        ],
    );
    let after = synthetic_capture(
        2,
        vec![authored_object(
            "duplicate",
            "node_a",
            EguiPaintKind::Rect,
            3,
        )],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert_eq!(diff.authored.ambiguous_ids.len(), 1);
    let ambiguity = &diff.authored.ambiguous_ids[0];
    assert_eq!(ambiguity.id, "duplicate");
    assert_eq!(ambiguity.before_count, 2);
    assert_eq!(ambiguity.after_count, 1);
    assert!(diff.authored.objects_added.is_empty());
    assert!(diff.authored.objects_removed.is_empty());
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.execution_handle_churn.is_empty());
    assert!(!diff.is_materially_empty());
}

#[test]
fn material_binding_change_is_reported_without_using_shape_index_as_identity() {
    let before = synthetic_capture(
        1,
        vec![authored_object(
            "canvas:node",
            "diagram_node",
            EguiPaintKind::Rect,
            4,
        )],
    );
    let mut changed = authored_object("canvas:node", "diagram_node", EguiPaintKind::Circle, 17);
    changed.bindings[0].bounds = Some(Rect {
        x: 12.0,
        y: 10.0,
        width: 20.0,
        height: 20.0,
    });
    let after = synthetic_capture(2, vec![changed]);

    let diff = diff_correlated_captures(&before, &after);
    assert_eq!(diff.authored.objects_changed.len(), 1);
    let change = &diff.authored.objects_changed[0];
    assert_eq!(change.id, "canvas:node");
    assert_eq!(
        change.fields.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["bindings"]
    );
    assert!(diff.authored.execution_handle_churn.is_empty());
    assert!(!diff.is_materially_empty());
}

fn capture_object(
    ctx: &egui::Context,
    probe: &mut EguiFrameProbe,
    annotator: &EguiPaintAnnotator,
    insert_unrelated_prefix: bool,
) -> EguiCorrelatedCapture {
    use egui::epaint::RectShape;

    probe
        .request_capture()
        .expect("request correlated diff capture");
    let bounds = egui::Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(60.0, 40.0));
    let prefix = egui::Rect::from_min_size(egui::pos2(5.0, 5.0), egui::vec2(10.0, 10.0));

    let output = ctx.run_ui(test_input(), |ui| {
        let painter = ui.painter().clone();
        if insert_unrelated_prefix {
            painter.add(RectShape::filled(prefix, 0.0, egui::Color32::DARK_GRAY));
        }
        annotator.add_shape(
            &painter,
            EguiPaintObjectDescriptor::new("canvas:persistent", "diagram_node")
                .with_name("Persistent node"),
            RectShape::filled(bounds, 0.0, egui::Color32::WHITE),
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive correlated diff capture");
    output.drop_without_applying_deltas();
    evidence
        .into_correlated_capture()
        .expect("convert correlated diff capture")
}

fn synthetic_capture(
    pass_nr: u64,
    authored_objects: Vec<EguiAuthoredPaintObject>,
) -> EguiCorrelatedCapture {
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
    .expect("parse synthetic semantic witness");

    EguiCorrelatedCapture {
        request_id: pass_nr,
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
        authored_objects,
    }
}

fn authored_object(
    id: &str,
    role: &str,
    kind: EguiPaintKind,
    shape_index: usize,
) -> EguiAuthoredPaintObject {
    EguiAuthoredPaintObject {
        id: id.into(),
        role: role.into(),
        name: None,
        semantic_evidence: "intended".into(),
        bindings: vec![EguiAuthoredPaintBinding {
            binding_evidence: "observed".into(),
            layer_order: EguiLayerOrder::Background,
            layer_id: 42,
            shape_index,
            verified_at_end_pass: true,
            kind: Some(kind),
            bounds: Some(Rect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            }),
            clip_rect: None,
        }],
    }
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
