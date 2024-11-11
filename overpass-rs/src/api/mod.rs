// src/api/mod.rs

// src/api/mod.rs
pub mod api_epoch;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod stats;
pub mod validation;

// Re-export for convenience

pub use routes::*;
pub use stats::*;
pub use validation::*;
