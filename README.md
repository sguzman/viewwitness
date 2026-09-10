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

- `src/` — canonical Rust model, serialization, geometry derivation, and optional egui capture adapter.
- `docs/ontology.md` — the small vocabulary ViewWitness commits to.
- `docs/format-v0.md` — draft wire-format contract.
- `docs/geometry.md` — deterministic geometry relation semantics.
- `docs/egui.md` — egui/AccessKit capture architecture and UI-thread boundary.
- `examples/snapshots/` — a growing corpus of representative GUI witnesses.
- `examples/transitions/` — before/after witnesses for state-transition and diff work.
- `examples/egui_capture.rs` — real headless egui frame → ViewWitness → YAML.
- `tests/` — executable checks over the corpus and actual egui output.

## Current egui slice

The optional `egui` feature translates egui-produced AccessKit output into the canonical ViewWitness model. It preserves semantic hierarchy, roles, names/values, bounds, visibility, focus, selection/toggle state, actions, and selected semantic relations.

Run the current end-to-end example with:

```text
cargo run --example egui_capture --features egui
```

The example builds a real headless egui frame, enables AccessKit generation, converts the resulting frame output into a `Witness`, and prints its YAML projection.

This is intentionally only the semantic capture slice. ViewWitness does not claim that AccessKit exhausts rendered reality; layering, clipping, custom-painted canvas content, and other visual evidence remain active design work.

## Status

ViewWitness has moved beyond format-only exploration. The current project has:

- an executable synthetic corpus;
- deterministic geometry derivation;
- real egui/AccessKit capture translation behind an optional feature;
- headless tests against actual egui output;
- an end-to-end capture example;
- CI over both the default and all-features builds.

The v0 schema is still intentionally provisional. The next major pressure source is the living egui showcase and live external-observer path.

## Non-goals for the first phase

ViewWitness is not currently trying to become:

- a universal GUI standard;
- a replacement for AccessKit or platform accessibility APIs;
- a pixel-perfect rendering format;
- a browser DOM representation;
- a cross-language SDK ecosystem;
- an autonomous GUI agent.

Those may become integration surfaces later. First we want one very good answer for Rust + egui.
