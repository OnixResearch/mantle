## Why

The native closure materializer now supports a musl-host Rust provider, but `bootstrap rust-source-provider` still derives its route plan implicitly from `bootstrap/rust-source-plan.ncl`, whose host triple is GNU. Operators need an explicit, validated musl-host route plan without replacing the existing GNU-host plan.

## What Changes

- Add an explicit route-plan selection path to Rust source provider materialization.
- Add a checked-in `bootstrap/rust-source-musl-host-plan.ncl` with `host_triple = target_triple = x86_64-unknown-linux-musl`.
- Validate both checked-in route plans in focused tests and keep the default GNU-host route unchanged.

## Impact

- **Files**: Rust source provider route loading/CLI plumbing, new Nickel route plan, Cairn spec/evidence.
- **Validation**: focused route-plan tests, CLI parse tests, formatting/whitespace checks, Cairn gates.
