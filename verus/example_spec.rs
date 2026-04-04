use vstd::prelude::*;

verus! {
    /// Example specification — replace with real invariants.
    pub open spec fn valid_state(x: int) -> bool {
        x >= 0
    }

    pub proof fn valid_state_is_non_negative(x: int)
        requires valid_state(x),
        ensures x >= 0,
    {
    }
}

fn main() {}
