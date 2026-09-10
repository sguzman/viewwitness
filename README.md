# ViewWitness

**A Rust-first witness format for graphical user interfaces.**

ViewWitness turns an observed GUI frame into structured, serializable, diffable evidence that humans, tests, tools, and software agents can reason about.

The project begins deliberately narrow: **Rust + egui**. Broader GUI backends and language bindings are allowed by the architecture, but they are not current work.

## Why

Screenshots are rich but expensive to interpret. Accessibility trees are semantically useful but do not fully describe rendered reality. Source code describes intent, not necessarily what appeared on screen.

ViewWitness aims to preserve the useful intersection:

- semantic identity and hierarchy;
- actual geometry and visibility;
- interaction state and affordances;
- salient visual facts;
- spatial and semantic relationships;
- capture provenance;
- explicit distinction between observed and derived facts;
- deterministic textual serialization;
- state-to-state diffs suitable for debugging and agent verification.

The long-term test is simple: an agent should be able to inspect a broken egui application, explain what is visibly wrong, change the code, capture another witness, and demonstrate from the resulting state that the defect changed or disappeared.

## Repository shape

- `src/` — canonical Rust model and serialization support.
- `docs/ontology.md` — the small vocabulary ViewWitness commits to.
- `docs/format-v0.md` — draft wire-format contract.
- `examples/snapshots/` — a growing corpus of representative GUI witnesses.
- `examples/transitions/` — before/after witnesses for state-transition and diff work.
- `tests/` — executable checks over the example corpus.

## Status

ViewWitness is at **format exploration / executable-specification** stage. The v0 schema is intentionally provisional. Examples are expected to pressure the model and expose missing concepts before an egui capture implementation is frozen.

## Non-goals for the first phase

ViewWitness is not currently trying to become:

- a universal GUI standard;
- a replacement for AccessKit or platform accessibility APIs;
- a pixel-perfect rendering format;
- a browser DOM representation;
- a cross-language SDK ecosystem;
- an autonomous GUI agent.

Those may become integration surfaces later. First we want one very good answer for Rust + egui.
