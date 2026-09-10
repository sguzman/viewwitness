use std::collections::BTreeMap;

use serde_json::json;

use crate::{Node, Rect, Relation, Witness};

/// Controls deterministic relations derived from observed node rectangles.
///
/// Defaults are intentionally conservative. In particular, relations are only
/// derived between siblings so a deeply nested control does not redundantly
/// overlap every ancestor-adjacent region in a large interface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryOptions {
    /// Restrict pairwise derivation to nodes with the same parent.
    pub siblings_only: bool,
    /// Include nodes explicitly marked `visible: false`.
    pub include_hidden: bool,
    /// Derive positive-area `overlaps` relations.
    pub derive_overlap: bool,
    /// Derive `left_of` and `above` for axis-separated nodes whose projection
    /// overlaps on the other axis.
    pub derive_separation: bool,
    /// Derive edge-alignment relations.
    pub derive_alignment: bool,
    /// Maximum logical-coordinate difference considered aligned.
    pub alignment_tolerance: f32,
}

impl Default for GeometryOptions {
    fn default() -> Self {
        Self {
            siblings_only: true,
            include_hidden: false,
            derive_overlap: true,
            derive_separation: true,
            derive_alignment: true,
            alignment_tolerance: 0.5,
        }
    }
}

/// Derive geometry relations using [`GeometryOptions::default`].
#[must_use]
pub fn derive_geometry_relations(witness: &Witness) -> Vec<Relation> {
    derive_geometry_relations_with_options(witness, GeometryOptions::default())
}

/// Derive deterministic relations from observed axis-aligned bounds.
///
/// The result is independent of node input order. Symmetric relations such as
/// `overlaps` and alignments use lexical node-id order. Directional relations
/// use their geometric direction and intentionally do not emit the redundant
/// inverse (`left_of` is emitted, not an additional `right_of`).
#[must_use]
pub fn derive_geometry_relations_with_options(
    witness: &Witness,
    options: GeometryOptions,
) -> Vec<Relation> {
    let mut nodes: Vec<&Node> = witness
        .nodes
        .iter()
        .filter(|node| {
            node.bounds.is_some() && (options.include_hidden || node.visible != Some(false))
        })
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut relations = Vec::new();

    for (index, a) in nodes.iter().enumerate() {
        for b in nodes.iter().skip(index + 1) {
            if options.siblings_only && a.parent != b.parent {
                continue;
            }

            let a_bounds = a.bounds.expect("filtered to bounded nodes");
            let b_bounds = b.bounds.expect("filtered to bounded nodes");

            if options.derive_overlap
                && let Some(intersection) = a_bounds.intersection(b_bounds)
            {
                let mut properties = BTreeMap::new();
                properties.insert("overlap_width".into(), json!(intersection.width));
                properties.insert("overlap_height".into(), json!(intersection.height));
                properties.insert("overlap_area".into(), json!(intersection.area()));

                if a_bounds.area() > 0.0 {
                    properties.insert(
                        "from_fraction".into(),
                        json!(intersection.area() / a_bounds.area()),
                    );
                }
                if b_bounds.area() > 0.0 {
                    properties.insert(
                        "to_fraction".into(),
                        json!(intersection.area() / b_bounds.area()),
                    );
                }

                relations.push(Relation {
                    kind: "overlaps".into(),
                    from: a.id.clone(),
                    to: b.id.clone(),
                    evidence: "derived".into(),
                    properties,
                });
            }

            if options.derive_separation {
                derive_separation(a, a_bounds, b, b_bounds, &mut relations);
            }

            if options.derive_alignment {
                derive_alignment(a, a_bounds, b, b_bounds, options, &mut relations);
            }
        }
    }

    relations
}

fn derive_separation(a: &Node, a_bounds: Rect, b: &Node, b_bounds: Rect, out: &mut Vec<Relation>) {
    if ranges_overlap(a_bounds.y, a_bounds.bottom(), b_bounds.y, b_bounds.bottom()) {
        if a_bounds.right() <= b_bounds.x {
            out.push(relation("left_of", &a.id, &b.id));
        } else if b_bounds.right() <= a_bounds.x {
            out.push(relation("left_of", &b.id, &a.id));
        }
    }

    if ranges_overlap(a_bounds.x, a_bounds.right(), b_bounds.x, b_bounds.right()) {
        if a_bounds.bottom() <= b_bounds.y {
            out.push(relation("above", &a.id, &b.id));
        } else if b_bounds.bottom() <= a_bounds.y {
            out.push(relation("above", &b.id, &a.id));
        }
    }
}

fn derive_alignment(
    a: &Node,
    a_bounds: Rect,
    b: &Node,
    b_bounds: Rect,
    options: GeometryOptions,
    out: &mut Vec<Relation>,
) {
    let tolerance = options.alignment_tolerance.max(0.0);

    if approximately_equal(a_bounds.x, b_bounds.x, tolerance) {
        out.push(relation("aligned_left", &a.id, &b.id));
    }
    if approximately_equal(a_bounds.right(), b_bounds.right(), tolerance) {
        out.push(relation("aligned_right", &a.id, &b.id));
    }
    if approximately_equal(a_bounds.y, b_bounds.y, tolerance) {
        out.push(relation("aligned_top", &a.id, &b.id));
    }
    if approximately_equal(a_bounds.bottom(), b_bounds.bottom(), tolerance) {
        out.push(relation("aligned_bottom", &a.id, &b.id));
    }
}

fn relation(kind: &str, from: &str, to: &str) -> Relation {
    Relation {
        kind: kind.into(),
        from: from.into(),
        to: to.into(),
        evidence: "derived".into(),
        properties: BTreeMap::new(),
    }
}

fn ranges_overlap(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> bool {
    a_min < b_max && b_min < a_max
}

fn approximately_equal(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

impl Rect {
    #[must_use]
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    #[must_use]
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    #[must_use]
    pub fn area(self) -> f32 {
        self.width * self.height
    }

    /// Positive-area intersection. Merely touching edges do not intersect.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        let width = right - x;
        let height = bottom - y;

        (width > 0.0 && height > 0.0).then_some(Self {
            x,
            y,
            width,
            height,
        })
    }
}
