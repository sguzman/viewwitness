use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Node, Relation, Viewport, Witness};

/// Material state transition between two witnesses.
///
/// Capture frame numbers are preserved as context but are intentionally not
/// themselves treated as changes: consecutive frames are expected to differ in
/// frame number even when the GUI is otherwise identical.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WitnessDiff {
    pub before_frame: Option<u64>,
    pub after_frame: Option<u64>,
    #[serde(default)]
    pub version_change: Option<FieldChange>,
    #[serde(default)]
    pub viewport_change: Option<ViewportChange>,
    #[serde(default)]
    pub nodes_added: Vec<Node>,
    #[serde(default)]
    pub nodes_removed: Vec<Node>,
    #[serde(default)]
    pub nodes_changed: Vec<NodeChange>,
    #[serde(default)]
    pub relations_added: Vec<Relation>,
    #[serde(default)]
    pub relations_removed: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewportChange {
    pub before: Viewport,
    pub after: Viewport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeChange {
    pub id: String,
    pub fields: BTreeMap<String, FieldChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldChange {
    pub before: Value,
    pub after: Value,
}

impl WitnessDiff {
    /// True when there is no material difference represented by this diff.
    /// Frame-number changes alone do not make a diff non-empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.version_change.is_none()
            && self.viewport_change.is_none()
            && self.nodes_added.is_empty()
            && self.nodes_removed.is_empty()
            && self.nodes_changed.is_empty()
            && self.relations_added.is_empty()
            && self.relations_removed.is_empty()
    }
}

/// Compare two witnesses using stable node IDs and full relation identity.
///
/// Node changes are represented field-by-field so an agent can distinguish a
/// moved/resized node from remove+add churn. Nodes and relations are returned
/// in deterministic order.
#[must_use]
pub fn diff_witnesses(before: &Witness, after: &Witness) -> WitnessDiff {
    let before_nodes: BTreeMap<&str, &Node> = before
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let after_nodes: BTreeMap<&str, &Node> = after
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();

    let before_ids: BTreeSet<&str> = before_nodes.keys().copied().collect();
    let after_ids: BTreeSet<&str> = after_nodes.keys().copied().collect();

    let nodes_added = after_ids
        .difference(&before_ids)
        .map(|id| (*after_nodes.get(id).expect("id came from map")).clone())
        .collect();
    let nodes_removed = before_ids
        .difference(&after_ids)
        .map(|id| (*before_nodes.get(id).expect("id came from map")).clone())
        .collect();

    let nodes_changed = before_ids
        .intersection(&after_ids)
        .filter_map(|id| {
            let before_node = before_nodes.get(id).expect("id came from map");
            let after_node = after_nodes.get(id).expect("id came from map");
            node_change(before_node, after_node)
        })
        .collect();

    let (relations_added, relations_removed) = relation_diff(&before.relations, &after.relations);

    WitnessDiff {
        before_frame: before.capture.frame,
        after_frame: after.capture.frame,
        version_change: (before.viewwitness_version != after.viewwitness_version).then(|| {
            FieldChange {
                before: Value::String(before.viewwitness_version.clone()),
                after: Value::String(after.viewwitness_version.clone()),
            }
        }),
        viewport_change: (before.capture.viewport != after.capture.viewport).then(|| {
            ViewportChange {
                before: before.capture.viewport,
                after: after.capture.viewport,
            }
        }),
        nodes_added,
        nodes_removed,
        nodes_changed,
        relations_added,
        relations_removed,
    }
}

fn node_change(before: &Node, after: &Node) -> Option<NodeChange> {
    let before_value = serde_json::to_value(before).expect("Node serialization is infallible");
    let after_value = serde_json::to_value(after).expect("Node serialization is infallible");
    let before_fields = before_value
        .as_object()
        .expect("serialized Node must be an object");
    let after_fields = after_value
        .as_object()
        .expect("serialized Node must be an object");

    let keys: BTreeSet<&str> = before_fields
        .keys()
        .chain(after_fields.keys())
        .map(String::as_str)
        .filter(|key| *key != "id")
        .collect();

    let fields: BTreeMap<String, FieldChange> = keys
        .into_iter()
        .filter_map(|key| {
            let before = before_fields.get(key).cloned().unwrap_or(Value::Null);
            let after = after_fields.get(key).cloned().unwrap_or(Value::Null);
            (before != after).then(|| (key.to_owned(), FieldChange { before, after }))
        })
        .collect();

    (!fields.is_empty()).then(|| NodeChange {
        id: before.id.clone(),
        fields,
    })
}

fn relation_diff(before: &[Relation], after: &[Relation]) -> (Vec<Relation>, Vec<Relation>) {
    let before_map = relation_map(before);
    let after_map = relation_map(after);
    let before_keys: BTreeSet<&str> = before_map.keys().map(String::as_str).collect();
    let after_keys: BTreeSet<&str> = after_map.keys().map(String::as_str).collect();

    let added = after_keys
        .difference(&before_keys)
        .map(|key| (*after_map.get(*key).expect("key came from map")).clone())
        .collect();
    let removed = before_keys
        .difference(&after_keys)
        .map(|key| (*before_map.get(*key).expect("key came from map")).clone())
        .collect();

    (added, removed)
}

fn relation_map(relations: &[Relation]) -> BTreeMap<String, &Relation> {
    relations
        .iter()
        .map(|relation| {
            let key =
                serde_json::to_string(relation).expect("Relation serialization is infallible");
            (key, relation)
        })
        .collect()
}
