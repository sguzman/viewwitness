# Rendered evidence — egui v0 research

ViewWitness cannot stop at accessibility semantics. A GUI can be semantically well described while still be visibly broken: clipped, covered, painted in the wrong order, overflowing, or custom-drawn without meaningful accessibility nodes.

This document records executable rendered-evidence findings from egui. These findings remain separate from the canonical `Witness` schema until the concepts survive enough pressure to justify cross-backend names.

## Evidence layers must remain separate

The current experiments establish at least five distinct kinds of GUI testimony:

1. **Semantic evidence** — AccessKit nodes, roles, values, hierarchy, focus, actions, and semantic relations.
2. **Paint-submission evidence** — shapes egui submitted to the renderer, including visual bounding rectangles, clips, and flattened paint order.
3. **Clip-survival evidence** — deterministic bounding-box survival through an observed scissor rectangle.
4. **Authored custom-paint evidence** — application-declared logical-object semantics, optional authored sub-binding identity, and concrete egui paint handles.
5. **Raster evidence** — what pixels finally appear in a screenshot/framebuffer.

These are related but not interchangeable.

A submitted paint shape can exist while being fully clipped. A semantic node can exist without a unique paint primitive. A paint primitive can exist without any AccessKit node. An application can explicitly declare that several paint handles belong to one custom object without proving an AccessKit-node mapping. A screenshot can show final pixels without explaining which semantic object or paint submission produced them.

## Generic egui paint evidence

`egui::FullOutput` exposes `shapes: Vec<ClippedShape>` as the renderer-facing flattened paint list. Each `ClippedShape` contains a `Shape` and clip/scissor rectangle.

`Shape::visual_bounding_rect()` provides an axis-aligned visual bounding box. The current research adapter exposes these facts as:

```rust
pub struct EguiPaintObservation {
    pub order: usize,
    pub kind: EguiPaintKind,
    pub bounds: Rect,
    pub clip_rect: Option<Rect>,
}
```

This remains egui-specific and provisional.

## Generic clipping experiment

The executable generic-paint probe submits paint through finite clips and establishes:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

Useful states include complete bounding-box survival, partial survival, and zero survival while the paint submission still exists.

`visible_fraction` is a **derived bounding-box measure**, not exact pixel/alpha coverage. A circle whose bounding box is half clipped does not necessarily have exactly half of its painted pixels visible.

For an effectively unbounded/non-finite egui clip, `clip_rect` is represented as `None` rather than inventing finite coordinates.

## Paint order is not occlusion

Paint order plus overlapping visible bounds is stronger evidence than geometry alone, but it still does not prove semantic occlusion. Transparency, strokes, meshes, callbacks, multiple primitives per object, and one primitive spanning several concepts all break the naive inference.

Therefore ViewWitness does not derive canonical `occludes` from `later paint order + rectangle overlap`.

## Continuous reporting without render-thread work

`EguiPaintReporter` observes the public `FullOutput` boundary and emits compact `EguiPaintFrame` values into a bounded `sync_channel` using `try_send`.

It performs no JSON/YAML serialization, persistence, network I/O, diffing, searching, or model work. Executable backpressure tests establish that a saturated reporter drops evidence rather than blocking a GUI pass, and later delivery exposes the loss count.

The worker-side `run_egui_paint_server` / `EguiPaintObserver` channel is read-only and loopback-oriented on `127.0.0.1:5720`.

## Independent streams are not exact correlation

The external `egui_inspection` semantic stream and continuous ViewWitness paint stream expose different clocks:

- `egui_inspection::Response::Tree.step` is inspection-plugin state;
- `EguiPaintFrame.pass_nr` is egui's cumulative viewport-aware pass number.

Their values must **not** be equated or joined by an assumed fixed offset.

## Exact same-pass correlation

`EguiFrameProbe` avoids clock reconciliation by providing an on-demand shared capture point.

