# Isolated authority-caveat evidence (2026-10-04)

## Branch and provenance boundary

All edited source is in `/home/brittonr/.cargo-target/mantle-rust-script-pin-20261004/source`, branch `work/adopt-casita-store-backend-20261004`, observed clean at `e7d91c31abab268852c09097b581e13c7a47d24b` before edits. At inspection `git merge-base HEAD origin/main` and `git rev-parse origin/main` were both `7dcc8a84882a28aee8871898feba1a4c63ecef2a`. That proves this already-existing isolated branch descends from current `origin/main`, **not** that a new authority-specific worktree was created; T1.1 stays open. The original Mantle root was not modified or built. The shared Cargo manifests, readiness-owned coordination/daemon/remote-serve/proof-stage/doctor sources, root README and ADR index were untouched by this slice.

## Baseline source observations before core insertion

- `src/remote_build.rs:12542-12570` commits remote ticket admission: authenticate against receiver state/time/endpoint and validate concrete request before redeeming and persisting a use. `TicketAuthRequest` at `src/remote_build.rs:335-343` contains bearer ticket id/secret and untrusted compatibility claims; it does **not** carry UCAN token, holder signature/nonce, or a delegated job-set caveat.
- `crates/crunch-store/src/capability.rs:719-766` allows complete store-path reads through `OutputLookup::find`, `find_with_layer`, and `find_remote`; no pattern-limited lookup guard is present. `src/store_cmd.rs:590-646` lists all PathInfo records, and `store info` substring-filters the complete list afterward. A restricted view cannot safely reuse that path: an exact normalized logical path must be checked before a PathInfo read, and unrestricted listing must be denied.
- `src/project_build.rs:83-95,130-176` rejects empty selector components and resolves a project selector to an extraction plan, but neither carries nor verifies a signed project identity or a rewritten goal grant.

After the signed fixture passed, production-carrier source was inspected again:
`src/remote_credentials.rs:603-611,673-714` issues a bearer from entropy
and a keyed verifier. The typed `TicketIssueInput` and persisted `RemoteTicket`
(`:445-460`) contain validity, use/build/upload limits and optional endpoint,
but **no issuer-signed producer identity, delegated job set, output class or
job deadline**. `src/remote_service_secrets.rs:47-49,275-283` resolves a
ticket-verifier key and a **result** signing key, not a UCAN grant issuer
trust chain. `src/remote_build.rs:335-343,860-876,9613-9629` transmits only
ticket id/secret and untrusted endpoint/time compatibility claims in the
`AuthTicket` frame; `ConcreteBuildRequest` (`:383-407`) supplies request and
expected-output fields without a signed producer scope. Its actual
pre-effect insertion point is `read_remote_production_opening` then
`commit_remote_ticket_admission_for_state` (`:12435-12486,12542-12571`);
that function verifies the bearer against server time/endpoint and commits a
ticket use before sending `AuthOk`, but receives no holder-signed grant.
The root `Cargo.toml` does not declare UCAN, Basalt or this standalone crate.
UCAN's fixture verification API is therefore not yet wired to an authentic
receiver-supplied issuer trust set, revocation/replay state or signed
job-set/output/deadline request payload. The readiness owner owns those
remote wire/admission sources, and Casita owns the live root Cargo lease.
Attaching our pure filter to current client-controlled request fields or
the bearer secret would falsely grant non-bypassable authority, so **no
production receiver edit or CLI effect test is claimed**, and T3/T4 remain
unchecked.

The exact pinned UCAN verifier source at
`src/token/holder_invocation.rs:509-557,751-829` requires caller-owned
time, issuer/audience key resolution, proof store, revocations, operation
profile, caveat policy and replay admission; its authorized result explicitly
does not prove downstream request semantics. The exact pinned Basalt
`src/authority_core.rs:280-299,451-480` consumes caller-asserted verified
grant/receipt, caveat/replay and trusted-policy facts. Its closed
`src/ucan_enforcement/types.rs:31-50` has no Mantle remote action. Neither
provides the missing Mantle-signed producer-to-job-set authority or permission
to invent a verified receipt from the existing ticket verifier/result signer.

