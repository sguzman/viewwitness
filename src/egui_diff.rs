use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

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

/// Diff of application-authored custom-paint objects and their bindings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredDiff {
    #[serde(default)]
    pub objects_added: Vec<EguiAuthoredPaintObject>,
    #[serde(default)]
    pub objects_removed: Vec<EguiAuthoredPaintObject>,
    /// Material changes to object-level authored semantics only. Binding changes
    /// are first-class below rather than being collapsed into one `bindings` blob.
    #[serde(default)]
    pub objects_changed: Vec<EguiAuthoredObjectChange>,
    #[serde(default)]
    pub bindings_added: Vec<EguiAuthoredBindingDelta>,
    #[serde(default)]
    pub bindings_removed: Vec<EguiAuthoredBindingDelta>,
    #[serde(default)]
    pub bindings_changed: Vec<EguiAuthoredBindingChange>,
    #[serde(default)]
    pub ambiguous_ids: Vec<EguiAuthoredIdAmbiguity>,
    /// Authored sub-binding IDs that cannot be trusted as continuity keys because
    /// they are duplicated within a uniquely matched logical object on at least
    /// one side of the comparison.
    #[serde(default)]
    pub binding_ambiguities: Vec<EguiAuthoredBindingIdAmbiguity>,
    /// Non-material layer-local slot churn for uniquely matched object/binding
    /// identities whose authored semantics and material binding state remained
    /// otherwise unchanged.
    #[serde(default)]
    pub execution_handle_churn: Vec<EguiAuthoredExecutionHandleChange>,
}

impl EguiAuthoredDiff {
    #[must_use]
    pub fn is_materially_empty(&self) -> bool {
        self.objects_added.is_empty()
            && self.objects_removed.is_empty()
            && self.objects_changed.is_empty()
            && self.bindings_added.is_empty()
            && self.bindings_removed.is_empty()
            && self.bindings_changed.is_empty()
            && self.ambiguous_ids.is_empty()
            && self.binding_ambiguities.is_empty()
    }
}

/// Material field changes for one uniquely matched authored object.
///
/// This type deliberately excludes paint bindings. Object semantics and rendered
/// sub-part state are separate evidence surfaces and are diffed separately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredObjectChange {
    pub id: String,
    pub fields: BTreeMap<String, FieldChange>,
}

/// One authored binding that exists on only one side of the comparison.
///
/// `binding_ordinal` is that side's vector position and is diagnostic only. A
/// keyed binding's continuity comes from `binding.authored_binding_id`, not from
/// this ordinal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredBindingDelta {
    pub object_id: String,
    pub binding_ordinal: usize,
    pub binding: EguiAuthoredPaintBinding,
}

/// Material field changes for one uniquely matched authored binding.
///
/// Exactly one continuity description is normally populated:
/// `authored_binding_id` for explicitly keyed bindings, or `unkeyed_ordinal` for
/// legacy bindings matched conservatively by relative unkeyed order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredBindingChange {
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unkeyed_ordinal: Option<usize>,
    pub fields: BTreeMap<String, FieldChange>,
}

/// An authored object ID that cannot be used as an automatic continuity key
/// because it is duplicated on at least one side of the comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredIdAmbiguity {
    pub id: String,
    pub before_count: usize,
    pub after_count: usize,
}

/// An authored binding ID that cannot be used as an automatic sub-object
/// continuity key because it is duplicated within its logical object on at least
/// one side of the comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredBindingIdAmbiguity {
    pub object_id: String,
    pub binding_id: String,
    pub before_count: usize,
    pub after_count: usize,
}

/// Change in a layer-local egui slot for one material-equivalent authored binding.
///
/// `authored_binding_id` is present when continuity came from an explicit
/// application-authored sub-binding key. `binding_ordinal` records the binding's
/// before-side vector position as diagnostic context; for unkeyed bindings the
/// actual continuity basis is their relative unkeyed ordinal.
///
/// This is diagnostic execution evidence, not an object-identity change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredExecutionHandleChange {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_binding_id: Option<String>,
    pub binding_ordinal: usize,
    pub before_shape_index: usize,
    pub after_shape_index: usize,
}