A worker requests one capture. Capture eligibility is fixed at the start of a pass so a request arriving halfway through a frame cannot collect only a suffix of authored annotations.

One requested pass supplies semantic AccessKit output, generic renderer paint, viewport identity, pass number, scale, observed viewport rectangle, and explicitly authored custom-paint evidence. A bounded channel hands that evidence to worker code, which produces `EguiCorrelatedCapture`.

An early implementation attempted to derive viewport dimensions from AccessKit root bounds. An executable headless test disproved that assumption: semantic root bounds can be unavailable while egui still knows the viewport. The exact probe therefore copies `InputState::viewport_rect()` directly and rejects invalid geometry instead of reconstructing it from weaker evidence.

## Explicit authored custom-paint identity

Exact same-pass capture still does **not** reveal a generic AccessKit-node → paint-shape mapping. ViewWitness solves a narrower case: applications may explicitly identify custom-painted logical objects and, optionally, stable rendered sub-parts.

```text
logical authored object
    intended id / role / optional name
    bindings[]
        optional intended authored_binding_id
        observed LayerId + ShapeIdx
        final verification state
        final kind / bounds / clip
```

`EguiPaintAnnotator::paint_object(...)` explicitly establishes object membership. Inside that scope:

- `add_shape` / `bind_shape` create unkeyed bindings;
- `add_shape_with_id` / `bind_shape_with_id` create bindings with an authored sub-part key.

### Duplicate object IDs are not grouping

A negative-control test performs two independent annotations with the same authored object ID. Exact capture returns two authored object records.

```text
same authored object ID              != same logical object
same geometry                         != same logical object
same capture pass                     != same logical object
explicit paint_object grouping        == authored membership claim
```

Cross-frame diffing only auto-matches an authored object ID when it is unique on both sides. Duplicate IDs become explicit ambiguity.

### Handle identity and replacement experiment

Two authored objects can have identical overlapping bounds while remaining distinct because their bindings point to different layer-local paint slots.

A separate probe annotates a rectangle and replaces that exact slot through `Painter::set`. End-of-pass evidence reports the final shape as a **circle**. The binding follows the egui handle to the slot's final state rather than remembering the originally submitted shape.

### Multi-shape object experiment

One authored logical object can own several concrete bindings. Tests prove bindings can share the same final bounds while differing in kind and clipping. Visibility therefore remains per binding; ViewWitness does not invent one aggregate object-level `visible_fraction` whose semantics would depend on how the constituent parts compose.

### Multi-layer binding experiment

An authored object may own bindings on different egui layers. The exact capture keeps each binding's `layer_order`, raw layer ID, and layer-local `ShapeIdx` distinct rather than flattening those identities into a guessed global order.

This is important because a general authored-binding → flattened `FullOutput::shapes` order mapping is still not established across all layer/window cases.

## Reset versus missing-handle experiment

A bound handle can stop painting in two materially different ways.

When egui resets a **valid existing slot**, the final slot becomes `Shape::Noop`. ViewWitness resolves it successfully:

```text
verified_at_end_pass = true
kind = noop
visible_fraction = 0
```

When a binding cannot resolve to a real slot at all:

```text
verified_at_end_pass = false
kind = absent
bounds = absent
clip = absent
visible_fraction = 0
```

The visible fraction happens to agree, but the provenance does not. ViewWitness therefore does not collapse “verified handle whose final shape paints nothing” into “handle could not be verified.”

## Cross-frame `ShapeIdx` experiment

A dedicated real-egui probe captures the same authored object across frames.

- with unchanged paint structure, the same layer-local `ShapeIdx` may recur;
- inserting one unrelated paint submission before the object shifts that `ShapeIdx`;
- authored object ID, role, final kind, final bounds, and layer remain unchanged.

Therefore:

```text
ShapeIdx = frame-local, structure-sensitive execution evidence
ShapeIdx != durable authored-object identity
```

