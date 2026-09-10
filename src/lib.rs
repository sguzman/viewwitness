//! ViewWitness: structured witnesses of observed GUI state.
//!
//! The v0 API is intentionally small. The example corpus is part of the design
//! process: concepts should earn their place by surviving varied GUI fixtures.

mod diff;
#[cfg(feature = "egui")]
mod egui_capture;
mod format;
mod geometry;
mod model;

pub use diff::{FieldChange, NodeChange, ViewportChange, WitnessDiff, diff_witnesses};
#[cfg(feature = "egui")]
pub use egui_capture::{
    EguiCaptureContext, witness_from_egui_output, witness_from_egui_tree_update,
};
pub use format::{diff_from_yaml, diff_to_yaml, from_yaml, to_yaml};
pub use geometry::{
    GeometryOptions, derive_geometry_relations, derive_geometry_relations_with_options,
};
pub use model::{Capture, Node, NodeIdentity, Rect, Relation, Viewport, Witness};
