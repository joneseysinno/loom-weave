//! Stateless composer that assembles [`Block`]s from [`Atom`]s and a schema.

use std::collections::HashSet;

use loom_domain::{Atom, Block, BlockSchema};
use plexus_core::BlockId;

use crate::error::WeaveError;

/// Assembles a [`Block`] from a list of atoms, a schema, and a target block id.
///
/// The composer validates that:
/// - Every atom id is unique (no duplicates).
/// - All schema port names are unique across inputs and outputs.
///
/// Domain-level schema validation (port direction checks) is delegated to
/// [`BlockSchema::validate`] which is called internally.
pub struct Composer;

impl Composer {
    /// Compose a new block.
    ///
    /// Returns `WeaveError::ValidationFailed` if atom ids are duplicated or
    /// schema ports have duplicate names.
    pub fn compose(
        atoms: Vec<Atom>,
        schema: BlockSchema,
        id: BlockId,
    ) -> Result<Block, WeaveError> {
        // Validate schema (direction correctness + duplicate port names).
        schema
            .validate()
            .map_err(|e| WeaveError::ValidationFailed(e.to_string()))?;

        // Check for duplicate atom ids.
        let mut seen: HashSet<plexus_core::AtomId> = HashSet::new();
        for atom in &atoms {
            if !seen.insert(atom.id) {
                return Err(WeaveError::ValidationFailed(format!(
                    "duplicate atom id: {:?}",
                    atom.id
                )));
            }
        }

        let atom_ids: Vec<_> = atoms.iter().map(|a| a.id).collect();
        let meta = loom_domain::Meta::default();

        Ok(Block {
            id,
            schema,
            atoms: atom_ids,
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loom_domain::{
        AtomKind, AtomMeta, BlockSchema, Port,
        port::PortDirection as PD,
    };
    use plexus_core::{AtomId, BlockId, LayerTag, PortId, TypeSig};

    fn make_port(id: u64, name: &str, dir: PD) -> Port {
        match dir {
            PD::Input => Port::new_input(PortId::new(id), name.to_string(), TypeSig::Any),
            PD::Output => Port::new_output(PortId::new(id), name.to_string(), TypeSig::Any),
        }
    }

    fn make_atom(id: u64) -> Atom {
        Atom::new(
            AtomId::new(id),
            AtomKind::Transform,
            AtomMeta::new("test".to_string(), LayerTag::Core),
        )
    }

    #[test]
    fn compose_success() {
        let schema = BlockSchema::new(
            vec![make_port(1, "in", PD::Input)],
            vec![make_port(2, "out", PD::Output)],
        );
        let atoms = vec![make_atom(10), make_atom(20)];
        let block = Composer::compose(atoms, schema, BlockId::new(99)).unwrap();
        assert_eq!(block.id, BlockId::new(99));
        assert_eq!(block.atoms.len(), 2);
    }

    #[test]
    fn compose_duplicate_atom_ids_fails() {
        let schema = BlockSchema::new(
            vec![make_port(1, "in", PD::Input)],
            vec![make_port(2, "out", PD::Output)],
        );
        let atoms = vec![make_atom(10), make_atom(10)];
        let err = Composer::compose(atoms, schema, BlockId::new(1)).unwrap_err();
        assert!(matches!(err, WeaveError::ValidationFailed(_)));
    }

    #[test]
    fn compose_schema_duplicate_port_names_fails() {
        let schema = BlockSchema::new(
            vec![
                make_port(1, "dup", PD::Input),
                make_port(2, "dup", PD::Input),
            ],
            vec![],
        );
        let err = Composer::compose(vec![], schema, BlockId::new(1)).unwrap_err();
        assert!(matches!(err, WeaveError::ValidationFailed(_)));
    }
}
