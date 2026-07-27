//! One source of randomness for the whole program.
//!
//! Flourish needs unpredictability in two places — the per-performance shader
//! seed, and choosing a flourish at random — and neither needs a random number
//! generator crate. Both need the same thing: a `u32` that differs from the
//! last one, including when the two calls are microseconds apart.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// A fresh value, different from the one before it.
///
/// Falls back to a fixed constant if the clock is unavailable. A repeated
/// flourish is a far smaller problem than a panic mid-presentation.
#[must_use]
pub fn fresh_u32() -> u32 {
    // The wall clock alone is not enough: its granularity is coarser than the
    // gap between two quick calls, so back-to-back values can land in the same
    // tick and repeat. A counter guarantees they always differ, while the clock
    // keeps separate launches from sharing a sequence.
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9e37_79b9, |elapsed| elapsed.subsec_nanos());
    let counted = COUNTER
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_mul(0x9e37_79b9);
    // Avalanche the low-entropy inputs so nearby calls do not produce visually
    // similar output.
    let mut mixed = nanos ^ counted ^ 0x2545_f491;
    mixed ^= mixed >> 16;
    mixed = mixed.wrapping_mul(0x7feb_352d);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x846c_a68b);
    mixed ^ (mixed >> 16)
}

#[cfg(test)]
mod tests {
    use super::fresh_u32;
    use std::collections::HashSet;

    #[test]
    fn consecutive_calls_differ() {
        // The failure this guards is the reason the counter exists: two calls
        // inside one clock tick used to return the same value, which would show
        // up as the same "random" flourish twice in a row.
        let values: HashSet<u32> = (0..1_000).map(|_| fresh_u32()).collect();
        assert_eq!(values.len(), 1_000, "fresh_u32 repeated within 1000 calls");
    }

    #[test]
    fn values_spread_across_the_range() {
        // A mixer that avalanches badly would still pass the test above while
        // clustering every value in one corner of the range, which would bias
        // any modulo drawn from it.
        let mut low = 0;
        let mut high = 0;
        for _ in 0..1_000 {
            if fresh_u32() < u32::MAX / 2 {
                low += 1;
            } else {
                high += 1;
            }
        }
        assert!(
            low > 300 && high > 300,
            "lopsided split: {low} low, {high} high"
        );
    }
}
