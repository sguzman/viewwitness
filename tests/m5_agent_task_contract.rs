use std::fs;
use std::path::Path;

#[test]
fn stage_e_handle_task_contains_gui_goal_without_source_recipe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let task = fs::read_to_string(root.join("prompts/m5-handle-repair.md"))
        .expect("read Stage E handle coding-agent task");

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
            "Stage E handle task must not leak expected repair cue {forbidden:?}"
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
fn stage_e_clip_task_contains_gui_goal_without_source_recipe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let task = fs::read_to_string(root.join("prompts/m5-clip-repair.md"))
        .expect("read Stage E clip coding-agent task");

    for forbidden in [
        "examples/showcase.rs",
        "with_clip_rect",
        "painter.clone()",
        "m5-clip-repair-loop",
        "BROKEN_EXPR",
        "FIXED_EXPR",
    ] {
        assert!(
            !task.contains(forbidden),
            "Stage E clip task must not leak expected repair cue {forbidden:?}"
        );
    }

    assert!(task.contains("object_id=\"showcase:painted-circle\""));
    assert!(task.contains("binding_id=\"ring\""));
    assert!(task.contains("binding_id=\"center\""));
    assert!(task.contains("fully invisible according to its observed clipping evidence"));
    assert!(task.contains("the center becomes fully visible"));
    assert!(task.contains("the ring remains materially unchanged"));
    assert!(task.contains("git diff --binary > ../agent.patch"));
}

#[test]
fn stage_e_sandbox_builder_uses_committed_head_and_excludes_answer_history() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("scripts/m5-prepare-agent-sandbox.sh"))
        .expect("read Stage E sandbox builder");

    assert!(script.contains("archive_paths=(Cargo.toml src examples)"));
    assert!(script.contains("cat-file -e HEAD:Cargo.lock"));
    assert!(script.contains("archive_paths+=(Cargo.lock)"));
    assert!(script.contains("archive HEAD \"${archive_paths[@]}\""));
    assert!(script.contains("TASK_KIND=\"${3:-handle}\""));
    assert!(script.contains("TASK_SOURCE=\"$TRUSTED_ROOT/prompts/m5-handle-repair.md\""));
    assert!(script.contains("TASK_SOURCE=\"$TRUSTED_ROOT/prompts/m5-clip-repair.md\""));
    assert!(script.contains("FOCUS_A=\"handle-focus.txt\""));
    assert!(script.contains("FOCUS_A=\"center-focus.txt\""));
    assert!(script.contains("$WORKSPACE/tests"));
    assert!(script.contains("$WORKSPACE/docs"));
    assert!(script.contains("$WORKSPACE/scripts"));
    assert!(script.contains("$WORKSPACE/prompts"));
    assert!(script.contains("$WORKSPACE/.github"));
    assert!(script.contains("task-kind.txt"));
    assert!(script.contains("mktemp -d \"${TMPDIR:-/tmp}/viewwitness-m5-sandbox-target.XXXXXX\""));
    assert!(script.contains("CARGO_TARGET_DIR=\"$SANDBOX_CARGO_TARGET\""));
    assert!(script.contains("cargo check --example showcase --features showcase"));
}

#[test]
fn trusted_agent_patch_gate_rejects_non_example_rust_mutations_and_selects_verifier() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("scripts/m5-verify-agent-patch.sh"))
        .expect("read Stage E trusted patch gate");

    assert!(script.contains("before.startswith(\"examples/\")"));
    assert!(script.contains("before.endswith(\".rs\")"));
    assert!(script.contains("TASK_KIND=\"${4:-handle}\""));
    assert!(script.contains("VERIFIER_NAME=\"m5-verify-handle-candidate.sh\""));
    assert!(script.contains("VERIFIER_NAME=\"m5-verify-clip-candidate.sh\""));
    assert!(script.contains("VERIFIER_PATH=\"$TRUSTED_ROOT/scripts/$VERIFIER_NAME\""));
    assert!(script.contains("worktree add --detach"));
    assert!(script.contains("candidate worktree only as the application build root"));
    assert!(script.contains("bash \"$VERIFIER_PATH\""));
}
