# Agent patch contract

This document defines the M5 boundary between evidence production, source mutation, and acceptance. A coding agent may propose and apply a source repair, but ViewWitness verification must remain independent of the repair recipe.

The purpose is to prevent a fake success mode in which the “verifier” already contains the exact source search string, replacement string, file path, or patch that makes the acceptance case pass.

## Roles

### Observer / evidence producer

The observer may:

- launch the known broken application;
- request exact captures;
- preserve complete correlated envelopes;
- project focused evidence for a named authored object/binding;
- provide the repository plus evidence to a coding agent.

The observer must not mutate application source.

### Patcher / coding agent

The patcher may:

- inspect repository source;
- search for code related to the observed object/binding or defect;
- choose which file(s) and expression(s) to change;
- modify source through the normal coding workflow;
- rebuild locally if useful before handing the candidate to verification.

The patcher owns the repair hypothesis. ViewWitness does not.

For Stage E and later, the coding task itself must not encode the expected source location or transformation. The agent must locate and choose its patch from the evidence plus ordinary repository inspection.

### Independent verifier

The verifier may:

- build whatever candidate source tree it is given;
- launch that candidate application;
- capture exact evidence;
- compare candidate evidence with the preserved broken baseline;
- enforce state-level acceptance predicates;
- reject ambiguity, collateral material change, or loss of provenance.

The verifier must not:

- edit application source;
- contain the source file path of the expected fix unless that path is independently part of the acceptance surface;
- contain the broken source expression;
- contain the expected replacement expression;
- invoke `git checkout` or otherwise repair/restore application source as part of deciding success;
- infer success from the candidate patch text instead of the running candidate's evidence.

Cleanup of temporary processes/artifacts is allowed. Source mutation is not.

## Evidence handoff

A repair task should give the coding agent at least:

1. the repository at a reproducible broken state;
2. the full broken `EguiCorrelatedCapture` envelope;
3. focused projections for the relevant authored object/binding(s) when known;
4. a human-readable acceptance goal expressed in GUI/evidence terms rather than source terms;
5. the command for independent verification.

For the misplaced-handle case, a valid goal is conceptually:

```text
The keyed rectangle handle is displaced from its rectangle.
Repair the application so the handle is attached to the rectangle's right edge.
Do not materially move the rectangle outline.
```

For the clipped-center case, a valid goal is conceptually:

```text
The keyed circle center exists geometrically but is fully invisible according to observed clipping evidence.
Repair the application so the center is fully visible.
Keep the sibling ring materially unchanged.
```

A bad goal is:

```text
Replace the known broken expression in the known showcase source file with the expected fixed expression.
```

That is a patch recipe, not a diagnosis task.

## Verifier acceptance semantics — misplaced handle

The independent verifier operates only on evidence from the broken baseline and the rebuilt candidate.

Required candidate properties:

- `showcase:painted-rectangle` remains uniquely identifiable;
- keyed `handle` remains uniquely identifiable;
- keyed `outline` remains uniquely identifiable;
- `outline` material geometry is unchanged from the broken baseline;
- `handle` size and vertical placement remain stable;
- `handle` materially changes from the broken baseline;
- candidate handle and outline geometrically reconnect: their horizontal spans overlap and their vertical centers align within tolerance;
- no object/binding ambiguity is introduced;
- no additional authored binding material change is introduced;
- exact capture still reports `same_full_output` correlation.

Notice what is intentionally absent:

- no expected displacement magnitude in source;
- no expected source file;
- no expected Rust expression;
- no required patch shape.

A coding agent could remove a conditional, change an offset calculation, introduce a helper, derive the center differently, or refactor the canvas code entirely. If the rebuilt GUI satisfies the evidence contract without collateral material damage, verification may accept it.

Canonical AccessKit state is still preserved in the complete exact envelopes and diff, but restart-sensitive generated AccessKit identity/layout is not a blanket veto for this authored custom-paint case. A native pressure run showed the inspector subtree can reflow between application processes while the authored rendered evidence remains clean. Acceptance therefore follows the strongest continuity source actually available instead of pretending generated semantic identity is stronger than it is.

## Verifier acceptance semantics — clipped center

The independent clipping verifier follows the same recipe-free rule.

Required candidate properties:

- `showcase:painted-circle` remains uniquely identifiable;
- keyed `center` and `ring` remain uniquely identifiable;
- center kind and material bounds remain stable;
- center becomes fully visible according to observed clip evidence;
- sibling ring evidence remains materially stable;
- no extra authored-binding material change or identity ambiguity is introduced;
- exact capture remains correlated.

