#![cfg(feature = "egui")]

use std::time::Duration;

use egui::{Id, LayerId, Order};
use viewwitness::{EguiFrameProbe, EguiLayerOrder, EguiPaintKind, EguiPaintObjectDescriptor, Rect};

#[test]
fn one_authored_object_can_bind_shapes_across_distinct_egui_layers() {
    use egui::epaint::RectShape;

    let ctx = egui::Context::default();
    let mut probe = EguiFrameProbe::install(&ctx, 1);
    let annotator = probe.annotator();
    probe.request_capture().expect("request multi-layer capture");

    let background_layer = LayerId::new(Order::Background, Id::new("viewwitness-test-background"));
    let foreground_layer = LayerId::new(Order::Foreground, Id::new("viewwitness-test-foreground"));
    let background_bounds =
        egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(60.0, 30.0));
    let foreground_bounds =
        egui::Rect::from_min_size(egui::pos2(30.0, 25.0), egui::vec2(40.0, 20.0));

    let output = ctx.run_ui(test_input(), |ui| {
        let background = ui.ctx().layer_painter(background_layer);
        let foreground = ui.ctx().layer_painter(foreground_layer);

        annotator.paint_object(
            EguiPaintObjectDescriptor::new("canvas:layered", "diagram_node")
                .with_name("Layered node"),
            |object| {
                object.add_shape(
                    &background,
                    RectShape::filled(background_bounds, 0.0, egui::Color32::DARK_GRAY),
                );
                object.add_shape(
                    &foreground,
                    RectShape::filled(foreground_bounds, 0.0, egui::Color32::WHITE),
                );
            },
        );
    });

    let evidence = probe
        .recv_timeout(Duration::from_millis(100))
        .expect("receive multi-layer authored evidence");
    output.drop_without_applying_deltas();

    assert_eq!(evidence.authored_objects.len(), 1);
    let object = &evidence.authored_objects[0];
    assert_eq!(object.id, "canvas:layered");
    assert_eq!(object.bindings.len(), 2);

    let background = &object.bindings[0];
    let foreground = &object.bindings[1];

    assert!(background.verified_at_end_pass);
    assert!(foreground.verified_at_end_pass);
    assert_eq!(background.layer_order, EguiLayerOrder::Background);
    assert_eq!(foreground.layer_order, EguiLayerOrder::Foreground);
    assert_eq!(background.layer_id, background_layer.id.value());
    assert_eq!(foreground.layer_id, foreground_layer.id.value());
    assert_ne!(background.layer_id, foreground.layer_id);
    assert_eq!(background.kind, Some(EguiPaintKind::Rect));
    assert_eq!(foreground.kind, Some(EguiPaintKind::Rect));
    assert_eq!(
        background.bounds,
        Some(Rect {
            x: 20.0,
            y: 20.0,
            width: 60.0,
            height: 30.0,
        })
    );
    assert_eq!(
        foreground.bounds,
        Some(Rect {
            x: 30.0,
            y: 25.0,
            width: 40.0,
            height: 20.0,
        })
    );

    assert!(
        !evidence.paint.is_empty(),
        "generic flattened paint should still coexist with authored layer-local bindings"
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
