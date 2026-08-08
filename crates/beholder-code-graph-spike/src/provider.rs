// ---
// relationships:
//   implements: compiler-backed-code-graph
// ---

//! Provider boundary consumed without compiler-specific values.

use crate::model::CodeGraph;

pub trait Provider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn produce(&self, input: &[u8]) -> Result<CodeGraph, Self::Error>;
}
