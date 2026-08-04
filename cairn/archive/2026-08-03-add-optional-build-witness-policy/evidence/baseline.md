# Baseline: optional build-witness policy

Date: 2026-08-03

The repository flake command stopped before Cargo ran:

```text
nix develop -c sh -c 'cargo test -p crunch-attestation-core; cargo test -p mantle --bin mantle release_attestation::; cargo test -p mantle --test release_cli attest_policy -- --nocapture'
```

Nix could not fetch the existing private Git input:

```text
error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'
```

I ran the same Cargo tests from the detached cached Mantle dev shell at commit `401251b8`. The tests used this change worktree as the source tree.

Results:

```text
crunch-attestation-core: 73 passed; 0 failed
mantle release_attestation: 11 passed; 0 failed
release_cli attest_policy: 3 passed; 0 failed
```

This baseline records the existing zero-threshold behavior before the policy-profile changes. It does not clear the private-input Nix blocker.
