// Planning-only fixture. No compiler or target execution claim.
@probe fn observe_exit(ctx: *pt_regs) -> i32 {
    return 0
}

fn main() -> i32 {
    return 0
}
