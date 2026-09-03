# Committed-source validation

Source commit: `0d2f75b7dff042c6b1ec42cdede4caa754971d7f`

## Successful checks

- `nix flake check --no-build -L` returned status `0`.
- The no-build evaluation includes `cli-application-architecture`, `application-core`, and `application-core-wasm`.
- Cairn validation reported `"valid": true`.
- Proposal, design, and tasks gates returned `PASS`.
- The final tasks gate recorded all 12 tasks as complete.
- Tracey reported `155/155` before synchronization of the new accepted spec.

Pueue task `2844` ran the no-build check. The lifecycle logs were refreshed after the final task was marked complete.

## Ordinary full-flake boundary

`nix flake check -L` returned status `1` in pueue task `2845`.

The exact primary blocker is unchanged:

```text
error: hash mismatch importing path '/nix/store/3r2nwafkx9xha0y6xsd0w0557ba0c294-rust-src-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu';
         specified: sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=
         got:       sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=
```

Mantle did not change the pin, expected hash, builder policy, or gate scope.

## Local-builder full-flake boundary

`nix flake check -L --builders ''` returned status `1` in pueue task `2846`.

This run reached the root test suite. It did not reproduce the earlier filtered-source compile omission. The result was:

```text
test result: FAILED. 2499 passed; 8 failed; 72 ignored; 0 measured; 0 filtered out
```

The eight failures were:

- `oci_projection_shell::tests::frontend_cas_rejects_special_files_before_oci_projection`;
- six `protected_exec_ptrace::linux::tests::ptrace_supervisor_*` host tests;
- `rust_source_provider::tests::rust_bootstrap_openssl_no_asm_patch_covers_all_configs_and_is_idempotent`.

The OCI test received a different special-file diagnostic. The six ptrace tests received `No such file or directory`. The OpenSSL patch test reported `/bin/sh: python3: not found`.

These tests do not touch the CLI split, application core, application ports, or presentation adapters. This change does not weaken, skip, or relabel them as passing.

## Evidence files

- `nix-no-build.log` and `nix-no-build.status`;
- `nix-full.log` and `nix-full.status`;
- `nix-local.log` and `nix-local.status`;
- `cairn-validate.log`;
- `cairn-proposal.log`;
- `cairn-design.log`;
- `cairn-tasks.log`;
- `tracey.log`.

The successful focused gates remain recorded under `../validation-2026-09-03/`.

## Conclusion

Committed-source flake evaluation and all focused CLI architecture gates pass. Both full build failures remain separate host, cache, or fixture boundaries.

This evidence does not prove the failed full checks, current-source fixed-point parity, external effect success, provider correctness, deployment, or release eligibility.
