//! Core domain types: scope, project, note creation, scope resolution.

pub mod note;
pub mod project;
pub mod resolve;
pub mod scope;

pub use note::Note;
pub use project::Project;
pub use scope::Scope;
