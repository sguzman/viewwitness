#![cfg(feature = "egui")]

use std::time::Duration;

use viewwitness::{EguiAuthoredPaintObject, EguiFrameProbe, EguiPaintAnnotator, EguiPaintObjectDescriptor};

#[test]
fn authored_binding_ids_survive_reordered_submissions() {
    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();

    let first = capture_object(&ctx, &mut probe, &annotator, false);
    let reversed = capture_object(&ctx, &mut probe, &annotator, true);

    assert_eq!(first.id, reversed.id);
    assert_eq!(first.bindings.len(), 2);
    assert_eq!(reversed.bindings.len(), 2);

    let first_ids: Vec<_> = first
        .bindings
        .iter()
        .map(|binding| binding.authored_binding_id.as_deref())
        .collect();
    let reversed_ids: Vec<_> = reversed
        .bindings
        .iter()
        .map(|binding| binding.authored_binding_id.as_deref())
        .collect();

    assert_eq!(first_ids, vec![Some("outline"), Some("handle")]);
    assert_eq!(reversed_ids, vec![Some("handle"), Some("outline")]);

    let first_outline = first
        .bindings
        .iter()
        .find(|binding| binding.authored_binding_id.as_deref() == Some("outline"))
        .expect("first outline binding");
    let reversed_outline = reversed
        .bindings
        .iter()
        .find(|binding| binding.authored_binding_id.as_deref() == Some("outline"))
        .expect("reversed outline binding");
    assert_eq!(first_outline.kind, reversed_outline.kind);
    assert_eq!(first_outline.bounds, reversed_outline.bounds);
    assert_ne!(
        first_outline.shape_index, reversed_outline.shape_index,
        "reordering submissions may change the exact-pass slot while authored binding identity persists"
    );
}

fn capture_object(
    ctx: &egui::Context,
    probe: &mut EguiFrameProbe,
    annotator: &EguiPaintAnnotator,
    reverse: bool,
) -> EguiAuthoredPaintObject {
    use egui::epaint::{CircleShape, RectShape};

    probe
        .request_capture()
        .expect("request authored binding identity capture");

    let outline_bounds =
        egui::Rect::from_min_size(egui::pos2(30.0, 30.0), egui::vec2(70.0, 50.0));
    let handle_center = egui::pos2(95.0, 75.0);

    let output = ctx.run_ui(test_input(), |ui| {
        let painter = ui.painter().clone();
        annotator.paint_object(
            EguiPaintObjectDescriptor::new("canvas:keyed", "diagram_node")
                .with_name("Keyed node"),
            |object| {
                let mut add_outline = || {
                    object.add_shape_with_id(
                        &painter,
                        "outline",
                        RectShape::stroke(
                            outline_bounds,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                            egui::StrokeKind::Middle,
                        ),
                    );
                };
                let mut add_handle = || {
                    object.add_shape_with_id(
                        &painter,
                        "handle",
                        CircleShape::filled(handle_center, 5.0, egui::Color32::GRAY),
                    );
                };

                if reverse {
                    add_handle();
                    add_outline();
                } else {
                    add_outline();
                    add_handle();
                }
            },
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive authored binding identity evidence");
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
