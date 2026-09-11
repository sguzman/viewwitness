#![cfg(feature = "egui")]

use std::sync::mpsc::TryRecvError;

use viewwitness::{EguiPaintReporter, Rect};

#[test]
fn bounded_reporter_drops_instead_of_backpressuring_and_reports_the_gap() {
    let ctx = egui::Context::default();
    let (reporter, receiver) = EguiPaintReporter::channel(1);
    ctx.add_plugin(reporter);

    run_painted_pass(&ctx, 10.0);
    run_painted_pass(&ctx, 20.0);

    let first = receiver.try_recv().expect("first paint frame delivered");
    assert_eq!(first.dropped_before, 0);
    assert_eq!(first.pixels_per_point, 1.0);
    assert!(
        first.observations.iter().any(|observation| {
            observation.kind == "rect"
                && observation.bounds
                    == Rect {
                        x: 10.0,
                        y: 20.0,
                        width: 40.0,
                        height: 30.0,
                    }
        }),
        "reporter carries renderer-facing paint observations"
    );
    assert!(matches!(receiver.try_recv(), Err(TryRecvError::Empty)));

    run_painted_pass(&ctx, 30.0);
    let after_gap = receiver
        .try_recv()
        .expect("paint frame after consumer catches up");

    assert_eq!(
        after_gap.dropped_before, 1,
        "one full-queue frame must be reported as dropped rather than blocking rendering"
    );
    assert!(after_gap.pass_nr > first.pass_nr);
    assert_eq!(after_gap.viewport_id, first.viewport_id);
    assert!(
        after_gap.observations.iter().any(|observation| {
            observation.kind == "rect"
                && observation.bounds
                    == Rect {
                        x: 30.0,
                        y: 20.0,
                        width: 40.0,
                        height: 30.0,
                    }
        }),
        "the next delivered frame is fresh rather than the dropped frame"
    );
}

fn run_painted_pass(ctx: &egui::Context, x: f32) {
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            ..Default::default()
        },
        |ui| {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(x, 20.0), egui::vec2(40.0, 30.0)),
                0.0,
                egui::Color32::WHITE,
            );
        },
    );
    output.drop_without_applying_deltas();
}
