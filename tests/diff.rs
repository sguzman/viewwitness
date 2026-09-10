use std::fs;

use viewwitness::{Witness, diff_from_yaml, diff_to_yaml, diff_witnesses, from_yaml};

#[test]
fn inspector_resize_is_field_change_not_identity_churn() {
    let before = load("examples/transitions/01-inspector-resize/before.yaml");
    let after = load("examples/transitions/01-inspector-resize/after.yaml");
    let diff = diff_witnesses(&before, &after);

    assert_eq!(diff.before_frame, Some(500));
    assert_eq!(diff.after_frame, Some(501));
    assert!(diff.nodes_added.is_empty());
    assert!(diff.nodes_removed.is_empty());
    assert!(diff.relations_added.is_empty());
    assert!(diff.relations_removed.is_empty());

    let preview = changed(&diff, "preview");
    assert_eq!(preview.fields.len(), 1);
    assert!(preview.fields.contains_key("bounds"));

    let inspector = changed(&diff, "inspector");
    assert_eq!(inspector.fields.len(), 1);
    assert!(inspector.fields.contains_key("bounds"));
}

#[test]
fn opening_context_menu_reports_only_new_transient_structure() {
    let before = load("examples/transitions/02-context-menu-open/before.yaml");
    let after = load("examples/transitions/02-context-menu-open/after.yaml");
    let diff = diff_witnesses(&before, &after);

    let added_ids: Vec<&str> = diff
        .nodes_added
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    assert_eq!(added_ids, vec!["delete", "duplicate", "object-menu"]);
    assert!(diff.nodes_removed.is_empty());
    assert!(diff.nodes_changed.is_empty());
    assert_eq!(diff.relations_added.len(), 1);
    assert_eq!(diff.relations_added[0].kind, "controls");
    assert_eq!(diff.relations_added[0].from, "object-menu");
    assert_eq!(diff.relations_added[0].to, "object-7");
    assert!(diff.relations_removed.is_empty());
}

#[test]
fn busy_transition_is_compact_and_preserves_button_identity() {
    let before = load("examples/transitions/03-busy-state/before.yaml");
    let after = load("examples/transitions/03-busy-state/after.yaml");
    let diff = diff_witnesses(&before, &after);

    let added_ids: Vec<&str> = diff
        .nodes_added
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    assert_eq!(added_ids, vec!["export-progress", "export-status"]);
    assert!(diff.nodes_removed.is_empty());

    let export = changed(&diff, "export");
    assert_eq!(export.fields.len(), 2);
    assert!(export.fields.contains_key("enabled"));
    assert!(export.fields.contains_key("actions"));

    assert_eq!(diff.relations_added.len(), 1);
    assert_eq!(diff.relations_added[0].kind, "describes");
    assert!(diff.relations_removed.is_empty());
}

#[test]
fn diff_yaml_round_trip_preserves_material_change() {
    let before = load("examples/transitions/03-busy-state/before.yaml");
    let after = load("examples/transitions/03-busy-state/after.yaml");
    let diff = diff_witnesses(&before, &after);

    let yaml = diff_to_yaml(&diff).expect("diff serializes");
    let reparsed = diff_from_yaml(&yaml).expect("diff reparses");

    assert_eq!(reparsed, diff);
}

#[test]
fn frame_number_alone_is_not_a_material_diff() {
    let before = load("examples/transitions/01-inspector-resize/before.yaml");
    let mut after = before.clone();
    after.capture.frame = before.capture.frame.map(|frame| frame + 1);

    let diff = diff_witnesses(&before, &after);
    assert!(diff.is_empty());
    assert_ne!(diff.before_frame, diff.after_frame);
}

fn changed<'a>(diff: &'a viewwitness::WitnessDiff, id: &str) -> &'a viewwitness::NodeChange {
    diff.nodes_changed
        .iter()
        .find(|change| change.id == id)
        .unwrap_or_else(|| panic!("missing change for {id}"))
}

fn load(path: &str) -> Witness {
    let source =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    from_yaml(&source).unwrap_or_else(|error| panic!("failed to parse {path}: {error}"))
}
