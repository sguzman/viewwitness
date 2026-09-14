# M5 acceptance provenance — Stage F multi-defect independent-agent repair

Status: **Stage F accepted**

Stage E established one genuine blinded coding-agent repair for the `handle` geometry defect. Stage F asks whether the same trust topology succeeds across a materially different defect class rather than collapsing into one memorized source idiom.

This record answers **yes** by adding an independently chosen `clip` repair to the previously accepted `handle` repair.

## Accepted independent-agent repairs

```text
handle
  defect class: geometry / placement
  accepted run: 34845296759
  artifact:     10347108708

clip
  defect class: clipping / visibility
  accepted run: 34849119878
  job:          103992277586
  artifact:     10348969634
```

Both tasks used the same blinded handoff discipline:

```text
sanitized task package
    -> independent coding agent chooses repair
    -> opaque agent.patch
    -> restricted mutation-surface gate
    -> fresh detached candidate worktree
    -> rebuilt application
    -> trusted ViewWitness exact capture
    -> task-specific mutation-free verifier
```

The coding agent never owned its judge.

## Stage-F clip handoff

Codex received only the sanitized clipping task package produced from the already-validated Stage-E arena. The package contained ordinary buildable source plus the broken exact envelope and focused `center` / `ring` evidence. It excluded historical tests, docs, scripts, prompts, CI, verifier machinery, reference-control output, and original ViewWitness Git history.

The returned candidate patch was treated as opaque before verification.

Candidate patch SHA-256:

```text
d891febd5a2202e6cba2c3470f41b0dba308c74b82d4d2d3d6219f07a1394394
```

The trusted gate accepted only this mutation surface:

```text
examples/showcase.rs
```

Trusted clip verifier SHA-256 recorded by the gate:

```text
fd5694f88cdc6004a8c85ca0297dc3be48c9b6db5ec6767c99b4703f939f04ce
```

## Broken clip baseline

The mutation-free baseline reproduced the intended clipping defect:

```text
object:                  showcase:painted-circle
center binding:          unique
ring binding:            unique
correlation:             same_full_output

center bounds:           [639,321,8,8]
center clip:             [238,100,0,0]
center visible_fraction: 0
ring bounds:             [590,272,106,106]
ring visible_fraction:   1
```

The center geometry existed and remained identifiable, but its observed clip rectangle had zero area and its derived visible fraction was zero.

## Candidate evidence

The rebuilt candidate produced:

```text
center bounds:           [639,321,8,8] unchanged
center clip:             [238,100,554,360]
center visible_fraction: 1
ring bounds:             [590,272,106,106] unchanged
ring visible_fraction:   1
correlation:             same_full_output
```

The exact authored diff contained one intended material change:

```text
authored-binding-change object_id="showcase:painted-circle" authored_binding_id="center" field="clip_rect"
```

with:

```text
before: { x: 238, y: 100, width: 0,   height: 0 }
after:  { x: 238, y: 100, width: 554, height: 360 }
```

Center kind and geometry stayed stable. The sibling ring stayed materially unchanged. No extra authored-binding material change or identity ambiguity was introduced.

The trusted verifier emitted:

```text
M5 independent clip verifier accepted candidate: center became fully visible, center_bounds=(639.0, 321.0, 8.0, 8.0), ring unchanged
M5 mutation-free clipped-center candidate verification succeeded
M5 trusted agent-patch verification succeeded: kind=clip
```

## Preserved Stage-F evidence

```text
run:          34849119878
job:          103992277586
artifact:     m5-stage-f-codex-clip-verification
artifact id:  10348969634
size:         16,561 bytes
artifact sha: cf2b2e08e2f9b1517c08f53fae65306561efd56157fa43d696a31ed99cbb059e
```

The temporary verification branch was reset to the pre-experiment trusted project head after evidence preservation, so the candidate payload and temporary workflow plumbing are not retained as active project state.

## Why this accepts Stage F

Stage E proved one coding agent could infer a valid repair from ViewWitness evidence plus ordinary source inspection while remaining blind to the answer key.

Stage F required the same independent-agent protocol to succeed across more than one defect class/task kind. That condition is now satisfied:

```text
geometry / placement
  handle.bounds changes
  sibling outline remains stable

clipping / visibility
  center.clip_rect changes
  center geometry remains stable
  sibling ring remains stable
```

The second success is not merely another coordinate-offset instance. Its accepted state transition is expressed in clipping and visibility evidence rather than object geometry.

**Stage F is accepted.**

## Next pressure — Stage G

Stage G should no longer ask whether an agent can repair a cleanly evidenced defect. It should pressure what happens when the evidence itself is incomplete, contradictory, ambiguous, or differently strong across layers.

Candidate pressure cases include:

- semantic evidence and authored-paint evidence disagreeing about apparent state;
- duplicate or ambiguous authored identity preventing a unique continuity claim;
- geometry evidence present while raster evidence is missing or weak;
- raster appearance suggesting a problem while semantic/custom-paint evidence cannot uniquely attribute it;
- partial capture or missing binding verification;
- two plausible repair hypotheses that the current evidence cannot distinguish.

The acceptance criterion should be epistemic rather than heroic: ViewWitness and the consuming agent must preserve uncertainty, refuse invented linkage, and distinguish what is observed from what is merely plausible.