Correlated diffs consequently exclude `shape_index` from material binding state and report pure slot movement as diagnostic `execution_handle_churn`.

## Authored binding identity experiment

Object identity alone is insufficient for a multi-shape object when its constituent paint submissions can reorder.

A real-egui experiment creates one object with two keyed bindings:

```text
outline
handle
```

The first capture submits `outline` then `handle`. The second capture submits `handle` then `outline`.

The result:

- the object ID remains stable;
- the two authored binding IDs remain stable;
- binding vector order reverses;
- layer-local `ShapeIdx` changes;
- final kind/bounds for each keyed sub-part remain stable.

This justifies `authored_binding_id` as optional intended continuity evidence distinct from the observed exact-pass renderer handle.

## Binding-aware diff rules

Within a uniquely matched authored object:

```text
unique authored_binding_id on both sides  -> match by key, order-independent
duplicate authored_binding_id             -> explicit ambiguity, matching refused
no authored_binding_id                     -> conservative relative-unkeyed-ordinal match
keyed + unkeyed bindings                   -> allowed together
binding appears/disappears                 -> first-class add/remove record
ShapeIdx-only change                        -> diagnostic, material=false
kind/bounds/clip/layer/verification change -> first-class field-granular material change
```

Object semantics and rendered binding state are separate diff surfaces. A binding move is no longer emitted as one opaque object-level `field="bindings"` replacement. If the application supplied a unique sub-binding key, ViewWitness can say exactly which named part and field changed.

Duplicate binding IDs are not silently deduplicated or repaired. `EguiAuthoredBindingIdAmbiguity` preserves the conflict explicitly.

Generic anonymous paint still lacks this continuity source and is therefore not naively list-diffed.

## First agent-verification pressure experiment

The first M5 real-egui pressure test constructs one logical object with keyed `body` and `handle` bindings.

Broken capture:

```text
body   bounds=[40,30,60,40]
handle bounds=[145,45,10,10]   # visibly/logically far from body edge
```

Fixed capture:

```text
body   unchanged
handle bounds=[95,45,10,10]    # moved to right edge of body
```

The fixed frame also inserts unrelated anonymous paint **before** the authored object. This deliberately perturbs layer-local renderer slots so the test must distinguish the application fix from execution churn.

The resulting correlated diff is required to contain:

```text
authored-binding-change object_id="agent-loop:node" authored_binding_id="handle" field="bounds" ...
authored-handle-churn id="agent-loop:node" authored_binding_id="body" ... material=false
```

It must not contain a material change for `body`, and it must not collapse the change into `field="bindings"`.

This is a stronger result than merely proving IDs persist. It demonstrates that ViewWitness can isolate a concrete rendered sub-part fix while suppressing unrelated renderer-slot noise in the same transition.

## Agent projections

Correlated capture text emits one logical `authored-object` line followed by one line per binding. A keyed binding carries `authored_binding_id`; an unkeyed binding omits it.

Correlated diff text separates:

- `authored-change` — object-level authored semantics changed;
- `+authored-binding` / `-authored-binding` — sub-part appeared/disappeared;
- `authored-binding-change` — a named or ordinal binding field materially changed;
- `authored-ambiguity` — object ID is not a trustworthy match key;
- `authored-binding-ambiguity` — sub-binding ID is not a trustworthy match key;
- `authored-handle-churn` — renderer slot changed but material state did not.

This distinction is designed specifically so an agent does not mistake unrelated prefix paint for a logical object mutation.

### Focused authored projection

The full correlated capture is still the complete evidence envelope. A diagnosis task that already knows its authored target can request a smaller projection through `correlated_capture_authored_focus_to_agent_text`.

The projection retains exact request/pass/viewport correlation metadata, then explicitly labels what it omitted:

```text
projection=authored_focus omitted=canonical_semantics,generic_paint
```

