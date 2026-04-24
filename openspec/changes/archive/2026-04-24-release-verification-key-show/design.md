## Context

The repo now exposes the main witnessed-self-hosting workflow under checked-in
commands, but `release-verify` still expects `--trusted-public-key
<name:base64>` strings. The underlying signing-key files already contain enough
information to derive those verifier-ready strings, and existing code can load
those keypairs, but there is no operator-facing command to print them.

That leaves the cross-machine witness flow awkward:
- the release signer must somehow derive their public key string,
- the witness rebuilder must somehow derive theirs,
- and the verifier must assemble those strings manually for `release-verify`.

This change adds the missing inspect/export step without changing the trust
model or the attestation formats.

## Goals / Non-Goals

**Goals**
- Add a CLI command that prints a verifier-ready trusted public key string from
  an existing signing keypair.
- Support both explicit `--signing-key` input and the existing default config
  lookup path.
- Keep the command side-effect free: inspection only, no key generation.
- Update witnessed-self-hosting docs to include the concrete key-export step.

**Non-Goals**
- Replacing `--trusted-public-key` with some other verifier input format.
- Automatically editing trust files.
- Adding key generation, rotation, or revocation commands here.
- Adding remote key exchange services.

## Decisions

### 1. Put key inspection under `crunch attest`

**Choice:** add `crunch attest key-show`.

**Rationale:** the command serves the same operator surface as `release-show`,
`witness-show`, `policy-init`, and `release-verify`: inspect or prepare
verification material around attestations.

**Alternative:** add a top-level `crunch key-show` or `crunch release key-show`.

**Why not:** a top-level command would fragment the verification workflow, while
`release` would imply the key is part of one release artifact rather than a
reusable signing identity.

### 2. Reuse the existing default config lookup path, but never generate

**Choice:** reuse the same explicit-path / default-config resolution pattern as
other signing-key commands, but fail if no key exists instead of generating one.

**Rationale:** operators already expect `--signing-key` or the default config
location to work. The only wrong behavior here would be surprise key creation,
because inspection must not mutate trust roots.

### 3. Output the exact verifier token first

**Choice:** human output will print the full `name:base64` trusted public key as
its primary line, and JSON output will include both the raw verifier token and
its source path.

**Rationale:** the common operator action is copy/paste into
`--trusted-public-key`. The command should optimize for that direct reuse.

### 4. Validate through release CLI integration tests

**Choice:** put the coverage in `tests/release_cli.rs` instead of unit tests
alone.

**Rationale:** this is a CLI workflow affordance. The important proof is that
users can invoke the command with explicit/default paths and get the expected
machine- and human-readable output.

## CLI shape

Planned surface:

```text
crunch attest key-show [--signing-key <path>]
```

Behavior:
- with `--signing-key`, read that keypair file;
- without it, look for the default config key path;
- if no key exists, fail with a clear missing-key error;
- if `--json` is active, print a structured object containing the verifier-ready
  trusted public key string, key name, and source path.

## Verification strategy

- explicit-path success test in `tests/release_cli.rs`
- default-config success test in `tests/release_cli.rs`
- missing-key failure test in `tests/release_cli.rs`
- docs update showing `key-show` in the witnessed self-hosting flow

## Risks / Trade-offs

**One more subcommand in `attest`**
That is acceptable because it closes a concrete operator gap in the same
workflow family.

**Default-config lookup might surprise users who expected explicit-only**
The docs must state the lookup order clearly, and JSON output should name the
source path so there is no ambiguity.
