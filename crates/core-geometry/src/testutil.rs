//! Test-only deterministic helpers: property-test PRNG and serde round-trip.
//!
//! Per the frozen v1.1 architecture §11 ("Reproducibility"): no wall-clock,
//! no `HashMap` iteration, no uncontrolled randomness. The property-test PRNG
//! uses a FIXED seed and a deterministic stream (splitmix64, public domain).
//!
//! `serde_json` is a `[dev-dependencies]`-only crate — it never ships in the
//! geometry runtime, so the "no file-format dependency" invariant
//! (WO-002-AC05) is preserved. It exists solely to exercise every primitive's
//! `Serialize`+`Deserialize` impls through a standard JSON round-trip.
//!
//! This module is `#[cfg(test)]`-only.

#![cfg(test)]

use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

// ---------------------------------------------------------------------------
// splitmix64 PRNG (fixed seed, deterministic, reproducible)
// ---------------------------------------------------------------------------

/// Deterministic splitmix64-based pseudo-random generator.
///
/// Fixed seed: `0x0123_4567_89ab_cdef`. Reproducible across runs and across
/// platforms (no `StdRandom`, no wall-clock, no OS entropy).
#[derive(Debug, Clone)]
pub struct Prng {
    state: u64,
}

impl Prng {
    /// The fixed seed used by all property tests in this crate.
    pub const FIXED_SEED: u64 = 0x0123_4567_89ab_cdef_u64;

    /// Create a PRNG with the canonical fixed seed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: Self::FIXED_SEED,
        }
    }

    /// Create a PRNG with a caller-provided seed (still deterministic).
    #[must_use]
    pub const fn with_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Return the next raw `u64`.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        // splitmix64 (Sebastiano Vigna, public domain).
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_4276_d1ce_d664);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_1ebb);
        z ^ (z >> 31)
    }

    /// Return a finite `f64` in `[lo, hi)`. NaN/Inf are never produced.
    #[must_use]
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        let bits = self.next_u64() >> 11; // top 53 bits -> [0, 2^53)
        let unit = (bits as f64) / (1u64 << 53) as f64; // [0, 1)
        lo + unit * (hi - lo)
    }

    /// Convenience: a finite `f64` in `[-bound, bound]`.
    #[must_use]
    pub fn signed_f64(&mut self, bound: f64) -> f64 {
        self.range_f64(-bound, bound)
    }
}

impl Default for Prng {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// serde round-trip helper (serde_json; dev/test only)
// ---------------------------------------------------------------------------

/// Error returned by [`roundtrip`] when serialization or deserialization
/// fails.
#[derive(Debug)]
pub struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TestError {}

/// Round-trip `value` through a stable JSON representation: serialize to a
/// `String`, deserialize back, and return the result. The caller asserts
/// equality. Exercises every primitive's `Serialize`+`Deserialize` impls.
///
/// Evidence: WO-002-AC01 — serialization tests for geometry types.
///
/// `serde_json` is a `[dev-dependencies]`-only crate; it does not enter the
/// geometry runtime (WO-002-AC05 preserved).
pub fn roundtrip<T>(value: &T) -> Result<T, TestError>
where
    T: Serialize + DeserializeOwned,
{
    let json = serde_json::to_string(value).map_err(|e| TestError(e.to_string()))?;
    let back: T = serde_json::from_str(&json).map_err(|e| TestError(e.to_string()))?;
    Ok(back)
}
