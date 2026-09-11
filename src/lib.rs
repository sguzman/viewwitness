//! ViewWitness: structured witnesses of observed GUI state.
//!
//! The v0 API is intentionally small. The example corpus is part of the design
//! process: concepts should earn their place by surviving varied GUI fixtures.

mod agent_text;
mod diff;
#[cfg(feature = "egui")]
mod egui_agent_text;
#[cfg(feature = "egui")]
mod egui_capture;
#[cfg(feature = "egui")]
mod egui_capture_transport;
#[cfg(feature = "egui")]
mod egui_frame_probe;
#[cfg(feature = "egui")]
mod egui_paint;
#[cfg(feature = "egui")]
mod egui_paint_annotation;
#[cfg(feature = "egui")]
mod egui_paint_reporter;
#[cfg(feature = "egui")]
mod egui_paint_transport;
mod format;
mod geometry;
#[cfg(feature = "observer")]
mod inspection_observer;
mod model;

pub use agent_text::{diff_to_agent_text, to_agent_text};
pub use diff::{FieldChange, NodeChange, ViewportChange, WitnessDiff, diff_witnesses};
#[cfg(feature = "egui")]
pub use egui_agent_text::correlated_capture_to_agent_text;
#[cfg(feature = "egui")]
pub use egui_capture::{
    EguiCaptureContext, witness_from_egui_output, witness_from_egui_tree_update,
};
#[cfg(feature = "egui")]
pub use egui_capture_transport::{
    DEFAULT_EGUI_CAPTURE_ADDR, EGUI_CAPTURE_PROTOCOL_MAGIC, EGUI_CAPTURE_PROTOCOL_VERSION,
    EguiCaptureObserver, MAX_EGUI_CAPTURE_MESSAGE_BYTES, run_egui_capture_server,
};
#[cfg(feature = "egui")]
pub use egui_frame_probe::{EguiCorrelatedCapture, EguiFrameEvidence, EguiFrameProbe};
#[cfg(feature = "egui")]
pub use egui_paint::{EguiPaintKind, EguiPaintObservation, paint_observations_from_egui_output};
#[cfg(feature = "egui")]
pub use egui_paint_annotation::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiLayerOrder, EguiPaintAnnotator,
    EguiPaintObjectDescriptor, EguiPaintObjectScope,
};
#[cfg(feature = "egui")]
pub use egui_paint_reporter::{EguiPaintFrame, EguiPaintReporter};
#[cfg(feature = "egui")]
pub use egui_paint_transport::{
    DEFAULT_EGUI_PAINT_ADDR, EGUI_PAINT_PROTOCOL_MAGIC, EGUI_PAINT_PROTOCOL_VERSION,
    EguiPaintObserver, MAX_EGUI_PAINT_MESSAGE_BYTES, run_egui_paint_server,
};
pub use format::{diff_from_yaml, diff_to_yaml, from_yaml, to_yaml};
pub use geometry::{
    GeometryOptions, derive_geometry_relations, derive_geometry_relations_with_options,
};
#[cfg(feature = "observer")]
pub use inspection_observer::{InspectionObserver, ScreenshotEvidence, SettleResult};
pub use model::{Capture, Node, NodeIdentity, Rect, Relation, Viewport, Witness};
