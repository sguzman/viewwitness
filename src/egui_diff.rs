use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, FieldChange,
    WitnessDiff, diff_witnesses,
};

/// Material transition between two exact correlated egui captures.
///
/// Canonical semantic state uses the ordinary [`WitnessDiff`]. Explicit
/// custom-paint objects are compared separately because they remain egui-specific
/// evidence rather than canonical `Witness` nodes. Generic anonymous paint is
/// intentionally not diffed here: it has no durable identity and naive list
/// comparison would mostly report renderer churn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiCorrelatedDiff {
    pub before_request_id: u64,
    pub after_request_id: u64,
    pub before_pass_nr: u64,
    pub after_pass_nr: u64,
    pub semantic: WitnessDiff,
    pub authored: EguiAuthoredDiff,
}

impl EguiCorrelatedDiff {
    /// True only when no material semantic/authored change is known and no
    /// authored-ID ambiguity prevents a trustworthy comparison.
    ///
    /// Pure layer-local `ShapeIdx` churn is execution evidence and does not make
    /// this false by itself.
    #[must_use]
    pub fn is_materially_empty(&self) -> bool {
        self.semantic.is_empty() && self.authored.is_materially_empty()
    }
}

/// Diff of application-authored custom-paint objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredDiff {
    #[serde(default)]
    pub objects_added: Vec<EguiAuthoredPaintObject>,
    #[serde(default)]
    pub objects_removed: Vec<EguiAuthoredPaintObject>,
    #[serde(default)]
    pub objects_changed: Vec<EguiAuthoredObjectChange>,
    #[serde(default)]
    pub ambiguous_ids: Vec<EguiAuthoredIdAmbiguity>,
    /// Non-material layer-local slot churn for uniquely matched objects whose
    /// authored semantics and binding material remained otherwise unchanged.
    #[serde(default)]
    pub execution_handle_churn: Vec<EguiAuthoredExecutionHandleChange>,
}

impl EguiAuthoredDiff {
    #[must_use]
    pub fn is_materially_empty(&self) -> bool {
        self.objects_added.is_empty()
            && self.objects_removed.is_empty()
            && self.objects_changed.is_empty()
            && self.ambiguous_ids.is_empty()
    }
}

/// Material field changes for one uniquely matched authored object.
///
/// `bindings` is compared as an authored ordered collection of material binding
/// snapshots with `shape_index` intentionally excluded. ViewWitness therefore
/// reports renderer-slot churn separately instead of laundering it into object
/// identity or material visual change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredObjectChange {
    pub id: String,
    pub fields: BTreeMap<String, FieldChange>,
}

/// An authored ID that cannot be used as an automatic continuity key because it
/// is duplicated on at least one side of the comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredIdAmbiguity {
    pub id: String,
    pub before_count: usize,
    pub after_count: usize,
}

/// Change in a layer-local egui slot for one material-equivalent binding ordinal.
///
/// This is diagnostic execution evidence, not an object-identity change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredExecutionHandleChange {
    pub id: String,
    pub binding_ordinal: usize,
    pub before_shape_index: usize,
    pub after_shape_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MaterialBinding {
    binding_evidence: String,
    layer_order: crate::EguiLayerOrder,
    layer_id: u64,
    verified_at_end_pass: bool,
    kind: Option<crate::EguiPaintKind>,
    bounds: Option<crate::Rect>,
    clip_rect: Option<crate::Rect>,
}

impl From<&EguiAuthoredPaintBinding> for MaterialBinding {
    fn from(binding: &EguiAuthoredPaintBinding) -> Self {
        Self {
            binding_evidence: binding.binding_evidence.clone(),
            layer_order: binding.layer_order,
            layer_id: binding.layer_id,
            verified_at_end_pass: binding.verified_at_end_pass,
            kind: binding.kind,
            bounds: binding.bounds,
            clip_rect: binding.clip_rect,
        }
    }
}

/// Compare two exact correlated captures without pretending renderer slots are
/// durable object identity.
///
/// Authored object IDs are used as continuity evidence only when an ID appears
/// exactly once on both sides. Any duplicate ID becomes explicit ambiguity.
/// Layer-local `ShapeIdx` values are excluded from material binding comparison;
/// if material state is otherwise identical, ordinal slot churn is surfaced in
/// `execution_handle_churn` instead.
#[must_use]
pub fn diff_correlated_captures(
    before: &EguiCorrelatedCapture,
    after: &EguiCorrelatedCapture,
) -> EguiCorrelatedDiff {
    EguiCorrelatedDiff {
        before_request_id: before.request_id,
        after_request_id: after.request_id,
        before_pass_nr: before.pass_nr,
        after_pass_nr: after.pass_nr,
        semantic: diff_witnesses(&before.witness, &after.witness),
        authored: diff_authored_objects(&before.authored_objects, &after.authored_objects),
    }
}

