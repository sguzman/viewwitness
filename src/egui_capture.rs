use std::collections::{BTreeMap, BTreeSet};

use egui::accesskit::{Action, NodeId, Role, Toggled, TreeUpdate};
use serde_json::{Value, json};

use crate::{Capture, Node, Rect, Relation, Viewport, Witness};

/// Capture context supplied by the egui integration.
///
/// `viewport` uses ViewWitness logical coordinates. When converting a complete
/// `egui::FullOutput`, its `scale_factor` is replaced with egui's reported
/// `pixels_per_point` so physical/logical scale remains observed rather than
/// caller-guessed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EguiCaptureContext {
    pub frame: Option<u64>,
    pub viewport: Viewport,
}

impl EguiCaptureContext {
    #[must_use]
    pub const fn new(viewport: Viewport) -> Self {
        Self {
            frame: None,
            viewport,
        }
    }

    #[must_use]
    pub const fn with_frame(mut self, frame: u64) -> Self {
        self.frame = Some(frame);
        self
    }
}

/// Convert the AccessKit tree embedded in an egui frame into a ViewWitness.
///
/// Returns `None` when AccessKit generation was not active for this frame.
#[must_use]
pub fn witness_from_egui_output(
    output: &egui::FullOutput,
    mut context: EguiCaptureContext,
) -> Option<Witness> {
    context.viewport.scale_factor = output.pixels_per_point;
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .map(|update| witness_from_egui_tree_update(update, context))
}

/// Convert an egui-produced AccessKit tree update into a ViewWitness.
///
/// egui currently emits a complete AccessKit tree for each UI frame. This
/// function relies on that stronger egui guarantee: it is not intended as a
/// generic converter for arbitrary incremental AccessKit updates.
#[must_use]
pub fn witness_from_egui_tree_update(
    update: &TreeUpdate,
    context: EguiCaptureContext,
) -> Witness {
    let known_ids: BTreeSet<NodeId> = update.nodes.iter().map(|(id, _)| *id).collect();
    let mut parents = BTreeMap::new();

    for (parent_id, node) in &update.nodes {
        for child_id in node.children() {
            if known_ids.contains(child_id) {
                parents.entry(*child_id).or_insert(*parent_id);
            }
        }
    }

    let mut nodes: Vec<Node> = update
        .nodes
        .iter()
        .map(|(id, node)| {
            let role = role_name(node.role());
            let actions = actions(node);
            let mut properties = BTreeMap::new();

            insert_string(&mut properties, "author_id", node.author_id());
            insert_string(&mut properties, "description", node.description());
            insert_string(&mut properties, "placeholder", node.placeholder());
            insert_string(
                &mut properties,
                "keyboard_shortcut",
                node.keyboard_shortcut(),
            );

            if node.is_busy() {
                properties.insert("busy".into(), json!(true));
            }
            if node.is_read_only() {
                properties.insert("read_only".into(), json!(true));
            }
            if node.is_required() {
                properties.insert("required".into(), json!(true));
            }
            if node.is_modal() {
                properties.insert("modal".into(), json!(true));
            }
            if node.clips_children() {
                properties.insert("clips_children".into(), json!(true));
            }
            if let Some(expanded) = node.is_expanded() {
                properties.insert("expanded".into(), json!(expanded));
            }
            if let Some(toggled) = node.toggled() {
                properties.insert("toggled".into(), json!(toggled_name(toggled)));
            }
            if node.transform().is_some() {
                properties.insert("accesskit_transform_present".into(), json!(true));
            }

            let string_value = node.value().map(str::to_owned);
            let numeric_value = node.numeric_value();
            let text = if role == "label" || role == "text_run" {
                string_value.clone()
            } else {
                None
            };
            let value = if let Some(value) = numeric_value {
                Some(json!(value))
            } else if text.is_none() {
                string_value.map(Value::String)
            } else {
                None
            };

            let focused = if *id == update.focus {
                Some(true)
            } else if node.supports_action(Action::Focus) {
                Some(false)
            } else {
                None
            };

            let enabled = if node.is_disabled() {
                Some(false)
            } else if !actions.is_empty() {
                Some(true)
            } else {
                None
            };

            Node {
                id: node_id(*id),
                role,
                parent: parents.get(id).copied().map(node_id),
                name: node.label().map(str::to_owned).or_else(|| {
                    (node.role() == Role::Label)
                        .then(|| node.value().map(str::to_owned))
                        .flatten()
                }),
                bounds: node.bounds().map(|bounds| Rect {
                    x: bounds.x0 as f32,
                    y: bounds.y0 as f32,
                    width: (bounds.x1 - bounds.x0) as f32,
                    height: (bounds.y1 - bounds.y0) as f32,
                }),
                visible: Some(!node.is_hidden()),
                enabled,
                focused,
                selected: node.is_selected(),
                text,
                value,
                actions,
                properties,
            }
        })
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut relations = semantic_relations(update, &known_ids);
    relations.sort_by(|a, b| {
        (&a.kind, &a.from, &a.to, &a.evidence).cmp(&(&b.kind, &b.from, &b.to, &b.evidence))
    });

    let mut metadata = BTreeMap::new();
    metadata.insert("semantic_source".into(), json!("accesskit"));
    metadata.insert("accesskit_tree_id".into(), json!(format!("{:?}", update.tree_id)));
    metadata.insert("accesskit_focus_id".into(), json!(node_id(update.focus)));

    if let Some(tree) = &update.tree {
        metadata.insert("accesskit_root_id".into(), json!(node_id(tree.root)));
        if let Some(toolkit_name) = &tree.toolkit_name {
            metadata.insert("toolkit_name".into(), json!(toolkit_name));
        }
        if let Some(toolkit_version) = &tree.toolkit_version {
            metadata.insert("toolkit_version".into(), json!(toolkit_version));
        }
    }

    Witness {
        viewwitness_version: "0.1".into(),
        capture: Capture {
            source: "egui".into(),
            frame: context.frame,
            viewport: context.viewport,
            metadata,
        },
        nodes,
        relations,
    }
}

