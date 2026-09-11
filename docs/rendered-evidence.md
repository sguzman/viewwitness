# Rendered evidence — egui v0 research

ViewWitness cannot stop at accessibility semantics. A GUI can be semantically well described while still being visibly broken: clipped, covered, painted in the wrong order, overflowing, or custom-drawn without meaningful accessibility nodes.

This document records executable rendered-evidence findings from egui. These findings are intentionally kept separate from the canonical `Witness` schema until the concepts survive more pressure.

## Evidence layers must remain separate

The current experiments establish at least five distinct kinds of GUI testimony:

1. **Semantic evidence** — AccessKit nodes, roles, values, hierarchy, focus, actions, and semantic relations.
2. **Paint-submission evidence** — shapes egui submitted to the renderer, including visual bounding rectangles, clips, and flattened paint order.
3. **Clip-survival evidence** — deterministic bounding-box survival through an observed scissor rectangle.
4. **Authored custom-paint evidence** — application-declared logical-object semantics plus one or more concrete paint bindings explicitly grouped by the application.
5. **Raster evidence** — what pixels finally appear in a screenshot/framebuffer.

These are related but not interchangeable.

A submitted paint shape can exist while being fully clipped. A semantic node can exist without a unique paint primitive. A paint primitive can exist without any AccessKit node. An application can explicitly declare that several paint handles belong to one custom object without thereby proving an AccessKit-node mapping. A screenshot can show final pixels without explaining which semantic object or paint submission produced them.

ViewWitness preserves these distinctions rather than choosing one representation and pretending it exhausts GUI reality.

## Generic egui paint evidence

`egui::FullOutput` exposes `shapes: Vec<ClippedShape>` as the renderer-facing flattened paint list. Each `ClippedShape` contains a `Shape` and a clip/scissor rectangle.

`Shape::visual_bounding_rect()` provides an axis-aligned visual bounding box. egui's graphics drain flattens ordered layer paint lists into `FullOutput::shapes`, so sequence position is usable observed back-to-front paint-order evidence for generic paint.

The current ViewWitness research adapter exposes these facts as `EguiPaintObservation`:

```rust
pub struct EguiPaintObservation {
    pub order: usize,
    pub kind: EguiPaintKind,
    pub bounds: Rect,
    pub clip_rect: Option<Rect>,
}
```

This type remains egui-specific and provisional. It is not embedded in canonical `Witness` documents.

## Generic clipping experiment

The executable generic-paint probe submits two custom-painted shapes through the same finite clip rectangle:

- a rectangle that extends beyond the clip and therefore remains partially visible;
- a circle entirely outside the clip and therefore becomes fully clipped.

The renderer-facing shape list still contains both submissions.

ViewWitness derives:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

For an effectively unbounded/non-finite egui clip, `clip_rect` is represented as `None` instead of inventing finite coordinates.

The experiment establishes three useful states:

- `visible_fraction == 1.0` — the bounding rectangle survives clipping completely;
- `0.0 < visible_fraction < 1.0` — partial clipping;
- `visible_fraction == 0.0` — paint was submitted but none of its bounding rectangle survives clipping.

`visible_fraction` is a **derived bounding-box measure**, not exact pixel/alpha coverage. A circle whose bounding box is half clipped does not necessarily have exactly half of its painted pixels visible.

## Paint order is not yet occlusion

Paint order plus overlapping visible bounds is stronger evidence than geometry alone, but it still does not automatically prove semantic occlusion.

Reasons include transparency, hollow/stroked shapes, complex meshes/callbacks, multiple primitives belonging to one logical object, and one primitive spanning several concepts.

Therefore ViewWitness does not derive canonical `occludes` merely from `later paint order + rectangle overlap`.

A future lower-level relation such as `painted_after` or a paint-overlap diagnostic may be justified, but it should remain clearly weaker than a proven visual occlusion claim.

## Continuous reporting without render-thread work

`EguiPaintReporter` observes the public `FullOutput` boundary and emits compact `EguiPaintFrame` values into a bounded `sync_channel` using `try_send`.

It performs no JSON/YAML serialization, persistence, network I/O, diffing, searching, or model work. Executable backpressure tests establish that a saturated reporter drops evidence rather than blocking a GUI pass, and a later delivered frame exposes the drop count.

The worker-side `run_egui_paint_server` / `EguiPaintObserver` side channel is read-only and loopback-oriented on `127.0.0.1:5720`. It is evidence transport, not an input/control protocol.

## Independent streams are not exact correlation

The external `egui_inspection` semantic stream and continuous ViewWitness paint stream expose different clocks:

- `egui_inspection::Response::Tree.step` is inspection-plugin state;
- `EguiPaintFrame.pass_nr` is egui's cumulative viewport-aware pass number.

Their values must **not** be equated or joined by an assumed fixed offset.

## Exact same-pass correlation

`EguiFrameProbe` solves exact semantic↔paint correlation by avoiding clock reconciliation entirely.

A worker requests one capture. Capture eligibility is fixed at the **start** of a pass so a request arriving halfway through a frame cannot collect only a suffix of authored annotations.

One requested pass supplies:

- the AccessKit update attached to that `FullOutput`;
- renderer-facing generic paint observations from that `FullOutput`;
- egui viewport identity;
- egui cumulative pass number;
- pixels per point;
- current viewport rectangle;
- any explicitly authored custom-paint objects and their bindings, verified at end-of-pass.

The result is sent through a bounded channel using `try_send`. If the response queue is full, the request is explicitly reported as dropped; the GUI hook never blocks.

Worker-side conversion yields `EguiCorrelatedCapture`, containing a canonical semantic `Witness` plus provisional egui paint/authored evidence from the same requested pass.

## Viewport evidence finding

