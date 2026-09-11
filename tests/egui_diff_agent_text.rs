#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredDiff, EguiAuthoredExecutionHandleChange, EguiAuthoredIdAmbiguity,
    EguiCorrelatedDiff, WitnessDiff, correlated_diff_to_agent_text,
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
            ambiguous_ids: vec![EguiAuthoredIdAmbiguity {
                id: "duplicate".into(),
                before_count: 2,
                after_count: 1,
            }],
            execution_handle_churn: vec![EguiAuthoredExecutionHandleChange {
                id: "canvas:stable".into(),
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
        "authored-handle-churn id=\"canvas:stable\" binding_ordinal=0 before_shape_index=3 after_shape_index=4 material=false continuity=frame_local_structure_sensitive"
    ));
}

#[test]
fn handle_churn_alone_projects_as_materially_empty() {
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
            ambiguous_ids: Vec::new(),
            execution_handle_churn: vec![EguiAuthoredExecutionHandleChange {
                id: "canvas:stable".into(),
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
    assert!(text.contains("material=false"));
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