/// Compare two exact correlated captures without pretending renderer slots are
/// durable object identity.
///
/// Authored object IDs are used as continuity evidence only when an ID appears
/// exactly once on both sides. Within such an object, an authored binding ID is
/// used only when it is unique within that object on both sides. Duplicate
/// object or binding IDs become explicit ambiguity instead of heuristic matching.
/// Unkeyed bindings retain relative-ordinal matching. Layer-local `ShapeIdx`
/// values are excluded from material binding comparison and surfaced as
/// diagnostic execution churn only when the binding is otherwise unchanged.
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
    let mut bindings_added = Vec::new();
    let mut bindings_removed = Vec::new();
    let mut bindings_changed = Vec::new();
    let mut ambiguous_ids = Vec::new();
    let mut binding_ambiguities = Vec::new();
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
                if !fields.is_empty() {
                    objects_changed.push(EguiAuthoredObjectChange {
                        id: id.to_owned(),
                        fields,
                    });
                }

                let object_binding_ambiguities =
                    binding_id_ambiguities(before_object, after_object);
                if !object_binding_ambiguities.is_empty() {
                    binding_ambiguities.extend(object_binding_ambiguities);
                    continue;
                }

                diff_object_bindings(
                    before_object,
                    after_object,
                    &mut bindings_added,
                    &mut bindings_removed,
                    &mut bindings_changed,
                    &mut execution_handle_churn,
                );
            }
            (None, None) => unreachable!("id came from the union of object maps"),
        }
    }

    bindings_added.sort_by(binding_delta_order);
    bindings_removed.sort_by(binding_delta_order);
    bindings_changed.sort_by(|a, b| {
        (&a.object_id, &a.authored_binding_id, a.unkeyed_ordinal).cmp(&(
            &b.object_id,
            &b.authored_binding_id,
            b.unkeyed_ordinal,
        ))
    });
    execution_handle_churn.sort_by(|a, b| {
        (&a.id, &a.authored_binding_id, a.binding_ordinal).cmp(&(
            &b.id,
            &b.authored_binding_id,
            b.binding_ordinal,
        ))
    });

    EguiAuthoredDiff {
        objects_added,
        objects_removed,
        objects_changed,
        bindings_added,
        bindings_removed,
        bindings_changed,
        ambiguous_ids,
        binding_ambiguities,
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
    fields
}

fn diff_object_bindings(
    before: &EguiAuthoredPaintObject,
    after: &EguiAuthoredPaintObject,
    bindings_added: &mut Vec<EguiAuthoredBindingDelta>,
    bindings_removed: &mut Vec<EguiAuthoredBindingDelta>,
    bindings_changed: &mut Vec<EguiAuthoredBindingChange>,
    execution_handle_churn: &mut Vec<EguiAuthoredExecutionHandleChange>,
) {
    let before_keyed = keyed_bindings(before);
    let after_keyed = keyed_bindings(after);
    let keyed_ids: BTreeSet<&str> = before_keyed
        .keys()
        .chain(after_keyed.keys())
        .copied()
        .collect();

    for binding_id in keyed_ids {
        match (before_keyed.get(binding_id), after_keyed.get(binding_id)) {
            (None, Some((after_ordinal, after_binding))) => {
                bindings_added.push(EguiAuthoredBindingDelta {
                    object_id: after.id.clone(),
                    binding_ordinal: *after_ordinal,
                    binding: (*after_binding).clone(),
                });
            }
            (Some((before_ordinal, before_binding)), None) => {
                bindings_removed.push(EguiAuthoredBindingDelta {
                    object_id: before.id.clone(),
                    binding_ordinal: *before_ordinal,
                    binding: (*before_binding).clone(),
                });
            }
            (Some((before_ordinal, before_binding)), Some((_, after_binding))) => {
                let fields = material_binding_fields(before_binding, after_binding);
                if fields.is_empty() {
                    if before_binding.shape_index != after_binding.shape_index {
                        execution_handle_churn.push(EguiAuthoredExecutionHandleChange {
                            id: before.id.clone(),
                            authored_binding_id: Some(binding_id.to_owned()),
                            binding_ordinal: *before_ordinal,
                            before_shape_index: before_binding.shape_index,
                            after_shape_index: after_binding.shape_index,
                        });
                    }
                } else {
                    bindings_changed.push(EguiAuthoredBindingChange {
                        object_id: before.id.clone(),
                        authored_binding_id: Some(binding_id.to_owned()),
                        unkeyed_ordinal: None,
                        fields,
                    });
                }
            }
            (None, None) => unreachable!("binding id came from the union of keyed maps"),
        }
    }

    let before_unkeyed = unkeyed_bindings(before);
    let after_unkeyed = unkeyed_bindings(after);
    let common = before_unkeyed.len().min(after_unkeyed.len());

    for relative_ordinal in 0..common {
        let (before_ordinal, before_binding) = before_unkeyed[relative_ordinal];
        let (_, after_binding) = after_unkeyed[relative_ordinal];
        let fields = material_binding_fields(before_binding, after_binding);
        if fields.is_empty() {
            if before_binding.shape_index != after_binding.shape_index {
                execution_handle_churn.push(EguiAuthoredExecutionHandleChange {
                    id: before.id.clone(),
                    authored_binding_id: None,
                    binding_ordinal: before_ordinal,
                    before_shape_index: before_binding.shape_index,
                    after_shape_index: after_binding.shape_index,
                });
            }
        } else {
            bindings_changed.push(EguiAuthoredBindingChange {
                object_id: before.id.clone(),
                authored_binding_id: None,
                unkeyed_ordinal: Some(relative_ordinal),
                fields,
            });
        }
    }

    for (binding_ordinal, binding) in before_unkeyed.into_iter().skip(common) {
        bindings_removed.push(EguiAuthoredBindingDelta {
            object_id: before.id.clone(),
            binding_ordinal,
            binding: binding.clone(),
        });
    }
    for (binding_ordinal, binding) in after_unkeyed.into_iter().skip(common) {
        bindings_added.push(EguiAuthoredBindingDelta {
            object_id: after.id.clone(),
            binding_ordinal,
            binding: binding.clone(),
        });
    }
}