It also reports object and binding match counts. A duplicate authored object ID therefore yields multiple object records and an `object_match_count > 1`; a duplicate binding key likewise remains multiple binding records. Zero matches remain explicit. The projection never chooses an arbitrary “best” duplicate and never falls back to geometry.

That refusal matters epistemically: a focused rendering is a **view over evidence**, not evidence erasure. The complete YAML envelope can still be preserved for later diffing or broader diagnosis.

Focused YAML is intentionally unsupported because serializing the projection as though it were a complete exact envelope would make its omissions too easy to forget.

## Exact transport versioning

The exact protocol is:

```text
VIEWWITNESS-EGUI-CAPTURE 2
```

v2 was required when the envelope changed from repeated object semantics per handle to one object with `bindings[]`.

`authored_binding_id` is an optional additive field inside a v2 binding. Captures that omit it retain their previous meaning as unkeyed bindings, so no v3 bump is required.

## Live showcase pressure case

The native showcase contains two authored Canvas objects with four explicitly keyed paint bindings:

```text
showcase:painted-rectangle -> outline, handle
showcase:painted-circle    -> ring, center
```

A source-level regression test guards those four keys. The canvas background and text labels remain ordinary anonymous paint, preserving the distinction between instrumented identity and generic renderer evidence.

The showcase now also contains a guarded broken-state switch:

```text
Misplaced canvas handle
```

When active, it moves only `showcase:painted-rectangle`'s keyed `handle` by 60 logical pixels while leaving its keyed `outline` fixed. This makes the headless `body`/`handle` verification experiment available as a real external diagnosis target.

A live agent can therefore preserve a complete broken capture and separately inspect only the relevant part:

```text
capture-exact --yaml > broken.yaml
inspect-exact broken.yaml --object=showcase:painted-rectangle --binding=handle
```

or focus a live exact capture directly:

```text
capture-exact --object=showcase:painted-rectangle --binding=handle
```

Turning the switch off demonstrates live state/action verification, but it is **not** evidence that an agent repaired source code. Source-edit proof requires changing the application source, rebuilding/restarting as needed, recapturing a complete envelope, and verifying the transition through `diff-exact`.

## What remains unknown

The generic widget-to-paint problem remains open. ViewWitness must not invent AccessKit-node → paint-shape mapping from coincident geometry, labels, or same-frame occurrence.

Likewise, an authored binding proves a **layer-local** paint handle. ViewWitness has not proven a general mapping from arbitrary `(LayerId, ShapeIdx)` to flattened `FullOutput::shapes` global order across all layer/window cases.

The next useful pressure cases are therefore:

- perform the real source-edit loop against the existing live `Misplaced canvas handle` defect;
- decide whether a focused diff projection is justified by measured agent noise in that loop;
- safe layer-local-handle → flattened-renderer-order mapping, if one can be proven;
- more multi-window/viewport exact-capture pressure;
- stronger raster evidence before any canonical visual-occlusion claim.

## Promotion rule

Rendered concepts should enter the canonical cross-backend `Witness` model only after repeated examples show that they are general, useful, and nameable without egui-specific leakage.

For now:

- AccessKit semantic evidence belongs in canonical witnesses;
- generic egui paint observations remain integration/research evidence;
- bounding-box clip survival is deterministic derived evidence for generic paint and authored bindings;
- `EguiCorrelatedCapture` proves same-pass origin without implying generic widget↔paint identity;
- explicitly instrumented custom paint can carry authored object identity plus optional authored binding identity and observed end-of-pass renderer bindings;
- authored binding diffs may be field-granular when continuity evidence supports that claim;
- focused authored output is a labeled projection over a complete exact capture, not a replacement evidence model;
- duplicate object or binding IDs remain explicit ambiguity rather than heuristic matches;
- screenshots remain supporting raster evidence unless their transport supplies trustworthy same-frame correlation;
- generic widget↔paint association remains unknown unless a source explicitly supplies it.
