# Verification

All commands ran in the dedicated change worktree on 2026-08-09. The pinned producer parity transcript is in `producer-fixture.md`.

## nix develop -c cargo -Zscript scripts/check-nario-v2-fixtures.rs

```text
Nario v2 fixture corpus valid: positive_bytes=424 negatives=7

```

exit_status: 0

## nix develop -c cargo -Zscript scripts/check-nario-v2-fixtures.rs --self-test

```text
Nario v2 fixture checker self-test passed

```

exit_status: 0

## nix develop -c nickel typecheck config/nario-v2/authority.ncl

```text

```

exit_status: 0

## nix develop -c cargo test -q -p crunch-store nario::tests:: --lib

```text

running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 345 filtered out; finished in 0.01s


```

exit_status: 0

## nix develop -c cargo test -q -p crunch-store --lib --tests

```text

running 353 tests
....................................................................................... 87/353
....................................................................................... 174/353
....................................................................................... 261/353
....................................................................................... 348/353
.....
test result: ok. 353 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s


running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


```

exit_status: 0

## nix develop -c cargo test -q -p snix-store pathinfoservice --lib

```text

running 50 tests
..................................................
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.07s


```

exit_status: 0

## nix develop -c cargo test -q -p mantle --bin mantle nario_source_request_core

```text

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2332 filtered out; finished in 0.00s


```

exit_status: 0

## nix develop -c cargo test -q -p mantle --test store_archive_cli

```text

running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s


```

exit_status: 0

## nix develop -c cargo test -q -p mantle --test foreign_import_cli

```text

running 18 tests
i.................
test result: ok. 17 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.50s


```

exit_status: 0

## nix develop -c cargo clippy -q -p crunch-store --lib --no-deps -- -D warnings

```text

```

exit_status: 0

## nix develop -c cargo clippy -q -p mantle --bin mantle --test store_archive_cli --test foreign_import_cli --no-deps -- -D warnings

```text

```

exit_status: 0

## nix develop -c cargo check -q --workspace --all-targets

```text

```

exit_status: 0

## nix develop -c cargo fmt --check -p crunch-store

```text

```

exit_status: 0

## nix develop -c rustfmt --edition 2024 --check --config skip_children=true src/foreign_import_cmd.rs src/main.rs src/store_cmd.rs tests/foreign_import_cli.rs tests/store_archive_cli.rs

```text

```

exit_status: 0

## nix develop -c ./scripts/check-operator-command-contract.sh

```text
operator command contract generator self-test: PASS
operator command contract: PASS (commands=169)

```

exit_status: 0

## nix flake check --no-build --option secret-key-files  -L

