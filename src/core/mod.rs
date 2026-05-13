//! Core domain types: scope, project, note creation.

pub mod note;
pub mod project;
pub mod scope;

pub use note::Note;
pub use project::Project;
pub use scope::Scope;
