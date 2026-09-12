# M5 coding-agent task — repair the invisible rendered center

You are working in a deliberately broken Rust + egui application workspace.

Your job is to diagnose and repair the application source from the supplied ViewWitness evidence. Do not modify or replace the evidence files.

## Observed defect

The exact live capture identifies one authored rendered object:

```text
object_id="showcase:painted-circle"
```

Two keyed rendered parts matter:

```text
binding_id="ring"
binding_id="center"
```

The broken evidence shows that the center has valid geometry at the middle of the circle but is fully invisible according to its observed clipping evidence. The surrounding ring remains visible and is the stable sibling reference.

The complete broken correlated capture and focused projections are available in the sibling `evidence/` directory:

```text
../evidence/broken.yaml
../evidence/center-focus.txt
../evidence/ring-focus.txt
```

Use those files as observed GUI testimony. You may inspect the application source normally and decide for yourself which code is responsible.

## Repair goal

Change the application so that, when launched in the same broken reproduction scenario:

- `showcase:painted-circle` remains uniquely identifiable;
- its keyed `center` remains uniquely identifiable;
- its keyed `ring` remains uniquely identifiable;
- center geometry and kind remain materially stable;
- the center becomes fully visible according to observed clipping evidence;
- the ring remains materially unchanged;
- no unrelated authored rendered part is materially changed;
- exact capture provenance remains valid.

Do not optimize for a particular patch shape. A small repair is preferable, but choose the source change that best expresses the intended application rendering.

## Mutation boundary

You may edit Rust application example sources in this workspace. Do not modify the ViewWitness library implementation, Cargo metadata, evidence, task text, or any verification tooling.

The independent verifier is intentionally outside this workspace and will judge the rebuilt application's evidence, not your patch text.

## Deliverable

When you are satisfied with the candidate, leave your source edits in the workspace and emit a binary-safe Git patch from the workspace root:

```text
git diff --binary > ../agent.patch
```

Do not edit `agent.patch` by hand after generating it.

Your patch will be applied to a fresh trusted checkout and verified independently.
