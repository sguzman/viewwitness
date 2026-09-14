use std::fmt::Write as _;

use serde::Serialize;

use crate::{EguiCorrelatedDiff, assess_authored_continuity};

/// Deterministic agent projection for a correlated egui diff with authored
/// continuity uncertainty promoted into the summary line.
///
/// The underlying diff projection remains responsible for material semantic and
/// authored changes, explicit ambiguity records, and non-material execution
/// churn. This wrapper adds only the narrow Stage-G continuity assessment so an
/// agent cannot mistake an ambiguity-blocked comparison for an ordinary material
/// change merely because `materially_empty=false`.
#[must_use]
pub fn correlated_diff_to_agent_text(diff: &EguiCorrelatedDiff) -> String {
    let continuity = assess_authored_continuity(diff);
    let mut output = crate::egui_agent_text::correlated_diff_to_agent_text(diff);
    let header_end = output.find('\n').unwrap_or(output.len());
    let mut epistemic = String::new();
    write!(
        epistemic,
        " authored_continuity={} unique_attribution_blocked={} object_ambiguity_count={} binding_ambiguity_count={}",
        json(&continuity.status),
        continuity.unique_attribution_blocked(),
        continuity.object_ambiguity_count,
        continuity.binding_ambiguity_count,
    )
    .expect("writing to String cannot fail");
    output.insert_str(header_end, &epistemic);
    output
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("ViewWitness egui epistemic values must serialize to JSON")
}
