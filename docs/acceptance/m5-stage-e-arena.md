# M5 acceptance provenance — Stage E agent arena

Status: **arena validated; Stage E not yet accepted**

Validated code head: `3eda2d79c67e325152609db2ed10063de85775a8`

Ordinary CI: run `34723502505` — format, default tests, and all-features tests passed.

Native workflow: `M5 Live Source Repair`, run `34723502520` — all steps passed.

Artifact: `m5-live-source-repair-evidence`, artifact id `10307520117`, SHA-256 `428a5e2d0a705b5a3d2e87667a27e20b814e44e6eec3c7f097f3bab1f03e318e`.

## Validation question

Is the Stage-E environment trustworthy enough to hand a GUI repair task to a real coding agent **without leaking the historical repair answer or letting the agent control its own judge**?

This record answers **yes** for the arena and verifier boundary. It does **not** claim that a coding agent has yet solved the task.

## Validated architecture

The native workflow exercised this separation:

```text
committed broken HEAD
    |
    +--> mutation-free broken evidence
    |
    +--> sanitized coding-agent workspace
            Cargo.toml + src/ + examples/
            no tests/docs/scripts/prompts/.github history
            sibling TASK.md + evidence/
                |
                v
          candidate patch boundary
                |
                v
trusted patch gate
    -> restrict mutation surface to examples/*.rs
    -> create fresh detached worktree from trusted HEAD
    -> apply candidate patch there
    -> build candidate showcase there
    -> observe candidate with trusted ViewWitness CLI
    -> decide acceptance with trusted verifier
```

The sandbox is generated from committed `HEAD`, not the caller's mutable working tree. This matters because the Stage-D reference patcher intentionally leaves a modified application tree during its own experiment; that state must never contaminate the Stage-E coding-agent baseline.

## Recipe-free task surface

`prompts/m5-handle-repair.md` describes the rendered defect and evidence-level acceptance goal. Static contract tests reject known repair leaks including:

- the expected source path;
- the historical broken expression;
- the historical replacement expression;
- the reference patcher name/output;
- the geometry expression that previously implemented the defect.

The coding-agent package contains the broken correlated envelope plus focused `handle` and `outline` projections. It contains ordinary buildable application/library source so an agent may inspect the code normally, but historical tests, docs, scripts, prompts, CI, and verifier machinery are deliberately absent.

The native run proved the sanitized workspace still builds independently.

## Trusted candidate gate

The control patch entered through `scripts/m5-verify-agent-patch.sh`.

The gate accepted only this candidate mutation surface:

```text
examples/showcase.rs
```

It recorded the trusted verifier SHA-256 and trusted repository head, then created a detached worktree from:

```text
3eda2d79c67e325152609db2ed10063de85775a8
```

Only the supplied candidate patch was applied to that worktree. The candidate showcase was built there. The observer CLI and verifier were invoked from the trusted checkout, so candidate source could not redefine the evidence machinery or its acceptance predicates.

The verifier script is invoked through `bash`; executable-file metadata is therefore not part of the trust contract.

## Control result

The Stage-D reference patch was used **only as a control input** to prove the Stage-E gate can transport and verify a valid patch. It is not evidence of coding-agent inference.

The trusted verifier observed:

```text
baseline handle:  [513,215,10,10]
candidate handle: [453,215,10,10]
outline before:   [307,169,152,102]
outline after:    [307,169,152,102]
correlation:      same_full_output
```

The exact authored diff retained one material rendered change:

```text
authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds"
```

The verifier accepted because the handle materially changed and geometrically reconnected while its size/y placement and the outline geometry remained stable, with no additional authored-binding material change or identity ambiguity.

## Restart-sensitive canonical semantics

During development of this arena, an intentionally stronger verifier briefly rejected any canonical AccessKit semantic diff across application restarts. A real native run disproved that rule: the debug inspector reflowed between processes, producing structure-sensitive AccessKit node/bounds churn while the authored rendered object evidence remained clean.

That veto was removed rather than hidden. The full canonical semantic diff remains evidence, but restart-sensitive generated identity/layout is not treated as a trustworthy collateral-change oracle for this authored custom-paint acceptance case.

This preserves the existing ViewWitness epistemic rule: evidence strength must match the actual continuity source.

## What this validates

The Stage-E **arena** is executable and independently guarded:

- broken evidence is produced without source mutation;
- the coding task does not encode the known patch recipe;
- the agent-visible workspace excludes historical answer surfaces;
- agent changes are constrained before verification;
- candidate application state is reconstructed in a clean worktree;
- observer/verifier code remains trusted and outside candidate mutation authority;
- acceptance is based on the rebuilt running GUI's evidence, not patch text;
- ordinary default/all-features CI and the native control workflow pass on the same code head.

## What this does not validate

**Stage E itself is not accepted.**

No general coding-agent handoff was available in the current execution environment, so no claim of agent inference is made. The reference patcher/control must not be relabeled as an autonomous solution.

Stage E becomes accepted only when a real coding agent receives the sanitized package, uses the ViewWitness evidence plus ordinary source inspection to locate the responsible code, chooses and emits its own candidate patch, and that patch passes this trusted gate.

The next experiment is therefore not another scripted transformation. It is the first genuine agent-produced patch through the already-validated arena.

## Artifact hygiene follow-up

The original arena-validation artifact was `217,951,549` bytes because the sandbox's independent `cargo check` wrote compilation intermediates beneath the preserved Stage-E workspace.

That workflow hygiene debt was removed at code head `6a7ee2b33060ca0c79f7e259465925005d65f4b5`. The sandbox still performs the independent buildability check, but sets `CARGO_TARGET_DIR` to a temporary directory outside the uploaded evidence tree and deletes it afterward.

The follow-up validation remained fully green:

```text
ordinary CI run: 34723867313
native M5 run:   34723867314
```

The same native workflow again passed both repair classes, mutation-free baseline capture, independent verification, recipe-free sandbox preparation, and the trusted clean-worktree candidate gate.

The resulting `m5-live-source-repair-evidence` artifact is:

```text
artifact id: 10307550539
size:        152,332 bytes
sha256:      d4852b1574c20110d5eb7c401fc7e0d6a5ff87fa5fed54ad3905d1cea44f6c52
```

That is about `99.93%` smaller than the original artifact while preserving the acceptance evidence and task/gate products. Build intermediates are no longer treated as evidence.
