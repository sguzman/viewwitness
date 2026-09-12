#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiLayerOrder,
    EguiPaintKind, Rect, diff_correlated_captures, from_yaml,
};

#[test]
fn keyed_binding_reorder_is_non_material_and_reports_handle_churn_by_id() {
    let before = capture(
        1,
        vec![object(vec![
            binding("outline", EguiPaintKind::Rect, 3, 10.0),
            binding("handle", EguiPaintKind::Circle, 4, 30.0),
        ])],
    );
    let after = capture(
        2,
        vec![object(vec![
            binding("handle", EguiPaintKind::Circle, 3, 30.0),
            binding("outline", EguiPaintKind::Rect, 4, 10.0),
        ])],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.bindings_added.is_empty());
    assert!(diff.authored.bindings_removed.is_empty());
    assert!(diff.authored.bindings_changed.is_empty());
    assert!(diff.authored.binding_ambiguities.is_empty());
    assert_eq!(diff.authored.execution_handle_churn.len(), 2);
    assert!(diff.is_materially_empty());

    let ids: Vec<_> = diff
        .authored
        .execution_handle_churn
        .iter()
        .map(|change| change.authored_binding_id.as_deref())
        .collect();
    assert_eq!(ids, vec![Some("handle"), Some("outline")]);
}

