use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use egui::{LayerId, Order, Painter, layers::ShapeIdx};
use serde::{Deserialize, Serialize};

use crate::{EguiPaintKind, Rect};

/// Application-declared semantics for one custom-painted object.
///
/// These fields are authored/intended semantics. Concrete renderer bindings are
/// recorded separately so one logical object can intentionally own more than
/// one egui paint submission without duplicating its identity metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EguiPaintObjectDescriptor {
    pub id: String,
    pub role: String,
    pub name: Option<String>,
}

impl EguiPaintObjectDescriptor {
    #[must_use]
    pub fn new(id: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: role.into(),
            name: None,
        }
    }

    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Stable serialized projection of egui's layer category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EguiLayerOrder {
    Background,
    Middle,
    Foreground,
    Tooltip,
    Debug,
}

impl From<Order> for EguiLayerOrder {
    fn from(order: Order) -> Self {
        match order {
            Order::Background => Self::Background,
            Order::Middle => Self::Middle,
            Order::Foreground => Self::Foreground,
            Order::Tooltip => Self::Tooltip,
            Order::Debug => Self::Debug,
        }
    }
}

/// One observed binding between an authored logical object and a concrete egui
/// paint slot used during the captured pass.
///
/// `binding_evidence` is `observed` because ViewWitness records the actual
/// `LayerId + ShapeIdx` returned by egui and verifies that slot against the final
/// paint list at end-of-pass. Bounds, clip, and kind are likewise read from that
/// final slot instead of remembered from the original submission call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredPaintBinding {
    pub binding_evidence: String,
    pub layer_order: EguiLayerOrder,
    pub layer_id: u64,
    pub shape_index: usize,
    pub verified_at_end_pass: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<EguiPaintKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip_rect: Option<Rect>,
}

impl EguiAuthoredPaintBinding {
    /// Intersect the final observed visual bounds with the final observed finite
    /// clip rectangle for this verified paint slot.
    ///
    /// `None` means the handle could not provide usable bounds. An unbounded
    /// clip leaves the observed bounds unchanged.
    #[must_use]
    pub fn visible_bounds(&self) -> Option<Rect> {
        let bounds = self.bounds?;
        match self.clip_rect {
            Some(clip) => bounds.intersection(clip),
            None => Some(bounds),
        }
    }

    /// Fraction of this binding's final axis-aligned visual bounds that survives
    /// the final observed clip rectangle.
    ///
    /// Like [`crate::EguiPaintObservation::visible_fraction`], this is derived
    /// bounding-box evidence, not exact painted-pixel or alpha coverage.
    #[must_use]
    pub fn visible_fraction(&self) -> f32 {
        let Some(bounds) = self.bounds else {
            return 0.0;
        };
        let area = bounds.area();
        if area <= 0.0 || !area.is_finite() {
            return 0.0;
        }

        self.visible_bounds()
            .map_or(0.0, |visible| (visible.area() / area).clamp(0.0, 1.0))
    }
}

