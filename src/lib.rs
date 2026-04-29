//! Weave layer — stateless assembly and validation for the Loom graph domain.
//!
//! Provides:
//! - [`Composer`] — assembles [`loom_domain::Block`]s from atoms and a schema.
//! - [`Validator`] — validates connections and block/schema consistency.
//! - [`Archetype`] / [`ArchetypeRegistry`] — named, versioned block blueprints.
//! - [`BlockTemplate`] / [`PortDef`] — serialisable templates convertible to archetypes.
//! - [`WeaveError`] — unified error type for this crate.

pub mod archetype;
pub mod composer;
pub mod error;
pub mod template;
pub mod validator;

pub use archetype::{Archetype, ArchetypeRegistry};
pub use composer::Composer;
pub use error::WeaveError;
pub use template::{BlockTemplate, PortDef};
pub use validator::Validator;
