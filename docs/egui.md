# egui integration

ViewWitness initially targets Rust + egui. The integration deliberately keeps semantic, rendered, authored, and raster evidence distinct. Those layers can be correlated without being collapsed.

The central rule remains: **egui's GUI/output path may copy cheap evidence and emit bounded work; it must not perform serialization, networking, diffing, searching, persistence, or other heavy work.**

## Evidence layers

### Semantic evidence

egui already produces AccessKit output. ViewWitness consumes that output as observed semantic evidence rather than inventing a competing widget vocabulary at capture time.

The adapter preserves semantic roles, names/values, hierarchy, logical bounds, visibility/state, focus/selection/toggle state, actions, identity provenance, selected AccessKit properties, and selected semantic relations.

AccessKit IDs are projected as `ak:<u64>` in v0. Automatically generated identity is treated as `structure_sensitive`: executable probes show it can survive ordinary state changes while changing after structural insertion. Application-authored AccessKit IDs remain additional evidence rather than silently replacing the observed source ID.

### Generic paint-submission evidence

AccessKit does not exhaust rendered reality. egui's public `FullOutput::shapes` exposes flattened `ClippedShape` submissions heading toward the renderer.

`EguiPaintObservation` records flattened paint order, compact `EguiPaintKind`, visual bounding rectangle, and finite clip/scissor rectangle when available.

ViewWitness can derive:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

This is bounding-box clip evidence, **not** exact raster/alpha coverage. Paint order plus rectangle overlap also does not prove semantic occlusion.

### Explicit authored custom-paint evidence

Custom canvas objects may not exist meaningfully in AccessKit. The authored layer separates logical semantics, optional authored sub-part identity, and concrete renderer execution evidence:

```rust
EguiAuthoredPaintObject {
    id,
    role,
    name,
    semantic_evidence,
    bindings: Vec<EguiAuthoredPaintBinding>,
}

EguiAuthoredPaintBinding {
    authored_binding_id: Option<String>,
    binding_evidence,
    layer_order,
    layer_id,
    shape_index,
    verified_at_end_pass,
    kind,
    bounds,
    clip_rect,
}
```

The epistemic split is first-class:

```text
object id / role / name       semantic_evidence = intended
authored_binding_id           optional intended sub-part continuity evidence
LayerId + ShapeIdx            observed exact-pass execution evidence
final kind / bounds / clip    observed end-of-pass slot state
```

`EguiPaintAnnotator::paint_object(...)` is the explicit object-grouping operation. Its callback receives an `EguiPaintObjectScope`.

```text
add_shape / bind_shape                 unkeyed binding
add_shape_with_id / bind_shape_with_id keyed binding
```

Unkeyed bindings are useful when the application knows only object membership. Keyed bindings are useful when the application can additionally name stable rendered sub-parts such as `outline`, `handle`, `ring`, or `center`.

ViewWitness does **not** group independent objects merely because they repeat the same object ID or overlap geometrically. It likewise does not invent sub-binding identity from geometry, kind, or vector position.

At `Plugin::on_end_pass`, after application UI painting but before egui drains graphic layers into `FullOutput`, the exact frame probe verifies each real layer-local slot. Every binding preserves final verification state, shape kind, visual bounds, and finite clip rectangle.

Executable tests establish:

- two authored objects with identical overlapping bounds remain distinct because they occupy different paint slots;
- an annotated rectangle replaced through `Painter::set` is observed as the final **circle** in that same slot;
- one logical object can own several different paint handles, including handles on different egui layers;
- two bindings can share bounds while carrying different kind/clip evidence;
- annotations made on an ordinary unrequested frame do not leak into a later exact request;
- a reset valid slot remains verified and resolves to final `Shape::Noop`;
- a genuinely missing/unresolvable handle remains `verified_at_end_pass = false` with no final kind/bounds/clip;
- authored binding IDs survive reversed submission order while their `ShapeIdx` values may change.

Clip visibility belongs to the **binding**. `visible_bounds()` and `visible_fraction()` use each binding's final verified bounds and clip. ViewWitness deliberately does not synthesize a single object-level visibility number from constituent bindings that may disagree.

This authored evidence remains egui-specific. It is not automatically a canonical `WitnessNode` and it does not create an AccessKit-node ↔ paint-shape mapping.