fn semantic_relations(update: &TreeUpdate, known_ids: &BTreeSet<NodeId>) -> Vec<Relation> {
    let mut relations = Vec::new();

    for (id, node) in &update.nodes {
        for source in node.labelled_by() {
            push_relation(&mut relations, known_ids, "labels", *source, *id);
        }
        for source in node.described_by() {
            push_relation(&mut relations, known_ids, "describes", *source, *id);
        }
        for target in node.controls() {
            push_relation(&mut relations, known_ids, "controls", *id, *target);
        }
    }

    relations
}

fn push_relation(
    relations: &mut Vec<Relation>,
    known_ids: &BTreeSet<NodeId>,
    kind: &str,
    from: NodeId,
    to: NodeId,
) {
    if !known_ids.contains(&from) || !known_ids.contains(&to) {
        return;
    }

    relations.push(Relation {
        kind: kind.into(),
        from: node_id(from),
        to: node_id(to),
        evidence: "observed".into(),
        properties: BTreeMap::new(),
    });
}

fn actions(node: &egui::accesskit::Node) -> Vec<String> {
    const ACTIONS: &[(Action, &str)] = &[
        (Action::Click, "click"),
        (Action::Focus, "focus"),
        (Action::Blur, "blur"),
        (Action::Collapse, "collapse"),
        (Action::Expand, "expand"),
        (Action::CustomAction, "custom_action"),
        (Action::Decrement, "decrement"),
        (Action::Increment, "increment"),
        (Action::HideTooltip, "hide_tooltip"),
        (Action::ShowTooltip, "show_tooltip"),
        (Action::ReplaceSelectedText, "replace_selected_text"),
        (Action::ScrollDown, "scroll_down"),
        (Action::ScrollLeft, "scroll_left"),
        (Action::ScrollRight, "scroll_right"),
        (Action::ScrollUp, "scroll_up"),
        (Action::ScrollIntoView, "scroll_into_view"),
        (Action::ScrollToPoint, "scroll_to_point"),
        (Action::SetScrollOffset, "set_scroll_offset"),
        (Action::SetTextSelection, "set_text_selection"),
        (
            Action::SetSequentialFocusNavigationStartingPoint,
            "set_sequential_focus_navigation_starting_point",
        ),
        (Action::SetValue, "set_value"),
        (Action::ShowContextMenu, "show_context_menu"),
    ];

    ACTIONS
        .iter()
        .filter(|(action, _)| node.supports_action(*action))
        .map(|(_, name)| (*name).to_owned())
        .collect()
}

fn role_name(role: Role) -> String {
    let raw = pascal_to_snake(&format!("{role:?}"));
    match raw.as_str() {
        "generic_container" => "container".into(),
        "pane" => "panel".into(),
        "text_input" => "textbox".into(),
        "check_box" => "checkbox".into(),
        "radio_button" => "radio".into(),
        "radio_group" => "radiogroup".into(),
        "progress_indicator" => "progress".into(),
        "scroll_bar" => "scrollbar".into(),
        "spin_button" => "spinbutton".into(),
        _ => raw,
    }
}

fn pascal_to_snake(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let chars: Vec<char> = value.chars().collect();

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_ascii_uppercase() {
            let previous_is_lower = index > 0 && chars[index - 1].is_ascii_lowercase();
            let next_is_lower = chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_lowercase());
            if index > 0 && (previous_is_lower || next_is_lower) {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch);
        }
    }

    output
}

fn toggled_name(toggled: Toggled) -> &'static str {
    match toggled {
        Toggled::False => "false",
        Toggled::True => "true",
        Toggled::Mixed => "mixed",
    }
}

fn node_id(id: NodeId) -> String {
    format!("ak:{}", id.0)
}

fn insert_string(properties: &mut BTreeMap<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        properties.insert(key.into(), json!(value));
    }
}
