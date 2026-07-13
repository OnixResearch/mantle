// Reviewed KernelScript v0.1.2 probe example.
// eBPF probe functions return i32 due to the BPF_PROG() constraint.
@probe("do_exit")
fn do_exit(code: i64) -> i32 {
    print("Process exiting with code: %ld", code)
    return 0
}

fn main() -> i32 {
    var prog = load(do_exit)
    var result = attach(prog, "do_exit", 0)

    if (result == 0) {
        print("probe program attached to do_exit successfully")
        print("Monitoring process exits...")
        detach(prog)
        print("probe program detached")
    } else {
        print("Failed to attach probe program")
        return 1
    }

    return 0
}
