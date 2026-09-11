#![cfg(feature = "egui")]

use std::time::Duration;

use egui::layers::ShapeIdx;
use viewwitness::{EguiFrameProbe, EguiPaintKind, EguiPaintObjectDescriptor};

#[test]
fn reset_slot_and_missing_slot_remain_distinct_authored_binding_states() {
    use egui::epaint::RectShape;

    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();
    probe
        .request_capture()
        .expect("request reset/missing-handle capture");

    let bounds =
        egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(50.0, 30.0));

    let output = ctx.run_ui(test_input(), |ui| {
        let painter = ui.painter().clone();

        annotator.paint_object(
            EguiPaintObjectDescriptor::new("canvas:reset", "diagram_node")
                .with_name("Reset node"),
            |object| {
                let live = object.add_shape(
                    &painter,
                    RectShape::filled(bounds, 0.0, egui::Color32::WHITE),
                );

                // Preserve the real handle but explicitly replace its final
                // contents with a no-op shape.
                painter.set(live, egui::epaint::Shape::Noop);

                // Bind an impossible slot on the same real layer. The resolver
                // must preserve this as unverified evidence rather than
                // pretending it is equivalent to a verified no-op.
                object.bind_shape(&painter, ShapeIdx(10_000));
            },
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive reset/missing-handle evidence");
    output.drop_without_applying_deltas();

    assert_eq!(evidence.authored_objects.len(), 1);
    let object = &evidence.authored_objects[0];
    assert_eq!(object.id, "canvas:reset");
    assert_eq!(object.bindings.len(), 2);

    let reset = &object.bindings[0];
    assert!(reset.verified_at_end_pass);
    assert_eq!(reset.kind, Some(EguiPaintKind::Noop));
    assert_eq!(reset.visible_fraction(), 0.0);

    let missing = &object.bindings[1];
    assert!(!missing.verified_at_end_pass);
    assert_eq!(missing.kind, None);
    assert_eq!(missing.bounds, None);
    assert_eq!(missing.clip_rect, None);
    assert_eq!(missing.visible_fraction(), 0.0);

    assert_ne!(
        reset.verified_at_end_pass, missing.verified_at_end_pass,
        "verified no-op paint must remain distinguishable from an unresolved handle"
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
