# Agent patch contract

This document defines the next M5 boundary: a coding agent may propose and apply a source repair, but ViewWitness verification must remain independent of the repair recipe.

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

### Patcher

The patcher may:

- inspect repository source;
- search for code related to the observed object/binding or defect;
- choose which file(s) and expression(s) to change;
- modify source through the normal coding workflow;
- rebuild locally if useful before handing the candidate to verification.

The patcher owns the repair hypothesis. ViewWitness does not.

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

A repair task should give the patcher at least:

1. the repository at a reproducible broken state;
2. the full broken `EguiCorrelatedCapture` envelope;
3. a focused projection for the relevant authored object/binding when one is known;
4. a human-readable acceptance goal expressed in GUI/evidence terms rather than source terms;
5. the command for independent verification.

For the misplaced-handle case, a valid goal is conceptually:

```text
The keyed rectangle handle is displaced from its rectangle.
Repair the application so the handle is attached to the rectangle's right edge.
Do not materially move the rectangle outline.
```

A bad goal is:

```text
Replace `+ egui::vec2(60.0, 0.0)` with `+ egui::vec2(0.0, 0.0)` in examples/showcase.rs.
```

The latter is a patch recipe, not a diagnosis task.

## Verifier acceptance semantics — misplaced handle

The independent verifier should operate only on evidence from the broken baseline and the rebuilt candidate.

Required candidate properties:

- `showcase:painted-rectangle` remains uniquely identifiable;
- keyed `handle` remains uniquely identifiable;
- keyed `outline` remains uniquely identifiable;
- `outline` material geometry is unchanged from the broken baseline;
- `handle` size and vertical alignment remain stable;
- `handle` materially changes from the broken baseline;
- candidate handle and outline geometrically reconnect: their horizontal spans overlap and their vertical centers align within tolerance;
- no object/binding ambiguity is introduced;
- exact capture still reports `same_full_output` correlation.

Notice what is intentionally absent:

- no expected 60-pixel source constant;
- no expected source file;
- no expected Rust expression;
- no required patch shape.

A patcher could remove the conditional, change the offset calculation, introduce a helper, derive the center differently, or refactor the canvas code entirely. If the rebuilt GUI satisfies the evidence contract without collateral material damage, verification may accept it.

## Verifier acceptance semantics — clipping case

For future separated verification of the circle-center clipping defect, the same principle applies:

- center identity/bounds/kind remain stable;
- center becomes visible according to observed clip evidence;
- sibling ring evidence remains materially stable;
- the verifier does not care whether the patch removes a clip painter, changes a clip rectangle, moves clipping responsibility elsewhere, or refactors paint construction.

## Acceptance stages

M5 progression should now be described explicitly:

```text
Stage A  capture/diff semantics                    accepted
Stage B  harness-controlled source edit            accepted
Stage C  two qualitatively different repair classes accepted
Stage D  patcher/verifier separation                current
Stage E  coding agent locates source + chooses patch
Stage F  multiple defects without recipe-specific prompting
Stage G  incomplete/ambiguous evidence pressure
```

Stages are cumulative. Later stages must not weaken the evidence/provenance guarantees established earlier.

## Architectural rule

**The component that decides whether a GUI repair succeeded must not also encode how to perform that repair.**

That rule is now part of the M5 acceptance architecture.
