use crate::Witness;

/// Deserialize a ViewWitness YAML document.
pub fn from_yaml(input: &str) -> Result<Witness, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(input)
}

/// Serialize a witness as human-readable YAML.
pub fn to_yaml(witness: &Witness) -> Result<String, serde_yaml_ng::Error> {
    serde_yaml_ng::to_string(witness)
}
