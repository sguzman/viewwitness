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

## Transition probes

`transitions/01-inspector-resize/` contains a before/after pair using stable IDs. It is the first target for future `WitnessDiff` work.

## Corpus rule

Prefer adding a new example whenever a format or ontology question arises. A concept that cannot be demonstrated in a concrete witness probably does not belong in the core yet.