fn diff_authored_objects(
    before: &[EguiAuthoredPaintObject],
    after: &[EguiAuthoredPaintObject],
) -> EguiAuthoredDiff {
    let before_by_id = objects_by_id(before);
    let after_by_id = objects_by_id(after);
    let ids: BTreeSet<&str> = before_by_id
        .keys()
        .chain(after_by_id.keys())
        .copied()
        .collect();

    let mut objects_added = Vec::new();
    let mut objects_removed = Vec::new();
    let mut objects_changed = Vec::new();
    let mut ambiguous_ids = Vec::new();
    let mut execution_handle_churn = Vec::new();

    for id in ids {
        let before_objects = before_by_id.get(id).map(Vec::as_slice).unwrap_or_default();
        let after_objects = after_by_id.get(id).map(Vec::as_slice).unwrap_or_default();

        if before_objects.len() > 1 || after_objects.len() > 1 {
            ambiguous_ids.push(EguiAuthoredIdAmbiguity {
                id: id.to_owned(),
                before_count: before_objects.len(),
                after_count: after_objects.len(),
            });
            continue;
        }

        match (before_objects.first(), after_objects.first()) {
            (None, Some(after_object)) => objects_added.push((**after_object).clone()),
            (Some(before_object), None) => objects_removed.push((**before_object).clone()),
            (Some(before_object), Some(after_object)) => {
                let before_object = *before_object;
                let after_object = *after_object;
                let fields = authored_object_fields(before_object, after_object);
                if fields.is_empty() {
                    execution_handle_churn.extend(handle_churn(before_object, after_object));
                } else {
                    objects_changed.push(EguiAuthoredObjectChange {
                        id: id.to_owned(),
                        fields,
                    });
                }
            }
            (None, None) => unreachable!("id came from the union of object maps"),
        }
    }

    EguiAuthoredDiff {
        objects_added,
        objects_removed,
        objects_changed,
        ambiguous_ids,
        execution_handle_churn,
    }
}

fn objects_by_id<'a>(
    objects: &'a [EguiAuthoredPaintObject],
) -> BTreeMap<&'a str, Vec<&'a EguiAuthoredPaintObject>> {
    let mut map: BTreeMap<&str, Vec<&EguiAuthoredPaintObject>> = BTreeMap::new();
    for object in objects {
        map.entry(object.id.as_str()).or_default().push(object);
    }
    map
}

fn authored_object_fields(
    before: &EguiAuthoredPaintObject,
    after: &EguiAuthoredPaintObject,
) -> BTreeMap<String, FieldChange> {
    let mut fields = BTreeMap::new();
    field_change(&mut fields, "role", &before.role, &after.role);
    field_change(&mut fields, "name", &before.name, &after.name);
    field_change(
        &mut fields,
        "semantic_evidence",
        &before.semantic_evidence,
        &after.semantic_evidence,
    );

    let before_bindings: Vec<MaterialBinding> = before.bindings.iter().map(Into::into).collect();
    let after_bindings: Vec<MaterialBinding> = after.bindings.iter().map(Into::into).collect();
    field_change(&mut fields, "bindings", &before_bindings, &after_bindings);
    fields
}

fn field_change<T: Serialize + PartialEq>(
    fields: &mut BTreeMap<String, FieldChange>,
    key: &str,
    before: &T,
    after: &T,
) {
    if before == after {
        return;
    }
    fields.insert(
        key.to_owned(),
        FieldChange {
            before: serde_json::to_value(before).expect("egui diff field must serialize"),
            after: serde_json::to_value(after).expect("egui diff field must serialize"),
        },
    );
}

fn handle_churn(
    before: &EguiAuthoredPaintObject,
    after: &EguiAuthoredPaintObject,
) -> Vec<EguiAuthoredExecutionHandleChange> {
    if before.bindings.len() != after.bindings.len() {
        return Vec::new();
    }

    before
        .bindings
        .iter()
        .zip(&after.bindings)
        .enumerate()
        .filter_map(|(binding_ordinal, (before_binding, after_binding))| {
            (before_binding.shape_index != after_binding.shape_index).then(|| {
                EguiAuthoredExecutionHandleChange {
                    id: before.id.clone(),
                    binding_ordinal,
                    before_shape_index: before_binding.shape_index,
                    after_shape_index: after_binding.shape_index,
                }
            })
        })
        .collect()
}
