use serde::{Deserialize, Serialize};

use crate::Rect;

/// Compact classification of an egui paint primitive.
///
/// This is deliberately an enum instead of a `String`: paint capture runs at
/// egui's renderer-facing output boundary, so classifying a shape must not heap
/// allocate once per primitive just to preserve a fixed vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EguiPaintKind {
    Noop,
    Group,
    Circle,
    Ellipse,
    LineSegment,
    Path,
    Rect,
    Text,
    Mesh,
    QuadraticBezier,
    CubicBezier,
    Callback,
}

/// Provisional observation of one egui paint entry.
///
/// This is intentionally egui-specific research evidence, not yet part of the
/// canonical cross-backend `Witness` schema. `order` follows the flattened
/// `FullOutput::shapes` sequence passed to the renderer, so smaller values are
/// painted earlier (farther back) than larger values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiPaintObservation {
    pub order: usize,
    pub kind: EguiPaintKind,
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
                kind: shape_kind(&clipped.shape),
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

fn shape_kind(shape: &egui::epaint::Shape) -> EguiPaintKind {
    use egui::epaint::Shape;

    match shape {
        Shape::Noop => EguiPaintKind::Noop,
        Shape::Vec(_) => EguiPaintKind::Group,
        Shape::Circle(_) => EguiPaintKind::Circle,
        Shape::Ellipse(_) => EguiPaintKind::Ellipse,
        Shape::LineSegment { .. } => EguiPaintKind::LineSegment,
        Shape::Path(_) => EguiPaintKind::Path,
        Shape::Rect(_) => EguiPaintKind::Rect,
        Shape::Text(_) => EguiPaintKind::Text,
        Shape::Mesh(_) => EguiPaintKind::Mesh,
        Shape::QuadraticBezier(_) => EguiPaintKind::QuadraticBezier,
        Shape::CubicBezier(_) => EguiPaintKind::CubicBezier,
        Shape::Callback(_) => EguiPaintKind::Callback,
    }
}
