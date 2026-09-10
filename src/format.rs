use crate::{Witness, WitnessDiff};

/// Deserialize a ViewWitness YAML document.
pub fn from_yaml(input: &str) -> Result<Witness, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(input)
}

/// Serialize a witness as human-readable YAML.
pub fn to_yaml(witness: &Witness) -> Result<String, serde_yaml_ng::Error> {
    serde_yaml_ng::to_string(witness)
}

/// Deserialize a ViewWitness diff YAML document.
pub fn diff_from_yaml(input: &str) -> Result<WitnessDiff, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(input)
}

/// Serialize a witness diff as human-readable YAML.
pub fn diff_to_yaml(diff: &WitnessDiff) -> Result<String, serde_yaml_ng::Error> {
    serde_yaml_ng::to_string(diff)
}