The verifier deliberately does not care whether the repair removes a clip painter, changes a clip rectangle, moves clipping responsibility elsewhere, or refactors paint construction. It judges the rebuilt rendered state.

## Stage D acceptance

Stage D is accepted at head `bb393d7cc6bc8aaba6aad68d34a625ebff69fd66`.

The live native workflow executes three distinct actors:

```text
scripts/m5-capture-handle-baseline.sh
    -> scripts/m5-reference-handle-patcher.sh
    -> scripts/m5-verify-handle-candidate.sh
```

The first and third actors are mutation-free. The middle actor alone owns source mutation.

`tests/m5_verifier_contract.rs` guards the verifier against expected source paths, repair constants, text replacement, patch application, and source restoration logic. Ordinary default/all-features CI and the native workflow both passed on the accepted head.

Canonical evidence and workflow provenance live in `docs/acceptance/m5-separated-patcher-verifier.md`.

## Stage E task contract

Stage E removes the final recipe from the patching side.

ViewWitness currently has two recipe-free task families:

```text
handle
  prompt:   prompts/m5-handle-repair.md
  evidence: broken envelope + handle/outline focus
  verifier: scripts/m5-verify-handle-candidate.sh

clip
  prompt:   prompts/m5-clip-repair.md
  evidence: broken envelope + center/ring focus
  verifier: scripts/m5-verify-clip-candidate.sh
```

Neither task may receive:

```text
expected source path
broken source expression
replacement expression
reference patch
reference patcher output
```

The coding agent must discover the responsible source itself, choose its own repair, and emit a candidate patch for the independent verifier.

The reference/control patches remain useful only to prove transport, reconstruction, and verifier behavior. They are not evidence of Stage-E completion.

## Stage E arena validation

The infrastructure needed for a genuine Stage-E experiment is now validated against both task kinds.

The sanitized agent package is built from committed broken `HEAD`, not a dirty patcher checkout. It contains buildable Cargo metadata plus `src/` and `examples/`, with the selected task/evidence beside the workspace, while historical tests, docs, scripts, prompts, CI, and verifier machinery are absent. This prevents the agent from reading previous repair answers out of acceptance history.

The trusted candidate gate then:

```text
receive explicit task kind: handle | clip
    -> select trusted task-specific verifier
    -> validate patch paths
    -> allow only examples/*.rs for this experiment
    -> create fresh detached worktree from trusted HEAD
    -> apply candidate patch there
    -> build candidate application there
    -> observe it with trusted ViewWitness tooling
    -> judge it with the selected mutation-free verifier
```

The code-bearing dual-task integration head is `1a526beb26aa97e800fa32b481861d351410abde`. Native `M5 Live Source Repair` run `34725470386` passed both recipe-free sandbox builds and both trusted clean-worktree control gates. The handle control was accepted from geometry evidence; the clip control was accepted from clipping/visibility evidence while center geometry and sibling ring evidence remained stable.

The current post-contract-test head `4cd6bbfc64733c9077c9b0652ec49607375ba559` also passed ordinary format/default/all-features CI in run `34725563814`.

That closes the previous one-defect pressure on the **arena and verifier topology**. It does not accept Stage E itself: the successful patches in the native workflow were known-good controls rather than independently chosen coding-agent repairs.

Canonical arena provenance lives in `docs/acceptance/m5-stage-e-arena.md`.

## Acceptance stages

M5 progression is cumulative:

```text
Stage A  capture/diff semantics                       accepted
Stage B  harness-controlled source edit               accepted
Stage C  two qualitatively different repair classes  accepted
Stage D  patcher/verifier separation                  accepted
Stage E  coding agent locates source + chooses patch  current
Stage F  multiple agent-solved defects/tasks
Stage G  incomplete/ambiguous evidence pressure
```

Stage E becomes accepted only when a real coding agent receives one of the sanitized packages, locates the responsible source from ViewWitness evidence plus ordinary repository inspection, chooses/emits its own patch, and that patch passes the already-validated trusted gate.

Stage F should then require successful independent agent repair across more than one defect class/task kind so the result cannot collapse into one memorized repair idiom.

Later stages must not weaken the evidence/provenance guarantees established earlier.

## Architectural rules

**The component that decides whether a GUI repair succeeded must not also encode how to perform that repair.**

For Stage E and later:

**The task given to the coding agent describes the broken GUI state and acceptance evidence, not the expected source patch.**

And the candidate must not own its judge:

**Agent-authored application changes are reconstructed inside a fresh candidate tree while observation and acceptance execute from trusted code outside that mutation surface.**
