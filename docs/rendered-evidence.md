# Rendered evidence — egui v0 research

ViewWitness cannot stop at accessibility semantics. A GUI can be semantically well described while still being visibly broken: clipped, covered, painted in the wrong order, overflowing, or custom-drawn without meaningful accessibility nodes.

This document records executable rendered-evidence findings from egui. These findings are intentionally kept separate from the canonical `Witness` schema until the concepts survive more pressure.

## Evidence layers must remain separate

The current experiments establish at least five distinct kinds of GUI testimony:

1. **Semantic evidence** — AccessKit nodes, roles, values, hierarchy, focus, actions, and semantic relations.
2. **Paint-submission evidence** — shapes egui submitted to the renderer, including visual bounding rectangles, clips, and flattened paint order.
3. **Clip-survival evidence** — deterministic bounding-box survival through an observed scissor rectangle.
4. **Authored custom-paint evidence** — application-declared object semantics bound to the concrete egui paint handle actually used for that object.
5. **Raster evidence** — what pixels finally appear in a screenshot/framebuffer.

These are related but not interchangeable.

A submitted paint shape can exist while being fully clipped. A semantic node can exist without a unique paint primitive. A paint primitive can exist without any AccessKit node. An application can explicitly declare that a custom object owns a paint handle without thereby proving an AccessKit-node mapping. A screenshot can show final pixels without explaining which semantic object or paint submission produced them.

ViewWitness preserves these distinctions rather than choosing one representation and pretending it exhausts GUI reality.

## What egui publicly exposes

`egui::FullOutput` exposes `shapes: Vec<ClippedShape>` as the renderer-facing flattened paint list. Each `ClippedShape` contains a `Shape` and a clip/scissor rectangle.

`Shape::visual_bounding_rect()` provides an axis-aligned visual bounding box. egui's graphics drain flattens ordered layer paint lists into `FullOutput::shapes`, so sequence position is usable observed back-to-front paint-order evidence.

The current ViewWitness research adapter exposes these facts as `EguiPaintObservation`:

```rust
pub struct EguiPaintObservation {
    pub order: usize,
    pub kind: EguiPaintKind,
    pub bounds: Rect,
    pub clip_rect: Option<Rect>,
}
```

`EguiPaintKind` is a small serializable enum rather than a `String`. That is a deliberate render-path property: classifying a fixed egui shape vocabulary should not allocate a heap string for every submitted primitive.

This type remains egui-specific and provisional. It is not embedded in canonical `Witness` documents.

## Clipping experiment

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

`visible_fraction` is a **derived bounding-box measure**, not exact pixel/alpha coverage. A circle whose bounding box is half clipped does not necessarily have exactly half of its painted pixels visible. That stronger claim would require raster or shape-specific analysis.

## Paint order is not yet occlusion

Paint order plus overlapping visible bounds is stronger evidence than geometry alone, but it still does not automatically prove semantic occlusion.

Reasons include:

- transparent or partially transparent paint;
- hollow/stroked shapes whose bounding boxes overlap without covering the same pixels;
- complex meshes and callbacks;
- multiple paint primitives belonging to one widget or authored object;
- one paint primitive spanning several semantic concepts.

Therefore ViewWitness does not derive canonical `occludes` merely from `later paint order + rectangle overlap`.

A future lower-level relation such as `painted_after` or a paint-overlap diagnostic may be justified, but it should remain clearly weaker than a proven visual occlusion claim.

## Live reporting without render-thread work

`EguiPaintReporter` is an egui plugin that observes the public `FullOutput` boundary and emits `EguiPaintFrame` values into a bounded `sync_channel` using `try_send`.

A paint frame currently contains:

- raw numeric egui viewport identity;
- egui cumulative pass number for that viewport;
- pixels per point;
- `dropped_before`, the count of reporter frames lost since the previous successful delivery because the bounded queue was full;
- provisional paint observations.

The reporter performs no JSON/YAML serialization, persistence, network I/O, diffing, searching, or model work. The executable backpressure probe fills a one-frame queue, runs another GUI pass without consuming it, and establishes that:

- the GUI pass still completes;
- the saturated frame is dropped rather than blocking rendering;
- the next successfully delivered frame reports `dropped_before == 1`.

Loss is therefore observable evidence rather than a hidden consequence of protecting the render thread.

