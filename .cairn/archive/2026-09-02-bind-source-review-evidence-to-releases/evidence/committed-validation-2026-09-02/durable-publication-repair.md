# Durable-publication binding repair

The first committed-source full checks stopped at `checks.x86_64-linux.durable-file-publication-adoption` before reaching later checks.

The focused local check reproduced the failure. The adoption receipt still bound the previous root `Cargo.toml` and `Cargo.lock` bytes. This change adds pinned Artifact Auth dependencies, so both root Cargo files changed.

The repair updated only these current-root bindings in the checked Nickel receipt, JSON export, and validator:

- `Cargo.toml`: `a6a5d87ecf1b4efb64bf53767c3c906f4dfe06979fa8155cd3563f5524616066`
- `Cargo.lock`: `b20d14286dabc901ad311d831f398d329a76a7b44e48d57d3c085f729e7fe426`
- refreshed receipt JSON: `5b807e657b9366fd56ba6f46bbabbb5be5dc757a2ec417282098ab70ff4d1c99`

The Radicle RID, immutable revision, source URL, Nix lock binding, mapping, authority, validation scope, rollback policy, and non-claims did not change.

After the repair, the exact focused local Nix gate passed. See `durable-publication.command.txt`, `durable-publication.log`, and `durable-publication.status`.
