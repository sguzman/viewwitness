# Agent verification loop — live showcase

This runbook is the current M5 acceptance target. It exists to pressure ViewWitness as evidence for an agent debugging a real egui application, not merely as a library test.

The canonical live defect is the Canvas object's misplaced keyed handle:

```text
object:  showcase:painted-rectangle
binding: handle
defect:  handle displaced 60 logical pixels to the right
```

The rectangle's keyed `outline` remains fixed. The Canvas background and labels remain anonymous generic paint.

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

The agent should identify the application source responsible for the observed displacement and modify that source.

This is deliberately outside ViewWitness's observation authority. ViewWitness does not grant itself code-edit or GUI-control authority merely because an agent can consume its evidence.

The edit must occur outside the render/UI thread. Rebuild/restart the application as required by the host workflow.

Turning off the pressure checkbox is **not** source-edit proof. It demonstrates a state/action transition only.

## 5. Launch the fixed target and recapture

After the source change, run the showcase normally unless the edited code has introduced a different explicit reproduction mechanism:

```powershell
Remove-Item Env:VIEWWITNESS_SHOWCASE_SCENARIO -ErrorAction SilentlyContinue
cargo run --example showcase --features showcase
```

Then preserve a second complete envelope:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact --yaml > fixed.yaml
```

## 6. Verify with correlated diff evidence

```powershell
cargo run --features egui --bin viewwitness -- diff-exact broken.yaml fixed.yaml
```

The acceptance claim must be based on evidence, not on “looks fixed.” For the intended defect, the important material evidence should identify the named `handle` binding's geometry change. Unrelated `ShapeIdx` movement must remain diagnostic/non-material rather than masquerading as an application change.

A successful run should answer all of these:

- Was the same authored rectangle identifiable before and after?
- Was the same keyed `handle` identifiable before and after?
- Did the handle's material geometry change in the intended direction?
- Did the rectangle `outline` avoid false material change?
- Was renderer-slot churn kept separate from material state?
- Did the fixed capture still preserve exact same-pass provenance?

## Current proof boundary

Already proven and CI-guarded:

```text
real egui exact capture                         yes
stable authored object + keyed sub-part         yes
field-granular handle.bounds diff               yes
unrelated ShapeIdx churn separated              yes
live misplaced-handle defect                    yes
deterministic broken startup scenario           yes
focused object/binding inspection               yes
complete-envelope before/after diff             yes
```

Not yet proven:

```text
agent diagnoses live broken showcase from evidence
agent edits application source
rebuilt/restarted app is recaptured
source fix is accepted from resulting diff
```

That remaining sequence is the next M5 acceptance event. Do not mark it complete merely because the infrastructure exists.