## Read-only continuous paint side channel

The worker-side `run_egui_paint_server` and `EguiPaintObserver` expose paint frames outside the GUI process without extending or impersonating `egui_inspection`.

The v0 side channel is deliberately small:

- TCP;
- a ViewWitness-specific versioned handshake;
- compact newline-delimited JSON frames;
- loopback default `127.0.0.1:5720`, distinct from egui inspection's `5719`;
- bounded message size;
- short client write timeout;
- one active observer, with a successfully initialized newer connection replacing the older one;
- latest-frame retention while no observer is connected.

The server is a **blocking worker function**. It receives already-copied paint frames from the bounded reporter queue and performs serialization/network work off the GUI thread.

An end-to-end loopback test paints a frame before any observer connects, then establishes that a later observer receives that retained frame without another repaint and subsequently receives new live frames. A separate test rejects an unrecognized protocol handshake.

The side channel is intentionally read-only. It is evidence transport, not an input/control protocol.

## Independent live streams are not exact correlation

The external `egui_inspection` semantic stream and continuous ViewWitness paint stream expose different clocks:

- `egui_inspection::Response::Tree.step` is a counter owned by the inspection plugin and incremented in that plugin's `output_hook`;
- `EguiPaintFrame.pass_nr` is egui's cumulative pass number for the active viewport.

The inspection counter is plugin-global while egui pass numbering is viewport-aware. Their values must therefore **not** be equated or joined by an assumed fixed offset.

An external AccessKit witness and an independently streamed paint frame are valid observations, but ViewWitness does not claim they describe the exact same pass.

## Exact same-pass correlation

`EguiFrameProbe` solves exact semantic↔paint correlation by avoiding clock reconciliation entirely.

A worker requests one capture. Capture eligibility is fixed at the **start** of a pass. This matters for application-authored custom-paint annotations: a request arriving halfway through a pass waits for the next repaint rather than collecting a semantic/paint frame whose annotation history began before the request existed.

One requested pass then supplies:

- the AccessKit update attached to that `FullOutput`;
- renderer-facing paint observations from that same `FullOutput`;
- egui's active viewport identity;
- egui's cumulative pass number;
- pixels per point;
- egui's current viewport rectangle;
- any explicit authored custom-paint bindings verified at end-of-pass.

The result is sent through a bounded channel using `try_send`. If the response queue is full, the request is explicitly reported as dropped; the GUI hook never blocks and never retries expensive work in a repaint loop.

`EguiFrameEvidence::into_correlated_capture()` performs the heavier semantic conversion off-thread and yields `EguiCorrelatedCapture`, containing a canonical semantic `Witness`, provisional generic paint observations, and provisional authored custom-paint evidence known to share the capture pass.

The canonical witness metadata names the clock (`egui_cumulative_pass_nr`) and correlation basis (`same_full_output`) rather than asking consumers to infer them.

## Viewport evidence finding

An early correlated-capture implementation attempted to derive viewport dimensions from AccessKit root bounds, matching the external inspection observer's only currently available strategy.

The all-features executable test rejected that assumption: a valid headless egui `FullOutput` can contain useful AccessKit semantics while its root lacks usable bounds, even though egui itself knows the viewport.

The in-process same-pass probe now copies `InputState::viewport_rect()` directly. `EguiCorrelatedCapture` preserves the full observed rectangle, including origin, while canonical `Viewport` currently uses its width and height. Invalid/non-finite observed viewport geometry is an error; ViewWitness does not fall back to invented dimensions.

This produces an important source rule:

> Prefer the strongest direct evidence available at the capture boundary. Do not reconstruct a fact from a weaker representation merely because another transport is forced to do so.

## Explicit authored custom-paint identity

Exact same-pass capture still does **not** reveal a generic AccessKit-node → paint-shape mapping. ViewWitness now solves a narrower, important case without guessing: applications may explicitly identify custom-painted objects at paint time.

`EguiPaintAnnotator::add_shape` paints normally through `Painter::add` and receives egui's real layer-local `ShapeIdx`. The annotator records, only during a requested exact-capture pass:

- application-authored object ID;
- application-authored role and optional name;
- the actual `LayerId` of the painter;
- the actual `ShapeIdx` returned by egui.

The epistemic split is explicit:

```text
object id / role / name     semantic_evidence = intended
LayerId + ShapeIdx binding  binding_evidence = observed
```

