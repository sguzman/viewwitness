# Agent verification loop — live showcase

This runbook is the current M5 operating procedure for pressuring ViewWitness as evidence for an agent debugging a real egui application, not merely as a library test.

The first accepted live defect is the Canvas object's misplaced keyed handle:

```text
object:  showcase:painted-rectangle
binding: handle
defect:  handle displaced 60 logical pixels to the right
```

The rectangle's keyed `outline` remains fixed. The Canvas background and labels remain anonymous generic paint.

The first complete source-edit acceptance run is recorded in `docs/acceptance/m5-live-source-repair.md`.

## 1. Launch the known-broken target

The showcase can enter the broken state without mouse interaction.

PowerShell:

```powershell
$env:VIEWWITNESS_SHOWCASE_SCENARIO="misplaced-handle"
cargo run --example showcase --features showcase
```

The scenario opens on the Canvas page with `Misplaced canvas handle` active. Exact capture remains served from `127.0.0.1:5721` on a worker thread.

## 2. Preserve the complete broken evidence

In another shell:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact --yaml > broken.yaml
```

`broken.yaml` is the complete correlated envelope. Preserve it even if the immediate diagnosis will use a smaller projection.

## 3. Focus diagnosis on the named rendered part

```powershell
cargo run --features egui --bin viewwitness -- inspect-exact broken.yaml --object=showcase:painted-rectangle --binding=handle
```

The result is explicitly a projection:

```text
projection=authored_focus omitted=canonical_semantics,generic_paint
```

It retains exact request/pass/viewport origin and reports object/binding match counts. Duplicate IDs are not silently collapsed and zero matches are explicit.

A live focused capture is also available:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact --object=showcase:painted-rectangle --binding=handle
```

Use the saved full envelope for durable before/after verification; use focus to reduce diagnosis noise.

## 4. Diagnose and edit source

The coding agent should identify the application source responsible for the observed defect and modify that source.

This remains deliberately outside ViewWitness's observation authority. ViewWitness does not grant itself code-edit or GUI-control authority merely because an agent can consume its evidence.

The edit must occur outside the render/UI thread. Rebuild/restart the application as required by the host workflow.

Turning off the pressure checkbox is **not** source-edit proof. It demonstrates a state/action transition only.

For the first accepted M5 run, the observed source defect was repaired in an isolated CI checkout as:

```diff
-        first.right_center() + egui::vec2(60.0, 0.0)
+        first.right_center() + egui::vec2(0.0, 0.0)
```

The checked-in source intentionally retains the broken expression so the acceptance case is reproducible.

## 5. Rebuild the edited target and recapture

The first accepted workflow rebuilds the real native showcase after editing the Rust source, then relaunches the same deterministic `misplaced-handle` scenario. This is stronger than switching runtime state because the repaired observation comes from a newly compiled program.

For a manual experiment, rebuild/relaunch however the source-editing workflow normally does, then preserve a second complete envelope:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact --yaml > fixed.yaml
```

## 6. Verify with correlated diff evidence

```powershell
cargo run --features egui --bin viewwitness -- diff-exact broken.yaml fixed.yaml
```

The acceptance claim must be based on evidence, not on “looks fixed.” For the first defect, `diff-exact` identified exactly the keyed `handle` binding's geometry change:

```text
authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds" before={"height":10.0,"width":10.0,"x":513.0,"y":215.0} after={"height":10.0,"width":10.0,"x":453.0,"y":215.0}
```

The paired outline stayed at `[307,169,152,102]`. The harness independently checked that the handle moved exactly 60 logical pixels left, changed only its x position, and remained uniquely identifiable.

A successful source-repair run should answer all of these:

- Was the same authored object identifiable before and after?
- Was the same keyed rendered sub-part identifiable before and after?
- Did its material state change in the intended direction?
- Did unaffected sibling bindings avoid false material changes?
- Was renderer-slot churn kept separate from material state?
- Did both captures preserve exact same-pass provenance?
- Did the repaired capture come from rebuilt source rather than a runtime toggle?

## Automated acceptance harness

The canonical live source-repair experiment is executable:

```text
.github/workflows/m5-live-showcase.yml
scripts/m5-source-repair-loop.sh
```

The script:

```text
starts Xvfb
  -> launches checked-in broken native showcase
  -> captures broken.yaml
  -> records focused handle + outline evidence
  -> edits examples/showcase.rs
  -> records the source patch
  -> rebuilds the native showcase
  -> relaunches the same scenario
  -> captures fixed.yaml
  -> records focused handle + outline evidence
  -> runs diff-exact
  -> rejects ambiguity/collateral material change/wrong movement
  -> restores checked-in source during cleanup
```

CI uploads the complete run evidence as `m5-live-source-repair-evidence`.

## Current proof boundary

Proven and executable:

```text
real native eframe exact capture                 yes
stable authored object + keyed sub-part           yes
focused live object/binding inspection            yes
deterministic broken startup scenario             yes
application source edited after broken capture    yes
native application rebuilt from edited source     yes
rebuilt application recaptured                    yes
complete-envelope diff-exact                      yes
field-granular handle.bounds repair proof         yes
unaffected outline held materially stable         yes
source patch + before/after evidence preserved     yes
```

Still not proven:

```text
arbitrary coding agent locates source without pre-encoded repair target
agent chooses patch without harness knowing replacement string
multiple qualitatively different defect classes
arbitrary GUI repair from incomplete/ambiguous evidence
```

Those are now the next M5 pressure points. The first source-edit milestone itself is complete; future work should remove scaffolding rather than replaying the same offset recipe and calling it additional progress.
