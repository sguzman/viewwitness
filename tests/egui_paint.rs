#![cfg(feature = "egui")]

use viewwitness::{EguiPaintKind, Rect, paint_observations_from_egui_output};

#[test]
fn paint_probe_preserves_visual_bounds_clip_and_renderer_order() {
    let ctx = egui::Context::default();
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            ..Default::default()
        },
        |ui| {
            let clip = egui::Rect::from_min_max(egui::pos2(40.0, 30.0), egui::pos2(90.0, 70.0));
            let painter = ui.painter().with_clip_rect(clip);
            painter.rect_filled(
                egui::Rect::from_min_max(egui::pos2(20.0, 20.0), egui::pos2(120.0, 90.0)),
                0.0,
                egui::Color32::WHITE,
            );
            painter.circle_filled(egui::pos2(200.0, 100.0), 10.0, egui::Color32::WHITE);
        },
    );

    let observations = paint_observations_from_egui_output(&output);
    let clipped_rect = observations
        .iter()
        .find(|observation| {
            observation.kind == EguiPaintKind::Rect
                && observation.bounds
                    == Rect {
                        x: 20.0,
                        y: 20.0,
                        width: 100.0,
                        height: 70.0,
                    }
        })
        .expect("custom painted rectangle appears in renderer output");

    assert_eq!(
        clipped_rect.clip_rect,
        Some(Rect {
            x: 40.0,
            y: 30.0,
            width: 50.0,
            height: 40.0,
        })
    );
    assert_eq!(
        clipped_rect.visible_bounds(),
        Some(Rect {
            x: 40.0,
            y: 30.0,
            width: 50.0,
            height: 40.0,
        })
    );
    assert!((clipped_rect.visible_fraction() - (2_000.0 / 7_000.0)).abs() < f32::EPSILON);

    let circle = observations
        .iter()
        .find(|observation| {
            observation.kind == EguiPaintKind::Circle
                && observation.bounds.x == 190.0
                && observation.bounds.y == 90.0
        })
        .expect("later painted circle appears in renderer output even when fully clipped");

    assert_eq!(circle.visible_bounds(), None);
    assert_eq!(circle.visible_fraction(), 0.0);
    assert!(
        clipped_rect.order < circle.order,
        "flattened FullOutput shape order must preserve back-to-front paint order"
    );

    output.drop_without_applying_deltas();
}

#[test]
fn paint_kind_serialization_is_stable_and_allocation_free_at_capture_time() {
    assert_eq!(
        serde_json::to_string(&EguiPaintKind::LineSegment).expect("serialize paint kind"),
        "\"line_segment\""
    );
    assert_eq!(
        serde_json::from_str::<EguiPaintKind>("\"quadratic_bezier\"")
            .expect("deserialize paint kind"),
        EguiPaintKind::QuadraticBezier
    );
}
