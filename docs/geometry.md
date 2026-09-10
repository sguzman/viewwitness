# Geometry derivation — v0

ViewWitness distinguishes observed geometry from facts deterministically derived from that geometry. The capture backend reports rectangles; the geometry layer turns those rectangles into explicit relations that humans and agents can query without re-solving layout from pixels.

## Design rules

1. **Derivation never becomes observation.** Every relation emitted by the geometry engine uses `evidence: derived`.
2. **Determinism matters.** Reordering nodes in the serialized witness must not reorder the semantic meaning of symmetric relations. Symmetric relation endpoints are canonicalized by node ID.
3. **Do not emit gratuitous inverse facts.** If `a left_of b` is emitted, the engine does not also emit `b right_of a`. Consumers can invert the relation when needed.
4. **Default output should stay legible.** Pairwise derivation is sibling-only by default. Otherwise a control nested inside a panel can redundantly overlap the panel's siblings, their descendants, and much of the rest of the interface.
5. **Hidden means excluded unless requested.** A node explicitly marked `visible: false` is ignored by default. Unknown visibility (`visible: null`) is not treated as hidden.
6. **Geometry is not z-order.** Rectangle overlap alone cannot prove occlusion. `occludes` must wait for evidence about layering/paint order rather than being guessed from intersection.

## Current derived relations

### `overlaps`

Two rectangles have a positive-area intersection. Touching edges are not overlap.

The relation carries:

- `overlap_width`
- `overlap_height`
- `overlap_area`
- `from_fraction` when the source has non-zero area
- `to_fraction` when the target has non-zero area

Because overlap is symmetric, `from` and `to` use lexical node-ID order for stable serialization.

### `left_of`

The source ends at or before the target begins on the X axis, and the two rectangles have a positive projection overlap on Y. The projection requirement prevents diagonally separated nodes from creating noisy cardinal relations.

Only `left_of` is emitted; an additional inverse `right_of` relation is intentionally omitted.

### `above`

The source ends at or before the target begins on the Y axis, and the two rectangles have a positive projection overlap on X.

Only `above` is emitted; an additional inverse `below` relation is intentionally omitted.

### edge alignment

`aligned_left`, `aligned_right`, `aligned_top`, and `aligned_bottom` compare the corresponding edges within a configurable tolerance. The default is 0.5 logical coordinate units.

## Scope controls

`GeometryOptions` currently controls:

- sibling-only versus cross-parent derivation;
- whether explicitly hidden nodes participate;
- overlap derivation;
- axis-separation derivation;
- edge-alignment derivation;
- alignment tolerance.

These are analysis controls, not part of the serialized witness schema.

## Deliberately deferred

The following need stronger evidence or more corpus pressure before implementation:

- occlusion and visible-fraction after z-order/clipping;
- containment as a named relation distinct from semantic parentage;
- transforms and non-axis-aligned geometry;
- scroll-space versus viewport-space composition;
- nearest-neighbor or adjacency relations;
- relation pruning optimized specifically for LLM token budgets.

The governing rule remains: add an example first, then make the concept earn its place.
