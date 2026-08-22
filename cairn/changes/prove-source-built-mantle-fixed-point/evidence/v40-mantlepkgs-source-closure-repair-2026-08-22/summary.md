# V40 Mantlepkgs staged-source closure repair

## Question

Did the fresh V37 proof pass StageX and provider construction, and what stopped stage1?

## Inspected evidence

Remote pueue task `279` ran from `2026-08-21T23:55:30-04:00` through `2026-08-22T15:07:34-04:00`.

It completed and retained:

- the full StageX transition with no fallback;
- the normalized native provider at BLAKE3 `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`;
- the full Rust 1.90.0 through 1.94.0 source-built provider;
- the 17-member source-built toolchain closure; and
- 736 successful stage1 units before the root Mantle binary.

Stage1 then failed at unit 737. Rustc reported:

```text
error: couldn't read `src/../mantlepkgs/contracts.ncl`: No such file or directory
  --> src/mantlepkgs_version_cmd.rs:73:41
```

`src/mantlepkgs_version_cmd.rs` uses `include_str!("../mantlepkgs/contracts.ncl")`. `STAGED_SOURCE_TOP_LEVEL_ENTRIES` did not contain `mantlepkgs`, so the authenticated Mantle source record omitted that compile-time input.

## Decision

Add `mantlepkgs` to the staged-source allowlist. Keep the existing root-anchored transfer exclusions and vendor checks unchanged.

The completed provider stages are eligible to seed the new promoted checkpoint store after current source/profile validation. Do not rebuild those stages.

## Owner

The self-build staged-source boundary owns the missing top-level compile-time input. The promoted checkpoint shell owns provider reuse.

## Next action

Refresh the source profile with the repaired source, import task `279` into the checkpoint store, prune only the copied Rust scratch, and rerun from stage1.

## Non-claims

Task `279` did not produce a fixed point or final v2 receipt. Provider completion does not prove the Mantle stage1 build.
