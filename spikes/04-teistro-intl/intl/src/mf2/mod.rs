//! The `MessageFormat 2` subset: the data model, the parser with the
//! data-model checks, and serialisation back to source. Evaluation lives
//! in [`crate::render`], which binds the SDK's functions.

pub mod ast;
pub mod check;
pub mod display;
pub mod parser;

#[cfg(test)]
mod tests;

pub use ast::Message;
pub use parser::{ParseError, parse};
