//! Verified arithmetic operations.
//!
//! Each function has a `spec fn` (mathematical definition) and an `exec fn`
//! whose `ensures` clause ties it to the spec. Verus mechanically checks
//! that the implementation matches — drift is impossible.

use vstd::prelude::*;

verus! {

// --- abs_diff ---

// r[depends arithmetic.abs-diff]
/// Spec: absolute difference of two u64 values.
pub open spec fn abs_diff_spec(a: u64, b: u64) -> u64 {
    if a >= b { (a - b) as u64 } else { (b - a) as u64 }
}

// r[impl arithmetic.abs-diff]
/// Returns the absolute difference between two `u64` values.
pub fn abs_diff(a: u64, b: u64) -> (result: u64)
    ensures result == abs_diff_spec(a, b)
{
    if a >= b { a - b } else { b - a }
}

// r[verify arithmetic.abs-diff]
proof fn abs_diff_symmetric(a: u64, b: u64)
    ensures abs_diff_spec(a, b) == abs_diff_spec(b, a)
{}

// r[verify arithmetic.abs-diff]
proof fn abs_diff_self_is_zero(x: u64)
    ensures abs_diff_spec(x, x) == 0u64
{}

// --- saturating_add ---

// r[depends arithmetic.saturating-add]
/// Spec: saturating addition.
pub open spec fn saturating_add_spec(a: u64, b: u64) -> u64 {
    if a as int + b as int > u64::MAX as int {
        u64::MAX
    } else {
        (a + b) as u64
    }
}

// r[impl arithmetic.saturating-add]
/// Saturating addition: returns `a + b` clamped to `u64::MAX`.
pub fn saturating_add(a: u64, b: u64) -> (result: u64)
    ensures result == saturating_add_spec(a, b)
{
    match a.checked_add(b) {
        Some(sum) => sum,
        None => u64::MAX,
    }
}

// r[verify arithmetic.saturating-add]
proof fn saturating_add_bounded(a: u64, b: u64)
    ensures saturating_add_spec(a, b) <= u64::MAX
{}

// r[verify arithmetic.saturating-add]
proof fn saturating_add_commutative(a: u64, b: u64)
    ensures saturating_add_spec(a, b) == saturating_add_spec(b, a)
{}

// --- saturating_sub ---

// r[depends arithmetic.saturating-sub]
/// Spec: saturating subtraction.
pub open spec fn saturating_sub_spec(a: u64, b: u64) -> u64 {
    if b > a { 0 } else { (a - b) as u64 }
}

// r[impl arithmetic.saturating-sub]
/// Saturating subtraction: returns `a - b` clamped to `0`.
pub fn saturating_sub(a: u64, b: u64) -> (result: u64)
    ensures result == saturating_sub_spec(a, b)
{
    if b > a { 0 } else { a - b }
}

// r[verify arithmetic.saturating-sub]
proof fn saturating_sub_exact(a: u64, b: u64)
    requires a >= b
    ensures saturating_sub_spec(a, b) == (a - b) as u64
{}

// --- checked_mul ---

// r[depends arithmetic.checked-mul]
/// Spec: checked multiplication.
pub open spec fn checked_mul_spec(a: u64, b: u64) -> Option<u64> {
    if a as int * b as int > u64::MAX as int {
        None
    } else {
        Some((a * b) as u64)
    }
}

// r[impl arithmetic.checked-mul]
/// Checked multiplication: returns `None` on overflow.
pub fn checked_mul(a: u64, b: u64) -> (result: Option<u64>)
    ensures result == checked_mul_spec(a, b)
{
    a.checked_mul(b)
}

// r[verify arithmetic.checked-mul]
proof fn checked_mul_zero(a: u64)
    ensures
        checked_mul_spec(a, 0) == Some(0u64),
        checked_mul_spec(0, a) == Some(0u64),
{}

// --- gcd ---

// r[depends arithmetic.gcd]
/// Spec: greatest common divisor via Euclidean algorithm.
pub open spec fn gcd_spec(a: u64, b: u64) -> u64
    decreases b
{
    if b == 0 { a } else { gcd_spec(b, (a % b) as u64) }
}

// r[impl arithmetic.gcd]
/// Computes the greatest common divisor of `a` and `b`.
///
/// Uses the Euclidean algorithm. Terminates because `b` strictly
/// decreases each iteration (`a % b < b`).
pub fn gcd(a: u64, b: u64) -> (result: u64)
    ensures result == gcd_spec(a, b)
    decreases b
{
    if b == 0 { a } else { gcd(b, a % b) }
}

// r[verify arithmetic.gcd]
/// Proof: gcd(a, 0) == a.
proof fn gcd_zero_left(a: u64)
    ensures gcd_spec(a, 0) == a
{}

// r[verify arithmetic.gcd]
/// Proof: gcd(0, a) == a.
proof fn gcd_zero_right(a: u64)
    ensures gcd_spec(0, a) == a
    decreases a
{
    if a != 0 {
        assert(0u64 % a == 0u64);
        gcd_zero_left(a);
    }
}

// --- div_ceil ---

// r[depends arithmetic.div-ceil]
/// Spec: ceiling division.
pub open spec fn div_ceil_spec(a: u64, b: u64) -> u64
    recommends b > 0
{
    if a == 0 { 0 } else { (((a - 1) as int / b as int) + 1) as u64 }
}

// r[impl arithmetic.div-ceil]
/// Ceiling division: returns the smallest integer >= a/b.
///
/// Uses `(a - 1) / b + 1` to avoid overflow from `(a + b - 1) / b`.
pub fn div_ceil(a: u64, b: u64) -> (result: u64)
    requires b > 0
    ensures result == div_ceil_spec(a, b)
{
    if a == 0 { 0 } else { (a - 1) / b + 1 }
}

// r[verify arithmetic.div-ceil]
/// Proof: div_ceil(0, b) == 0 for all nonzero b.
proof fn div_ceil_zero()
    ensures forall|b: u64| b > 0 ==> div_ceil_spec(0, b) == 0u64
{}

} // verus!

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    // r[verify arithmetic.abs-diff]
    #[test]
    fn test_abs_diff() {
        assert_eq!(abs_diff(10, 3), 7);
        assert_eq!(abs_diff(3, 10), 7);
        assert_eq!(abs_diff(5, 5), 0);
        assert_eq!(abs_diff(0, u64::MAX), u64::MAX);
    }

    // r[verify arithmetic.saturating-add]
    #[test]
    fn test_saturating_add() {
        assert_eq!(saturating_add(1, 2), 3);
        assert_eq!(saturating_add(u64::MAX, 1), u64::MAX);
        assert_eq!(saturating_add(0, 0), 0);
    }

    // r[verify arithmetic.saturating-sub]
    #[test]
    fn test_saturating_sub() {
        assert_eq!(saturating_sub(10, 3), 7);
        assert_eq!(saturating_sub(3, 10), 0);
        assert_eq!(saturating_sub(0, u64::MAX), 0);
    }

    // r[verify arithmetic.checked-mul]
    #[test]
    fn test_checked_mul() {
        assert_eq!(checked_mul(3, 4), Some(12));
        assert_eq!(checked_mul(u64::MAX, 2), None);
        assert_eq!(checked_mul(0, u64::MAX), Some(0));
    }

    proptest! {
        // r[verify arithmetic.abs-diff]
        #[test]
        fn prop_abs_diff_symmetric(a: u64, b: u64) {
            prop_assert_eq!(abs_diff(a, b), abs_diff(b, a));
        }

        // r[verify arithmetic.saturating-add]
        #[test]
        fn prop_saturating_add_commutative(a: u64, b: u64) {
            prop_assert_eq!(saturating_add(a, b), saturating_add(b, a));
        }

        // r[verify arithmetic.saturating-sub]
        #[test]
        fn prop_saturating_sub_self_is_zero(a: u64) {
            prop_assert_eq!(saturating_sub(a, a), 0);
        }

        // r[verify arithmetic.checked-mul]
        #[test]
        fn prop_checked_mul_zero(a: u64) {
            prop_assert_eq!(checked_mul(a, 0), Some(0));
            prop_assert_eq!(checked_mul(0, a), Some(0));
        }

        // r[verify arithmetic.gcd]
        #[test]
        fn prop_gcd_commutative(a: u64, b: u64) {
            // GCD is commutative — verify empirically
            prop_assert_eq!(gcd(a, b), gcd(b, a));
        }

        // r[verify arithmetic.gcd]
        #[test]
        fn prop_gcd_divides(a in 1u64..=1_000_000, b in 1u64..=1_000_000) {
            let g = gcd(a, b);
            if g > 0 {
                prop_assert_eq!(a % g, 0);
                prop_assert_eq!(b % g, 0);
            }
        }

        // r[verify arithmetic.div-ceil]
        #[test]
        fn prop_div_ceil_ge_floor(a: u64, b in 1u64..=u64::MAX) {
            prop_assert!(div_ceil(a, b) >= a / b);
        }

        // r[verify arithmetic.div-ceil]
        #[test]
        fn prop_div_ceil_exact_when_divisible(
            b in 1u64..=1_000_000,
            factor in 0u64..=1_000_000,
        ) {
            let a = b.saturating_mul(factor);
            if a / b == factor {
                // a is exactly divisible by b
                prop_assert_eq!(div_ceil(a, b), factor);
            }
        }
    }

    // r[verify arithmetic.gcd]
    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(8, 12), 4);
        assert_eq!(gcd(7, 0), 7);
        assert_eq!(gcd(0, 7), 7);
        assert_eq!(gcd(0, 0), 0);
        assert_eq!(gcd(17, 13), 1);
        assert_eq!(gcd(100, 75), 25);
    }

    // r[verify arithmetic.div-ceil]
    #[test]
    fn test_div_ceil() {
        assert_eq!(div_ceil(0, 3), 0);
        assert_eq!(div_ceil(1, 3), 1);
        assert_eq!(div_ceil(3, 3), 1);
        assert_eq!(div_ceil(4, 3), 2);
        assert_eq!(div_ceil(6, 3), 2);
        assert_eq!(div_ceil(7, 3), 3);
        assert_eq!(div_ceil(u64::MAX, 1), u64::MAX);
    }
}
