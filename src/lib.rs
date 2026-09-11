//! ViewWitness: structured witnesses of observed GUI state.
//!
//! The v0 API is intentionally small. The example corpus is part of the design
//! process: concepts should earn their place by surviving varied GUI fixtures.

mod agent_text;
mod diff;
#[cfg(feature = "egui")]
mod egui_capture;
#[cfg(feature = "egui")]
mod egui_paint;
mod format;
mod geometry;
#[cfg(feature = "observer")]
mod inspection_observer;
mod model;

pub use agent_text::{diff_to_agent_text, to_agent_text};
pub use diff::{FieldChange, NodeChange, ViewportChange, WitnessDiff, diff_witnesses};
#[cfg(feature = "egui")]
pub use egui_capture::{
    EguiCaptureContext, witness_from_egui_output, witness_from_egui_tree_update,
};
#[cfg(feature = "egui")]
pub use egui_paint::{EguiPaintObservation, paint_observations_from_egui_output};
pub use format::{diff_from_yaml, diff_to_yaml, from_yaml, to_yaml};
pub use geometry::{
    GeometryOptions, derive_geometry_relations, derive_geometry_relations_with_options,
};
#[cfg(feature = "observer")]
pub use inspection_observer::{InspectionObserver, ScreenshotEvidence, SettleResult};
pub use model::{Capture, Node, NodeIdentity, Rect, Relation, Viewport, Witness};