```text
evaluating flake...
checking flake output 'packages'...
checking derivation packages.x86_64-linux.default...
derivation evaluated to /nix/store/15q7s90za0rc65w3ig3g1x35v2d70yml-mantle-0.1.0.drv
checking derivation packages.x86_64-linux.crunch...
derivation evaluated to /nix/store/15q7s90za0rc65w3ig3g1x35v2d70yml-mantle-0.1.0.drv
checking derivation packages.x86_64-linux.ast-grep-toolchain...
derivation evaluated to /nix/store/hwz43xm09gs5v38a4pfyc2dhs5n9h1pj-mantle-ast-grep-toolchain-0.42.1.drv
checking derivation packages.x86_64-linux.ast-grep-package-identity...
derivation evaluated to /nix/store/6f1xqfw3dc0hca3x412qqcgb4cca9d99-mantle-ast-grep-package-identity-smoke.drv
checking derivation packages.x86_64-linux.wasm-component-toolchain...
derivation evaluated to /nix/store/jip7bl880iii0mzaywr3jzfdmfpkb1gb-mantle-wasm-component-toolchain-v1.drv
checking derivation packages.x86_64-linux.wasm-component-toolchain-identity...
derivation evaluated to /nix/store/kqy8bp2zq2zzjnpxkqxaslph51gailg8-mantle-wasm-component-toolchain-identity.drv
checking derivation packages.x86_64-linux.wasm-component-toolchain-compatibility...
derivation evaluated to /nix/store/ja7yc1yr65jdwd66ixf1lr1flpy632mb-mantle-wasm-component-toolchain-compatibility.drv
checking derivation packages.x86_64-linux.mantle-transcript-quality...
fatal: repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git/' not found
warning: could not get HEAD ref for repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'; using expired cached ref 'refs/heads/main'
derivation evaluated to /nix/store/dmbxa994i6qcbw6wdvr44gdmkxgq24mm-mantle-transcript-quality-nextest-0.1.0.drv
checking derivation packages.x86_64-linux.release-nix-witness-quality...
derivation evaluated to /nix/store/3gbncz3hfpvla4hgp6vkjd585hxzjbp6-crunch-release-nix-witness-quality-nextest-0.1.0.drv
checking derivation packages.x86_64-linux.check-store-retention-policy...
derivation evaluated to /nix/store/5xc5yw2mnizcln8b4f52fmgxb861s4z7-check-store-retention-policy.drv
checking derivation packages.x86_64-linux.check-store-overlay-policy...
derivation evaluated to /nix/store/358z0ygz6qib9gpgs79sp2llm0d3q2ls-check-store-overlay-policy.drv
checking derivation packages.x86_64-linux.oci-distribution-registry...
derivation evaluated to /nix/store/7h1b5w2s2gqhb0dgmiydw4xcl2blxnnv-distribution-3.1.0.drv
checking derivation packages.x86_64-linux.rustc-wrapper...
derivation evaluated to /nix/store/pqs6wvv98hm79qm7s9nmvygfmrv48vmq-mantle-rustc-wrapper-0.1.0.drv
checking derivation packages.x86_64-linux.kernelscript-compiler...
derivation evaluated to /nix/store/z1dps3ryzhmn8iyx6jnpfyxfqlnzqpzq-ocaml5.2.1-kernelscript-0.1.2.drv
checking derivation packages.x86_64-linux.kernelscript-core-adapter...
derivation evaluated to /nix/store/bqmwskja62fgasskw950bipl81pslq17-crunch-kernelscript-adapter-0.1.0.drv
checking derivation packages.x86_64-linux.kernelscript-production...
derivation evaluated to /nix/store/4yg232pq5ppfvz28yb6lkvfgcbhbvdn6-mantle-kernelscript-production-artifacts.drv
checking derivation packages.x86_64-linux.kernelscript-production-cohort...
derivation evaluated to /nix/store/69dsdqngnmvhycs32jv57griqkxzsp0z-mantle-kernelscript-production-cohort.drv
checking derivation packages.x86_64-linux.kernelscript-production-shell...
derivation evaluated to /nix/store/9vn36yp3xlsmqw4jq18qbmag0c59l7qr-mantle-kernelscript-production.drv
checking derivation packages.x86_64-linux.kernelscript-production-runtime-check...
derivation evaluated to /nix/store/bdjbsw8lsjl87xn8j46p7az16g4srab2-vm-test-run-mantle-kernelscript-production-runtime.drv
checking derivation packages.x86_64-linux.spacewasm-reference-bundle...
derivation evaluated to /nix/store/l6r3ig41jrdqfivsvnl88z0w7gq1kfj0-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3.drv
checking derivation packages.x86_64-linux.spacewasm-reference-bundler...
derivation evaluated to /nix/store/ipyyk9zxh3kbvk9c1znx79jcdrrvvg38-mantle-spacewasm-reference-bundler-0.1.0.drv
checking derivation packages.x86_64-linux.spacewasm-reference-evidence...
derivation evaluated to /nix/store/y3py5q02d27fq2nywdidz9zj3vcfz3zm-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-evidence.drv
checking derivation packages.x86_64-linux.spacewasm-reference-rust-toolchain...
derivation evaluated to /nix/store/ib3maa2zj4d9ikmlx4x15xaiz9vxh55r-rust-minimal-1.91.1.drv
checking flake output 'apps'...
checking app 'apps.x86_64-linux.tigerstyle'...
warning: app 'apps.x86_64-linux.tigerstyle' lacks attribute 'meta'
checking flake output 'checks'...
checking derivation checks.x86_64-linux.tigerstyle...
derivation evaluated to /nix/store/wqabg5x1djpg9bd41l1a997sfamhrbik-tigerstyle-consumer-check.drv
checking derivation checks.x86_64-linux.crunch...
derivation evaluated to /nix/store/15q7s90za0rc65w3ig3g1x35v2d70yml-mantle-0.1.0.drv
checking derivation checks.x86_64-linux.ast-grep-package-identity...
derivation evaluated to /nix/store/6f1xqfw3dc0hca3x412qqcgb4cca9d99-mantle-ast-grep-package-identity-smoke.drv
checking derivation checks.x86_64-linux.wasm-component-toolchain-identity...
derivation evaluated to /nix/store/kqy8bp2zq2zzjnpxkqxaslph51gailg8-mantle-wasm-component-toolchain-identity.drv
checking derivation checks.x86_64-linux.wasm-component-toolchain-compatibility...
derivation evaluated to /nix/store/ja7yc1yr65jdwd66ixf1lr1flpy632mb-mantle-wasm-component-toolchain-compatibility.drv
checking derivation checks.x86_64-linux.mantle-transcript-quality...
derivation evaluated to /nix/store/dmbxa994i6qcbw6wdvr44gdmkxgq24mm-mantle-transcript-quality-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.release-nix-witness-quality...
derivation evaluated to /nix/store/3gbncz3hfpvla4hgp6vkjd585hxzjbp6-crunch-release-nix-witness-quality-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.kernelscript-production...
derivation evaluated to /nix/store/fwlb67w1wr794yd6kjb5bwfgsfy9n9iz-mantle-kernelscript-production-structural-check.drv
checking derivation checks.x86_64-linux.spacewasm-reference-bundle...
derivation evaluated to /nix/store/l6r3ig41jrdqfivsvnl88z0w7gq1kfj0-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3.drv
checking derivation checks.x86_64-linux.bootstrap-blocker-inventory...
derivation evaluated to /nix/store/maf4s5k0nlsiglaf2ppj6c9bv2i9jry8-bootstrap-blocker-inventory.drv
checking derivation checks.x86_64-linux.nickel-export-core-pin...
derivation evaluated to /nix/store/n4db5aph722fi44blnrg7vip1f4dxxl3-mantle-nickel-export-core-pin.drv
checking derivation checks.x86_64-linux.content-bound-requirement-source-closure...
derivation evaluated to /nix/store/jflijb343vap40lx9qhlrmzv80n4fs4g-mantle-content-bound-requirement-source-closure.drv
checking derivation checks.x86_64-linux.content-bound-requirement-evidence...
derivation evaluated to /nix/store/9b8f5idjmqxag8q2jdzdv1c17j608c87-mantle-content-bound-requirement-evidence.drv
checking derivation checks.x86_64-linux.store-retention-policy...
derivation evaluated to /nix/store/2q5z0pjkxjf2jsij9hqa1mdacxpyn7hk-mantle-store-retention-policy.drv
checking derivation checks.x86_64-linux.store-overlay-policy...
derivation evaluated to /nix/store/z189c1kp1h6id0p09r273hfsmsc3slj5-mantle-store-overlay-policy.drv
checking derivation checks.x86_64-linux.release-determinism-quality...
derivation evaluated to /nix/store/5srx4pl6zvri6gfzlkdlfhp7y07dsxjc-crunch-release-determinism-quality-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.artifact-auth-radicle-cutover...
derivation evaluated to /nix/store/pqbib3zsqkd8zk0qq06kam58z3gq1xy4-mantle-artifact-auth-radicle-cutover.drv
checking derivation checks.x86_64-linux.durable-file-publication-adoption...
derivation evaluated to /nix/store/n0jgm86pv7gv0k8krfxkq740c5gp60zp-mantle-durable-file-publication-adoption.drv
checking derivation checks.x86_64-linux.nextest...
derivation evaluated to /nix/store/abjqlrfz1vf55wa0g3zix7mln40yazx9-mantle-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.clippy...
derivation evaluated to /nix/store/38gzd54dwvinmf7va30xnl9aw7kg311n-mantle-clippy-0.1.0.drv
checking derivation checks.x86_64-linux.fmt...
derivation evaluated to /nix/store/xv2v0qw9w4aikbp187s63bsv497diyfs-mantle-fmt-0.1.0.drv
checking derivation checks.x86_64-linux.spacewasm-reference-profile...
derivation evaluated to /nix/store/n9c40y8a14cabyv75rpssccf8a9k7sxr-spacewasm-reference-profile-export.drv
checking derivation checks.x86_64-linux.spacewasm-reference-host-library...
derivation evaluated to /nix/store/j6r8vlls8h06h61gq52fgcg1b7593bib-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-host-library.drv
checking derivation checks.x86_64-linux.spacewasm-reference-wasm-library...
derivation evaluated to /nix/store/f7ghaq9l9f7jfvkipzq3mfqzpj716riv-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-wasm-library.drv
checking derivation checks.x86_64-linux.spacewasm-reference-host-runner...
derivation evaluated to /nix/store/xv1a7k2m125vv3003ii3gf2ymwygly2x-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-diagnostic-runner.drv
checking derivation checks.x86_64-linux.spacewasm-reference-upstream-unit-tests...
derivation evaluated to /nix/store/pqdh5xf8y0k76h7pjvaskxbq289crkwg-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-unit-tests.drv
checking derivation checks.x86_64-linux.spacewasm-reference-spectest-address...
derivation evaluated to /nix/store/b5pnd8m8xmnfgfgdxchr639gvgcl7yp9-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-spectest-address.drv
checking derivation checks.x86_64-linux.spacewasm-reference-fixtures...
derivation evaluated to /nix/store/77xymn526iq43h5ggnckzw6id3znjk5x-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-fixture-report.drv
checking derivation checks.x86_64-linux.spacewasm-reference-negative...
derivation evaluated to /nix/store/5r89n7f0qdvyp4335j6jaanx1y3vcmc1-mantle-spacewasm-reference-negative-check.drv
checking flake output 'devShells'...
checking derivation devShells.x86_64-linux.default...
derivation evaluated to /nix/store/63x0lqdzsl6yqmvg0k99b1sanjpcy0pr-nix-shell.drv
all checks passed!
warning: The check omitted these incompatible systems: aarch64-darwin, aarch64-linux, x86_64-darwin
Use '--all-systems' to check all.

```

