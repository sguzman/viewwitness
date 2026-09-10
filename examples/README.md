# Example corpus

The ViewWitness examples are **golden fixtures and design probes**, not decorative snippets.

Every `.yaml` witness in this tree is parsed and structurally validated by `tests/corpus.rs`. When the schema changes, the corpus tells us which kinds of GUI state the change invalidated.

## Snapshot probes

- `01-minimal-button.yaml` — smallest useful interactive witness.
- `02-login-form.yaml` — labels, text entry, checkbox state, focus, hierarchy.
- `03-editor-layout.yaml` — toolbar + sidebar + central canvas + inspector.
- `04-overlay-occlusion.yaml` — popup overlap and directional occlusion.
- `05-scroll-list.yaml` — scroll container and partially visible list content.
- `06-modal-dialog.yaml` — modal layering and disabled background interaction.
- `07-disabled-busy.yaml` — disabled controls plus progress/status state.
- `08-focus-keyboard.yaml` — focus and keyboard-oriented actions.
- `09-clipped-child.yaml` — clipping as a relation rather than vague prose.
- `10-pathological-overlap.yaml` — intentionally bad layout for agent diagnosis.
- `11-context-menu.yaml` — transient popup structure, menu actions, and semantic control relation.
- `12-table-selection.yaml` — dense tabular hierarchy, selected rows, and cells.
- `13-nested-scroll.yaml` — independent scroll spaces and a partially visible descendant.
- `14-tooltip-anchor.yaml` — ephemeral descriptive UI attached semantically to a hovered control.
- `15-tiny-viewport.yaml` — deliberately constrained viewport with an action escaping its usable layout.
- `16-canvas-selection.yaml` — freeform semantic canvas objects, selection indicators, and drag handles.

## Transition probes

- `transitions/01-inspector-resize/` — stable nodes whose bounds change between captures.
- `transitions/02-context-menu-open/` — stable background state plus newly added transient nodes and relation.

Transition fixtures are intended to become executable specifications for `WitnessDiff`, not merely pairs of screenshots in textual form.

## What the corpus is already teaching us

- Semantic hierarchy and visual overlap are separate facts.
- A node can be visually meaningful without being a conventional accessibility control (`shape`, `selection_indicator`, `drag_handle`).
- Transient lifetime belongs somewhere in the model, but a provisional property is enough until capture experience tells us whether it deserves a core field.
- Rectangle overlap cannot by itself prove occlusion; layering evidence is required.
- Scroll/clipping needs coordinate-space semantics beyond bare rectangles before it can be derived robustly.
- Pairwise geometry can become extremely noisy, so default derivation should be conservative and configurable.

## Corpus rule

Prefer adding a new example whenever a format or ontology question arises. A concept that cannot be demonstrated in a concrete witness probably does not belong in the core yet.

When possible, a new core behavior should arrive with both:

1. a concrete GUI fixture that makes the behavior necessary; and
2. an automated assertion over that fixture or a synthetic equivalent.

The corpus is therefore both a showcase and an executable pressure vessel for the schema.
