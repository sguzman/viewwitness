#![cfg(feature = "egui")]

use std::time::Duration;

use viewwitness::{
    EguiAuthoredPaintObject, EguiFrameProbe, EguiPaintAnnotator, EguiPaintObjectDescriptor,
};

#[test]
fn authored_object_id_is_continuity_evidence_while_shape_index_is_structure_sensitive() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();

    let first = capture_object(&ctx, &mut probe, &annotator, false);
    let second = capture_object(&ctx, &mut probe, &annotator, false);
    let shifted = capture_object(&ctx, &mut probe, &annotator, true);

    assert_eq!(first.id, "canvas:persistent");
    assert_eq!(second.id, first.id);
    assert_eq!(shifted.id, first.id);
    assert_eq!(first.bindings.len(), 1);
    assert_eq!(second.bindings.len(), 1);
    assert_eq!(shifted.bindings.len(), 1);

    let first_binding = &first.bindings[0];
    let second_binding = &second.bindings[0];
    let shifted_binding = &shifted.bindings[0];

    assert_eq!(first_binding.layer_id, second_binding.layer_id);
    assert_eq!(second_binding.layer_id, shifted_binding.layer_id);
    assert_eq!(
        first_binding.shape_index, second_binding.shape_index,
        "unchanged paint structure should reproduce the same layer-local slot"
    );
    assert_ne!(
        second_binding.shape_index, shifted_binding.shape_index,
        "inserting unrelated paint before the authored object must expose that ShapeIdx is structure-sensitive"
    );

    assert_eq!(first_binding.kind, second_binding.kind);
    assert_eq!(second_binding.kind, shifted_binding.kind);
    assert_eq!(first_binding.bounds, second_binding.bounds);
    assert_eq!(second_binding.bounds, shifted_binding.bounds);
}

fn capture_object(
    ctx: &egui::Context,
    probe: &mut EguiFrameProbe,
    annotator: &EguiPaintAnnotator,
    insert_unrelated_prefix: bool,
) -> EguiAuthoredPaintObject {
    use egui::epaint::RectShape;

    probe
        .request_capture()
        .expect("request authored cross-frame capture");

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
        .expect("receive authored cross-frame evidence");
    output.drop_without_applying_deltas();

    assert_eq!(evidence.authored_objects.len(), 1);
    evidence.authored_objects.into_iter().next().unwrap()
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