The pre-change focused store baseline ran from the isolated branch with a
dedicated target before touching any existing store source:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-authority-isolated-target nix develop --offline --no-write-lock-file -c cargo test -p crunch-store --lib capability::tests::output_lookup_and_selected_root_registration_share_exact_identity -- --exact --nocapture
test capability::tests::output_lookup_and_selected_root_registration_share_exact_identity ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 414 filtered out
```

That test proves existing exact-identity lookup/root registration, **not**
restricted pattern reads. Remote and project source observations above are
static; no corresponding isolated-branch command result is claimed here.

## Stack evaluation and explicit non-claims

Pinned UCAN source is Basalt's vendored `6f888f6c91a4ea26f0bd52b6486e6643c8f6d271`: signed proof chains preserve opaque caveat identities/payloads and holder invocation binds operation and request; caller supplies key resolution, revocation and replay. Basalt's closed `EnforcementAction` list in `basalt/src/ucan_enforcement/types.rs:31-50` has no Mantle build/store/project action. The generic `authority_core::evaluate_authority` consumes caller-supplied verified grant/receipt references, caveat/replay disposition, and one shell-trusted policy artifact reference; it cannot itself create verification freshness. ADR 0085 selects these boundaries and rejects pretending an unrelated closed Basalt action authorizes Mantle work. The Nickel policy under `config/authority-caveats/` is a **proposed** Mantle-specific source artifact, not an approved policy-owner contract until reviewed. The standalone core is not yet part of a production receiver.

## Observed Nickel contract exports

Under the isolated Mantle Nix development shell, each of the following
commands exited successfully without modifying the shared manifest:

```text
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c nickel export --format json config/authority-caveats/default.ncl
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c nickel export --format json config/authority-caveats/basalt-policy.ncl
```

The first output has exactly the four declared caveat kinds, eight-chain/eight-
alternative, 256-byte pattern, 8192-byte serialized payload, 512-byte
canonical-value, 16-segment, 32-job bounds, right-to-left ordering, silent
discard and original/parent-prefix recheck declaration. The second output
contains schema `ucan-nickel-contracts.policy.v1` and three distinct
`mantle-remote-build`, `mantle-store-view`, `mantle-project-goal` contracts
with exact resource prefixes and abilities. Both outputs were captured as
checked-in `config/authority-caveats/generated/*.json`; an exported policy
is **not** a production Basalt decision or policy-owner signoff.

## Scoped validation observed

`nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`
returned `changes: 34`, `change_issues: []`, `issues: []`,
`profile_issues: []`, `spec_issues: []` and `resolution.valid: true`.
The checked tool path is the resolved target of the repository guide's
`/home/brittonr/git/cairn` symlink, which Nix refused as a `path:` flake.

Direct Rust `--target wasm32-unknown-unknown --crate-type lib --emit=metadata`
on `crates/crunch-authority-core/src/lib.rs` in the pinned Mantle Nix shell
exited successfully. These observations compile and validate isolated
source only, not its placement into the shared workspace or any receiver.

The standalone `#![no_std]` crate was initially compiled as a direct test
harness independently of Cargo dependency resolution:

```text
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c rustc --test --edition=2024 --crate-name crunch_authority_core -o /home/brittonr/scratch/mantle-authority-core-tests-20261004 crates/crunch-authority-core/src/lib.rs
/home/brittonr/scratch/mantle-authority-core-tests-20261004 --nocapture
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```

That exercises rewrite binding and silent nonmatch, reject/alternatives,
unknown deny-all, strict ordinal chain and bound failures, parent-prefix
nonwidening, receiver-supplied job/deadline, logical store path escapes,
and cross-project goal isolation. It does **not** exercise an existing
Mantle receiver or claim that generic Basalt automatically knows these sites.

After the identity-rewrite fast path and owned-file formatting, the final
isolated source was rerun using
`nix develop --offline --no-write-lock-file --option min-free 0 --option max-free 0`.
Direct `rustc --test --edition=2024 -D warnings` again passed **11/11** core
tests; `rustc --crate-type lib -D warnings --emit=metadata` and
`rustc --target wasm32-unknown-unknown --crate-type lib --emit=metadata`
exited successfully. The bounded Nix GC override was needed because
parallel host shells queued behind a big collector lock; it did not change
Cargo manifests, locked dependency pins or root source.

Scoped `clippy-driver --crate-type lib -D warnings --emit=metadata` on the
final no-std core exited successfully. Pinned `rustfmt --edition 2024 --check`
passed for the three owned Rust files; `git diff --check` found no whitespace
errors in tracked authority files. The final Cairn `validate --root .`
reported `changes: 34`, `issues: []`, `change_issues: []`,
`spec_issues: []` and `resolution.valid: true`; transient Nix
`git.onix.computer`/remote-builder warnings fell back to a successful local
build. None is a production receiver or whole-workspace quality claim.

Scoped Cairn `gate proposal`, `gate design` and `gate tasks` for
`attenuate-build-authority-with-pattern-caveats` each returned
`"verdict": "PASS"`, `"valid": true`, `"issues": []`. The tasks gate reported
15 substantive tasks: **5 done, 10 still TODO**; a structural gate does
not complete those tasks. Final Nickel `typecheck` and JSON exports of both
`config/authority-caveats/default.ncl` and `basalt-policy.ncl` succeeded;
the exported values match the retained generated artifacts. Policy-owner
acceptance and production shell-trusted references remain unproven.

The checked crate-owned `crates/crunch-authority-core/check-signed-authority.sh`
was then executed with a private
`CARGO_HOME=/home/brittonr/.cargo-target/mantle-authority-fixture-cargo-home`,
private
`CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-authority-fixture-target`
and explicit pinned rustup `nightly-2026-07-22` on `PATH`. Git transport
overrides mapped the two GitHub URLs to verified local Git repositories;
the manifest retains exact UCAN `6f888f6c91a4ea26f0bd52b6486e6643c8f6d271`
(the revision vendored by the Basalt pin) and Basalt
`89675cd4f585f837323c049e4a25f7b94c903038`. Cargo fetched Radicle and
registry dependencies normally. The command passed **11/11 core tests and
3/3 signed UCAN/Basalt fixture tests; 0 failures in each**. The signed
tests observe parent/child nonwidening with rejected and altered child
caveats, holder signature/replay admission before an effect, and store/
project receiver-scope checks before filesystem effects.

Counting distinct fixture admission/effect scenarios (not duplicate
assertions), the three signed tests exercised **6 positive and 19 negative**
cases: remote signed attenuation and bare-token scope `2/7`, holder
invocation plus generic Basalt authority `2/8`, and restricted store/
project effects `2/4` (positive/negative, respectively). Remote negatives
cover rejected sibling, undeclared job, deadline, missing/altered parent,
unknown caveat and bare-token unauthorized job; holder negatives include
replay and seven mismatched or denied Basalt facts; store/project negatives
cover forbidden sibling read, traversal, wrong contract and cross-project
declaration. These are finite fixture observations, not policy completeness.

Critically, a
**signed zero-caveat grant** admits the declared `compile-1` job but cannot
write a `bare-unauthorized` effect: the receiver checks job-set scope
before UCAN's absent-caveat callback. Cargo generated a crate-local
`Cargo.lock` for this run; it was removed afterward without touching the
root manifests or root lock. This is an isolated fixture, not a production
signed grant or live receiver proof, and the crate is not yet included in
the current root workspace quality rail.

## Production gate still requiring owner handoff

A coherent root manifest insertion, reviewed Basalt policy reference, signed
UCAN issuance/holder invocation transport, issuer and holder key resolution,
live revocation/replay ports, and receiver-local effect checks are required
before T3.* can close. The readiness owner holds remote protocol and daemon
sources; `TicketAuthRequest` has no credential carrier. `RemoteJobScope`
currently takes a caller-supplied job set/output class/deadline, not an
agreed signed UCAN payload binding a producer identity to that job set.
T3.1 needs an owner-reviewed signed producer/issuer binding, scope payload,
transport and holder-request binding before a receiver can redeem a ticket.
Store and project receivers cannot be claimed integrated by exercising the
standalone core. No archive or StageX action is authorized here.

The readiness owner reported the source-identical remote stdio baseline PASS
(`artifact://31662`); this slice did not rerun it. The live-state owner holds
exclusive root Cargo manifest/lock edits. The nested `[workspace]` in
`crunch-authority-core` keeps this fixture standalone meanwhile.
`scripts/check-first-party-quality.sh` runs
`cargo test --workspace --lib --tests` and therefore does **not** include the
standalone fixture. Only after the live owner releases the graph **and** a
real production authority consumer exists, coordinate removing the nested
workspace marker and registering the crate with pinned signed UCAN/Basalt
test dependencies in the root lock. That cutover would join this same
fixture to the required first-party workspace test rail; do not claim it now.
`scripts/quality-gate-common.sh` also maintains a named package list for
rustfmt; add `crunch-authority-core` there only at that approved cutover.
