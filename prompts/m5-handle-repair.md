# M5 coding-agent task — repair the disconnected rendered handle

You are working in a deliberately broken Rust + egui application workspace. Work from the `workspace/` directory inside this task package.

Your job is to diagnose and repair the application source from the supplied ViewWitness evidence. Do not modify or replace the evidence files.

## Observed defect

The exact live capture identifies one authored rendered object:

```text
object_id="showcase:painted-rectangle"
```

Two keyed rendered parts matter:

```text
binding_id="outline"
binding_id="handle"
```

The broken evidence shows that the handle is vertically aligned with the rectangle but horizontally disconnected from it. The rectangle outline itself is the reference geometry and should not move materially.

The complete broken correlated capture and focused projections are available in the sibling `evidence/` directory. From `workspace/`, they are:

```text
../evidence/broken.yaml
../evidence/handle-focus.txt
../evidence/outline-focus.txt
```

Use those files as observed GUI testimony. You may inspect the application source normally and decide for yourself which code is responsible.

## Repair goal

Change the application so that, when launched in the same broken reproduction scenario:

- `showcase:painted-rectangle` remains uniquely identifiable;
- its keyed `handle` remains uniquely identifiable;
- its keyed `outline` remains uniquely identifiable;
- the outline stays materially where it was;
- the handle keeps its size and vertical placement;
- the handle becomes geometrically attached to the rectangle's right edge rather than remaining horizontally disconnected;
- no unrelated authored rendered part is materially changed;
- exact capture provenance remains valid.

Do not optimize for a particular patch shape. A small repair is preferable, but choose the source change that best expresses the intended application geometry.

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
