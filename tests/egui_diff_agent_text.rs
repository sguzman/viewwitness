#![cfg(feature = "egui")]

use std::collections::BTreeMap;

use viewwitness::{
    EguiAuthoredBindingChange, EguiAuthoredBindingIdAmbiguity, EguiAuthoredDiff,
    EguiAuthoredExecutionHandleChange, EguiAuthoredIdAmbiguity, EguiCorrelatedDiff, FieldChange,
    Rect, WitnessDiff, correlated_diff_to_agent_text,
};

#[test]
fn correlated_diff_text_separates_ambiguity_and_non_material_handle_churn() {
    let diff = EguiCorrelatedDiff {
        before_request_id: 7,
        after_request_id: 8,
        before_pass_nr: 9,
        after_pass_nr: 10,
        semantic: empty_witness_diff(9, 10),
        authored: EguiAuthoredDiff {
            objects_added: Vec::new(),
            objects_removed: Vec::new(),
            objects_changed: Vec::new(),
            bindings_added: Vec::new(),
            bindings_removed: Vec::new(),
            bindings_changed: Vec::new(),
            ambiguous_ids: vec![EguiAuthoredIdAmbiguity {
                id: "duplicate".into(),
                before_count: 2,
                after_count: 1,
            }],
            binding_ambiguities: vec![EguiAuthoredBindingIdAmbiguity {
                object_id: "canvas:stable".into(),
                binding_id: "outline".into(),
                before_count: 2,
                after_count: 1,
            }],
            execution_handle_churn: vec![EguiAuthoredExecutionHandleChange {
                id: "canvas:stable".into(),
                authored_binding_id: Some("handle".into()),
                binding_ordinal: 0,
                before_shape_index: 3,
                after_shape_index: 4,
            }],
        },
    };

    let text = correlated_diff_to_agent_text(&diff);
    assert!(text.starts_with(
        "egui-diff before_request=7 after_request=8 before_pass=9 after_pass=10 materially_empty=false\n"
    ));
    assert!(text.contains(
        "authored-ambiguity id=\"duplicate\" before_count=2 after_count=1 matching=refused"
    ));
    assert!(text.contains(
        "authored-binding-ambiguity object_id=\"canvas:stable\" authored_binding_id=\"outline\" before_count=2 after_count=1 matching=refused"
    ));
    assert!(text.contains(
        "authored-handle-churn id=\"canvas:stable\" authored_binding_id=\"handle\" binding_ordinal=0 before_shape_index=3 after_shape_index=4 material=false continuity=frame_local_structure_sensitive"
    ));
}

#[test]
fn keyed_binding_change_projects_the_named_subpart_and_field() {
    let mut fields = BTreeMap::new();
    fields.insert(
        "bounds".into(),
        FieldChange {
            before: serde_json::to_value(Rect {
                x: 120.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
            })
            .unwrap(),
            after: serde_json::to_value(Rect {
                x: 90.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
            })
            .unwrap(),
        },
    );

    let diff = EguiCorrelatedDiff {
        before_request_id: 1,
        after_request_id: 2,
        before_pass_nr: 5,
        after_pass_nr: 6,
        semantic: empty_witness_diff(5, 6),
        authored: EguiAuthoredDiff {
            objects_added: Vec::new(),
            objects_removed: Vec::new(),
            objects_changed: Vec::new(),
            bindings_added: Vec::new(),
            bindings_removed: Vec::new(),
            bindings_changed: vec![EguiAuthoredBindingChange {
                object_id: "canvas:node".into(),
                authored_binding_id: Some("handle".into()),
                unkeyed_ordinal: None,
                fields,
            }],
            ambiguous_ids: Vec::new(),
            binding_ambiguities: Vec::new(),
            execution_handle_churn: Vec::new(),
        },
    };

    let text = correlated_diff_to_agent_text(&diff);
    assert!(text.contains(
        "authored-binding-change object_id=\"canvas:node\" authored_binding_id=\"handle\" field=\"bounds\""
    ));
    assert!(!text.contains("field=\"bindings\""));
}

#[test]
fn unkeyed_handle_churn_keeps_legacy_projection_and_is_materially_empty() {
    let diff = EguiCorrelatedDiff {
        before_request_id: 1,
        after_request_id: 2,
        before_pass_nr: 5,
        after_pass_nr: 6,
        semantic: empty_witness_diff(5, 6),
        authored: EguiAuthoredDiff {
            objects_added: Vec::new(),
            objects_removed: Vec::new(),
            objects_changed: Vec::new(),
            bindings_added: Vec::new(),
            bindings_removed: Vec::new(),
            bindings_changed: Vec::new(),
            ambiguous_ids: Vec::new(),
            binding_ambiguities: Vec::new(),
            execution_handle_churn: vec![EguiAuthoredExecutionHandleChange {
                id: "canvas:stable".into(),
                authored_binding_id: None,
                binding_ordinal: 0,
                before_shape_index: 8,
                after_shape_index: 9,
            }],
        },
    };

    let text = correlated_diff_to_agent_text(&diff);
    assert!(
        text.lines()
            .next()
            .unwrap()
            .ends_with("materially_empty=true")
    );
    assert!(text.contains(
        "authored-handle-churn id=\"canvas:stable\" binding_ordinal=0 before_shape_index=8 after_shape_index=9 material=false"
    ));
    assert!(!text.contains("authored_binding_id="));
}

fn empty_witness_diff(before_frame: u64, after_frame: u64) -> WitnessDiff {
    WitnessDiff {
        before_frame: Some(before_frame),
        after_frame: Some(after_frame),
        version_change: None,
        viewport_change: None,
        nodes_added: Vec::new(),
        nodes_removed: Vec::new(),
        nodes_changed: Vec::new(),
        relations_added: Vec::new(),
        relations_removed: Vec::new(),
    }
}