#[test]
fn keyed_binding_material_change_is_field_granular() {
    let before = capture(
        1,
        vec![object(vec![binding(
            "outline",
            EguiPaintKind::Rect,
            3,
            10.0,
        )])],
    );
    let after = capture(
        2,
        vec![object(vec![binding(
            "outline",
            EguiPaintKind::Circle,
            9,
            10.0,
        )])],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert!(diff.authored.objects_changed.is_empty());
    assert_eq!(diff.authored.bindings_changed.len(), 1);
    let change = &diff.authored.bindings_changed[0];
    assert_eq!(change.object_id, "canvas:keyed");
    assert_eq!(change.authored_binding_id.as_deref(), Some("outline"));
    assert_eq!(change.unkeyed_ordinal, None);
    assert_eq!(
        change.fields.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["kind"]
    );
    assert!(diff.authored.execution_handle_churn.is_empty());
    assert!(!diff.is_materially_empty());
}

#[test]
fn keyed_binding_addition_and_removal_are_first_class() {
    let before = capture(
        1,
        vec![object(vec![binding(
            "outline",
            EguiPaintKind::Rect,
            3,
            10.0,
        )])],
    );
    let after = capture(
        2,
        vec![object(vec![binding(
            "handle",
            EguiPaintKind::Circle,
            8,
            30.0,
        )])],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert_eq!(diff.authored.bindings_removed.len(), 1);
    assert_eq!(diff.authored.bindings_added.len(), 1);
    assert_eq!(
        diff.authored.bindings_removed[0]
            .binding
            .authored_binding_id
            .as_deref(),
        Some("outline")
    );
    assert_eq!(
        diff.authored.bindings_added[0]
            .binding
            .authored_binding_id
            .as_deref(),
        Some("handle")
    );
    assert!(!diff.is_materially_empty());
}

#[test]
fn duplicate_binding_ids_are_explicit_ambiguity() {
    let before = capture(
        1,
        vec![object(vec![
            binding("outline", EguiPaintKind::Rect, 3, 10.0),
            binding("outline", EguiPaintKind::Circle, 4, 30.0),
        ])],
    );
    let after = capture(
        2,
        vec![object(vec![binding(
            "outline",
            EguiPaintKind::Rect,
            8,
            10.0,
        )])],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert_eq!(diff.authored.binding_ambiguities.len(), 1);
    let ambiguity = &diff.authored.binding_ambiguities[0];
    assert_eq!(ambiguity.object_id, "canvas:keyed");
    assert_eq!(ambiguity.binding_id, "outline");
    assert_eq!(ambiguity.before_count, 2);
    assert_eq!(ambiguity.after_count, 1);
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.bindings_added.is_empty());
    assert!(diff.authored.bindings_removed.is_empty());
    assert!(diff.authored.bindings_changed.is_empty());
    assert!(diff.authored.execution_handle_churn.is_empty());
    assert!(!diff.is_materially_empty());
}

#[test]
fn keyed_and_unkeyed_bindings_can_coexist_without_losing_legacy_order_semantics() {
    let before = capture(
        1,
        vec![object(vec![
            binding("outline", EguiPaintKind::Rect, 3, 10.0),
            unkeyed(EguiPaintKind::Circle, 4, 30.0),
        ])],
    );
    let after = capture(
        2,
        vec![object(vec![
            unkeyed(EguiPaintKind::Circle, 8, 30.0),
            binding("outline", EguiPaintKind::Rect, 9, 10.0),
        ])],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.bindings_changed.is_empty());
    assert_eq!(diff.authored.execution_handle_churn.len(), 2);
    assert!(diff.is_materially_empty());
}

#[test]
fn unkeyed_binding_change_reports_relative_unkeyed_ordinal() {
    let before = capture(
        1,
        vec![object(vec![
            binding("outline", EguiPaintKind::Rect, 3, 10.0),
            unkeyed(EguiPaintKind::Circle, 4, 30.0),
        ])],
    );
    let after = capture(
        2,
        vec![object(vec![
            binding("outline", EguiPaintKind::Rect, 9, 10.0),
            unkeyed(EguiPaintKind::Circle, 10, 35.0),
        ])],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert_eq!(diff.authored.bindings_changed.len(), 1);
    let change = &diff.authored.bindings_changed[0];
    assert_eq!(change.authored_binding_id, None);
    assert_eq!(change.unkeyed_ordinal, Some(0));
    assert_eq!(
        change.fields.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["bounds"]
    );
    assert_eq!(diff.authored.execution_handle_churn.len(), 1);
    assert_eq!(
        diff.authored.execution_handle_churn[0]
            .authored_binding_id
            .as_deref(),
        Some("outline")
    );
}

fn object(bindings: Vec<EguiAuthoredPaintBinding>) -> EguiAuthoredPaintObject {
    EguiAuthoredPaintObject {
        id: "canvas:keyed".into(),
        role: "diagram_node".into(),
        name: Some("Keyed node".into()),
        semantic_evidence: "intended".into(),
        bindings,
    }
}

fn binding(id: &str, kind: EguiPaintKind, shape_index: usize, x: f32) -> EguiAuthoredPaintBinding {
    let mut binding = unkeyed(kind, shape_index, x);
    binding.authored_binding_id = Some(id.into());
    binding
}

fn unkeyed(kind: EguiPaintKind, shape_index: usize, x: f32) -> EguiAuthoredPaintBinding {
    EguiAuthoredPaintBinding {
        authored_binding_id: None,
        binding_evidence: "observed".into(),
        layer_order: EguiLayerOrder::Background,
        layer_id: 42,
        shape_index,
        verified_at_end_pass: true,
        kind: Some(kind),
        bounds: Some(Rect {
            x,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        }),
        clip_rect: None,
    }
}

fn capture(pass_nr: u64, authored_objects: Vec<EguiAuthoredPaintObject>) -> EguiCorrelatedCapture {
    let witness = from_yaml(&format!(
        r#"
viewwitness_version: "0.1"
capture:
  source: egui
  frame: {pass_nr}
  viewport:
    width: 100.0
    height: 50.0
    scale_factor: 1.0
nodes: []
relations: []
"#
    ))
    .expect("parse synthetic keyed-binding witness");

    EguiCorrelatedCapture {
        request_id: pass_nr,
        viewport_id: 1,
        pass_nr,
        viewport_rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        witness,
        paint: Vec::new(),
        authored_objects,
    }
}