exit_status: 0

## git diff --check

```text

```

exit_status: 0

FINAL_STATUS=0

# Cairn pre-archive validation and gates

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- validate --root .

```text
{
  "change_issues": [],
  "changes": 13,
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 61,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 84,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 36,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/durable-file-publication-adoption/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/fix-nix-producer/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 38,
      "scenario_blocks": 119,
      "substantive_requirement_blocks": 38
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/immutable-release-pointer/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/mantlepkgs-catalog-structure/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/mantlepkgs-impact-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/mantlepkgs-update-plans/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/mantlepkgs/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/nix-producer-adapter/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 18,
      "scenario_blocks": 42,
      "substantive_requirement_blocks": 18
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 83,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 17,
      "scenario_blocks": 37,
      "substantive_requirement_blocks": 17
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 76,
      "scenario_blocks": 116,
      "substantive_requirement_blocks": 76
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 44,
      "scenario_blocks": 129,
      "substantive_requirement_blocks": 44
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 145,
      "scenario_blocks": 486,
      "substantive_requirement_blocks": 145
    },
    {
      "path": "./cairn/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-lifecycle/spec.md",
      "requirement_blocks": 20,
      "scenario_blocks": 39,
      "substantive_requirement_blocks": 20
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 37,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 59,
      "scenario_blocks": 172,
      "substantive_requirement_blocks": 59
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 61,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 33,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 33,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 33
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 50,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 24,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_lines": 98,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 23,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 1,
      "task_in_progress": 0,
      "task_todo": 21
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 36,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 31,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 31,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 31,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 31
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_lines": 42,
      "substantive_requirement_blocks": 6,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 12,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 12,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 12
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 21,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 21,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 21
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_lines": 38,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 18,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 18
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_lines": 72,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 22
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 27,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_lines": 64,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 26,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 26,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 26
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 32,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 23,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 3,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 18,
      "task_done": 18,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 10,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 10,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 10
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 25,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 2,
      "task_in_progress": 0,
      "task_todo": 6
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 19,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 19,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 19
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}

```

