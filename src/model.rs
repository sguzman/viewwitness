use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One observed GUI state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Witness {
    pub viewwitness_version: String,
    pub capture: Capture,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capture {
    /// Capture backend, initially expected to be `egui` or `synthetic`.
    pub source: String,
    #[serde(default)]
    pub frame: Option<u64>,
    pub viewport: Viewport,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f32,
}

const fn default_scale_factor() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Stable within the witness and preferably stable across related witnesses.
    pub id: String,
    /// Semantic role. The v0 vocabulary is intentionally open-ended.
    pub role: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub bounds: Option<Rect>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub focused: Option<bool>,
    #[serde(default)]
    pub selected: Option<bool>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A fact relating two nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    pub kind: String,
    pub from: String,
    pub to: String,
    /// `observed` for facts supplied directly by capture; `derived` for facts
    /// computed from other evidence such as geometry.
    pub evidence: String,
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
}

impl Witness {
    /// Lightweight structural validation for format fixtures and capture output.
    ///
    /// Semantic validation will grow with the ontology. Returning all issues at
    /// once makes this useful for example-corpus development.
    #[must_use]
    pub fn validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let mut ids = HashSet::new();
        let viewport = self.capture.viewport;

        if self.viewwitness_version.trim().is_empty() {
            issues.push("viewwitness_version must not be empty".into());
        }
        if self.capture.source.trim().is_empty() {
            issues.push("capture source must not be empty".into());
        }
        if !viewport.width.is_finite()
            || !viewport.height.is_finite()
            || !viewport.scale_factor.is_finite()
        {
            issues.push("viewport geometry must be finite".into());
        }
        if viewport.width < 0.0 || viewport.height < 0.0 {
            issues.push("viewport dimensions must be non-negative".into());
        }
        if viewport.scale_factor <= 0.0 {
            issues.push("viewport scale_factor must be positive".into());
        }

        for node in &self.nodes {
            if node.id.trim().is_empty() {
                issues.push("node id must not be empty".into());
            } else if !ids.insert(node.id.as_str()) {
                issues.push(format!("duplicate node id: {}", node.id));
            }

            if node.role.trim().is_empty() {
                issues.push(format!("node {} role must not be empty", node.id));
            }

            if let Some(bounds) = node.bounds {
                if !bounds.x.is_finite()
                    || !bounds.y.is_finite()
                    || !bounds.width.is_finite()
                    || !bounds.height.is_finite()
                {
                    issues.push(format!("node {} has non-finite bounds", node.id));
                }
                if bounds.width < 0.0 || bounds.height < 0.0 {
                    issues.push(format!("node {} has negative bounds", node.id));
                }
            }
        }

        for node in &self.nodes {
            if let Some(parent) = &node.parent
                && !ids.contains(parent.as_str())
            {
                issues.push(format!(
                    "node {} references missing parent {}",
                    node.id, parent
                ));
            }
        }

        for relation in &self.relations {
            if relation.kind.trim().is_empty() {
                issues.push("relation kind must not be empty".into());
            }
            if !ids.contains(relation.from.as_str()) {
                issues.push(format!(
                    "relation {} references missing source {}",
                    relation.kind, relation.from
                ));
            }
            if !ids.contains(relation.to.as_str()) {
                issues.push(format!(
                    "relation {} references missing target {}",
                    relation.kind, relation.to
                ));
            }
            if relation.evidence != "observed" && relation.evidence != "derived" {
                issues.push(format!(
                    "relation {} has unknown evidence kind {}",
                    relation.kind, relation.evidence
                ));
            }
        }

        issues
    }
}