### Raster evidence

The upstream `egui_inspection` protocol can return screenshots as PNG bytes plus dimensions. Its screenshot response currently lacks a trustworthy shared frame token with the semantic tree, so ViewWitness keeps raster evidence separate rather than claiming exact semantic↔raster correlation.

## Continuous paint monitoring

`EguiPaintReporter` installs at egui's `Plugin::output_hook` boundary. It copies compact renderer evidence into a bounded standard-library channel with `try_send`.

The reporter performs no serialization, networking, persistence, diffing, searching, or inference. If the consumer falls behind, evidence is dropped rather than blocking egui. Executable backpressure tests make this a hard project invariant.

`run_egui_paint_server` moves transport work to a blocking worker thread. The paint stream uses `127.0.0.1:5720`, a ViewWitness-specific versioned handshake, bounded newline-delimited JSON frames, and latest-frame retention for a later observer.

Explicit authored-object bookkeeping is **not** collected continuously. Ordinary annotated frames paint normally but do not retain binding bookkeeping when no exact request is active. Rich identity is therefore an exact-diagnosis cost, not a permanent render-loop tax.

## Exact same-pass capture

Independent semantic and continuous paint streams have different clocks and must not be joined by guesswork. `EguiFrameProbe` provides ViewWitness's on-demand shared capture point.

A worker requests one capture and wakes egui. Capture eligibility is fixed at `on_begin_pass`; a request arriving midway through a pass waits for the next pass rather than collecting only a suffix of authored annotations.

During a requested pass:

1. `on_begin_pass` activates exact capture and authored-paint bookkeeping;
2. normal application UI code runs and paints;
3. `on_end_pass` resolves every explicitly grouped `(LayerId, ShapeIdx)` binding against final layer-local paint lists;
4. `output_hook` copies AccessKit, viewport, generic flattened paint, and resolved authored evidence;
5. a bounded channel hands the evidence to worker code.

Worker-side conversion produces `EguiCorrelatedCapture`, containing the canonical semantic `Witness`, provisional generic paint, provisional authored custom-paint evidence, request ID, viewport ID, pass number, and full observed viewport rectangle.

Its metadata records `semantic_paint_correlation: same_full_output`.

### Viewport evidence

The exact correlated path does **not** treat AccessKit root bounds as a viewport surrogate. A real headless probe demonstrated a valid semantic tree with unusable root bounds while egui still had trustworthy viewport geometry. `EguiFrameProbe` therefore copies `InputState::viewport_rect()` and rejects invalid/non-finite geometry instead of guessing.

## Exact external capture transport

Exact correlated evidence is available outside the application process through ViewWitness's request/response protocol on `127.0.0.1:5721`.

```text
VIEWWITNESS-EGUI-CAPTURE 2
```

Protocol v2 was introduced because one logical authored object now owns an explicit `bindings[]` collection. A v2 observer rejects a v1 peer rather than silently interpreting an incompatible JSON shape.

`authored_binding_id` is additive and optional inside the v2 binding object. Its absence means the binding is unkeyed; it does not require a protocol bump.

`run_egui_capture_server` owns worker-side request/wait/convert/serialize work. `EguiCaptureObserver` is a read-only external client. Timeouts, alien handshakes, and incompatible older protocol versions remain explicit errors.

## Cross-frame authored identity

A dedicated real-egui experiment captures the same authored object multiple times.

With unchanged paint structure, the same `ShapeIdx` may recur. When unrelated paint is inserted before the object, the object's layer-local `ShapeIdx` shifts even though authored identity, final kind, bounds, and layer remain unchanged.

Therefore:

```text
ShapeIdx = exact-pass execution handle
ShapeIdx != cross-frame authored identity
```

A second real-egui experiment paints two keyed sub-parts, `outline` and `handle`, then reverses their submission order on the next capture. The authored binding IDs persist while vector order and `ShapeIdx` change.

This establishes the matching basis used by `EguiCorrelatedDiff`.

## Correlated diff semantics

`diff_correlated_captures` combines canonical `WitnessDiff` with a separate egui-authored diff. Generic anonymous paint is intentionally not naively list-diffed because no durable identity has been established for it.

Authored object matching:

