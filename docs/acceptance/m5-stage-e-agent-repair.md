# M5 acceptance provenance — Stage E independent coding-agent repair

Status: **Stage E accepted**

Accepted task kind: `handle`

Sanitized task source head: `1a526beb26aa97e800fa32b481861d351410abde`

Independent coding agent: Codex, operating only inside the sanitized Stage-E package.

Candidate patch SHA-256: `2df25897244a5a9ef49fd62f3301f12756632542e4700e26b83f8495a45a4bc3`

Trusted verification run: `34845296759`

Trusted verification job: `103979649037`

Preserved artifact: `m5-stage-e-codex-handle-verification`

Artifact id: `10347108708`

Artifact size: `16,610` bytes

Artifact ZIP SHA-256: `c2517c9dbe77be4a2d4a0de7e64870764ddcb277af8ca859692b38d08de3dc9e`

## Acceptance question

Can a coding agent that does **not** have ViewWitness history, tests, verifier code, reference-patcher output, or the known repair recipe use only a sanitized ordinary source tree plus ViewWitness evidence to locate the responsible application code, choose its own repair, emit a patch, and have that patch accepted by the pre-existing trusted evidence verifier?

For the misplaced-handle task, this run answers **yes**.

That is the exact Stage-E boundary. The agent chose the repair; ViewWitness retained observation and acceptance authority.

## Blind handoff

The agent received the sanitized Stage-E handle package rather than the real repository checkout. The package contained:

```text
TASK.md
task-kind.txt
trusted-head.txt
visible-files.txt
evidence/broken.yaml
evidence/handle-focus.txt
evidence/outline-focus.txt
workspace/Cargo.toml
workspace/Cargo.lock
workspace/src/**
workspace/examples/**
```

The workspace contained a fresh local Git baseline for patch production, but no original ViewWitness Git history.

The package deliberately excluded:

```text
tests/
docs/
scripts/
prompts/
.github/
trusted verifier machinery
reference-control output
original Git history
```

The coding agent was told only to read `TASK.md`, work inside that isolated package, and emit the required `agent.patch`. Its explanation was not used as acceptance evidence.

## Candidate boundary

The returned `agent.patch` was treated as an opaque candidate until it entered the trusted gate.

The trusted gate accepted exactly one mutated application path:

```text
examples/showcase.rs
```

No ViewWitness library code, tests, docs, workflow logic, evidence, task text, or verifier was within the candidate mutation surface.

The candidate patch hash was checked before verification:

```text
2df25897244a5a9ef49fd62f3301f12756632542e4700e26b83f8495a45a4bc3
```

## Trusted execution topology

The ordinary execution sandbox used for the director session could not resolve GitHub over `git clone`, so the candidate was verified on a native GitHub Actions runner rather than weakening the experiment.

A temporary verification branch began from the accepted dual-task integration source head. Its only committed difference before the run was workflow plumbing needed to materialize the exact opaque candidate bytes and invoke the already-existing trusted Stage-E gate. Application source and verifier code were unchanged.

The run then executed:

```text
trusted broken source
    -> build trusted showcase + ViewWitness observer
    -> mutation-free broken baseline capture
    -> materialize exact agent.patch bytes
    -> verify patch SHA-256
    -> scripts/m5-verify-agent-patch.sh ... handle
        -> validate candidate mutation surface
        -> detached fresh worktree from trusted HEAD
        -> apply candidate only there
        -> build candidate showcase
        -> observe candidate with trusted ViewWitness CLI
        -> scripts/m5-verify-handle-candidate.sh decides acceptance
```

The trusted handle verifier SHA-256 recorded by the gate was:

```text
23e97ee3b6dc940a60a892a8b479b0e10966cdd253cb3ead9f37b301fb5d963b
```

After the run completed, the temporary verification branch was force-reset to the trusted source baseline so the candidate payload and temporary workflow plumbing were not retained as an active project branch state.

## Broken baseline

The mutation-free baseline capture reproduced the intended defect:

```text
object:            showcase:painted-rectangle
handle binding:    unique
outline binding:   unique
correlation:       same_full_output

handle bounds:     [513,215,10,10]
outline bounds:    [307,169,152,102]
horizontal gap:    54
```

The handle was vertically aligned with the rectangle but horizontally disconnected.

## Candidate evidence

The rebuilt candidate produced:

```text
handle bounds:     [453,215,10,10]
outline bounds:    [307,169,152,102]
correlation:       same_full_output
```

The exact authored diff contained the intended material change:

```text
authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds"
```

with:

```text
before: { x: 513, y: 215, width: 10, height: 10 }
after:  { x: 453, y: 215, width: 10, height: 10 }
```

No second authored-binding material change was present. The outline remained exactly stable. Handle y-position and size remained stable. The handle geometrically reconnected to the rectangle.

The trusted verifier emitted:

```text
M5 independent verifier accepted candidate: handle reconnected, outline unchanged
M5 mutation-free candidate verification succeeded
M5 trusted agent-patch verification succeeded: kind=handle
```

## Why this accepts Stage E

This run satisfies the Stage-E requirements that the earlier reference controls deliberately did not satisfy:

- a real independent coding agent received the recipe-free sanitized package;
- the agent did not receive historical tests, docs, scripts, verifier code, reference patch, or project Git history;
- the agent inspected ordinary source and ViewWitness evidence and chose its own source repair;
- the returned patch crossed the pre-existing restricted mutation boundary;
- the patch was reconstructed inside a fresh detached candidate worktree;
- trusted ViewWitness code observed the rebuilt candidate;
- the mutation-free trusted verifier judged live evidence rather than patch text;
- the verifier accepted the intended GUI state transition with collateral authored evidence stable.

The distinction between proposer and judge therefore survived a genuine coding-agent handoff.

**Stage E is accepted.**

## What this does not yet prove

One successful independent repair does not establish generality across defect classes.

The accepted agent task was the `handle` geometry defect. The clipping task has a separately validated sanitized package, evidence contract, and trusted verifier, but its successful runs so far are reference controls rather than independently chosen agent repairs.

That becomes Stage F pressure:

```text
same independent-agent protocol
    -> succeeds on more than one defect class/task kind
```

The natural next specimen is the already-prepared `clip` task because it requires a visibility/clipping repair while preserving center geometry and sibling-ring evidence, rather than repeating the handle geometry idiom.
