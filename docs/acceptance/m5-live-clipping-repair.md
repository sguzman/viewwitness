# M5 acceptance — live clipping repair

Date: 2026-09-12

This record captures the second ViewWitness live source-repair class against a real native eframe application. Unlike the first positional repair, this defect preserves object geometry and identity while making one keyed rendered sub-part fully invisible through clipping.

The purpose is to prove that ViewWitness's repair story is not merely a memorized coordinate-delta recipe.

## Acceptance run

- repository head: `f490300a0ba714f7394875170a170a40ca2774af`
- workflow: `M5 Live Source Repair`
- workflow run: `34719898781`
- job: `103623702427`
- evidence artifact: `m5-live-source-repair-evidence`
- artifact id: `10305243349`
- artifact digest: `sha256:8096c6b61dd7adea644b3ec3c119d06ec1058d5994babbde1421a8e54aabd17f`

The same workflow also reran and passed the earlier misplaced-handle repair. The clipping case is a separate source mutation/rebuild/recapture sequence in the same isolated native X11 environment.

## Injected broken source

The checked-in circle center normally paints through the canvas painter. The acceptance harness temporarily changes only the keyed `center` binding to use a zero-area clip:

```diff
-                    &painter,
+                    &painter.with_clip_rect(egui::Rect::from_min_max(rect.min, rect.min)),
                     "center",
```

The center's shape geometry remains unchanged. Only its observed clip evidence changes.

## Broken observation

Focused exact evidence for `showcase:painted-circle` / `center`:

```text
bounds=[639,321,8,8]
clip=[238,100,0,0]
visible_fraction=0
visible_bounds=none
verified=true
kind="circle"
correlation=same_full_output
```

The sibling keyed `ring` remained fully visible:

```text
bounds=[590,272,106,106]
clip=[238,100,554,360]
visible_fraction=1
verified=true
kind="circle"
```

This is a deliberately different defect from the misplaced rectangle handle: the center still exists at the correct bounds and with the correct identity/kind, but its observed clip eliminates its entire bounding-box visibility.

## Source repair

The harness repairs the source by restoring the original painter binding:

```diff
-                    &painter.with_clip_rect(egui::Rect::from_min_max(rect.min, rect.min)),
+                    &painter,
                     "center",
```

It then requires the working tree source to match the checked-in file exactly before rebuilding.

## Repaired observation

After rebuild and relaunch, focused exact evidence reported:

```text
center bounds=[639,321,8,8]
center clip=[238,100,554,360]
center visible_fraction=1
center visible_bounds=[639,321,8,8]
```

The ring remained unchanged:

```text
ring bounds=[590,272,106,106]
ring clip=[238,100,554,360]
ring visible_fraction=1
```

Therefore:

```text
center identity:          unchanged
center bounds:            unchanged
center kind:              unchanged
center clip:              repaired
center visible_fraction:  0 -> 1
ring evidence:            unchanged
```

## ViewWitness diff proof

`viewwitness diff-exact broken.yaml fixed.yaml` emitted:

```text
authored-binding-change object_id="showcase:painted-circle" authored_binding_id="center" field="clip_rect" before={"height":0.0,"width":0.0,"x":238.0,"y":100.0} after={"height":360.0,"width":554.0,"x":238.0,"y":100.0}
```

The acceptance harness additionally rejects:

- any material `center.bounds` change;
- any material `ring` change;
- authored object ambiguity for `showcase:painted-circle`;
- authored binding ambiguity for `center`;
- a broken center whose visible fraction is not exactly zero;
- a repaired center whose visible fraction is not exactly one;
- any difference in ring bounds/clip/visibility between captures.

All assertions passed.

## What this proves

M5 now contains two qualitatively different native source-repair acceptance classes:

```text
1. positional geometry defect
   evidence field: handle.bounds
   repair result: 60 px x movement, sibling outline stable

2. clipping/visibility defect
   evidence field: center.clip_rect
   repair result: visible_fraction 0 -> 1, center bounds stable, sibling ring stable
```

This establishes that authored per-binding clip evidence and derived bounding-box visibility are operational in a real repair loop, not merely synthetic model features.

It also demonstrates why ViewWitness keeps bounds and clipping separate: the broken center's geometry was perfectly stable while its visibility changed from none to complete.

## What this does not prove

The source-edit harness still knows the exact mutation and repair strings. It verifies the observation/edit/rebuild/recapture/diff machinery, but it does not prove that an arbitrary coding agent can locate the responsible source and choose the repair without pre-encoded guidance.

The next M5 milestone should therefore remove that scaffolding by separating the patcher from an independent verifier.

## Reproduction machinery

- `scripts/m5-clip-repair-loop.sh` owns this clipping-specific source mutation and acceptance assertions.
- `.github/workflows/m5-live-showcase.yml` executes both the positional and clipping repair classes under native X11/Xvfb.
- all source mutation occurs in the isolated workflow checkout and is restored during cleanup.
