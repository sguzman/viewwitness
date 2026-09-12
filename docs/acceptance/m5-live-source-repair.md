# M5 acceptance — live source repair

Date: 2026-09-12

This record captures the first ViewWitness experiment that crossed the source-edit boundary against a real native eframe application.

It is an acceptance record, not a synthetic fixture. The checked-in showcase remains intentionally broken so the experiment can be rerun. The repair is applied inside the isolated CI workspace, the application is rebuilt, and the workspace source is restored during cleanup.

## Acceptance run

- repository head: `1e833bd49fdc327322a615dadd1d5deb26755a7a`
- workflow: `M5 Live Source Repair`
- workflow run: `34719588530`
- job: `103622858981`
- evidence artifact: `m5-live-source-repair-evidence`
- artifact id: `10305574058`
- artifact digest: `sha256:9bf891a38ccbb9e52ae0f1b2a66cb6c4da44629fff12e3c21e81d302788447ab`

The workflow runs the native showcase under Xvfb with the deterministic `misplaced-handle` startup scenario. Both before and after states are captured from the application's exact ViewWitness endpoint, not reconstructed from test fixtures.

## Broken observation

Focused exact evidence for `showcase:painted-rectangle` / `handle`:

```text
bounds=[513,215,10,10]
verified=true
kind="rect"
visible_fraction=1
correlation=same_full_output
```

The paired `outline` binding was:

```text
bounds=[307,169,152,102]
verified=true
kind="rect"
visible_fraction=1
```

The observed defect was positional rather than clipping or disappearance. The handle was fully visible but displaced from the rectangle edge.

## Source repair

After observing the broken application, the workflow modifies the application source itself:

```diff
-        first.right_center() + egui::vec2(60.0, 0.0)
+        first.right_center() + egui::vec2(0.0, 0.0)
```

The native showcase is then rebuilt from that changed Rust source before any repaired-state capture is allowed.

This is deliberately different from clicking the showcase checkbox or changing runtime state. The compiled program changes.

## Repaired observation

The rebuilt application's focused exact evidence reported:

```text
handle bounds=[453,215,10,10]
outline bounds=[307,169,152,102]
```

Therefore:

```text
handle movement: 60 px left
handle size/y:    unchanged
outline bounds:   unchanged
```

## ViewWitness diff proof

`viewwitness diff-exact broken.yaml fixed.yaml` emitted:

```text
authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds" before={"height":10.0,"width":10.0,"x":513.0,"y":215.0} after={"height":10.0,"width":10.0,"x":453.0,"y":215.0}
```

The acceptance harness additionally rejects:

- a material `outline` binding change;
- authored object ambiguity for `showcase:painted-rectangle`;
- authored binding ambiguity for `handle`;
- any repair movement other than 60 logical pixels left;
- any handle change outside its x position;
- any change in outline bounds.

All assertions passed.

## What this proves

ViewWitness has now demonstrated this concrete loop against a running native GUI:

```text
broken source
  -> launch real native application
  -> exact same-pass capture
  -> focused diagnosis of named rendered sub-part
  -> modify application source
  -> rebuild application
  -> relaunch application
  -> exact same-pass recapture
  -> identity-aware exact diff
  -> machine-check the intended repair and absence of collateral outline movement
```

This establishes the first source-edit acceptance slice of M5.

It does **not** establish that an arbitrary autonomous coding agent can discover and repair arbitrary GUI defects without task scaffolding. In this experiment the repair target is intentionally constrained and the CI harness applies the source edit selected from the observed evidence. Future M5 pressure should progressively remove that scaffolding rather than overstating what this run proves.

## Reproduction machinery

- `scripts/m5-source-repair-loop.sh` owns the capture → edit → rebuild → recapture → diff assertions.
- `.github/workflows/m5-live-showcase.yml` provides the native Linux/X11 execution environment.
- `VIEWWITNESS_SHOWCASE_SCENARIO=misplaced-handle` makes the broken application state deterministic before the repair.
- the checked-in source remains broken intentionally so every workflow run begins from the same known defect.
