use std::fmt::Write as _;

use serde::Serialize;

use crate::{
    EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiCorrelatedDiff, Rect, diff_to_agent_text,
    to_agent_text,
};

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
                "authored-binding object_index={} binding_index={} object_id={}",
                object_index,
                binding_index,
                json(&object.id),
            )
            .expect("writing to String cannot fail");
            if let Some(binding_id) = &binding.authored_binding_id {
                write!(output, " authored_binding_id={}", json(binding_id))
                    .expect("writing to String cannot fail");
            }
            write!(
                output,
                " binding_evidence={} layer_order={} layer_id={} shape_index={} verified={}",
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

/// Deterministic agent projection for a correlated egui diff.
///
/// Generic anonymous paint is intentionally absent because the correlated diff
/// does not claim durable identity for those submissions. Layer-local ShapeIdx
/// churn is emitted separately and labeled non-material. Authored binding IDs
/// are included only when the application supplied them.
#[must_use]
pub fn correlated_diff_to_agent_text(diff: &EguiCorrelatedDiff) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "egui-diff before_request={} after_request={} before_pass={} after_pass={} materially_empty={}",
        diff.before_request_id,
        diff.after_request_id,
        diff.before_pass_nr,
        diff.after_pass_nr,
        diff.is_materially_empty(),
    )
    .expect("writing to String cannot fail");
    output.push_str(&diff_to_agent_text(&diff.semantic));

    for object in &diff.authored.objects_added {
        write_authored_delta(&mut output, "+authored-object", object);
    }
    for object in &diff.authored.objects_removed {
        write_authored_delta(&mut output, "-authored-object", object);
    }
    for change in &diff.authored.objects_changed {
        for (field, value) in &change.fields {
            writeln!(
                output,
                "authored-change id={} field={} before={} after={}",
                json(&change.id),
                json(field),
                json(&value.before),
                json(&value.after),
            )
            .expect("writing to String cannot fail");
        }
    }
    for ambiguity in &diff.authored.ambiguous_ids {
        writeln!(
            output,
            "authored-ambiguity id={} before_count={} after_count={} matching=refused",
            json(&ambiguity.id),
            ambiguity.before_count,
            ambiguity.after_count,
        )
        .expect("writing to String cannot fail");
    }
    for ambiguity in &diff.authored.binding_ambiguities {
        writeln!(
            output,
            "authored-binding-ambiguity object_id={} authored_binding_id={} before_count={} after_count={} matching=refused",
            json(&ambiguity.object_id),
            json(&ambiguity.binding_id),
            ambiguity.before_count,
            ambiguity.after_count,
        )
        .expect("writing to String cannot fail");
    }
    for churn in &diff.authored.execution_handle_churn {
        write!(output, "authored-handle-churn id={}", json(&churn.id))
            .expect("writing to String cannot fail");
        if let Some(binding_id) = &churn.authored_binding_id {
            write!(output, " authored_binding_id={}", json(binding_id))
                .expect("writing to String cannot fail");
        }
        writeln!(
            output,
            " binding_ordinal={} before_shape_index={} after_shape_index={} material=false continuity=frame_local_structure_sensitive",
            churn.binding_ordinal,
            churn.before_shape_index,
            churn.after_shape_index,
        )
        .expect("writing to String cannot fail");
    }

    output
}

fn write_authored_delta(output: &mut String, prefix: &str, object: &EguiAuthoredPaintObject) {
    write!(
        output,
        "{prefix} id={} role={} semantic_evidence={} binding_count={}",
        json(&object.id),
        json(&object.role),
        json(&object.semantic_evidence),
        object.bindings.len(),
    )
    .expect("writing to String cannot fail");
    if let Some(name) = &object.name {
        write!(output, " name={}", json(name)).expect("writing to String cannot fail");
    }
    output.push('\n');
}

fn rect(rect: Rect) -> String {
    format!("[{},{},{},{}]", rect.x, rect.y, rect.width, rect.height)
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("ViewWitness egui agent-text values must serialize to JSON")
}
