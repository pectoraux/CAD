//! Stable error contract for geometry operations.
//!
//! Mirrors the canonical `GeometryInvalid` error variant required by
//! `spec/api.md` ("Error contract"). Variants are closed and stable within a
//! major contract version; no caller may invent a new error code.

use core::fmt;

/// Stable, closed error type returned by every canonical-model boundary in
/// `core-geometry`.
///
/// Variants:
/// - [`GeometryError::NonFinite`] — input contained NaN or ±infinity.
/// - [`GeometryError::Degenerate`] — input was finite but geometrically
///   degenerate (e.g. zero-length segment, zero-radius circle, coincident
///   control points, knot multiplicity > degree+1).
/// - [`GeometryError::InvalidInput`] — input violated a structural invariant
///   not covered by the two categories above (e.g. wrong knot count, mismatched
///   weights).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryError {
    /// A coordinate or scalar was NaN or ±infinity.
    NonFinite,
    /// Input was finite but geometrically degenerate (zero-length, zero-radius,
    /// coincident, parallel-but-equal, etc.). The `&'static str` is a stable,
    /// human-readable reason identifier (no caller-supplied text).
    Degenerate(&'static str),
    /// Input violated a structural invariant (wrong length, wrong shape, etc.).
    /// The `&'static str` is a stable, human-readable reason identifier.
    InvalidInput(&'static str),
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => write!(f, "geometry error: non-finite (NaN or infinity) input"),
            Self::Degenerate(reason) => {
                write!(f, "geometry error: degenerate input ({reason})")
            }
            Self::InvalidInput(reason) => {
                write!(f, "geometry error: invalid input ({reason})")
            }
        }
    }
}

impl core::error::Error for GeometryError {}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC03 — Degenerate inputs are explicitly handled (error variants).
    use super::GeometryError;

    #[test]
    fn error_variants_display_stably() {
        assert_eq!(
            GeometryError::NonFinite.to_string(),
            "geometry error: non-finite (NaN or infinity) input"
        );
        assert_eq!(
            GeometryError::Degenerate("zero-length segment").to_string(),
            "geometry error: degenerate input (zero-length segment)"
        );
        assert_eq!(
            GeometryError::InvalidInput("knot count mismatch").to_string(),
            "geometry error: invalid input (knot count mismatch)"
        );
    }

    #[test]
    fn error_is_clone_and_eq() {
        let a = GeometryError::Degenerate("zero-radius");
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, GeometryError::NonFinite);
    }
}
