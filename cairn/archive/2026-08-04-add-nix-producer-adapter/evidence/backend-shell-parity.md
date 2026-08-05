# Backend shell and parity evidence (I5, I10, I11, I12, I13, I14, I15, I16, I20)

Date: 2026-08-04. Worktree `.pi/worktrees/add-fix-nix-producer`, branch `cairn/add-fix-nix-producer`.

## Contract core and shell unit tests

```text
cargo test -p mantle --bin mantle nix_producer
→ test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 2167 filtered out
```

Covers: request admission (schema, path, expression, budget, control chars), backend selection (unknown, duplicate, unavailable, unsupported system, wrong identity shape), identity facts (backend mismatch, posture mismatch, eval-args digest drift), success acceptance (non-drv root, empty closure, budget overflow), and shell behavior with a fake backend (two-node closure collection, ambient host-nix posture, evaluation failure, garbage output, timeout, missing closure member, count budget, unspawnable binary).

## CLI surface

`mantle foreign-import produce-backend` runs one registered backend (`fix` or `host-nix`) under the bounded policy, copies the concrete `.drv` closure, and lowers it through the existing direct-`.drv` producer into `foreign-derivation-graph-v1` and `foreign-package-index-v1`. Backend kinds, identity, and revision land in `producer.{kind,identity,revision}`; no backend-specific field enters the artifacts.

```text
cargo test -p mantle --test foreign_import_cli
→ test result: ok. 16 passed; 0 failed; 1 ignored
```

The ignored test is the live parity fixture (below).

## Live backend parity (I20)

Expression: two-node `let dep = derivation ...; in derivation { ... ${dep} ... }`.

```text
fix backend:      accepted=true, closure {h5rncf1cq3h967p8w0rc3kn5qw6qqyh9-fix-dep.drv,
                  nbjjgi2b2r3vad3f4fb7ny7wfjcj9z4n-fix-top.drv}
host-nix backend: accepted=true, identical closure basenames
graph diff:       only producer.{identity,kind,revision} differ
fix identity:     a8e67ddf32a4b6dab90590bcedf4701297320c302f3be5397ca75dc1d3b7fd50 (BLAKE3 of the Mantle-built binary)
host-nix identity: /run/current-system/sw/bin/nix-instantiate (ambient posture)
```

Checked-in fixture: `produce_backend_fix_and_host_nix_emit_parity_artifacts` in `tests/foreign_import_cli.rs`, gated on `MANTLE_TEST_FIX_BACKEND_BINARY` + `MANTLE_TEST_NIX_INSTANTIATE_BINARY` + a reachable daemon. Verified passing with `--ignored --nocapture` (1 passed).

## Memory-limit finding (design correction)

The default 8 GiB `RLIMIT_AS` killed the fix backend at startup: its parallel GC reserves between 64 GiB and 256 GiB of virtual address space (measured with `prlimit --as=... fix eval -E '1 + 2'`). `RLIMIT_AS` counts reservation, not resident use, so it is the wrong mechanism for arena-reserving backends. `memory_bytes_max` is now 0-by-default (disabled) and never reported as enforcement; honest RSS enforcement needs a cgroup mechanism, deferred.

## Negative-fixture mapping (I11)

- Source hash mismatch: proven live twice by the `--fix` workflow (dummy hashes rejected with exact expected/got reports); pipeline FOD-mismatch tests cover the class.
- Floating revision: `crunch.fetchGit` requires an exact rev plus content hash; a floating ref cannot satisfy the fixed-output contract.
- Host-toolchain leakage / undeclared inputs: the fix build sandbox mounts only declared inputs; the spike build reported `hermeticity: practical (no degraded facts)`.

## Non-claims

This evidence covers producer mechanics and two-node parity only. It does not prove evaluator correctness, nixpkgs-scale parity for the Mantle-built binary, memory enforcement, or realization readiness.
