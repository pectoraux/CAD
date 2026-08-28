//! domain-electrical — frozen architectural module boundary.
//!
//! Implementation is intentionally introduced through the corresponding
//! Work Order. Do not place domain logic here before its Work Order is active.

#![forbid(unsafe_code)]

/// Returns the module name for baseline architecture tests.
pub const MODULE_NAME: &str = "domain-electrical";

#[cfg(test)]
mod tests {
    // Evidence: WO-001-AC02 — module boundary matches `spec/architecture.md` §2.
    // Evidence: WO-001-AC04 — deterministic baseline unit test harness.

    #[test]
    fn module_boundary_matches_spec() {
        assert_eq!(super::MODULE_NAME, "domain-electrical");
    }
}
