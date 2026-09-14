use serde::{Deserialize, Serialize};

use crate::EguiCorrelatedDiff;

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

/// Assess only the authored-identity continuity evidence already present in a
/// correlated diff.
///
/// This deliberately does not synthesize a confidence score or promote
/// `NoKnownAmbiguity` into a claim of evidential completeness. Later Stage-G
/// pressure may add other orthogonal epistemic assessments without weakening
/// this narrow identity rule.
#[must_use]
pub fn assess_authored_continuity(
    diff: &EguiCorrelatedDiff,
) -> EguiAuthoredContinuityAssessment {
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
