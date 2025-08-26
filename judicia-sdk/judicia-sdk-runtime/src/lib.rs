//! Runtime types and utilities for Judicia Plugin development

pub mod types;
pub mod platform;
pub mod error;
pub mod frontend;
pub mod utils;

// Public API exports
pub use types::*;
pub use platform::*;
pub use error::*;
pub use frontend::*;
pub use utils::*;

// Re-export common external types
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
pub use anyhow::Result;
pub use tracing::{debug, error, info, trace, warn};
pub use wasm_bindgen::prelude::*;