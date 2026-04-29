//! Serialisable block templates that can be converted to [`Archetype`]s.

use serde::{Deserialize, Serialize};

use loom_domain::{AtomKind, BlockSchema, Port, port::PortDirection};
use plexus_base::{BlockId, PortId, TypeSig};

use crate::archetype::Archetype;
use crate::error::WeaveError;

/// A serialisable description of a single port in a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    /// Port name.
    pub name: String,
    /// `"input"` or `"output"`.
    pub direction: String,
    /// Human-readable type signature string (e.g. `"Any"`, `"Int"`, `"Bool"`).
    pub type_sig: String,
}

impl PortDef {
    /// Parse direction + type_sig strings into a [`Port`].
    ///
    /// Returns `WeaveError::TemplateError` for unknown direction / type values.
    pub fn to_port(&self, id: PortId) -> Result<Port, WeaveError> {
        let type_sig = parse_type_sig(&self.type_sig)?;
        match self.direction.to_lowercase().as_str() {
            "input" => Ok(Port::new_input(id, self.name.clone(), type_sig)),
            "output" => Ok(Port::new_output(id, self.name.clone(), type_sig)),
            other => Err(WeaveError::TemplateError(format!(
                "unknown direction: '{other}'"
            ))),
        }
    }
}

/// A serialisable template that can be turned into an [`Archetype`] or used to
/// instantiate a [`loom_domain::Block`] directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplate {
    /// Archetype name.
    pub name: String,
    /// Version.
    pub version: u32,
    /// Port definitions.
    pub ports: Vec<PortDef>,
    /// Required atom kind names (e.g. `["Transform", "Source"]`).
    pub required_atoms: Vec<String>,
}

impl BlockTemplate {
    /// Convert the template to an [`Archetype`].
    ///
    /// Port ids are assigned sequentially starting from 1.
    pub fn to_archetype(&self) -> Result<Archetype, WeaveError> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for (i, def) in self.ports.iter().enumerate() {
            let port = def.to_port(PortId::new((i + 1) as u64))?;
            match port.direction {
                PortDirection::Input => inputs.push(port),
                PortDirection::Output => outputs.push(port),
            }
        }

        let schema = BlockSchema::new(inputs, outputs);
        schema
            .validate()
            .map_err(|e| WeaveError::ValidationFailed(e.to_string()))?;

        let required = self
            .required_atoms
            .iter()
            .map(|s| parse_atom_kind(s))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Archetype::new(
            self.name.clone(),
            self.version,
            schema,
            required,
        ))
    }

    /// Shorthand — convert to archetype and immediately instantiate a block.
    pub fn instantiate(&self, block_id: BlockId) -> Result<loom_domain::Block, WeaveError> {
        self.to_archetype()?.instantiate(block_id)
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn parse_type_sig(s: &str) -> Result<TypeSig, WeaveError> {
    match s.to_lowercase().as_str() {
        "any" => Ok(TypeSig::Any),
        "null" => Ok(TypeSig::Null),
        "bool" => Ok(TypeSig::Bool),
        "int" => Ok(TypeSig::Int),
        "float" => Ok(TypeSig::Float),
        "string" | "str" => Ok(TypeSig::String),
        "bytes" => Ok(TypeSig::Bytes),
        other => Err(WeaveError::TemplateError(format!(
            "unknown type sig: '{other}'"
        ))),
    }
}

fn parse_atom_kind(s: &str) -> Result<AtomKind, WeaveError> {
    match s.to_lowercase().as_str() {
        "source" => Ok(AtomKind::Source),
        "sink" => Ok(AtomKind::Sink),
        "transform" => Ok(AtomKind::Transform),
        "state" => Ok(AtomKind::State),
        "trigger" => Ok(AtomKind::Trigger),
        other => Err(WeaveError::TemplateError(format!(
            "unknown atom kind: '{other}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plexus_base::BlockId;

    fn basic_template() -> BlockTemplate {
        BlockTemplate {
            name: "my_block".to_string(),
            version: 1,
            ports: vec![
                PortDef {
                    name: "in".to_string(),
                    direction: "input".to_string(),
                    type_sig: "Any".to_string(),
                },
                PortDef {
                    name: "out".to_string(),
                    direction: "output".to_string(),
                    type_sig: "Int".to_string(),
                },
            ],
            required_atoms: vec!["Transform".to_string()],
        }
    }

    #[test]
    fn to_archetype_success() {
        let arch = basic_template().to_archetype().unwrap();
        assert_eq!(arch.id, "my_block");
        assert_eq!(arch.version, 1);
        assert_eq!(arch.required_atoms, vec![AtomKind::Transform]);
    }

    #[test]
    fn instantiate_success() {
        let block = basic_template().instantiate(BlockId::new(5)).unwrap();
        assert_eq!(block.id, BlockId::new(5));
    }

    #[test]
    fn unknown_direction_fails() {
        let mut t = basic_template();
        t.ports[0].direction = "sideways".to_string();
        let err = t.to_archetype().unwrap_err();
        assert!(matches!(err, WeaveError::TemplateError(_)));
    }

    #[test]
    fn unknown_type_sig_fails() {
        let mut t = basic_template();
        t.ports[0].type_sig = "Turbo".to_string();
        let err = t.to_archetype().unwrap_err();
        assert!(matches!(err, WeaveError::TemplateError(_)));
    }

    #[test]
    fn unknown_atom_kind_fails() {
        let mut t = basic_template();
        t.required_atoms = vec!["Gadget".to_string()];
        let err = t.to_archetype().unwrap_err();
        assert!(matches!(err, WeaveError::TemplateError(_)));
    }

    #[test]
    fn round_trip_serde() {
        let t = basic_template();
        let json = serde_json::to_string(&t).unwrap();
        let back: BlockTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, t.name);
        assert_eq!(back.ports.len(), t.ports.len());
    }
}
