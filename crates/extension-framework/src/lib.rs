//! Contracts and runtime support for statically composed knowledge-base extensions.
//!
//! Start with [`contracts`] to define an extension, use [`registry`] to select
//! the extensions enabled for a repository, and use [`bindings`] to supply the
//! repository-specific ontology identifiers they need.

#![forbid(unsafe_code)]

pub mod bindings;
pub mod contracts;
pub mod error;
pub mod registry;