exit_status: 0

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- gate proposal import-nario-v2-store-archives --root .

```text
{
  "change": "import-nario-v2-store-archives",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "254b7ed4a59f5ba32bdd0042a5015d5c7b6034e7531c174704ebf8b6331ab41c",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "receipt_hash": "a351e19b345a99c7b7e40620f6fb1ac781a1f794a656cb0a7d2d279802685c8f",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}

```

exit_status: 0

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- gate design import-nario-v2-store-archives --root .

```text
{
  "change": "import-nario-v2-store-archives",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "01f4a3fdb6edec3a3f34b07edc3433e22c9ec3557490710d65c135bb90d4817e",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "receipt_hash": "f024b9225bf4966e9264b1b5522e84f16c94d38ac4b2e485e69ba84ac1df9166",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 32,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 23,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 3,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}

```

exit_status: 0

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- gate tasks import-nario-v2-store-archives --root .

```text
{
  "change": "import-nario-v2-store-archives",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e13b672084a9fcb88315a5aa5e436237e1fdcd8209c839976338283a5bb753e9",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "receipt_hash": "80d10177e74fb231ddcd9733edce9e6de972d99adfe87a153d4c8af0de770c75",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 32,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 23,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 3,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 18,
      "task_done": 18,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}

```

exit_status: 0

