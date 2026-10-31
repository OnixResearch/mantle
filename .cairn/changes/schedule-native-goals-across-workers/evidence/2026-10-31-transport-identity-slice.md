# Transport executable identity slice (2026-10-31)

Scope: bind each selected endpoint's transport executable to a declared BLAKE3
identity and remeasure it before admission reports success. Nothing is executed
and no transport is opened.

## Contract

- `lib/remote-builders.ncl`: `RemoteEndpoint.transport_program_digest_blake3`,
  admitted through the endpoint shape.
- Rust evaluation: a declared program requires a lowercase 64-hex BLAKE3
  identity; a digest without a program is rejected.
- Pool preflight (`remote pool check`): for every selected endpoint, the program
  must be an absolute regular file within 268 MiB, executable, and its BLAKE3
  must equal the declared identity. Denials are
  `remote-pool:pool-transport-program-unreadable`,
  `remote-pool:pool-transport-program-not-executable`, and
  `remote-pool:pool-transport-program-drift`.
- The preflight report gains `transport_programs`, each entry carrying the
  endpoint name, the program path, and the remeasured digest.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle remote_farm_config::tests::` | 36 passed (adds program-without-identity denial) |
| `cargo test -p mantle --bin mantle remote_build::pool_config::tests::` | 4 passed (adds drift and non-executable denials) |
| real CLI: matching digest, two owned descriptors | exit 0, report lists both remeasured digests |
| real CLI: program bytes changed after the digest was declared | exit 3, `remote-pool:pool-transport-program-drift` |
| strict Clippy (mantle targets), rustfmt | exit 0 |

## Not claimed

- The digest proves only that the bytes at the configured path match the declared
  identity at preflight time; S4 must recheck at execution and bind the identity
  into the dispatch receipt.
- Being executable and digest-matched is not sandboxing, trust, or authorization.
