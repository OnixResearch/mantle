# Dependency audit baseline

## Success contract

Goal: make Mantle's checked-policy dependency audit pass by resolving the actionable vulnerability and admitting only the already-reviewed source and exact SPDX expression.

Completion requires:

- `crossbeam-epoch 0.9.18` absent and `RUSTSEC-2026-0204` unwaived;
- `unknown-git = "deny"` retained while only the immutable Nickel export repository is added;
- the exact `Apache-2.0 WITH LLVM-exception` expression admitted without crate-wide license bypass;
- checked-policy audit, locked consumer checks, immutable source-pin checks, and negative policy evidence passing.

False completion includes adding an advisory waiver, allowing arbitrary Git sources, floating the Nickel export revision, disabling license checks, citing default-policy output, or treating policy admission as proof of upstream correctness or legal advice.

## Baseline commands and findings

The authoritative baseline command was:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
```

It exited non-zero with these actionable findings:

```text
advisories FAILED: RUSTSEC-2026-0204, crossbeam-epoch 0.9.18
licenses FAILED: winx 0.36.4, Apache-2.0 WITH LLVM-exception is not explicitly allowed
sources FAILED: git source https://github.com/OnixResearch/nickel-export is not explicitly allowed
bans ok
```

Targeted resolution feasibility was checked without changing the lockfile:

```text
nix develop -c cargo update -p crossbeam-epoch --precise 0.9.20 --dry-run
Updating crossbeam-epoch v0.9.18 -> v0.9.20
warning: not updating lockfile due to dry run
```

## Dependency paths

The vulnerable dependency path is runtime-reachable through the workspace's BLAKE3 usage:

```text
crossbeam-epoch 0.9.18
└── crossbeam-deque 0.8.6
    └── rayon-core 1.13.0
        └── blake3 1.8.2
            ├── mantle and first-party core/shell crates
            └── vendored snix/nix compatibility crates
```

The license finding is target-specific but part of the locked all-target graph:

```text
winx 0.36.4
└── cap-primitives 4.0.2
    ├── cap-fs-ext 4.0.2 -> mantle
    └── cap-std 4.0.2 -> mantle
```

The source finding is a direct Mantle dependency:

```text
nickel-export-core 0.1.0
(https://github.com/OnixResearch/nickel-export?rev=257fafc1c746f1faf156207043a4c826bfb16d49#257fafc1)
└── mantle 0.1.0
```

Accepted requirement `mantle.nickel_export_cutover.source`, Cargo, Nix, generated source metadata, and dedicated pin checks already mandate that exact revision. Source-policy admission will not replace those revision checks.

## Classification

| Finding | Classification | Action |
|---|---|---|
| `RUSTSEC-2026-0204` | transitive but tractable | generate the compatible `0.9.20` lock update; no waiver |
| `winx` compound SPDX expression | first-party policy gap | admit the exact expression only |
| immutable Nickel export repository | first-party policy gap | admit the exact repository only; retain revision checks |

## Non-claims

This baseline does not claim dependency correctness, absence of future advisories, legal approval for downstream distribution, trust in arbitrary commits from an admitted repository, or release eligibility.
