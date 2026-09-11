use serde::{Deserialize, Serialize};

use crate::Rect;

/// Provisional observation of one egui paint entry.
///
/// This is intentionally egui-specific research evidence, not yet part of the
/// canonical cross-backend `Witness` schema. `order` follows the flattened
/// `FullOutput::shapes` sequence passed to the renderer, so smaller values are
/// painted earlier (farther back) than larger values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiPaintObservation {
    pub order: usize,
    pub kind: String,
    pub bounds: Rect,
    /// egui's observed scissor/clip rectangle when it is finite.
    ///
    /// `None` represents an effectively unbounded/non-finite clip rather than
    /// inventing finite coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip_rect: Option<Rect>,
}

impl EguiPaintObservation {
    /// Deterministically intersect the visual bounds with the observed finite
    /// clip rectangle. An unbounded clip leaves the visual bounds unchanged.
    #[must_use]
    pub fn visible_bounds(&self) -> Option<Rect> {
        match self.clip_rect {
            Some(clip) => self.bounds.intersection(clip),
            None => Some(self.bounds),
        }
    }

    /// Fraction of the shape's axis-aligned visual bounds surviving the clip.
    ///
    /// This is a geometric derivation, not a claim about alpha coverage inside
    /// those bounds. A circle clipped to half its bounding box, for example,
    /// reports bounding-box visibility rather than exact painted-pixel area.
    #[must_use]
    pub fn visible_fraction(&self) -> f32 {
        let area = self.bounds.area();
        if area <= 0.0 || !area.is_finite() {
            return 0.0;
        }

        self.visible_bounds()
            .map_or(0.0, |visible| (visible.area() / area).clamp(0.0, 1.0))
    }
}

/// Read the renderer-facing egui paint list as provisional visual evidence.
///
/// Empty/non-finite shapes are omitted. No attempt is made to associate a paint
/// entry with an AccessKit node or widget because egui does not publicly expose
/// that mapping at this boundary.
#[must_use]
pub fn paint_observations_from_egui_output(output: &egui::FullOutput) -> Vec<EguiPaintObservation> {
    output
        .shapes
        .iter()
        .enumerate()
        .filter_map(|(order, clipped)| {
            let bounds = rect_from_egui(clipped.shape.visual_bounding_rect())?;
            if bounds.width <= 0.0 || bounds.height <= 0.0 {
                return None;
            }

            Some(EguiPaintObservation {
                order,
                kind: shape_kind(&clipped.shape).into(),
                bounds,
                clip_rect: rect_from_egui(clipped.clip_rect),
            })
        })
        .collect()
}

fn rect_from_egui(rect: egui::Rect) -> Option<Rect> {
    let width = rect.width();
    let height = rect.height();
    let values = [rect.min.x, rect.min.y, width, height];
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(Rect {
            x: rect.min.x,
            y: rect.min.y,
            width,
            height,
        })
}

fn shape_kind(shape: &egui::epaint::Shape) -> &'static str {
    use egui::epaint::Shape;

    match shape {
        Shape::Noop => "noop",
        Shape::Vec(_) => "group",
        Shape::Circle(_) => "circle",
        Shape::Ellipse(_) => "ellipse",
        Shape::LineSegment { .. } => "line_segment",
        Shape::Path(_) => "path",
        Shape::Rect(_) => "rect",
        Shape::Text(_) => "text",
        Shape::Mesh(_) => "mesh",
        Shape::QuadraticBezier(_) => "quadratic_bezier",
        Shape::CubicBezier(_) => "cubic_bezier",
        Shape::Callback(_) => "callback",
    }
}
