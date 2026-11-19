# Contributing to Mantle

## Development shell

Work from the repository root inside the checked-in shell:

```bash
nix develop
```

The shell supplies the pinned Rust nightly, clang/mold, Nickel, nextest,
cargo-deny, and the repository's Tiger Style tooling. Bare host Cargo is not
the supported build path.

## Before sending a change

Read `AGENTS.md` for the repository-specific architecture, evidence, and
lifecycle rules. Keep the change scoped to one capability and repository.

Run the narrowest check that exercises the change, then the applicable gates:

```bash
./scripts/check-first-party-quality.sh
./scripts/check-first-party-tigerstyle.sh
nix flake check
```

Focused test legs may use `cargo test -p <package> --lib <filter>` or
`cargo test -p mantle --test <suite> <filter>`. Do not run the full self-hosting
proof from a routine CLI test. Capability-dependent tests must fail or report a
skip explicitly; a skip is not execution evidence.

## Cairn lifecycle

Use Cairn for changes that affect behavior, policy, or release evidence. Create
a change under `.cairn/changes/<name>/` with the proposal, design, tasks, and
spec delta; record focused evidence, and validate with:

```bash
nix develop -c cairn validate --root .
```

Keep implementation commits separate from `cairn archive`. Archive only after
the gates recorded in the change pass and the implementation is committed.
Legacy OpenSpec trees are archive-only unless a repository-local note says
otherwise.

## Evidence and claims

- Quote exact commands and `test result:` lines from captured output.
- Do not promote a passing build into a bootstrap, release, or correctness
  claim; read `docs/operator-proof-guide.md` first.
- Preserve non-claims in receipts, audits, and architecture records.
- Do not commit secrets, signing keys, credentials, private store contents, or
  whole proof store/state trees.

## Licensing

Mantle is licensed under `AGPL-3.0-or-later`; see `LICENSE`. By submitting a
contribution, you agree that it may be distributed under the same license.
There is no separate contributor license agreement in this repository.
Vendored components keep their own recorded licenses and notices; do not
relicense them under the workspace expression.

## Security

Do not report exploitable findings in a public issue. Follow `SECURITY.md`.
