use std::fmt::Write as _;

use serde::Serialize;

use crate::{EguiCorrelatedCapture, Rect, to_agent_text};

/// Project an exact correlated egui capture into deterministic agent-oriented
/// text without collapsing semantic nodes, authored custom-paint objects, and
/// generic renderer submissions into guessed identity mappings.
///
/// The canonical semantic witness is emitted using the ordinary ViewWitness
/// agent projection. Explicit application-authored logical objects follow, each
/// with zero or more separately observed paint-binding lines, then renderer
/// evidence as one ordered line per generic paint submission. Clip survival is
/// explicitly labeled as a derived bounding-box measure rather than exact raster
/// coverage.
#[must_use]
pub fn correlated_capture_to_agent_text(capture: &EguiCorrelatedCapture) -> String {
    let mut output = String::new();
    let authored_binding_count: usize = capture
        .authored_objects
        .iter()
        .map(|object| object.bindings.len())
        .sum();
    writeln!(
        output,
        "egui-correlated request={} viewport_id={} pass={} viewport_rect=[{},{},{},{}] paint_count={} authored_count={} authored_binding_count={} correlation=same_full_output",
        capture.request_id,
        capture.viewport_id,
        capture.pass_nr,
        capture.viewport_rect.x,
        capture.viewport_rect.y,
        capture.viewport_rect.width,
        capture.viewport_rect.height,
        capture.paint.len(),
        capture.authored_objects.len(),
        authored_binding_count,
    )
    .expect("writing to String cannot fail");

    output.push_str(&to_agent_text(&capture.witness));

    let mut authored: Vec<_> = capture.authored_objects.iter().collect();
    authored.sort_by(|a, b| (&a.id, &a.role, &a.name).cmp(&(&b.id, &b.role, &b.name)));
    for (object_index, object) in authored.into_iter().enumerate() {
        write!(
            output,
            "authored-object index={} id={} role={}",
            object_index,
            json(&object.id),
            json(&object.role),
        )
        .expect("writing to String cannot fail");
        if let Some(name) = &object.name {
            write!(output, " name={}", json(name)).expect("writing to String cannot fail");
        }
        writeln!(
            output,
            " semantic_evidence={} binding_count={}",
            json(&object.semantic_evidence),
            object.bindings.len(),
        )
        .expect("writing to String cannot fail");

        for (binding_index, binding) in object.bindings.iter().enumerate() {
            write!(
                output,
                "authored-binding object_index={} binding_index={} object_id={} binding_evidence={} layer_order={} layer_id={} shape_index={} verified={}",
                object_index,
                binding_index,
                json(&object.id),
                json(&binding.binding_evidence),
                json(&binding.layer_order),
                binding.layer_id,
                binding.shape_index,
                binding.verified_at_end_pass,
            )
            .expect("writing to String cannot fail");
            if let Some(kind) = binding.kind {
                write!(output, " kind={}", json(&kind)).expect("writing to String cannot fail");
            }
            if let Some(bounds) = binding.bounds {
                write!(output, " bounds={}", rect(bounds)).expect("writing to String cannot fail");
            }
            match binding.clip_rect {
                Some(clip) => write!(output, " clip={}", rect(clip)),
                None => write!(output, " clip=unbounded"),
            }
            .expect("writing to String cannot fail");

            if binding.bounds.is_some() {
                write!(
                    output,
                    " visible_fraction={} visible_fraction_evidence=derived_bbox_clip",
                    binding.visible_fraction(),
                )
                .expect("writing to String cannot fail");
                if let Some(visible) = binding.visible_bounds() {
                    write!(output, " visible_bounds={}", rect(visible))
                        .expect("writing to String cannot fail");
                } else {
                    write!(output, " visible_bounds=none").expect("writing to String cannot fail");
                }
            }
            output.push('\n');
        }
    }

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