- unique object ID on both sides → automatic object continuity;
- duplicate object ID on either side → `EguiAuthoredIdAmbiguity`, matching refused;
- object-level changes describe object semantics only, not the full binding vector.

Binding matching inside a uniquely matched object:

- unique `authored_binding_id` on both sides → match by key independent of submission order;
- duplicate authored binding ID on either side → `EguiAuthoredBindingIdAmbiguity`, matching refused for that object's bindings;
- no authored binding ID → match conservatively by relative unkeyed ordinal;
- keyed and unkeyed bindings may coexist.

Material binding state excludes `shape_index`. It includes evidence/layer/verification/kind/bounds/clip. Binding additions and removals are first-class. Material changes are field-granular through `EguiAuthoredBindingChange`, so a move can be represented as exactly `handle.bounds` rather than one opaque object-level `bindings` replacement.

Pure slot churn becomes `EguiAuthoredExecutionHandleChange` with `material=false`. If a binding has a material field change and its execution slot also churns, ViewWitness reports the material change and does not double-count the slot movement for that binding.

## Agent projection and CLI

`correlated_capture_to_agent_text` emits semantic witness lines, authored-object lines, authored-binding lines, and generic paint lines separately. When a binding has an authored key, the binding line includes `authored_binding_id=...`; unkeyed bindings omit it.

`correlated_diff_to_agent_text` distinguishes:

```text
authored-change                  object-level authored semantics changed
+authored-binding                rendered sub-part added
-authored-binding                rendered sub-part removed
authored-binding-change          named/ordinal binding field changed
authored-ambiguity               object continuity cannot be trusted
authored-binding-ambiguity       sub-binding continuity cannot be trusted
authored-handle-churn            exact-pass renderer slot changed, material=false
```

For example, a keyed handle movement is projected directly as:

```text
authored-binding-change object_id="agent-loop:node" authored_binding_id="handle" field="bounds" ...
```

### Focused exact projection

The full `EguiCorrelatedCapture` remains the exact evidence envelope. `correlated_capture_authored_focus_to_agent_text` is a deliberately smaller projection for a diagnosis task that already knows the authored object of interest.

It selects a required object ID and optional binding ID while retaining the exact capture's request/pass/viewport origin. The first line identifies its epistemic limits:

```text
projection=authored_focus omitted=canonical_semantics,generic_paint
```

The header also carries `object_match_count` and `binding_match_count`. These counts are not cosmetic: duplicate authored IDs are preserved as multiple records rather than silently choosing one, while zero matches are explicit. Original object and binding indices remain visible.

This projection is intentionally **agent text only**. It is not serialized as YAML because it is not a complete correlated envelope and should not be mistaken for one. Likewise, canonical geometry derivation is not run over a projection that explicitly omitted canonical semantics.

The unified CLI exposes:

```text
viewwitness inspect-exact <file> [--agent|--yaml] [--object=ID [--binding=ID]]
viewwitness capture-exact [address] [--agent|--yaml] [--derive] [--object=ID [--binding=ID]]
viewwitness diff-exact <before> <after> [--agent|--yaml]
```

A full composable verification loop is:

```text
viewwitness capture-exact --yaml > before.yaml
# change/interact
viewwitness capture-exact --yaml > after.yaml
viewwitness diff-exact before.yaml after.yaml
```

A focused diagnosis can be performed on either the live capture or a saved full envelope:

```text
viewwitness capture-exact --object=showcase:painted-rectangle --binding=handle
viewwitness inspect-exact before.yaml --object=showcase:painted-rectangle --binding=handle
```

`--binding` requires `--object`. Focus cannot be combined with `--yaml` or `--derive`. `diff-exact` still compares saved complete envelopes only; it does not secretly recapture or mutate the application.

## First executable agent-verification pressure case

The first M5 real-egui probe creates one authored `diagram_node` with two keyed sub-parts:

```text
body
handle
```

The broken capture places `handle` far away from `body`. The fixed capture moves the same keyed handle onto the body edge **and** inserts unrelated anonymous paint earlier in the same layer to force renderer-slot churn.

The accepted diff is:

```text
material:     handle.bounds changed
non-material: body ShapeIdx changed
not reported: body material change
not reported: object-level field="bindings"
```

