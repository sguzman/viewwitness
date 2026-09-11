use std::fmt::Write as _;

use serde::Serialize;

use crate::{EguiCorrelatedCapture, Rect, to_agent_text};

/// Project an exact correlated egui capture into deterministic agent-oriented
/// text without collapsing semantic nodes and paint submissions into a guessed
/// identity mapping.
///
/// The canonical semantic witness is emitted using the ordinary ViewWitness
/// agent projection. Renderer evidence then follows as one ordered line per
/// paint submission. `visible_fraction` is explicitly labeled as a derived
/// bounding-box/clip measure rather than exact raster coverage.
#[must_use]
pub fn correlated_capture_to_agent_text(capture: &EguiCorrelatedCapture) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "egui-correlated request={} viewport_id={} pass={} viewport_rect=[{},{},{},{}] paint_count={} correlation=same_full_output",
        capture.request_id,
        capture.viewport_id,
        capture.pass_nr,
        capture.viewport_rect.x,
        capture.viewport_rect.y,
        capture.viewport_rect.width,
        capture.viewport_rect.height,
        capture.paint.len(),
    )
    .expect("writing to String cannot fail");

    output.push_str(&to_agent_text(&capture.witness));

    let mut paint: Vec<_> = capture.paint.iter().collect();
    paint.sort_by_key(|observation| observation.order);
    for observation in paint {
        write!(
            output,
            "paint order={} kind={} bounds={}",
            observation.order,
            json(&observation.kind),
            rect(observation.bounds),
        )
        .expect("writing to String cannot fail");

        match observation.clip_rect {
            Some(clip) => write!(output, " clip={}", rect(clip)),
            None => write!(output, " clip=unbounded"),
        }
        .expect("writing to String cannot fail");

        write!(
            output,
            " visible_fraction={} visible_fraction_evidence=derived_bbox_clip",
            observation.visible_fraction(),
        )
        .expect("writing to String cannot fail");

        if let Some(visible) = observation.visible_bounds() {
            write!(output, " visible_bounds={}", rect(visible))
                .expect("writing to String cannot fail");
        } else {
            write!(output, " visible_bounds=none").expect("writing to String cannot fail");
        }
        output.push('\n');
    }

    output
}

fn rect(rect: Rect) -> String {
    format!("[{},{},{},{}]", rect.x, rect.y, rect.width, rect.height)
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("ViewWitness egui agent-text values must serialize to JSON")
}
