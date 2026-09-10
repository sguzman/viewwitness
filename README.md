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
- explicit identity provenance and stability;
- explicit distinction between observed and derived facts;
- deterministic textual serialization;
- state-to-state diffs suitable for debugging and agent verification.

The long-term test is simple: an agent should be able to inspect a broken egui application, explain what is visibly wrong, change the code, capture another witness, and demonstrate from the resulting state that the defect changed or disappeared.

## Repository shape

- `src/` — canonical Rust model, serialization, geometry derivation, optional egui capture adapter, and optional live inspection observer.
- `docs/ontology.md` — the small vocabulary ViewWitness commits to.
- `docs/format-v0.md` — draft wire-format contract.
- `docs/geometry.md` — deterministic geometry relation semantics.
- `docs/egui.md` — egui/AccessKit capture architecture, live observation, and UI-thread boundary.
- `examples/snapshots/` — a growing corpus of representative GUI witnesses.
- `examples/transitions/` — before/after witnesses for state-transition and diff work.
- `examples/egui_capture.rs` — real headless egui frame → ViewWitness → YAML.
- `examples/inspection_capture.rs` — running inspected egui app → external ViewWitness observer → YAML.
- `examples/showcase.rs` — native eframe pressure surface for capture and diagnosis.
- `tests/` — executable checks over the corpus, actual egui output, diffs, identity behavior, and live-protocol framing.

## Current egui slice

The optional `egui` feature translates egui-produced AccessKit output into the canonical ViewWitness model. It preserves semantic hierarchy, roles, names/values, bounds, visibility, focus, selection/toggle state, actions, selected semantic relations, and explicit identity evidence.

Run the headless end-to-end example with:

```text
cargo run --example egui_capture --features egui
```

The optional `observer` feature speaks the versioned `egui_inspection` protocol directly from an external process. With an inspected eframe application running, this prints one live witness:

```text
cargo run --example inspection_capture --features observer
```

The observer path is deliberately external: GUI code reports semantic state, while ViewWitness performs conversion, serialization, diffing, derivation, and future agent-facing work outside the render/update thread.

AccessKit/egui identity is treated as evidence rather than absolute truth. Automatically generated identity has been empirically shown to survive ordinary state changes while remaining sensitive to structural insertion, so captured nodes state that stability explicitly. Application-authored IDs are preserved as additional evidence when available.

This remains only part of rendered reality. ViewWitness does not claim that AccessKit exhausts layering, clipping, styling, custom-painted canvas content, or other visual evidence.

## Status

ViewWitness has moved beyond format-only exploration. The current project has:

- an executable synthetic snapshot corpus and transition corpus;
- deterministic geometry derivation;
- first-class `WitnessDiff` with YAML projection;
- real egui/AccessKit capture translation behind an optional feature;
- explicit node identity provenance/stability;
- headless tests against actual egui output, including cross-frame identity behavior;
- a living native eframe showcase;
- a read-only external `egui_inspection` observer;
- a loopback integration test exercising the actual inspection handshake, TCP framing, and tree response;
- CI over both the default and all-features builds.

The v0 schema is still intentionally provisional. The next major pressure is no longer whether a running egui app can testify about itself; it can. The next question is how much additional rendered evidence—layering, clipping, custom paint, and selective screenshots—is required for robust agent diagnosis, and how compactly that evidence should be projected for agents.

## Non-goals for the first phase

ViewWitness is not currently trying to become:

- a universal GUI standard;
- a replacement for AccessKit or platform accessibility APIs;
- a pixel-perfect rendering format;
- a browser DOM representation;
- a cross-language SDK ecosystem;
- an autonomous GUI agent.

Those may become integration surfaces later. First we want one very good answer for Rust + egui.
