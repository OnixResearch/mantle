// Reviewed KernelScript v0.1.2 private/kfunc example.
include "xdp.kh"

@private
fn validate_input(value: u32) -> bool {
    return value > 0 && value < 1000
}

@kfunc
fn process_value(input: u32) -> u32 {
    if (!validate_input(input)) {
        return 0
    }
    return input * 2
}

@xdp
fn xdp_main(ctx: *xdp_md) -> xdp_action {
    var value: u32 = 42
    var result = process_value(value)

    if (result > 0) {
        return XDP_PASS
    }
    return XDP_DROP
}

fn main() -> i32 {
    var prog = load(xdp_main)
    attach(prog, "lo", 0)

    print("TCP monitor attached with private kfunc capabilities")
    print("Monitoring TCP connections on loopback...")
    detach(prog)
    print("TCP monitor detached")

    return 0
}
