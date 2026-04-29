//! Stateless validators for connections and block/schema consistency.

use loom_domain::{Block, BlockSchema, Port, PortDirection};

use crate::error::WeaveError;

/// Stateless structural validator.
pub struct Validator;

impl Validator {
    /// Validate that `output` and `input` can be connected.
    ///
    /// Checks:
    /// 1. `output` must have direction `Output`.
    /// 2. `input` must have direction `Input`.
    /// 3. The output type must be compatible with the input type.
    pub fn validate_connection(output: &Port, input: &Port) -> Result<(), WeaveError> {
        if output.direction != PortDirection::Output {
            return Err(WeaveError::IncompatiblePorts {
                from: output.name.clone(),
                to: input.name.clone(),
            });
        }
        if input.direction != PortDirection::Input {
            return Err(WeaveError::IncompatiblePorts {
                from: output.name.clone(),
                to: input.name.clone(),
            });
        }
        if !output.type_sig.is_compatible_with(&input.type_sig) {
            return Err(WeaveError::IncompatiblePorts {
                from: format!("{} ({:?})", output.name, output.type_sig),
                to: format!("{} ({:?})", input.name, input.type_sig),
            });
        }
        Ok(())
    }

    /// Validate that a block's atom list is non-empty and its schema is valid.
    pub fn validate_block(block: &Block, schema: &BlockSchema) -> Result<(), WeaveError> {
        if block.atoms.is_empty() {
            return Err(WeaveError::ValidationFailed(
                "block must contain at least one atom".to_string(),
            ));
        }
        schema
            .validate()
            .map_err(|e| WeaveError::ValidationFailed(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loom_domain::Port;
    use plexus_base::{PortId, TypeSig};

    fn in_port(id: u64, name: &str, ty: TypeSig) -> Port {
        Port::new_input(PortId::new(id), name.to_string(), ty)
    }

    fn out_port(id: u64, name: &str, ty: TypeSig) -> Port {
        Port::new_output(PortId::new(id), name.to_string(), ty)
    }

    #[test]
    fn compatible_any_types_pass() {
        let out = out_port(1, "o", TypeSig::Any);
        let inp = in_port(2, "i", TypeSig::Int);
        Validator::validate_connection(&out, &inp).unwrap();
    }

    #[test]
    fn matching_concrete_types_pass() {
        let out = out_port(1, "o", TypeSig::Int);
        let inp = in_port(2, "i", TypeSig::Int);
        Validator::validate_connection(&out, &inp).unwrap();
    }

    #[test]
    fn mismatched_types_fail() {
        let out = out_port(1, "o", TypeSig::Int);
        let inp = in_port(2, "i", TypeSig::Bool);
        let err = Validator::validate_connection(&out, &inp).unwrap_err();
        assert!(matches!(err, WeaveError::IncompatiblePorts { .. }));
    }

    #[test]
    fn wrong_directions_fail() {
        // Both inputs — output arg has wrong direction
        let out = in_port(1, "o", TypeSig::Any);
        let inp = in_port(2, "i", TypeSig::Any);
        let err = Validator::validate_connection(&out, &inp).unwrap_err();
        assert!(matches!(err, WeaveError::IncompatiblePorts { .. }));
    }
}