FINAL_STATUS=0

# Cairn specification sync

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- sync import-nario-v2-store-archives --root .

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md"
    },
    {
      "description": "merge delta spec into main specs: ./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
    }
  ],
  "blocked": false,
  "change": "import-nario-v2-store-archives",
  "delta_specs": [
    "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
    "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
  ],
  "dry_run": true,
  "input_hash": "846c4ca0fc01c923eeaf0dbbb8706a4d20397c82c4c199f069d10b5f00adb5f0",
  "layout": "cairn",
  "merge_preflight": {
    "documents": [
      {
        "accepted_before_exists": true,
        "accepted_before_hash": "32de8f5faf4fffe2db50963ff449a5bede3041f8c166742bf298febb59caea26",
        "blocked": false,
        "changed": true,
        "delta_hash": "fb433b425690e93b043a59591a401cddf7c3ad3f9b3d9437065213035194c8e4",
        "destination": "cairn/specs/foreign-derivation-import/spec.md",
        "diagnostics": [],
        "expected_after_hash": "9865e1b0a4f4e03d4c2caa76590dbca5cb335d7af895829e05c363aa7c414e7f",
        "operations": [
          {
            "block_hash": "9d9a8036e6cdef991d85d191105a7425b069b7fd0ae266d458b4b82c2dfbcca1",
            "heading": "### Requirement: Nario v2 can prepare exact foreign source requirements",
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_source_preparation"
          },
          {
            "block_hash": "08be205d8ae3f292fc13734573386ed2df450b064cff0dd8b8e97ed05d078db5",
            "heading": "### Requirement: Nario v2 evidence keeps explicit non-claims",
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_non_claims"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_source_preparation",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_non_claims",
            "state": "applied"
          }
        ],
        "source": "cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md"
      },
      {
        "accepted_before_exists": true,
        "accepted_before_hash": "67d5c3c178fb7ad007b1e5b5094bb010409949559176391631e27097758932ba",
        "blocked": false,
        "changed": true,
        "delta_hash": "b8deb2f1fc457755eaa1be738a1f106ad2e6de3cf8d3c521fbbd72b9f6942b66",
        "destination": "cairn/specs/store-transports/spec.md",
        "diagnostics": [],
        "expected_after_hash": "495513c517dc0683128282d9b76e5e3eeee7b1cf8637463136c347bdf7602276",
        "operations": [
          {
            "block_hash": "e19dab9d1a212d074de4bd65e2757f34cac291ed3af32517c431e64d308cae6f",
            "heading": "### Requirement: Nario v2 read compatibility is version-bound",
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_read_compatibility"
          },
          {
            "block_hash": "3b0f736702d7fae30b12b5dab9a6131ce4ed475b4f7ea7d4f42a6d23007e8849",
            "heading": "### Requirement: Nario v2 admission is bounded and fail-closed",
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_bounded_admission"
          },
          {
            "block_hash": "cb17b0fd2f9e43244738c658e9209a007736ebd357a097685e7289617fa3ed41",
            "heading": "### Requirement: Nario v2 compatibility has durable validation",
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_validation"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_read_compatibility",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_bounded_admission",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_validation",
            "state": "applied"
          }
        ],
        "source": "cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
      }
    ]
  },
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "4ae71308c5f859ebdca29c1e371b27ca1fc4cd9678ca2b7f6f026299b81b9919",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "reasons": [],
  "receipt_hash": "9adfcb6674e841c685a3a37eab0e622c98e737030cea027df0b7eb932c381eaf"
}

