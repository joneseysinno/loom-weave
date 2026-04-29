//! Error types for the weave layer.

use plexus_core::{AtomId, BlockId, EdgeId, PortId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WeaveError {
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("missing atom: {0:?}")]
    MissingAtom(AtomId),

    #[error("incompatible ports: '{from}' -> '{to}'")]
    IncompatiblePorts { from: String, to: String },

    #[error("archetype not found: {0}")]
    ArchetypeNotFound(String),

    #[error("block not found: {0:?}")]
    BlockNotFound(BlockId),

    #[error("edge not found: {0:?}")]
    EdgeNotFound(EdgeId),

    #[error("port not found: {0:?}")]
    PortNotFound(PortId),

    #[error("template error: {0}")]
    TemplateError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_validation_failed() {
        let e = WeaveError::ValidationFailed("bad schema".to_string());
        assert_eq!(e.to_string(), "validation failed: bad schema");
    }

    #[test]
    fn display_missing_atom() {
        let e = WeaveError::MissingAtom(AtomId::new(42));
        assert!(e.to_string().contains("42"));
    }

    #[test]
    fn display_incompatible_ports() {
        let e = WeaveError::IncompatiblePorts {
            from: "output_a".to_string(),
            to: "input_b".to_string(),
        };
        assert_eq!(e.to_string(), "incompatible ports: 'output_a' -> 'input_b'");
    }

    #[test]
    fn display_archetype_not_found() {
        let e = WeaveError::ArchetypeNotFound("my_arch".to_string());
        assert_eq!(e.to_string(), "archetype not found: my_arch");
    }
}
