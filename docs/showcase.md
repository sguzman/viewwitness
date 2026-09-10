# Living showcase

The native showcase is not a product demo. It is a **GUI evidence laboratory** whose states should force ViewWitness to answer concrete questions.

Run it with:

```text
cargo run --example showcase --features showcase
```

The `showcase` feature enables eframe's inspection integration. Set `EGUI_INSPECTION=1` when the external inspection path is being exercised.

## Operating rule

Every showcase state should serve at least one of these purposes:

1. prove a semantic/visual fact survives capture;
2. expose a fact that the current capture path loses;
3. pressure stable identity across a transition;
4. pressure geometry, clipping, layering, or visibility derivation;
5. demonstrate an agent-facing debugging case.

A state that merely makes the application prettier does not belong here.

## Current galleries

| Gallery / switch | What a human sees | What ViewWitness should learn | Current expectation |
| --- | --- | --- | --- |
| Controls / text field | label + editable value | naming, value, focusability, label relation | AccessKit should carry most semantics |
| Controls / slider | named numeric range/value | value and increment/decrement/set-value affordance | semantic capture expected |
| Controls / checkbox | toggled control | boolean/toggled state | semantic capture expected |
| Controls / radio mode | mutually exclusive choices | selected/toggled state and group semantics | inspect actual egui output before freezing mapping |
| Busy switch | disabled Apply + spinner/status | availability and busy distinction | disabled action is captureable; spinner/status semantics need observation |
| Tooltip switch | hover descriptions | ephemeral descriptive node/relation | capture only while tooltip exists; stability is deliberately weak |
| Long labels | stretched form/table/scroll content | text-pressure effects on geometry and clipping | semantics should survive; geometry may change materially |
| Table gallery | repeated rows/cells + one selected row | dense hierarchy and selection without relation explosion | semantic tree needs inspection; geometry derivation should remain conservative |
| Scrolling gallery | 60 rows in bounded scroll area | scroll container, partial visibility, clip space | intentionally unresolved beyond basic semantics |
| Custom canvas | painted rectangle + circle | evidence absent from accessibility-only capture | negative control: shapes should NOT magically appear as semantic objects |
| Floating inspector | foreground window over application | independent surface/layer + geometry | AccessKit gives semantics/bounds; exact occlusion requires layer evidence |
| Pathological overlap | inspector moved across primary content | diagnose real visual collision | cross-parent geometry should detect overlap when the involved bounds are available |
| Modal-like window | centered destructive confirmation | foreground/dialog/modal semantics and blocked background | inspect what egui/AccessKit reports before claiming background blocking |

## Transition probes to add

The living application should eventually support deterministic scripted or manually reproducible transitions that mirror the YAML transition corpus.

### Inspector movement

Before:

- inspector does not substantially overlap primary content.

After:

- `Pathological overlap` is enabled;
- inspector moves over primary content;
- stable inspector identity should survive;
- a witness diff should primarily report bounds/geometry changes rather than remove + add.

### Context or popup appearance

Before:

- stable background control exists;
- popup absent.

After:

- popup node(s) appear;
- semantic background identities remain stable;
- popup lifetime is transient;
- overlap may be derivable from geometry;
- occlusion remains unproven without layer/paint evidence.

### Busy state

Before:

- Apply action enabled.

After:

- same control remains identifiable;
- control becomes disabled;
- status/spinner appears or changes;
- a diff should capture state change compactly.

### Selection

Before/after:

- same table/canvas population;
- selected identity changes;
- diff should not resend the entire repeated structure as though it changed.

## Negative controls

Negative controls are first-class. They prevent ViewWitness from confusing absence of evidence with evidence of absence.

### Custom paint

The custom canvas intentionally paints objects that AccessKit cannot know are meaningful GUI objects unless the application supplies semantics. The correct accessibility-only witness may therefore contain the canvas interaction surface but not the rectangle and circle as distinct semantic nodes.

That is not a capture bug. It identifies the exact boundary where ViewWitness needs another evidence source or explicit application annotation.

### Occlusion

Two rectangles overlapping is not sufficient evidence that one visually occludes the other. Showcase overlap cases should be used to obtain and test real layer/paint evidence before `occludes` is derived automatically.

### Clipping

A child extending beyond a scroll/container rectangle is not by itself a complete clipping model. The showcase should pressure actual egui clip rectangles and coordinate spaces before ViewWitness makes exact visible-fraction claims.

## Corpus growth target

The showcase should grow toward these families as implementation becomes useful:

- buttons/links/toggles/radio/slider/text entry;
- focus and keyboard traversal;
- read-only, required, disabled, busy, error and validation states;
- combo boxes, menus, context menus and tooltips;
- tabs and collapsing sections;
- trees and hierarchical selection;
- dense tables/grids;
- horizontal/vertical/nested scrolling;
- resizable panels and separators;
- drag/drop and handles;
- multiple floating windows;
- modal and non-modal dialogs;
- custom painting and canvas objects;
- clipping and overflow;
- tiny and huge viewports;
- DPI/scale changes;
- long, empty, Unicode and localized text;
- transient one-frame or short-lived nodes;
- intentionally broken layouts;
- stable-state repetition where nothing meaningful changes.

Each family should eventually have at least one golden witness, invariant test, or explicit negative-control expectation.