```

exit_status: 0

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- sync import-nario-v2-store-archives --root . --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md"
    },
    {
      "description": "merge delta spec into main specs: ./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
    }
  ],
  "blocked": false,
  "change": "import-nario-v2-store-archives",
  "delta_specs": [
    "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
    "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
  ],
  "dry_run": false,
  "input_hash": "846c4ca0fc01c923eeaf0dbbb8706a4d20397c82c4c199f069d10b5f00adb5f0",
  "layout": "cairn",
  "merge_preflight": {
    "documents": [
      {
        "accepted_before_exists": true,
        "accepted_before_hash": "32de8f5faf4fffe2db50963ff449a5bede3041f8c166742bf298febb59caea26",
        "blocked": false,
        "changed": true,
        "delta_hash": "fb433b425690e93b043a59591a401cddf7c3ad3f9b3d9437065213035194c8e4",
        "destination": "cairn/specs/foreign-derivation-import/spec.md",
        "diagnostics": [],
        "expected_after_hash": "9865e1b0a4f4e03d4c2caa76590dbca5cb335d7af895829e05c363aa7c414e7f",
        "operations": [
          {
            "block_hash": "9d9a8036e6cdef991d85d191105a7425b069b7fd0ae266d458b4b82c2dfbcca1",
            "heading": "### Requirement: Nario v2 can prepare exact foreign source requirements",
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_source_preparation"
          },
          {
            "block_hash": "08be205d8ae3f292fc13734573386ed2df450b064cff0dd8b8e97ed05d078db5",
            "heading": "### Requirement: Nario v2 evidence keeps explicit non-claims",
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_non_claims"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_source_preparation",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "foreign_derivation_import.nario_v2_non_claims",
            "state": "applied"
          }
        ],
        "source": "cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md"
      },
      {
        "accepted_before_exists": true,
        "accepted_before_hash": "67d5c3c178fb7ad007b1e5b5094bb010409949559176391631e27097758932ba",
        "blocked": false,
        "changed": true,
        "delta_hash": "b8deb2f1fc457755eaa1be738a1f106ad2e6de3cf8d3c521fbbd72b9f6942b66",
        "destination": "cairn/specs/store-transports/spec.md",
        "diagnostics": [],
        "expected_after_hash": "495513c517dc0683128282d9b76e5e3eeee7b1cf8637463136c347bdf7602276",
        "operations": [
          {
            "block_hash": "e19dab9d1a212d074de4bd65e2757f34cac291ed3af32517c431e64d308cae6f",
            "heading": "### Requirement: Nario v2 read compatibility is version-bound",
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_read_compatibility"
          },
          {
            "block_hash": "3b0f736702d7fae30b12b5dab9a6131ce4ed475b4f7ea7d4f42a6d23007e8849",
            "heading": "### Requirement: Nario v2 admission is bounded and fail-closed",
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_bounded_admission"
          },
          {
            "block_hash": "cb17b0fd2f9e43244738c658e9209a007736ebd357a097685e7289617fa3ed41",
            "heading": "### Requirement: Nario v2 compatibility has durable validation",
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_validation"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_read_compatibility",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_bounded_admission",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "store_transports.nario_v2_validation",
            "state": "applied"
          }
        ],
        "source": "cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
      }
    ]
  },
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "9865e1b0a4f4e03d4c2caa76590dbca5cb335d7af895829e05c363aa7c414e7f",
          "exists": true,
          "path": "./cairn/specs/foreign-derivation-import/spec.md"
        },
        {
          "content_hash": "495513c517dc0683128282d9b76e5e3eeee7b1cf8637463136c347bdf7602276",
          "exists": true,
          "path": "./cairn/specs/store-transports/spec.md"
        }
      ],
      "manifest_hash": "6a46abe0d93c65b14b2c74bf810685deb79816f70bdf4c372f03404fe388bf0b"
    },
    "before": {
      "entries": [
        {
          "content_hash": "32de8f5faf4fffe2db50963ff449a5bede3041f8c166742bf298febb59caea26",
          "exists": true,
          "path": "./cairn/specs/foreign-derivation-import/spec.md"
        },
        {
          "content_hash": "67d5c3c178fb7ad007b1e5b5094bb010409949559176391631e27097758932ba",
          "exists": true,
          "path": "./cairn/specs/store-transports/spec.md"
        }
      ],
      "manifest_hash": "4c64d4cf0da8aff1bd6454f15baab30643a19a79133ef0b9be4fc93f8530483d"
    },
    "kind": "sync",
    "manifest_hash": "4b4012318cff2d4cad9a77e925a2620307a90ddd92111593131ffd82ccb41725"
  },
  "plan_hash": "2bf083ac9b635b6b0c3abd7b61d42f1e5a697e05efec9dff2caa3b066e7f09f4",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "reasons": [],
  "receipt_hash": "07f78d6ec3ca147e74372a41d8d226701c19b4679d753f32d4f1e0f3d69b9acb"
}

```

