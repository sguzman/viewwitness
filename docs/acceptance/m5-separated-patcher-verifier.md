# M5 acceptance — separated patcher and verifier

Status: **accepted**

Accepted head: `bb393d7cc6bc8aaba6aad68d34a625ebff69fd66`

Native workflow: `M5 Live Source Repair`, run `34721172502`

Artifact: `m5-live-source-repair-evidence`, artifact id `10306276885`, SHA-256 `85d46a5c32c0804fc824f8195b14229f6a689e8a359d99254477356be983cc9d`

Ordinary CI on the same head also passed default and all-features tests.

## Acceptance question

Can ViewWitness verification decide whether a GUI source repair succeeded **without the verifier containing the repair recipe or mutating source itself**?

For this acceptance case the checked-in showcase starts with the keyed rectangle handle visibly disconnected from its keyed rectangle outline.

The run separates the workflow into three executable actors:

```text
mutation-free baseline observer
    -> isolated reference patcher
    -> mutation-free independent verifier
```

The reference patcher is deliberately allowed to know a repair recipe. The verifier is not.

## Baseline evidence

The observer captured the real native eframe showcase through exact same-pass capture:

```text
object_id="showcase:painted-rectangle"
binding_id="handle"
object_match_count=1
binding_match_count=1
correlation=same_full_output
bounds=[513,215,10,10]
```

The sibling outline was:

```text
object_id="showcase:painted-rectangle"
binding_id="outline"
bounds=[307,169,152,102]
```

The baseline observer independently checked that the handle was vertically aligned with the outline but horizontally disconnected, with a 54 logical-pixel bounding-box gap.

## Patcher evidence

The isolated reference patcher searched application Rust surfaces rather than being handed the expected file path. It located:

```text
examples/showcase.rs
```

Its candidate patch changed the known pressure defect in its own candidate source tree. This recipe is intentionally confined to the patcher role and artifact.

## Independent verifier evidence

The verifier rebuilt the candidate tree it was given, launched the same deterministic native scenario, captured a new exact envelope, and observed:

```text
candidate handle:  [453,215,10,10]
candidate outline: [307,169,152,102]
correlation:        same_full_output
```

The full correlated diff contained exactly one material authored-binding change:

```text
authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds" before={"height":10.0,"width":10.0,"x":513.0,"y":215.0} after={"height":10.0,"width":10.0,"x":453.0,"y":215.0}
```

The verifier accepted because the candidate satisfied state-level evidence predicates:

- rectangle object, `handle`, and `outline` remained uniquely identifiable;
- exact capture retained `same_full_output` provenance;
- outline geometry remained unchanged;
- handle y-position and size remained unchanged;
- handle geometry materially changed from the broken baseline;
- candidate handle geometrically reconnected with the outline;
- no authored object/binding ambiguity was introduced;
- no additional authored binding material change was present.

## Anti-recipe guard

`tests/m5_verifier_contract.rs` permanently guards the role boundary. The independent verifier is forbidden from containing, among other source-repair cues:

```text
examples/showcase.rs
vec2(60.0
vec2(0.0
git checkout
git restore
git apply
sed -i
write_text(
.replace(
```

The same test explicitly confirms that the reference patcher may contain the repair constants while the verifier may not.

This is not merely documentation: default and all-features CI passed with that contract test on the accepted head.

## What this proves

Stage D is accepted:

```text
observer owns evidence
patcher owns repair hypothesis and source mutation
verifier owns acceptance predicates
```

The actor that decides whether the GUI repair succeeded no longer encodes how to perform the repair.

## What this does not prove

The patcher is still a reference harness with a pre-encoded source transformation. This acceptance does **not** prove that a coding agent can infer the responsible source and choose a patch from ViewWitness evidence.

That is Stage E.

The next acceptance must give the coding agent:

- the broken repository state;
- complete and focused ViewWitness evidence;
- a GUI/evidence-level repair goal;
- access to ordinary source search/edit/build tools;
- the independent verifier command.

It must **not** give the agent the expected source path, broken expression, replacement expression, or patch.

The candidate is accepted only by the independent verifier after the coding agent has chosen and applied its own source change.
