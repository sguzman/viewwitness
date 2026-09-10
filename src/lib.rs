//! ViewWitness: structured witnesses of observed GUI state.
//!
//! The v0 API is intentionally small. The example corpus is part of the design
//! process: concepts should earn their place by surviving varied GUI fixtures.

mod format;
mod model;

pub use format::{from_yaml, to_yaml};
pub use model::{Capture, Node, Rect, Relation, Viewport, Witness};