exit_status: 0

FINAL_STATUS=0

# Tracey coverage after sync

## `nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- tracey coverage --root .`

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)

FINAL_STATUS=0
```

# Broad-check boundary

`nix run .#tigerstyle -- check -- --manifest-path crates/crunch-store/Cargo.toml --lib` did not reach the Nario target. It failed first on existing findings in `crates/crunch-gc-core` and `crates/crunch-overlay-core`. This change does not claim whole-tree Tiger Style success. `git diff origin/main -- crates/crunch-gc-core crates/crunch-overlay-core` is empty. Focused first-party Clippy with warnings denied passed above.

# Archive execution

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- archive import-nario-v2-store-archives --root .

```text
{
  "actions": [
    {
      "description": "move active change to archive: import-nario-v2-store-archives",
      "kind": "archive_change",
      "path": "./cairn/changes/import-nario-v2-store-archives"
    }
  ],
  "blocked": false,
  "change": "import-nario-v2-store-archives",
  "dry_run": true,
  "input_hash": "1a10a0457a66b7907f9bf1001d1ddeb17128e908c20bbe7a82839996eb785f90",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "f3411a0fb838a64e0b4232be240dd7b30c3589f2633e57e33d8fd4ce16e0773d",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "reasons": [],
  "receipt_hash": "2c65818d8d757fae1053f6aa6a18209e9119715b9662a015789600144af08936"
}

```

exit_status: 0

## env CAIRN_ARCHIVE_DATE=2026-08-09 nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- archive import-nario-v2-store-archives --root . --execute

```text
{
  "actions": [
    {
      "description": "move active change to archive: import-nario-v2-store-archives",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives"
    }
  ],
  "blocked": false,
  "change": "import-nario-v2-store-archives",
  "dry_run": false,
  "input_hash": "1a10a0457a66b7907f9bf1001d1ddeb17128e908c20bbe7a82839996eb785f90",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "805c23c41bcdf6ef2cd170b83d515db986da5ec567c7f17d22f1ac496b3f8969",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/design.md"
        },
        {
          "content_hash": "20a8c4e66ce049462e54bf6c659bb4cf5872fb8b6f24d0fba1e8b0c70d9f4e04",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/evidence/baseline.md"
        },
        {
          "content_hash": "d7d15a58c2da2aeaa578563c3ab7446d6a5458910d3c5dcb75cea393cfaa1b8b",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/evidence/format-boundary-review.md"
        },
        {
          "content_hash": "9cf9295cd5bb370fe2b5abe3ee29417216e66709464cbe6558212c1083805ec1",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/evidence/producer-fixture.md"
        },
        {
          "content_hash": "a977752dd499bc8bca98c3e7a5176f987eb0fad0c4520f7f54f986af4ff93615",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "5ce43886d0489acbe79c2724a9c96743e2ed8907ef7ba0301816d224423db016",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/evidence/verification.md"
        },
        {
          "content_hash": "921a27b267a39c16e6ab7bf33ae00527e568e68e62716406602deacd01a33042",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/proposal.md"
        },
        {
          "content_hash": "fb433b425690e93b043a59591a401cddf7c3ad3f9b3d9437065213035194c8e4",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md"
        },
        {
          "content_hash": "b8deb2f1fc457755eaa1be738a1f106ad2e6de3cf8d3c521fbbd72b9f6942b66",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/specs/store-transports/spec.md"
        },
        {
          "content_hash": "78f6a970e6a03530f5dc8f00634e6529008d05a5899b216ea43ce3c9ceeb5610",
          "exists": true,
          "path": "./cairn/archive/2026-08-09-import-nario-v2-store-archives/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/import-nario-v2-store-archives"
        }
      ],
      "manifest_hash": "645c34d44fc8bf911fce69e70986ebbb0f18f4c906224a302017a9baeeb4da2e"
    },
    "before": {
      "entries": [
        {
          "content_hash": "805c23c41bcdf6ef2cd170b83d515db986da5ec567c7f17d22f1ac496b3f8969",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/design.md"
        },
        {
          "content_hash": "20a8c4e66ce049462e54bf6c659bb4cf5872fb8b6f24d0fba1e8b0c70d9f4e04",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/evidence/baseline.md"
        },
        {
          "content_hash": "d7d15a58c2da2aeaa578563c3ab7446d6a5458910d3c5dcb75cea393cfaa1b8b",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/evidence/format-boundary-review.md"
        },
        {
          "content_hash": "9cf9295cd5bb370fe2b5abe3ee29417216e66709464cbe6558212c1083805ec1",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/evidence/producer-fixture.md"
        },
        {
          "content_hash": "a977752dd499bc8bca98c3e7a5176f987eb0fad0c4520f7f54f986af4ff93615",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "5ce43886d0489acbe79c2724a9c96743e2ed8907ef7ba0301816d224423db016",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/evidence/verification.md"
        },
        {
          "content_hash": "921a27b267a39c16e6ab7bf33ae00527e568e68e62716406602deacd01a33042",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/proposal.md"
        },
        {
          "content_hash": "fb433b425690e93b043a59591a401cddf7c3ad3f9b3d9437065213035194c8e4",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md"
        },
        {
          "content_hash": "b8deb2f1fc457755eaa1be738a1f106ad2e6de3cf8d3c521fbbd72b9f6942b66",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
        },
        {
          "content_hash": "78f6a970e6a03530f5dc8f00634e6529008d05a5899b216ea43ce3c9ceeb5610",
          "exists": true,
          "path": "./cairn/changes/import-nario-v2-store-archives/tasks.md"
        }
      ],
      "manifest_hash": "f4c48260a0801c68e1afd5e581dd1b14ac3830ed7fc27e4a2b678595f725a2d2"
    },
    "kind": "archive",
    "manifest_hash": "5bf7b800e1bffae8e5415a4fec5b41445fdec042d2c503de4d78614781da2696"
  },
  "plan_hash": "d78e5b2cc9c8aa043f0fb35d6dee59bd940ac5becde217404753534e424e668b",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "reasons": [],
  "receipt_hash": "60fc3d07174799fa7e2f20c0a8c924ea8ef3f5f1acdb49e2afee3bd2e5d03aeb"
}

```

