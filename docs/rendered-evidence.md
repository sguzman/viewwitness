# Rendered evidence — egui v0 research

ViewWitness cannot stop at accessibility semantics. A GUI can be semantically well described while still being visibly broken: clipped, covered, painted in the wrong order, overflowing, or custom-drawn without meaningful accessibility nodes.

This document records executable rendered-evidence findings from egui. These findings are intentionally kept separate from the canonical `Witness` schema until the concepts survive more pressure.

## Evidence layers must remain separate

The current experiments establish at least four distinct kinds of GUI testimony:

1. **Semantic evidence** — AccessKit nodes, roles, values, hierarchy, focus, actions, and semantic relations.
2. **Paint-submission evidence** — shapes egui submitted to the renderer, including their visual bounding rectangles and flattened paint order.
3. **Clip-survival evidence** — how much of a submitted shape's axis-aligned visual bounds survives its observed scissor rectangle.
4. **Raster evidence** — what pixels finally appear in a screenshot/framebuffer.

These are related but not interchangeable.

A submitted paint shape can exist while being fully clipped. A semantic node can exist without a unique paint primitive. A paint primitive can exist without any AccessKit node. A screenshot can show the final pixels without explaining which semantic object or paint submission produced them.

ViewWitness should preserve these distinctions rather than choosing one representation and pretending it exhausts GUI reality.

## What egui publicly exposes

`egui::FullOutput` exposes `shapes: Vec<ClippedShape>` as the renderer-facing paint list. Each `ClippedShape` contains:

- a `Shape`;
- a clip/scissor rectangle.

`Shape::visual_bounding_rect()` provides an axis-aligned visual bounding box. egui's graphics drain flattens ordered layer paint lists into the `FullOutput::shapes` sequence, so sequence position is usable observed back-to-front paint-order evidence.

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

The executable paint probe submits two custom-painted shapes through the same finite clip rectangle:

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
- multiple paint primitives belonging to one widget;
- one paint primitive spanning several semantic concepts.

Therefore ViewWitness should not derive canonical `occludes` merely from `later paint order + rectangle overlap`.

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

## Read-only paint side channel

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

## Temporal correlation is not yet solved

The external semantic and paint channels expose different clocks:

- `egui_inspection::Response::Tree.step` is a counter owned by the inspection plugin and incremented in that plugin's `output_hook`;
- `EguiPaintFrame.pass_nr` is egui's cumulative pass number for the active viewport.

The inspection counter is plugin-global while egui pass numbering is viewport-aware. Their values must therefore **not** be equated or joined by an assumed fixed offset.

This means an external AccessKit witness and an external paint frame are currently independently valid observations, but ViewWitness does not claim they describe the exact same pass.

Exact semantic↔paint correlation will require a shared ViewWitness capture point or an upstream transport that exposes a trustworthy shared frame token. Clock guessing is not an acceptable substitute.

## The widget-to-paint gap

egui internally has rich `WidgetRect` / `WidgetRects` information including widget ID, parent UI ID, layer ID, full rectangle, clipped interaction rectangle, enabled state, interaction sense, and back-to-front ordering within layers.

That data is highly relevant to ViewWitness, but the complete collection is not currently exposed through the public generic plugin hooks or the `egui_inspection` tree protocol. The inspection protocol exposes AccessKit state and screenshot capture, not the renderer paint list or the full widget-rectangle table.

ViewWitness must therefore **not invent an automatic AccessKit-node → paint-shape mapping** from coincident geometry or labels and then report it as observation.

Potential future paths include:

- a shared ViewWitness-specific capture request that copies semantic + paint evidence from one `FullOutput` only when requested, then immediately hands it to an off-thread worker;
- an upstream egui/inspection extension exposing the missing structured evidence and a shared frame token;
- explicit application annotations for custom-painted objects.

Any such path must preserve the project rule that the GUI thread reports cheap state and does not perform serialization, searching, diffing, networking, or analysis.

## Promotion rule

Rendered concepts should enter the canonical cross-backend `Witness` model only after repeated examples show that they are general, useful, and nameable without egui-specific leakage.

For now:

- AccessKit semantic evidence belongs in canonical witnesses;
- egui paint observations remain a research/integration structure;
- bounding-box clip survival is deterministic derived evidence;
- the live paint side channel transports egui-specific evidence without promoting it into the canonical model;
- screenshots remain supporting raster evidence;
- semantic↔paint same-frame correlation remains unproven across the two external channels;
- widget↔paint association remains unknown unless a source explicitly supplies it.
