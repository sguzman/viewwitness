use std::fmt::Write as _;

use serde::Serialize;

use crate::{Node, Relation, Witness, WitnessDiff};

/// Project a witness into compact deterministic text intended for repeated
/// software-agent consumption.
///
/// This is a lossy presentation in the sense that it does not promise a parser
/// or round-trip contract. The canonical Rust model and YAML projection remain
/// the interchange representation. Agent text preserves the current witness's
/// material node/relation evidence while avoiding YAML's structural overhead.
#[must_use]
pub fn to_agent_text(witness: &Witness) -> String {
    let mut output = String::new();
    write!(
        output,
        "view version={} source={} viewport=[{},{},{}]",
        json(&witness.viewwitness_version),
        json(&witness.capture.source),
        witness.capture.viewport.width,
        witness.capture.viewport.height,
        witness.capture.viewport.scale_factor
    )
    .expect("writing to String cannot fail");
    if let Some(frame) = witness.capture.frame {
        write!(output, " frame={frame}").expect("writing to String cannot fail");
    }
    output.push('\n');

    if !witness.capture.metadata.is_empty() {
        writeln!(output, "meta {}", json(&witness.capture.metadata))
            .expect("writing to String cannot fail");
    }

    let mut nodes: Vec<&Node> = witness.nodes.iter().collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    for node in nodes {
        append_node(&mut output, "node", node);
    }

    let mut relations: Vec<&Relation> = witness.relations.iter().collect();
    relations.sort_by(|a, b| relation_key(a).cmp(&relation_key(b)));
    for relation in relations {
        append_relation(&mut output, "relation", relation);
    }

    output
}

/// Project a material witness transition into compact deterministic text.
///
/// Changed node fields are emitted individually so an agent can consume a
/// small state transition without reparsing two complete snapshots.
#[must_use]
pub fn diff_to_agent_text(diff: &WitnessDiff) -> String {
    let mut output = String::new();
    write!(output, "diff").expect("writing to String cannot fail");
    if let Some(frame) = diff.before_frame {
        write!(output, " before_frame={frame}").expect("writing to String cannot fail");
    }
    if let Some(frame) = diff.after_frame {
        write!(output, " after_frame={frame}").expect("writing to String cannot fail");
    }
    output.push('\n');

    if let Some(change) = &diff.version_change {
        writeln!(
            output,
            "change scope=view field=version before={} after={}",
            json(&change.before),
            json(&change.after)
        )
        .expect("writing to String cannot fail");
    }

    if let Some(change) = &diff.viewport_change {
        writeln!(
            output,
            "change scope=view field=viewport before=[{},{},{}] after=[{},{},{}]",
            change.before.width,
            change.before.height,
            change.before.scale_factor,
            change.after.width,
            change.after.height,
            change.after.scale_factor
        )
        .expect("writing to String cannot fail");
    }

    for node in &diff.nodes_added {
        append_node(&mut output, "+node", node);
    }
    for node in &diff.nodes_removed {
        append_node(&mut output, "-node", node);
    }
    for change in &diff.nodes_changed {
        for (field, value) in &change.fields {
            writeln!(
                output,
                "change node={} field={} before={} after={}",
                json(&change.id),
                json(field),
                json(&value.before),
                json(&value.after)
            )
            .expect("writing to String cannot fail");
        }
    }
    for relation in &diff.relations_added {
        append_relation(&mut output, "+relation", relation);
    }
    for relation in &diff.relations_removed {
        append_relation(&mut output, "-relation", relation);
    }

    output
}

fn append_node(output: &mut String, prefix: &str, node: &Node) {
    write!(
        output,
        "{prefix} id={} role={}",
        json(&node.id),
        json(&node.role)
    )
    .expect("writing to String cannot fail");

    if let Some(parent) = &node.parent {
        write!(output, " parent={}", json(parent)).expect("writing to String cannot fail");
    }
    if let Some(identity) = &node.identity {
        write!(output, " identity={}", json(identity)).expect("writing to String cannot fail");
    }
    if let Some(name) = &node.name {
        write!(output, " name={}", json(name)).expect("writing to String cannot fail");
    }
    if let Some(bounds) = node.bounds {
        write!(
            output,
            " bounds=[{},{},{},{}]",
            bounds.x, bounds.y, bounds.width, bounds.height
        )
        .expect("writing to String cannot fail");
    }
    if let Some(visible) = node.visible {
        write!(output, " visible={visible}").expect("writing to String cannot fail");
    }
    if let Some(enabled) = node.enabled {
        write!(output, " enabled={enabled}").expect("writing to String cannot fail");
    }
    if let Some(focused) = node.focused {
        write!(output, " focused={focused}").expect("writing to String cannot fail");
    }
    if let Some(selected) = node.selected {
        write!(output, " selected={selected}").expect("writing to String cannot fail");
    }
    if let Some(text) = &node.text {
        write!(output, " text={}", json(text)).expect("writing to String cannot fail");
    }
    if let Some(value) = &node.value {
        write!(output, " value={}", json(value)).expect("writing to String cannot fail");
    }
    if !node.actions.is_empty() {
        write!(output, " actions={}", json(&node.actions)).expect("writing to String cannot fail");
    }
    if !node.properties.is_empty() {
        write!(output, " props={}", json(&node.properties)).expect("writing to String cannot fail");
    }
    output.push('\n');
}

fn append_relation(output: &mut String, prefix: &str, relation: &Relation) {
    write!(
        output,
        "{prefix} kind={} from={} to={} evidence={}",
        json(&relation.kind),
        json(&relation.from),
        json(&relation.to),
        json(&relation.evidence)
    )
    .expect("writing to String cannot fail");
    if !relation.properties.is_empty() {
        write!(output, " props={}", json(&relation.properties))
            .expect("writing to String cannot fail");
    }
    output.push('\n');
}

fn relation_key(relation: &Relation) -> (&str, &str, &str, &str) {
    (
        relation.kind.as_str(),
        relation.from.as_str(),
        relation.to.as_str(),
        relation.evidence.as_str(),
    )
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("ViewWitness agent-text values must serialize to JSON")
}
