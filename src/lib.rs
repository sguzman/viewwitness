//! ViewWitness: structured witnesses of observed GUI state.
//!
//! The v0 API is intentionally small. The example corpus is part of the design
//! process: concepts should earn their place by surviving varied GUI fixtures.

mod format;
mod geometry;
mod model;

pub use format::{from_yaml, to_yaml};
pub use geometry::{
    GeometryOptions, derive_geometry_relations, derive_geometry_relations_with_options,
};
pub use model::{Capture, Node, Rect, Relation, Viewport, Witness};