exit_status: 0

FINAL_STATUS=0

# Post-archive validation

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- validate --root .

```text
{
  "change_issues": [],
  "changes": 12,
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 61,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 84,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 36,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/durable-file-publication-adoption/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/fix-nix-producer/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 40,
      "scenario_blocks": 124,
      "substantive_requirement_blocks": 40
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/immutable-release-pointer/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/mantlepkgs-catalog-structure/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/mantlepkgs-impact-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/mantlepkgs-update-plans/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/mantlepkgs/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/nix-producer-adapter/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 18,
      "scenario_blocks": 42,
      "substantive_requirement_blocks": 18
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 83,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 17,
      "scenario_blocks": 37,
      "substantive_requirement_blocks": 17
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 76,
      "scenario_blocks": 116,
      "substantive_requirement_blocks": 76
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 44,
      "scenario_blocks": 129,
      "substantive_requirement_blocks": 44
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 145,
      "scenario_blocks": 486,
      "substantive_requirement_blocks": 145
    },
    {
      "path": "./cairn/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-lifecycle/spec.md",
      "requirement_blocks": 20,
      "scenario_blocks": 39,
      "substantive_requirement_blocks": 20
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 16,
      "scenario_blocks": 46,
      "substantive_requirement_blocks": 16
    },
    {
      "path": "./cairn/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 59,
      "scenario_blocks": 172,
      "substantive_requirement_blocks": 59
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 59,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 33,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 33,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 33
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 50,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 24,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_lines": 98,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 23,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 1,
      "task_in_progress": 0,
      "task_todo": 21
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 36,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 31,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 31,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 31,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 31
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_lines": 42,
      "substantive_requirement_blocks": 6,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 12,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 12,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 12
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 21,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 21,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 21
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_lines": 38,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 18,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 18
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_lines": 72,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 22
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 27,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_lines": 64,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 26,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 26,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 26
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 10,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 10,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 10
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 25,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 2,
      "task_in_progress": 0,
      "task_todo": 6
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 19,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 19,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 19
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}

```

exit_status: 0

## nix run path:/home/brittonr/git/OnixResearch/.pi/worktrees/cairn-legacy-layout-e5ee2a6#cairn -- tracey coverage --root .

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)

```

exit_status: 0

## git diff --check

```text

```

exit_status: 0

FINAL_STATUS=0
