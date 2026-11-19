# Design: Add security and contribution guides

## Decision

| Concern | Decision |
| --- | --- |
| Private reporting | GitHub private security advisories, with owner contact fallback |
| Scope | Research-software boundaries, signed outputs, remote workers, sandbox, signing/store/substitution |
| Contribution shell | `nix develop` |
| Gates | First-party quality, Tiger Style, `nix flake check`, focused tests |
| Lifecycle | Cairn change with proposal/design/tasks/spec/evidence; archive after accepted gates |
| Licensing | Contributions under AGPL-3.0-or-later; no CLA; vendored notices retained |

## Failure behavior

A public issue for an exploitable finding is explicitly discouraged. A skipped
capability test is not execution evidence. Contribution commands must match
`AGENTS.md` and the repository scripts.

## Verification

Check file presence and README links; review commands against `AGENTS.md`.