An early correlated-capture implementation attempted to derive viewport dimensions from AccessKit root bounds. An executable test rejected that assumption: a valid headless egui semantic tree can lack usable root bounds even while egui itself knows the viewport.

The in-process same-pass probe therefore copies `InputState::viewport_rect()` directly. Invalid/non-finite observed viewport geometry is an error; ViewWitness does not fall back to invented dimensions.

This produces an important source rule:

> Prefer the strongest direct evidence available at the capture boundary. Do not reconstruct a fact from a weaker representation merely because another transport is forced to do so.

## Explicit authored custom-paint identity

Exact same-pass capture still does **not** reveal a generic AccessKit-node → paint-shape mapping. ViewWitness solves a narrower case without guessing: applications may explicitly identify custom-painted logical objects at paint time.

The accepted model is:

```text
logical authored object
    intended id / role / optional name
    bindings[]
        observed LayerId + ShapeIdx
        final verification state
        final kind / bounds / clip
```

The application creates multi-shape membership explicitly with `EguiPaintAnnotator::paint_object(...)`. The callback receives an object scope and every shape added or bound through that scope becomes one binding of that logical object.

The singular `add_shape(...)` API is merely the one-binding convenience form.

### Why duplicate IDs are not grouping

A dedicated executable negative-control test performs two independent annotations with exactly the same authored ID but different roles. Exact capture returns **two authored object records**, each with one binding.

Therefore:

```text
same authored ID                  != same logical object
same geometry                     != same logical object
same capture pass                 != same logical object
explicit paint_object grouping     == authored membership claim
```

This keeps grouping provenance visible. ViewWitness does not silently repair an application's duplicate IDs or infer identity from convenience.

### Handle identity experiment

The earlier identity pressure test deliberately creates two authored objects with **identical overlapping bounds**. They remain distinct because their bindings point to different real shape slots.

The test then annotates one rectangle and replaces that exact slot through `Painter::set`. End-of-pass evidence reports the final shape as a **circle**. The binding therefore follows egui's handle, not the originally submitted shape or coincident geometry.

A negative-control test also proves annotations from an ordinary unrequested frame do not leak into a later exact request.

### Multi-shape object experiment

The new core pressure test constructs **one authored logical object with two explicit paint bindings**.

Both bindings have the same final 40×40 visual bounds, but they are different renderer submissions:

- binding 0 is a rectangle on an ordinary painter and survives clipping fully;
- binding 1 is a circle submitted through a painter clipped to the right 20×40 half.

Exact capture proves:

- there is one authored object, not two duplicated object records;
- it has exactly two distinct `ShapeIdx` bindings;
- both final slots verify successfully;
- final kinds remain independently observable (`rect`, `circle`);
- both can share the same visual bounds without losing identity;
- binding 0 derives `visible_fraction = 1.0`;
- binding 1 derives `visible_fraction = 0.5` from its final clip.

This is why visibility remains **per binding**. A single object-level `visible_fraction` would erase a real disagreement among its constituent rendered parts and would require additional semantics about how those parts compose.

The correlated agent projection therefore emits a logical `authored-object` line followed by separate `authored-binding` lines, each carrying its own kind/bounds/clip/visibility evidence.

### Live showcase pressure case

The native showcase now contains exactly two authored Canvas objects with four explicit bindings:

- `showcase:painted-rectangle`
  - stroked outline;
  - small filled handle;
- `showcase:painted-circle`
  - outer ring;
  - center marker.

The canvas background and text labels remain ordinary anonymous paint. This keeps the demonstration epistemically useful: exact capture should expose two authored objects, four verified authored bindings, and unrelated generic paint rather than pretending instrumentation semanticizes the entire renderer output.

## Exact transport versioning

The authored-object envelope is serialized over ViewWitness's exact capture transport. Moving from one object record per handle to one object with `bindings[]` was therefore a real wire-format change.

The exact protocol was bumped to:

```text
VIEWWITNESS-EGUI-CAPTURE 2
```

A v2 observer explicitly rejects a v1 peer. ViewWitness does not reuse the same handshake for incompatible serialized meanings.

## What remains unknown

The generic widget-to-paint problem remains open.

egui internally has richer widget/layout data than AccessKit alone, but the complete collection is not exposed through the same public capture boundary used here. ViewWitness must therefore still **not invent an automatic AccessKit-node → paint-shape mapping** from coincident geometry, names, or same-frame occurrence.

Likewise, an authored binding currently proves a **layer-local** paint handle. ViewWitness has not proven a general mapping from arbitrary `(LayerId, ShapeIdx)` to the flattened `FullOutput::shapes` global order across all layer/window cases.

The next useful pressure cases are:

- one authored object with bindings on multiple egui layers;
- reset/removal semantics for bound handles;
- safe mapping, if possible, from layer-local handles to flattened renderer order;
- authored-object/binding diffs across exact captures;
- real agent debugging workflows against the live showcase.

## Promotion rule

Rendered concepts should enter the canonical cross-backend `Witness` model only after repeated examples show that they are general, useful, and nameable without egui-specific leakage.

For now:

- AccessKit semantic evidence belongs in canonical witnesses;
- egui paint observations remain integration/research evidence;
- bounding-box clip survival is deterministic derived evidence for generic paint and each authored binding;
- the continuous paint side channel transports egui-specific evidence without promoting it into the canonical model;
- `EguiCorrelatedCapture` proves same-pass origin without implying generic widget↔paint identity;
- explicitly instrumented custom paint can carry one authored logical identity plus one or more observed, end-of-pass-verified egui paint bindings;
- duplicate authored IDs are preserved, not silently merged;
- screenshots remain supporting raster evidence unless their transport supplies trustworthy same-frame correlation;
- generic widget↔paint association remains unknown unless a source explicitly supplies it.