/// One application-authored logical custom-paint object.
///
/// `semantic_evidence` is `intended` because identity/name/role are declared by
/// the application. Each entry in `bindings` is separate observed execution
/// evidence. ViewWitness does not infer object grouping from duplicate IDs or
/// coincident geometry: multi-shape membership exists only when the application
/// explicitly groups the paint submissions through [`EguiPaintAnnotator::paint_object`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredPaintObject {
    pub id: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub semantic_evidence: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<EguiAuthoredPaintBinding>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingPaintBinding {
    pub layer_id: LayerId,
    pub shape_index: ShapeIdx,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingPaintObject {
    pub descriptor: EguiPaintObjectDescriptor,
    pub bindings: Vec<PendingPaintBinding>,
}

pub(crate) type PendingPaintObjects = Arc<Mutex<Vec<PendingPaintObject>>>;

/// Pass-local scope for explicitly grouping several egui paint handles into one
/// application-authored logical object.
///
/// The scope is created by [`EguiPaintAnnotator::paint_object`]. On ordinary
/// unrequested frames it still performs painting normally but does not allocate
/// or retain binding bookkeeping.
pub struct EguiPaintObjectScope {
    bindings: Option<Vec<PendingPaintBinding>>,
}

impl EguiPaintObjectScope {
    /// Paint one shape and include its concrete egui handle in this logical
    /// object's explicit binding set when exact capture is active.
    pub fn add_shape(
        &mut self,
        painter: &Painter,
        shape: impl Into<egui::epaint::Shape>,
    ) -> ShapeIdx {
        let shape_index = painter.add(shape);
        self.bind_shape(painter, shape_index);
        shape_index
    }

    /// Include an already-created shape handle in this logical object's binding
    /// set when exact capture is active.
    pub fn bind_shape(&mut self, painter: &Painter, shape_index: ShapeIdx) {
        let Some(bindings) = &mut self.bindings else {
            return;
        };
        bindings.push(PendingPaintBinding {
            layer_id: painter.layer_id(),
            shape_index,
        });
    }
}

/// Lightweight application-side helper for explicitly identifying custom paint.
///
/// Painting always happens normally. Annotation bookkeeping is activated only
/// for a pass in which the exact frame probe has an outstanding capture request,
/// so ordinary frames pay only an atomic load for an annotated logical object or
/// singular `add_shape` call.
#[derive(Clone)]
pub struct EguiPaintAnnotator {
    active_request: Arc<AtomicU64>,
    pending: PendingPaintObjects,
}

impl EguiPaintAnnotator {
    pub(crate) fn new(active_request: Arc<AtomicU64>, pending: PendingPaintObjects) -> Self {
        Self {
            active_request,
            pending,
        }
    }

    /// Paint one application-authored logical object that may consist of several
    /// concrete egui paint submissions.
    ///
    /// Object membership is explicit through this closure. ViewWitness never
    /// later groups independent annotations merely because they repeat an ID or
    /// overlap geometrically. Objects that emit no bindings are omitted from
    /// paint evidence rather than being reported as if they rendered.
    pub fn paint_object<R>(
        &self,
        descriptor: EguiPaintObjectDescriptor,
        paint: impl FnOnce(&mut EguiPaintObjectScope) -> R,
    ) -> R {
        let active = self.active_request.load(Ordering::Acquire) != 0;
        let mut object = EguiPaintObjectScope {
            bindings: active.then(Vec::new),
        };
        let result = paint(&mut object);

        if let Some(bindings) = object.bindings
            && !bindings.is_empty()
        {
            lock_pending(&self.pending).push(PendingPaintObject {
                descriptor,
                bindings,
            });
        }

        result
    }

    /// Paint one custom shape and, during an exact capture pass, bind the
    /// supplied authored object descriptor to egui's returned paint handle.
    ///
    /// This remains the convenient one-binding form of [`Self::paint_object`].
    pub fn add_shape(
        &self,
        painter: &Painter,
        descriptor: EguiPaintObjectDescriptor,
        shape: impl Into<egui::epaint::Shape>,
    ) -> ShapeIdx {
        let shape_index = painter.add(shape);
        self.bind_shape(painter, descriptor, shape_index);
        shape_index
    }

    /// Bind one authored logical object to an already-created paint handle.
    ///
    /// The handle is verified against egui's final paint list at end-of-pass;
    /// invalid or reset handles remain explicit rather than being rematched by
    /// geometry or label heuristics.
    pub fn bind_shape(
        &self,
        painter: &Painter,
        descriptor: EguiPaintObjectDescriptor,
        shape_index: ShapeIdx,
    ) {
        if self.active_request.load(Ordering::Acquire) == 0 {
            return;
        }

        lock_pending(&self.pending).push(PendingPaintObject {
            descriptor,
            bindings: vec![PendingPaintBinding {
                layer_id: painter.layer_id(),
                shape_index,
            }],
        });
    }
}

pub(crate) fn new_pending_paint_objects() -> PendingPaintObjects {
    Arc::new(Mutex::new(Vec::new()))
}

pub(crate) fn clear_pending_paint_objects(pending: &PendingPaintObjects) {
    lock_pending(pending).clear();
}

pub(crate) fn resolve_pending_paint_objects(
    ctx: &egui::Context,
    pending: &PendingPaintObjects,
) -> Vec<EguiAuthoredPaintObject> {
    let pending = lock_pending(pending).clone();

    pending
        .into_iter()
        .map(|pending| EguiAuthoredPaintObject {
            id: pending.descriptor.id,
            role: pending.descriptor.role,
            name: pending.descriptor.name,
            semantic_evidence: "intended".into(),
            bindings: pending
                .bindings
                .into_iter()
                .map(|binding| resolve_binding(ctx, binding))
                .collect(),
        })
        .collect()
}

fn resolve_binding(ctx: &egui::Context, pending: PendingPaintBinding) -> EguiAuthoredPaintBinding {
    let resolved = ctx.graphics(|graphics| {
        graphics.get(pending.layer_id).and_then(|paint_list| {
            paint_list
                .all_entries()
                .nth(pending.shape_index.0)
                .map(|clipped| {
                    (
                        shape_kind(&clipped.shape),
                        rect_from_egui(clipped.shape.visual_bounding_rect()),
                        rect_from_egui(clipped.clip_rect),
                    )
                })
        })
    });

    let (verified_at_end_pass, kind, bounds, clip_rect) = match resolved {
        Some((kind, bounds, clip_rect)) => (true, Some(kind), bounds, clip_rect),
        None => (false, None, None, None),
    };

    EguiAuthoredPaintBinding {
        binding_evidence: "observed".into(),
        layer_order: pending.layer_id.order.into(),
        layer_id: pending.layer_id.id.value(),
        shape_index: pending.shape_index.0,
        verified_at_end_pass,
        kind,
        bounds,
        clip_rect,
    }
}

fn lock_pending(
    pending: &PendingPaintObjects,
) -> std::sync::MutexGuard<'_, Vec<PendingPaintObject>> {
    pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn rect_from_egui(rect: egui::Rect) -> Option<Rect> {
    let width = rect.width();
    let height = rect.height();
    [rect.min.x, rect.min.y, width, height]
        .into_iter()
        .all(f32::is_finite)
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
