use serde::{Deserialize, Serialize};

use crate::{EguiAuthoredPaintBinding, EguiCorrelatedDiff};

/// Narrow epistemic assessment of application-authored cross-frame continuity.
///
/// This does not claim that the overall GUI evidence is complete or that a
/// repair is safe to infer. It answers only whether the authored object/binding
/// identity evidence contains a known ambiguity that blocks unique attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EguiAuthoredContinuityStatus {
    /// No duplicate authored object or binding identity is known in this diff.
    /// Other evidence can still be missing, weak, or contradictory.
    NoKnownAmbiguity,
    /// At least one authored object or binding identity is duplicated, so a
    /// unique cross-frame attribution is not supported for the affected scope.
    Ambiguous,
}

/// Summary of whether authored identity can support unique cross-frame
/// attribution for a correlated diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredContinuityAssessment {
    pub status: EguiAuthoredContinuityStatus,
    pub object_ambiguity_count: usize,
    pub binding_ambiguity_count: usize,
}

impl EguiAuthoredContinuityAssessment {
    /// True when duplicate authored identity prevents a unique attribution.
    #[must_use]
    pub fn unique_attribution_blocked(&self) -> bool {
        self.status == EguiAuthoredContinuityStatus::Ambiguous
    }
}

/// Whether one authored binding's resolution flag and final paint testimony are
/// mutually compatible with the ViewWitness resolver contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EguiAuthoredBindingConsistencyStatus {
    Consistent,
    Contradictory,
}

/// One concrete contradiction inside an authored binding's resolution evidence.
///
/// The live resolver establishes a narrow contract:
///
/// - a verified slot always has a resolved paint `kind`;
/// - an unverified slot has no final `kind`, `bounds`, or `clip_rect` testimony.
///
/// Bounds and clip may still be absent on a verified slot when usable finite
/// geometry was unavailable, so that absence is incompleteness rather than a
/// contradiction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EguiAuthoredBindingConflictKind {
    VerifiedWithoutKind,
    UnverifiedWithKind,
    UnverifiedWithBounds,
    UnverifiedWithClipRect,
}

/// Narrow consistency assessment for one authored paint binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EguiAuthoredBindingConsistencyAssessment {
    pub status: EguiAuthoredBindingConsistencyStatus,
    pub conflicts: Vec<EguiAuthoredBindingConflictKind>,
}

impl EguiAuthoredBindingConsistencyAssessment {
    #[must_use]
    pub fn contradictory(&self) -> bool {
        self.status == EguiAuthoredBindingConsistencyStatus::Contradictory
    }
}

/// Whether a particular authored paint binding was resolved to final paint
/// evidence at the end of the captured pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EguiAuthoredBindingResolutionStatus {
    Verified,
    Unverified,
    /// The serialized binding makes mutually incompatible resolution claims.
    Contradictory,
}

/// Visibility testimony for one authored binding with resolution provenance
/// preserved.
///
/// `visible_fraction` is present only when the final slot is verified, the
/// binding is internally consistent, and usable bounds exist. The fraction is
/// still the existing derived bounding-box clip measure; this type does not
/// promote it to raster coverage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiAuthoredBindingVisibilityAssessment {
    pub resolution: EguiAuthoredBindingResolutionStatus,
    pub visible_fraction: Option<f32>,
}

impl EguiAuthoredBindingVisibilityAssessment {
    /// True only when enough consistent final paint geometry exists to support
    /// the derived visibility fraction.
    #[must_use]
    pub fn visibility_known(&self) -> bool {
        self.visible_fraction.is_some()
    }
}

/// Assess only the authored-identity continuity evidence already present in a
/// correlated diff.
///
/// This deliberately does not synthesize a confidence score or promote
/// `NoKnownAmbiguity` into a claim of evidential completeness. Later Stage-G
/// pressure may add other orthogonal epistemic assessments without weakening
/// this narrow identity rule.
#[must_use]
pub fn assess_authored_continuity(diff: &EguiCorrelatedDiff) -> EguiAuthoredContinuityAssessment {
    let object_ambiguity_count = diff.authored.ambiguous_ids.len();
    let binding_ambiguity_count = diff.authored.binding_ambiguities.len();
    let status = if object_ambiguity_count == 0 && binding_ambiguity_count == 0 {
        EguiAuthoredContinuityStatus::NoKnownAmbiguity
    } else {
        EguiAuthoredContinuityStatus::Ambiguous
    };

    EguiAuthoredContinuityAssessment {
        status,
        object_ambiguity_count,
        binding_ambiguity_count,
    }
}

/// Check whether one authored binding contains mutually incompatible final
/// resolution testimony.
///
/// This is deliberately a contract check, not a heuristic. It does not compare
/// authored intent with pixels and does not invent semantic↔paint linkage.
#[must_use]
pub fn assess_authored_binding_consistency(
    binding: &EguiAuthoredPaintBinding,
) -> EguiAuthoredBindingConsistencyAssessment {
    let mut conflicts = Vec::new();

    if binding.verified_at_end_pass {
        if binding.kind.is_none() {
            conflicts.push(EguiAuthoredBindingConflictKind::VerifiedWithoutKind);
        }
    } else {
        if binding.kind.is_some() {
            conflicts.push(EguiAuthoredBindingConflictKind::UnverifiedWithKind);
        }
        if binding.bounds.is_some() {
            conflicts.push(EguiAuthoredBindingConflictKind::UnverifiedWithBounds);
        }
        if binding.clip_rect.is_some() {
            conflicts.push(EguiAuthoredBindingConflictKind::UnverifiedWithClipRect);
        }
    }

    let status = if conflicts.is_empty() {
        EguiAuthoredBindingConsistencyStatus::Consistent
    } else {
        EguiAuthoredBindingConsistencyStatus::Contradictory
    };

    EguiAuthoredBindingConsistencyAssessment { status, conflicts }
}

/// Interpret authored-binding visibility without collapsing missing,
/// contradictory, or geometry-incomplete testimony into a false zero.
///
/// `EguiAuthoredPaintBinding::visible_fraction()` mechanically returns `0.0`
/// when usable bounds are unavailable. That remains useful as a geometry helper,
/// but an epistemic consumer must distinguish an observed zero from absence of
/// evidence. Contradictory resolver state also blocks a visibility claim.
#[must_use]
pub fn assess_authored_binding_visibility(
    binding: &EguiAuthoredPaintBinding,
) -> EguiAuthoredBindingVisibilityAssessment {
    if assess_authored_binding_consistency(binding).contradictory() {
        return EguiAuthoredBindingVisibilityAssessment {
            resolution: EguiAuthoredBindingResolutionStatus::Contradictory,
            visible_fraction: None,
        };
    }

    if !binding.verified_at_end_pass {
        return EguiAuthoredBindingVisibilityAssessment {
            resolution: EguiAuthoredBindingResolutionStatus::Unverified,
            visible_fraction: None,
        };
    }

    EguiAuthoredBindingVisibilityAssessment {
        resolution: EguiAuthoredBindingResolutionStatus::Verified,
        visible_fraction: binding.bounds.map(|_| binding.visible_fraction()),
    }
}