This is the first executable proof that ViewWitness can distinguish the application change an agent cares about from renderer bookkeeping churn in the same before/after pair. It does not yet prove autonomous source modification; it proves the capture/diff/verification language needed for that step.

## Showcase as a live integration target

The native eframe showcase is a real ViewWitness pressure target. At startup it installs the cheap paint reporter and exact frame probe while blocking server loops run on named worker threads.

```text
127.0.0.1:5719  optional upstream egui_inspection semantic/raster endpoint
127.0.0.1:5720  ViewWitness continuous generic paint stream
127.0.0.1:5721  ViewWitness exact correlated capture request/response
```

The Canvas page contains two explicit logical authored objects and four **keyed** concrete paint bindings:

```text
showcase:painted-rectangle -> outline, handle
showcase:painted-circle    -> ring, center
```

A source-level regression test guards those four keys. Canvas background and text labels remain anonymous generic paint by design. Instrumentation grants identity only where the application explicitly supplies it.

The same Canvas page now provides a guarded live defect: `Misplaced canvas handle`. Enabling it offsets only the rectangle's keyed `handle` by 60 logical pixels while its `outline` remains in place. That makes the running showcase a concrete target for focused capture, diagnosis, and later source-edit verification.

## Operator examples

```powershell
cargo run --example showcase --features showcase
cargo run --features egui --bin viewwitness -- capture-exact
cargo run --features egui --bin viewwitness -- capture-exact --yaml
cargo run --features egui --bin viewwitness -- capture-exact --object=showcase:painted-rectangle --binding=handle
cargo run --features egui --bin viewwitness -- inspect-exact before.yaml --object=showcase:painted-rectangle --binding=handle
cargo run --features egui --bin viewwitness -- diff-exact before.yaml after.yaml
```

Optional upstream semantic/raster observation remains separate:

```powershell
$env:EGUI_INSPECTION="1"
cargo run --example showcase --features showcase
cargo run --features observer --bin viewwitness -- capture --settle=8
cargo run --features observer --bin viewwitness -- screenshot witness.png --scale=1
```

## Proven boundary

```text
explicit custom logical object -> one or many paint bindings       proven
optional authored binding ID -> stable sub-part continuity         proven under reordered submission
binding -> concrete LayerId + ShapeIdx                              proven
LayerId + ShapeIdx -> final layer-local shape                       proven at on_end_pass
reset valid handle -> verified final Noop                           proven
missing/unresolvable handle -> unverified binding                   proven
final binding bounds + clip -> bbox visibility                      proven derived evidence
ShapeIdx -> cross-frame identity                                    disproven as a general assumption
named material binding field -> precise correlated diff             proven
unrelated slot churn -> separable non-material diagnostic           proven
full exact envelope -> focused authored object/binding projection   proven with explicit omissions/match counts
live keyed handle defect -> exact diagnosis target                  proven in showcase and regression-guarded
arbitrary AccessKit node -> paint binding                           not proven
layer-local binding -> flattened FullOutput global order            not yet proven generally
agent source edit -> recaptured proof                               not yet proven
```

## Agent boundary

MCP is not the ViewWitness data model. `egui_inspection` is not the ViewWitness data model. AccessKit is not the ViewWitness data model. Generic paint observations are not the ViewWitness data model. Authored egui paint objects/bindings are not the canonical ViewWitness data model. Screenshots are not the ViewWitness data model. A focused authored projection is not a replacement `Witness` or replacement correlated envelope.

They are evidence sources and integration surfaces around the canonical `Witness` representation.

## Next pressure points

The next egui work should stay example-driven:

1. use the existing `Misplaced canvas handle` live scenario for a real source-edit loop: focused inspect → diagnose source → edit → rebuild/restart → full recapture → `diff-exact` verification;
2. consider focused diff output only if that real loop demonstrates a concrete token/noise problem with the full correlated diff;
3. determine whether layer-local handles can be mapped safely to flattened `FullOutput` order without unstable/incomplete assumptions;
4. pressure exact evidence across more multi-window/viewport cases;
5. improve raster correlation/evidence before considering any stronger visual-occlusion claim;
6. decide only after cross-backend pressure whether a generic “authored visual object” concept deserves promotion beyond the egui-specific envelope;
7. continue refusing canonical occlusion until stronger evidence than rectangle overlap exists.
