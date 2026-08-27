//! core-annotation — frozen architectural module boundary.
//!
//! Implementation is intentionally introduced through the corresponding
//! Work Order. Do not place domain logic here before its Work Order is active.

#![forbid(unsafe_code)]

/// Returns the module name for baseline architecture tests.
pub const MODULE_NAME: &str = "core-annotation";
