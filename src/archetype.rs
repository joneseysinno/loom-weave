//! Named, versioned archetypes and a registry for instantiating blocks.

use std::collections::HashMap;

use frp_domain::{AtomKind, Block, BlockSchema, Meta};
use frp_plexus::BlockId;

use crate::error::WeaveError;

/// A named, versioned blueprint for a [`Block`].
#[derive(Debug, Clone)]
pub struct Archetype {
    /// Unique identifier for this archetype.
    pub id: String,
    /// Monotonically increasing version number.
    pub version: u32,
    /// The port schema that all instantiated blocks will share.
    pub schema: BlockSchema,
    /// The atom kinds that the archetype requires be present in any block
    /// built from it.  (Enforced at instantiation time if atoms are provided.)
    pub required_atoms: Vec<AtomKind>,
}

impl Archetype {
    /// Create a new archetype.
    pub fn new(
        id: impl Into<String>,
        version: u32,
        schema: BlockSchema,
        required_atoms: Vec<AtomKind>,
    ) -> Self {
        Archetype {
            id: id.into(),
            version,
            schema,
            required_atoms,
        }
    }

    /// Instantiate a bare block from this archetype (no atoms attached).
    ///
    /// Schema validation is performed; returns `WeaveError::ValidationFailed`
    /// if the schema is invalid.
    pub fn instantiate(&self, id: BlockId) -> Result<Block, WeaveError> {
        self.schema
            .validate()
            .map_err(|e| WeaveError::ValidationFailed(e.to_string()))?;

        Ok(Block {
            id,
            schema: self.schema.clone(),
            atoms: vec![],
            meta: Meta::default(),
        })
    }
}

/// A registry of named [`Archetype`]s.
#[derive(Debug, Default)]
pub struct ArchetypeRegistry {
    archetypes: HashMap<String, Archetype>,
}

impl ArchetypeRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        ArchetypeRegistry::default()
    }

    /// Register an archetype. Overwrites any existing entry with the same id.
    pub fn register(&mut self, archetype: Archetype) {
        self.archetypes.insert(archetype.id.clone(), archetype);
    }

    /// Retrieve an archetype by id.
    pub fn get(&self, id: &str) -> Option<&Archetype> {
        self.archetypes.get(id)
    }

    /// Instantiate a block from a named archetype.
    ///
    /// Returns `WeaveError::ArchetypeNotFound` if the id is not registered.
    pub fn instantiate(&self, archetype_id: &str, block_id: BlockId) -> Result<Block, WeaveError> {
        let arch = self
            .archetypes
            .get(archetype_id)
            .ok_or_else(|| WeaveError::ArchetypeNotFound(archetype_id.to_string()))?;
        arch.instantiate(block_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frp_domain::{BlockSchema, Port};
    use frp_plexus::{BlockId, PortId, TypeSig};

    fn simple_schema() -> BlockSchema {
        BlockSchema::new(
            vec![Port::new_input(PortId::new(1), "in".to_string(), TypeSig::Any)],
            vec![Port::new_output(PortId::new(2), "out".to_string(), TypeSig::Any)],
        )
    }

    #[test]
    fn archetype_instantiate_returns_block() {
        let arch = Archetype::new("test_arch", 1, simple_schema(), vec![AtomKind::Transform]);
        let block = arch.instantiate(BlockId::new(7)).unwrap();
        assert_eq!(block.id, BlockId::new(7));
        assert!(block.atoms.is_empty());
    }

    #[test]
    fn registry_get_and_instantiate() {
        let mut reg = ArchetypeRegistry::new();
        reg.register(Archetype::new("arch_a", 1, simple_schema(), vec![]));

        assert!(reg.get("arch_a").is_some());

        let block = reg.instantiate("arch_a", BlockId::new(99)).unwrap();
        assert_eq!(block.id, BlockId::new(99));
    }

    #[test]
    fn registry_not_found_returns_error() {
        let reg = ArchetypeRegistry::new();
        let err = reg.instantiate("missing", BlockId::new(1)).unwrap_err();
        assert!(matches!(err, WeaveError::ArchetypeNotFound(_)));
    }

    #[test]
    fn registry_overwrite_existing() {
        let mut reg = ArchetypeRegistry::new();
        reg.register(Archetype::new("arch_a", 1, simple_schema(), vec![]));
        reg.register(Archetype::new("arch_a", 2, simple_schema(), vec![]));
        assert_eq!(reg.get("arch_a").unwrap().version, 2);
    }
}
