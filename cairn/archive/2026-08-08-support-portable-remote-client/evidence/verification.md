# Portable remote client verification

Task-ID: I1,I2,I3,I4,I5,I6,V1,V2,V3,V4,V5

Covers: realization_routing.portable_client_command_matrix, realization_routing.portable_client_core, realization_routing.non_linux_remote_route, realization_routing.portable_client_concrete_inputs, realization_routing.no_local_execution_on_portable_client, realization_routing.portable_output_materialization, realization_routing.portable_client_credentials, realization_routing.portable_client_validation

## Implementation evidence

- `config/operator-surfaces.ncl` owns 39 reviewed root-command platform profiles.
- `crates/mantle-portable-client-core` owns pure bounded command admission and
  frontend-neutral remote request planning.
- `src/main.rs` runs portable command admission before the state-directory
  override and selects the existing remote path before local pipeline entry.
- The remote request validator keeps the client and target platform labels
  separate. It rejects raw frontend payload kinds, missing capability identity,
  missing trusted signer facts, invalid prefixes, and unbounded payload lists.
- The ordinary remote admission path still imports signed `PathInfo` state and
  materializes accepted output into the explicit physical store directory.
- `scripts/check-portable-client-boundary.rs` rejects Linux execution
  dependencies and side-effect APIs in the portable core.
- `MANTLE_TEST_LOCAL_ROUTE_SENTINEL` instruments the two client local-route
  entry points in debug builds. The production remote transfer test proves the
  sentinel stays absent after both rejected remote preflight and remote success.

## Focused validation

### Pure core and malformed request cases

Command:

```text
nix develop -c cargo test -p mantle-portable-client-core
```

Result:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The suite includes Darwin remote admission, Darwin local-route rejection,
client/target separation, custom store prefixes, raw frontend rejection, and
missing capability/trust rejection.

### Route policy

Command:

```text
nix develop -c cargo test -p mantle --bin mantle realization_routing:: -- --nocapture
```

Result:

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 2300 filtered out
```

### Remote transfer, materialization, and no-local-execution sentinel

Command:

```text
nix develop -c cargo test -p mantle --test remote_transfer_production production_stdio_resumes_missing_chunks_and_imports_output -- --nocapture
```

Result:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out
```

The test verifies resumed bounded transfer, admitted state, materialized output
bytes, and an untouched client local-route sentinel.

Negative command:

```text
nix develop -c cargo test -p mantle --test remote_transfer_production production_stdio_rejects_ticket_upload_quota_before_checkpoint_or_admission -- --nocapture
```

Result:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out
```

### Output trust

Command:

```text
nix develop -c cargo test -p mantle --bin mantle remote_output_admission_rejects -- --nocapture
```

Result:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2321 filtered out
```

These tests retain untrusted-key and malformed-digest rejection.

### Darwin cross-target checks

Commands:

```text
nix develop -c cargo check -p mantle-portable-client-core --target aarch64-apple-darwin
nix develop -c cargo check -p mantle-portable-client-core --target x86_64-apple-darwin
```

Both commands completed successfully. The checked dependency tree contains only
the portable core, `serde`, and proc-macro support.

Native Darwin tests were not available in this Linux session. Full root-package
cross checks were also attempted. They reached C dependencies, then failed
because the Linux dev shell selected its musl compiler and had no Darwin SDK.
Those failures are toolchain evidence only. They are not a Mantle source error,
and they do not establish native Darwin support.

### Boundary and policy checks

Commands:

```text
nix develop -c cargo -q -Zscript scripts/check-portable-client-boundary.rs .
nix develop -c ./scripts/check-operator-command-contract.sh
nix develop -c nickel typecheck fixtures/portable-client/platform-matrix.ncl
nix develop -c nickel typecheck fixtures/portable-client/requests.ncl
```

Results:

```text
portable client boundary: PASS
operator command contract generator self-test: PASS
operator command contract: PASS (commands=169)
```

Both Nickel fixtures typechecked successfully.

### Product-owned Clippy and formatting

Command:

```text
nix develop -c cargo clippy -p mantle-portable-client-core -p mantle --lib --bins --tests --no-deps -- -D warnings
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo]
```

Direct Rustfmt checks passed for the portable core, generator, root shell,
operator contract, and changed integration test. `git diff --check` also passed.

## Cairn validation and gates

Commands:

```text
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- gate proposal support-portable-remote-client --root .
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- gate design support-portable-remote-client --root .
nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- gate tasks support-portable-remote-client --root .
```

Exact verdict fields:

```text
validate: valid=true findings=[] issues=[]
proposal: valid=true verdict=PASS receipt_hash=0b8d200c7a55262eff359887c14eb129826cf35f86585d30a5ad2f1b8a6563c4
design: valid=true verdict=PASS receipt_hash=b39430e005f87f599f9c6176b65319bea4b55afcaf773246dd35f32915b2d079
tasks: valid=true verdict=PASS task_done=11 receipt_hash=a20630fed764096c440c346d0ca0324615b734d82ebedc67e97e74c44420e4d7
```

## Non-claims

This evidence does not claim native Darwin execution, Linux worker portability,
bootstrap portability, proof portability, full-source completion, fixed-point
success, or whole-repository release readiness.
