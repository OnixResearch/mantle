# Authority caveat baseline and isolated core evidence (2026-09-30)

## Isolation and scope

`origin/main` at `da00f58425740adea559ef926c9dfa97b2cb8240` was checked out detached at `/home/brittonr/scratch/mantle-pattern-caveats-baseline`. This checkout is distinct from the active Mantle worktree. All commands below used `TMPDIR=/home/brittonr/scratch`; isolated Cargo targets are named in each command. No existing ticket, store view, project selector, or production CLI has been changed by the scratch core tests. The core is not a new cryptographic token or a claim of working grant transport.

### Existing receiver paths observed before edits

- Remote: `src/remote_build.rs::commit_remote_ticket_admission_for_state` acquires a mutation guard, reloads ticket state, obtains receiver clock, authenticates the ticket, validates the concrete request, then redeems and persists state before any success frames. `src/remote_credentials.rs::apply_ticket_policy` rejects revoked, expired, exhausted, clock-before-issuance, and endpoint-mismatched tickets. There is no holder-supplied job-set caveat in these paths.
- Store: `crates/crunch-store/src/capability.rs::OutputLookup::{find,find_with_layer,find_remote}` reads by complete store-path identity but has no narrowed logical-path pattern view yet. A restricted wrapper must reject before these methods reach PathInfo service reads.
- Project: `src/project_build.rs::resolve_project_target` resolves a `.#selector` relative to the nearest discovered project file and `render_attribute_target` generates Nickel extraction. `parse_selector` rejects empty selector segments but the baseline has no transferred grant to rename a goal into a project namespace or authorize one project's goal against another project's capability.

### Observed baseline command

From the detached baseline checkout:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-pattern-caveats-baseline-target nix develop --offline --no-write-lock-file -c cargo test -p mantle --bin mantle remote_build::tests::malformed_request_does_not_consume_ticket -- --exact --nocapture
running 1 test
test remote_build::tests::malformed_request_does_not_consume_ticket ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2593 filtered out; finished in 0.00s
```

This observes an existing negative remote-ticket admission path. It does not test a new holder grant. The baseline store read also passed:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-pattern-caveats-baseline-target nix develop --offline --no-write-lock-file -c cargo test -p crunch-store --lib capability::tests::output_lookup_and_selected_root_registration_share_exact_identity -- --exact --nocapture
running 1 test
test capability::tests::output_lookup_and_selected_root_registration_share_exact_identity ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 395 filtered out; finished in 0.00s
```

This observes an existing exact-path read and root registration, not pattern-based view restriction. The baseline project parser restriction passed:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-pattern-caveats-project-target nix develop --offline --no-write-lock-file -c cargo test -p mantle --bin mantle project_build::tests::parse_selector_rejects_double_dot -- --exact --nocapture
running 1 test
test project_build::tests::parse_selector_rejects_double_dot ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2593 filtered out; finished in 0.00s
```

This rejects malformed selector segments before project lowering. It does **not** demonstrate a cross-project grant or namespace, absent in the original baseline.

### Stack decision and reviewed policy

`adr/0090-carry-pattern-caveats-in-ucan-and-enforce-at-mantle-receivers.md` records UCAN signed proof chains and opaque preserved caveat payloads plus Basalt reviewed exact policy as the stack boundary. Neither implements Mantle's pattern rewrites for us. Mantle's pure filter is an application rule, not independent crypto. `config/authority-caveats/default.ncl` was exported through `nickel export --format json` in the pinned Nix development shell; the observed JSON contained accepted variants `rewrite`, `reject`, `alternatives`, `unknown-deny-all`, bounds `max_chain=8`, `max_alternatives=8`, `max_pattern_bytes=256`, `max_caveat_bytes=8192`, `max_value_bytes=512`, `max_segments=16`, newest-first/right-to-left, silent discard, original and parent-prefix rechecks, and the three explicit non-claims.

### Isolated filter-core proof

The scratch-only `crates/crunch-authority-core` was compiled independently of the Mantle workspace. After adding ordinal-record checks, this scoped command returned:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-caveat-core-target nix develop --offline --no-write-lock-file -c cargo test --manifest-path /home/brittonr/scratch/mantle-pattern-caveats-baseline/crates/crunch-authority-core/Cargo.toml --lib
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

These tests exercise bound rewrites, rejects, alternatives, unknown deny-all, malformed/oversized inputs, strict ordinal identities, right-to-left parent-prefix admission, site policy rechecks, typed remote job/class/deadline decisions, restricted logical-path views and project namespace decisions. Both final scratch-core checks passed after the ordinal-record extension:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-caveat-core-target nix develop --offline --no-write-lock-file -c cargo check --manifest-path /home/brittonr/scratch/mantle-pattern-caveats-baseline/crates/crunch-authority-core/Cargo.toml --target wasm32-unknown-unknown --lib
Finished `dev` profile [unoptimized + debuginfo]
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-caveat-clippy-target nix develop --offline --no-write-lock-file -c cargo clippy --manifest-path /home/brittonr/scratch/mantle-pattern-caveats-baseline/crates/crunch-authority-core/Cargo.toml --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo]
```

These are pure observations only; no production shell admission was yet exercised. Re-run after integration before changing lifecycle acceptance status.

### Isolated UCAN signed-grant/holder/site fixtures (not Mantle receivers)

An independent scratch crate now depends on Basalt's **exact vendored UCAN snapshot** `c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e` and the scratch core. An earlier run used current sibling UCAN checkout `d46816f19606b518307ba21ab33e8b93b6ebf646`; that was a preliminary compatibility observation, not a change to Basalt's pin. The pinned snapshot's two Radicle Git dependencies were initially unavailable offline; running the scratch test once with Cargo network access fetched revision `a0de79303f3f45c18a34fff535b690a85cb3cf59` and built both crates, resolving that local prerequisite. Mantle's current dev shell still lacks `snafu` in its vendor registry, so the scratch test uses the UCAN dev shell; the Mantle workspace dependency integration remains untested.

One fixture issues a signed parent grant and signed attenuated child with parent caveat preserved, decodes the **verified** child's opaque Mantle caveat payloads under a strict JSON DTO, and checks the right-to-left chain against the declared job. It writes a real scratch file only for the admitted job; other jobs create no output; deadline and unknown variant deny; UCAN verification rejects a child that drops the parent caveat. Another fixture signs a holder invocation and rejects replay before a second effect. A third verifies separately signed store-read and project-declaration grants: a restricted logical view reads only its approved scratch file, rejects a private path and traversal; a project declaration writes one alpha goal and rejects beta's namespace without an output file. An offline repeat against the exact Basalt UCAN snapshot returned:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-caveat-basalt-target nix develop --offline --no-write-lock-file -c cargo test --manifest-path /home/brittonr/scratch/mantle-caveat-stack-proof/Cargo.toml --lib --offline -- --nocapture
running 3 tests
test tests::signed_view_and_project_declaration_gate_real_filesystem_effects ... ok
test tests::holder_signature_and_replay_precede_effect ... ok
test tests::signed_attenuation_admits_one_job_and_denies_effects_for_other_jobs ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

This remains a scratch-only integration observation, not a production CLI, transport, Basalt reviewed-policy invocation, durable revocation service, restricted store backend or live project evaluator. Neither the scratch compatibility check nor the cached Radicle revision changes Basalt's pinned authority source or proves that Mantle's root Cargo window is ready.