At egui's `on_end_pass` hook—after application UI code has finished painting but before graphic layers are drained into `FullOutput`—ViewWitness looks up that exact layer and slot through the public graphics API. It records the **final** shape kind, bounds, and clip rectangle occupying the handle and marks whether the handle was successfully verified.

This is stronger than storing geometry at annotation time.

The executable identity pressure test deliberately creates two authored objects with **identical overlapping bounds**. They remain distinct because their identities are tied to different real shape slots, not rectangle matching. The test also annotates one rectangle and subsequently replaces the exact slot through `Painter::set`; end-of-pass evidence reports the final shape as a **circle**. That demonstrates the binding follows egui's paint handle rather than the shape originally submitted or its coincident geometry.

A negative-control test annotates an ordinary frame when no exact capture was requested, then performs a later exact capture containing no authored objects. No old annotation leaks forward. Ordinary frames therefore do not accumulate custom-paint bookkeeping.

### Authored objects under clipping

The authored-object layer now composes with the same bounding-box clipping semantics as generic paint.

A separate executable test creates an authored 40×40 rectangle through a painter whose final finite clip preserves exactly a 20×40 half. End-of-pass handle verification observes:

- final object bounds: 40×40;
- final clip rect: 20×40 over the right half;
- derived `visible_bounds`: the 20×40 intersection;
- derived `visible_fraction`: `0.5`.

`EguiAuthoredPaintObject::visible_bounds()` and `visible_fraction()` are therefore derived from the **final verified handle evidence**, not from the descriptor or geometry remembered at annotation time.

The same caveat applies as for generic paint: `0.5` means half of the axis-aligned visual bounding rectangle survives the clip. It does **not** mean exactly half of the object's painted pixels or alpha survives.

The correlated agent projection labels the fraction as `visible_fraction_evidence=derived_bbox_clip` so an agent does not silently strengthen the claim.

The live showcase uses explicit authored identity for exactly two Canvas objects:

- `showcase:painted-rectangle`;
- `showcase:painted-circle`.

The canvas background and text labels remain ordinary anonymous paint. That is intentional: ViewWitness only grants custom-object identity where the application explicitly supplied it.

`EguiAuthoredPaintObject` remains egui-specific correlated evidence, not a canonical `WitnessNode`. The project is not claiming that all GUI concepts are accessibility nodes, nor that all authored graphics should be promoted into the cross-backend ontology.

## What remains unknown

The generic widget-to-paint problem remains open.

egui internally has rich `WidgetRect` / `WidgetRects` information including widget ID, parent UI ID, layer ID, full rectangle, clipped interaction rectangle, enabled state, interaction sense, and back-to-front ordering within layers. The complete collection is not currently exposed through the same public capture boundary used here.

ViewWitness must therefore still **not invent an automatic AccessKit-node → paint-shape mapping** from coincident geometry, names, or same-frame occurrence. Same time does not imply same identity.

Likewise, the new authored binding proves a layer-local paint handle. ViewWitness has not yet promoted a claim that every such handle can be resolved robustly to a flattened global `FullOutput` paint-order index across all unusual/missing-layer cases. That needs its own executable proof before becoming evidence.

The next useful pressure cases are:

- one authored object composed of multiple paint handles;
- authored objects across multiple layers/windows;
- reset/removal semantics for bound handles;
- safe mapping, if possible, from layer-local handles to flattened renderer order;
- interaction between authored object identity and diffs across frames.

## Promotion rule

Rendered concepts should enter the canonical cross-backend `Witness` model only after repeated examples show that they are general, useful, and nameable without egui-specific leakage.

For now:

- AccessKit semantic evidence belongs in canonical witnesses;
- egui paint observations remain a research/integration structure;
- bounding-box clip survival is deterministic derived evidence for both generic and explicitly authored paint;
- the continuous paint side channel transports egui-specific evidence without promoting it into the canonical model;
- `EguiCorrelatedCapture` proves same-pass origin without implying generic widget↔paint identity;
- explicitly instrumented custom paint can carry authored semantics plus an observed, end-of-pass-verified egui paint-handle binding;
- screenshots remain supporting raster evidence unless their transport supplies trustworthy same-frame correlation;
- generic widget↔paint association remains unknown unless a source explicitly supplies it.
