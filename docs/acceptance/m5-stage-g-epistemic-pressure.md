# M5 Stage G — epistemic pressure

Status: **current; initial pressure slices accepted. Stage G is not yet complete.**

Stage G asks whether ViewWitness preserves uncertainty and disagreement instead of manufacturing a convenient answer when the available evidence is incomplete, ambiguous, contradictory, or differently strong across layers.

The governing rule is:

> Absence, ambiguity, disagreement, and observation are different states. ViewWitness must not collapse one into another merely to produce a convenient answer.

## Accepted footholds

### Ambiguous authored continuity blocks unique attribution

Duplicate authored object or binding identities remain explicit ambiguity. ViewWitness refuses to guess a cross-frame match and exposes whether unique authored attribution is blocked.

`NoKnownAmbiguity` is deliberately narrow: it means no authored ambiguity is currently known. It does **not** mean the evidence is complete, globally trustworthy, or sufficient for a repair decision.

### Unresolved visibility is unknown, not zero

An unresolved authored paint binding may mechanically report zero through a low-level geometry helper because no usable resolved geometry exists. Stage G does not promote that mechanical value into an observation that the binding was invisible.

At the epistemic layer:

- unresolved or geometry-incomplete testimony yields `visible_fraction = None`;
- a verified binding that is genuinely fully clipped yields `Some(0.0)`.

This preserves the distinction between **not observed** and **observed invisible**.

### Contradictory resolver testimony is first-class

The authored-binding resolver contract now exposes impossible combinations rather than silently choosing one field as authoritative. Examples include a binding marked verified without final paint kind testimony, or an unverified binding that nevertheless carries final kind/bounds/clip testimony.

Contradictory evidence cannot produce a visibility claim. The contradiction and all detected conflict kinds remain available to downstream consumers.

A verified slot may still lack usable bounds; verification of the slot therefore does not itself imply that visibility is known.

### Agent text preserves epistemic state

The agent-facing correlated capture projection now carries the same distinctions instead of flattening them:

- `evidence_consistency=...`;
- explicit conflict kinds when contradictory;
- unresolved clip testimony as `clip=unknown`;
- unresolved, contradictory, or geometry-incomplete visibility as `visible_fraction=unknown`.

This prevents agent consumers from receiving stronger claims than the underlying evidence supports.

### Material change and blocked attribution can coexist

The correlated-diff summary now exposes authored continuity directly in its first line:

- `authored_continuity=...`;
- `unique_attribution_blocked=...`;
- object and binding ambiguity counts.

A mixed pressure test proves that a real material change for one uniquely matched binding can coexist with a separate authored identity whose matching is refused. ViewWitness preserves both truths: the valid material change remains actionable evidence, while the ambiguous identity remains unattributed.

The summary does not emit a generic confidence score and does not convert `NoKnownAmbiguity` into a claim of completeness.

## Non-claims

These accepted slices do **not** claim:

- a global confidence score for a capture or diff;
- that absence of known ambiguity means the evidence is complete;
- that an authored binding's existence implies an intention that it be visible;
- that a verified paint slot necessarily provides usable geometry;
- a generic semantic-node ↔ paint-shape identity mapping;
- that weak or missing raster evidence can be promoted into trustworthy authored attribution.

## Provenance

Accepted implementation and regression points include:

- `f30b9c43f90dc9bbe2c6736a3107e1c12ae53b10` — authored continuity ambiguity assessment;
- `20ec5b4af983db95504586350a18bd3df8189f5c` — unresolved versus invisible visibility evidence;
- `31225916e1a73d7c46d98fa1039a4732e27836cf` — contradictory authored-binding evidence assessment;
- `ada0cdd97e80aadafc112c92b2d91bf401b00171` — epistemic state exposed in authored agent text;
- `1c9a128d89ffae0a6ca990e98a0e1f205d42e28e` — formatted agent-text head with ordinary CI `34861804498` and full M5 native regression `34861804495` green;
- `55069759bd9724999df52137dcdf178bea190743` — mixed ambiguity/material-change diff projection, with ordinary CI `34864879293` green across format, default tests, and all-features tests.

Earlier focused ordinary CI runs also include `34850574384`, `34851162015`, and `34860779878` for the initial epistemic slices.

## Remaining Stage G pressure

Before Stage G can close, the remaining question is precedence under mixed weak evidence: when incomplete testimony and contradictory testimony coexist, contradiction must not be softened into mere absence, and absence must not be upgraded into contradiction without an actual conflicting claim.

That pressure should remain narrow and executable. It should not introduce a global confidence engine merely to close the milestone.
