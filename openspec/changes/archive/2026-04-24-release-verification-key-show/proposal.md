# release-verification-key-show

## Why

The witnessed self-hosting workflow is now mechanically complete through
`release attest`, `attest policy-init`, `attest witness-create`, and `attest
release-verify`, but one operator gap remains: the verifier still needs
`--trusted-public-key <name:base64>` values, and the repo has no checked-in CLI
for deriving those strings from release or witness signing keypair files.

Today the docs can only show placeholders such as
`<release-or-witness-name:base64>`, which means real cross-machine witness
exchange still depends on out-of-band key-format knowledge or hand-written
parsing. That is avoidable friction in exactly the workflow we are trying to
make routine.

## What Changes

- **Add a trusted-key display CLI.** Introduce `crunch attest key-show` to load
  a signing keypair file and print the corresponding trusted public key string.
- **Support both explicit and default key locations.** The command should work
  with `--signing-key <path>` and with the repo's default config location so
  operators can inspect whichever key source they are using.
- **Document the witness workflow with real key-export steps.** Update the
  release/witness docs to show how a release signer and witness rebuilder obtain
  the exact `--trusted-public-key` strings used by `release-verify`.
- **Add positive and negative CLI coverage.** Cover explicit-path success,
  default-config success, and missing-key failure.

## Non-Goals

- Generating new signing keys in this command.
- Mutating `trusted-public-keys` config files automatically.
- Replacing the existing `--trusted-public-key` verifier interface.
- Adding networked key discovery or distribution.

## Capabilities

### New Capabilities
- `release-verification-key-export-cli`: export verifier-ready public key
  strings from signing keypair files.
- `witness-workflow-key-exchange-docs`: show a complete witnessed self-hosting
  workflow including how trusted public keys are obtained.

## Impact

- **Files**: `src/main.rs`, `src/attest_cmd.rs`, signing-key loading helpers,
  `tests/release_cli.rs`, and release/operator docs.
- **APIs**: new `crunch attest key-show` CLI surface.
- **Dependencies**: no new dependencies.
- **Testing**: targeted release CLI coverage for key export paths.

## Constraints

- The command MUST read existing signing keys only; it MUST NOT auto-generate a
  new key as a side effect of inspection.
- Output MUST be directly reusable as `--trusted-public-key <name:base64>`.
- The docs MUST keep witnessed-self-hosting claims bounded to key exchange and
  witness verification, not full-source bootstrap.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| Trusted-public-key export CLI | `specs/release-verification-tech/spec.md` |
| Witnessed workflow docs include key export | `specs/release-evidence/spec.md` |

## How to validate

1. `openspec validate release-verification-key-show` succeeds.
2. `openspec_gate stage=proposal change=release-verification-key-show` passes.
3. `cargo test -p crunch --test release_cli attest_key_show_ -- --nocapture`
   proves explicit-path success, default-config success, and missing-key
   failure.
4. `openspec_gate stage=design change=release-verification-key-show` and
   `openspec_gate stage=tasks change=release-verification-key-show` pass once
   the design and task packet are complete.
