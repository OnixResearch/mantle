## Why

Mantle contains a pure Cairn release-handoff validator, but current call sites appear limited to tests, so production release verification can bypass the contract. The CLI also advertises a source-root path that unconditionally reports unavailable, while practical hermeticity remains easy to confuse with evidence suitable for strong release claims.

Mantle must connect its existing cores to real release paths, expose an honest source-root capability boundary, and require strict hermetic execution for Onix release evidence.

## What Changes

- Invoke Cairn handoff validation from production release assembly and verification paths before handoff evidence is admitted.
- Measure handoff artifact bytes in the shell and compare typed identities in the pure validator.
- Replace the permanently unavailable source-root path with an implemented bounded operation or an explicit unsupported capability that is not advertised as executable.
- Make strict hermetic mode required for Onix release profiles; practical mode remains explicit diagnostic or development evidence.
- Add positive production-path fixtures and negative bypass, stale-byte, unavailable-capability, host-influence, and practical-mode-promotion fixtures.
- Add a checked-in CI workflow whose verification scope is `nix flake check`.

## Impact

- **Files**: release handoff shell wiring, release verification, source-root CLI capability reporting, hermetic policy, fixtures, docs, Nix checks, and CI workflow.
- **Dependencies**: local implementation waits for opaque evidence sidecar binding; authenticated Cairn evidence consumption waits for Cairn's archived authentication change.
- **Claims**: passing Mantle verification proves bundle-local measured linkage and enforced execution-policy facts only, not Cairn correctness, source correctness, build semantics, deployment safety, or universal reproducibility.
