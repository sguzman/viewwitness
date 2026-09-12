use std::fs;
use std::path::Path;

#[test]
fn stage_e_agent_task_contains_gui_goal_without_source_recipe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let task = fs::read_to_string(root.join("prompts/m5-handle-repair.md"))
        .expect("read Stage E coding-agent task");

    for forbidden in [
        "examples/showcase.rs",
        "vec2(60.0",
        "vec2(0.0",
        "m5-reference-handle-patcher",
        "source-repair.patch",
        "first.right_center()",
    ] {
        assert!(
            !task.contains(forbidden),
            "Stage E task must not leak expected repair cue {forbidden:?}"
        );
    }

    assert!(task.contains("object_id=\"showcase:painted-rectangle\""));
    assert!(task.contains("binding_id=\"handle\""));
    assert!(task.contains("binding_id=\"outline\""));
    assert!(task.contains("horizontally disconnected"));
    assert!(task.contains("outline stays materially where it was"));
    assert!(task.contains("git diff --binary > ../agent.patch"));
}

#[test]
fn stage_e_sandbox_builder_uses_committed_head_and_excludes_answer_history() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("scripts/m5-prepare-agent-sandbox.sh"))
        .expect("read Stage E sandbox builder");

    assert!(
        script.contains("git -C \"$TRUSTED_ROOT\" archive HEAD Cargo.toml Cargo.lock src examples")
    );
    assert!(script.contains("$WORKSPACE/tests"));
    assert!(script.contains("$WORKSPACE/docs"));
    assert!(script.contains("$WORKSPACE/scripts"));
    assert!(script.contains("$WORKSPACE/prompts"));
    assert!(script.contains("$WORKSPACE/.github"));
    assert!(script.contains("cargo check --example showcase --features showcase"));
}

#[test]
fn trusted_agent_patch_gate_rejects_non_example_rust_mutations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("scripts/m5-verify-agent-patch.sh"))
        .expect("read Stage E trusted patch gate");

    assert!(script.contains("before.startswith(\"examples/\")"));
    assert!(script.contains("before.endswith(\".rs\")"));
    assert!(script.contains("worktree add --detach"));
    assert!(script.contains("m5-verify-handle-candidate.sh"));
    assert!(script.contains("candidate worktree only as the application build root"));
}