fn material_binding_fields(
    before: &EguiAuthoredPaintBinding,
    after: &EguiAuthoredPaintBinding,
) -> BTreeMap<String, FieldChange> {
    let mut fields = BTreeMap::new();
    field_change(
        &mut fields,
        "binding_evidence",
        &before.binding_evidence,
        &after.binding_evidence,
    );
    field_change(
        &mut fields,
        "layer_order",
        &before.layer_order,
        &after.layer_order,
    );
    field_change(&mut fields, "layer_id", &before.layer_id, &after.layer_id);
    field_change(
        &mut fields,
        "verified_at_end_pass",
        &before.verified_at_end_pass,
        &after.verified_at_end_pass,
    );
    field_change(&mut fields, "kind", &before.kind, &after.kind);
    field_change(&mut fields, "bounds", &before.bounds, &after.bounds);
    field_change(
        &mut fields,
        "clip_rect",
        &before.clip_rect,
        &after.clip_rect,
    );
    fields
}

fn binding_id_ambiguities(
    before: &EguiAuthoredPaintObject,
    after: &EguiAuthoredPaintObject,
) -> Vec<EguiAuthoredBindingIdAmbiguity> {
    let before_counts = binding_id_counts(before);
    let after_counts = binding_id_counts(after);
    let ids: BTreeSet<&str> = before_counts
        .keys()
        .chain(after_counts.keys())
        .copied()
        .collect();

    ids.into_iter()
        .filter_map(|binding_id| {
            let before_count = before_counts.get(binding_id).copied().unwrap_or(0);
            let after_count = after_counts.get(binding_id).copied().unwrap_or(0);
            (before_count > 1 || after_count > 1).then(|| EguiAuthoredBindingIdAmbiguity {
                object_id: before.id.clone(),
                binding_id: binding_id.to_owned(),
                before_count,
                after_count,
            })
        })
        .collect()
}

fn binding_id_counts(object: &EguiAuthoredPaintObject) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for binding in &object.bindings {
        if let Some(id) = binding.authored_binding_id.as_deref() {
            *counts.entry(id).or_insert(0) += 1;
        }
    }
    counts
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

fn keyed_bindings(
    object: &EguiAuthoredPaintObject,
) -> BTreeMap<&str, (usize, &EguiAuthoredPaintBinding)> {
    object
        .bindings
        .iter()
        .enumerate()
        .filter_map(|(ordinal, binding)| {
            binding
                .authored_binding_id
                .as_deref()
                .map(|id| (id, (ordinal, binding)))
        })
        .collect()
}

fn unkeyed_bindings(object: &EguiAuthoredPaintObject) -> Vec<(usize, &EguiAuthoredPaintBinding)> {
    object
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| binding.authored_binding_id.is_none())
        .collect()
}

fn binding_delta_order(
    a: &EguiAuthoredBindingDelta,
    b: &EguiAuthoredBindingDelta,
) -> std::cmp::Ordering {
    (
        &a.object_id,
        &a.binding.authored_binding_id,
        a.binding_ordinal,
    )
        .cmp(&(
            &b.object_id,
            &b.binding.authored_binding_id,
            b.binding_ordinal,
        ))
}
