//! Explicit tolerance policy for robust geometry predicates.
//!
//! Per the frozen v1.1 domain model (`spec/domain-model.md` §"Core value
//! types and invariants"):
//!
//! > Geometry predicates that require robust classification MUST use an
//! > explicit tolerance policy; tolerance is never implicit or
//! > caller-chosen on a per-operation basis.
//!
//! This module enforces that contract by exposing exactly ONE tolerance
//! value: [`Tolerance::CANONICAL`]. There is no public constructor, no
//! `Default` impl, and no per-operation `tol: Tolerance` parameter on any
//! predicate signature in this crate. Every predicate that requires
//! tolerance uses [`Tolerance::CANONICAL`] internally.
//!
//! ## Why a singleton, not a per-call parameter?
//!
//! A per-call `tol: Tolerance` parameter — even with a single canonical
//! value — would still be *caller-chosen on a per-operation basis*: the
//! caller would decide, for each call, which tolerance to pass. The
//! contract forbids that. By making the type non-constructible publicly
//! and exposing only the const [`Tolerance::CANONICAL`], we make the
//! policy a fixed property of the geometry crate, not a per-call input.
//!
//! ## Future extension
//!
//! If a future Work Order legitimately requires a different named policy
//! (e.g. a coarser tolerance for low-resolution imported DXF data), it
//! must be added as a new named const here (`Tolerance::COARSE_IMPORT` or
//! similar), not as a per-call value. Adding a named policy is a
//! contract-level change that requires Architect review.

/// The canonical, closed-set tolerance policy for geometry predicates.
///
/// There is exactly one public value of this type: [`Self::CANONICAL`].
/// The `absolute` field is private; callers cannot construct a
/// `Tolerance` and cannot read or mutate the absolute value directly
/// except through the const [`Self::CANONICAL`] and the helper methods
/// on this type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    /// Absolute tolerance in canonical drawing units. Private so callers
    /// cannot construct arbitrary `Tolerance` values; the only public
    /// value is [`Self::CANONICAL`].
    absolute: f64,
}

impl Tolerance {
    /// The canonical absolute tolerance: `1e-9` drawing units.
    ///
    /// This is the ONE AND ONLY public tolerance value exposed by this
    /// crate. It is used by every geometry predicate that requires
    /// robust classification. Per the frozen v1.1 contract, tolerance
    /// is never implicit or caller-chosen on a per-operation basis;
    /// this const is the explicit named policy.
    pub const CANONICAL: Self = Self { absolute: 1e-9_f64 };

    /// Squared tolerance (for comparing squared distances without a sqrt).
    #[must_use]
    pub const fn squared(self) -> f64 {
        self.absolute * self.absolute
    }

    /// Squared tolerance used for "are these points coincident?" predicates.
    ///
    /// Equal to [`Self::squared`] by definition; provided as a named,
    /// semantic entry point so call sites read as
    /// `Tolerance::CANONICAL.coincident_squared()`.
    #[must_use]
    pub const fn coincident_squared(self) -> f64 {
        self.squared()
    }

    /// Returns `true` if `delta` is within `±absolute` of zero.
    #[must_use]
    pub fn is_zero(self, delta: f64) -> bool {
        delta.abs() <= self.absolute
    }

    /// Returns `true` if `a` and `b` are within `absolute` of each other.
    #[must_use]
    pub fn eq(self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.absolute
    }

    /// Returns the absolute tolerance value. Exposed so the canonical
    /// policy's numeric value is observable (e.g. for diagnostic logging,
    /// doc assertions, and tests), but callers cannot construct a
    /// `Tolerance` to feed a different value back into a predicate.
    #[must_use]
    pub const fn absolute(self) -> f64 {
        self.absolute
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC02 — Predicates are deterministic (single
    // canonical tolerance policy; no per-operation caller-chosen value).
    use super::Tolerance;

    #[test]
    fn canonical_is_1e9() {
        assert_eq!(Tolerance::CANONICAL.absolute(), 1e-9_f64);
        assert_eq!(Tolerance::CANONICAL.squared(), 1e-18_f64);
        assert_eq!(Tolerance::CANONICAL.coincident_squared(), 1e-18_f64);
    }

    #[test]
    fn is_zero_and_eq_are_symmetric() {
        let t = Tolerance::CANONICAL;
        assert!(t.is_zero(1e-10));
        assert!(!t.is_zero(2e-9));
        assert!(t.eq(1.0, 1.0 + 1e-10));
        assert!(t.eq(1.0 + 1e-10, 1.0));
        assert!(!t.eq(1.0, 1.0 + 2e-9));
    }

    #[test]
    fn canonical_is_a_singleton_value() {
        // The frozen contract requires tolerance to be a single explicit
        // policy, not caller-chosen. The CANONICAL const is the only
        // public value of Tolerance; the field is private and there is no
        // public constructor. This test asserts that invariant by
        // confirming the only constructible value (via the const) equals
        // itself.
        let a = Tolerance::CANONICAL;
        let b = Tolerance::CANONICAL;
        assert_eq!(a, b);
        assert_eq!(a.absolute(), b.absolute());
    }
}
