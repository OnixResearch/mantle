# Verification evidence

Date: 2026-08-02

Implementation commit: `0e5680bf`

The repository used the current Cairn policy from the sibling Cairn checkout.
The repository-local generated policy lacks `nominal_identity_policy`.

## V1 focused positive and negative checks

| Command | Result |
|---|---|
| `nix develop -c cargo fmt --all -- --check` | pass |
| `nix develop -c cargo check --workspace --all-targets` | pass |
| `nix develop -c cargo test -p crunch-store provenance --no-fail-fast` | pass, 18 tests |
| `nix develop -c cargo test -p mantle --bin mantle build_correctness` | pass, 14 tests |
| `nix develop -c cargo test -p mantle --bin mantle foreign_provenance_audit -- --nocapture` | pass, 4 tests |
| `nix develop -c cargo test -p mantle --test foreign_import_cli -- --nocapture` | pass, 14 tests |
| `nix develop -c check-nickel-configs` | pass, including two expected policy rejections |
| `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs` | pass |
| `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test` | pass |
| `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test` | pass |
| `nix develop -c cargo clippy -p crunch-store --lib --no-deps -- -D warnings` | pass |
| `nix develop -c cargo clippy -p mantle --bin mantle --test foreign_import_cli --no-deps -- -D warnings` | pass |

The scanner tests cover pure payload classes, malformed inputs, arbitrary input
properties, castore fixtures, duplicate nodes, integrity mismatches, links,
containers, recursive archives, and limits.

The CLI test proves preflight rejection does not write an audit receipt. It also
proves that an existing receipt is not overwritten.

## V2 deterministic audit receipts

The two-node CLI audit returned `pass` and `provenance-audited`.

- Build report: `d43e02fbb71558fa67a938df41558be076a2ec53625795704a83fcc304e27a38`
- Audit receipt: `e3175cdbe4028d595d3664e831a18b38bbd5c1afbfa8c6bdb190af506c6f264e`

The end-to-end node-limit audit returned `fail` and retained `realized`.
Its audit receipt was `73da9c07ee54f9e329d33f43989f496cc3b6403ac80555fe87442d70b7ac04a7`.

The receipt-matrix test returned typed, self-digest-valid failed receipts.
Each receipt retained `realized`.

| Case | Audit BLAKE3 |
|---|---|
| leftover GNU store path | `e5e8e75793e13e3dedeee18915f53fdb906c559d17353f7a07957e86bbf66590` |
| missing shebang target | `5726ec84673f30068ed1c7f4a17e87c9d84bcdb3c61ec102cbc4861b9dcfb40a` |
| malformed ELF | `3807c1faf0c9e9fa1dce472301e404c525382f1ac0f2a7591c26869ee08a1f21` |
| symlink escape | `e795936f1fa8ceed6126b5f1552ffc024c00d86a7e3de48e5857faa53edeea76` |
| hidden archive executable | `8b06a7506912ad1eb2f332f50f95704b418c63b2a94c3f69c66722c22c08ee15` |
| incomplete closure | `25aa21aad6c62f61dba363c789d7856e31f15ad9f73c5e97ba465d66bbeefdc2` |
| PathInfo limit | `22866d44e519761c162f29bee24035793cc98f56f86d0590fbb16424f30e06ec` |
| node limit | `ad1678e91a65b7f66aaa772b3776d603e79ba04c64dfb0427fa5e887beb8cdb3` |
| blob limit | `939af4adc8c1be05299fc180860bb741141ad9102864b58294485ea7c8a898c1` |
| per-blob byte limit | `6e51178dcd1effaa9eb010117ddea38ac2bd03890c7098f839e3492ede4778b5` |
| total byte limit | `949a8332af803e6d538b261588c042161ae89ad0c6e7859fd5704a231a4fa3c5` |
| traversal depth limit | `47aa3ab6221a358063e350cd6b9ef913da56ebb0cf1fa128e670f3b63cf8de0e` |
| finding limit | `9050d55b357cc3a18fc0be60482fa2abde86008f112024f9eec0f50ed93c5a9e` |
| duplicate limit | `5b57e68a6d0b11377f0d570bb4c559cae54ef68387f31b7a819c6396f80cab05` |
| container entry limit | `afab7bf7b19772e84ddbb0fd9c92254e365cb410a7cf1686df3ed9da77164e5e` |
| container byte limit | `aafd1791c6f95b6042a1f05781a066a0e0669773a6135159ca3f9a6b9f705193` |
| container depth limit | `229d7afd6c4f51014ba5039ec9e209deea1abaa877d4b2ad9fe5ccb2a0fe4aba` |
| path byte limit | `24e59842dfc1875523c9cb9b94f39ad76dd999db79e14d7029e4730b155b264d` |
| shebang byte limit | `ecf72dafddb0ccbc4a3672c3554d6bbcdd86b69008b9f719f4d40b2e35b4a73a` |

The fixed-output mismatch report remained
`f6c3a38233fc1be5df15293d7bec37258a4af578890b99afc000412fce20054d`.
The Guix no-`/bin/sh` report remained
`eb97a276c7768c7c8d85cb78fc2a3d233a63965233f529ff32308446fdcc0c9a`.

## V3 lifecycle and policy checks

These commands used:

`--policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`

Strict Cairn validation passed. Proposal, design, and tasks gates each returned
`PASS` and `valid: true`.

Pre-sync Tracey reported 263 referenced requirements and 690 accepted
requirements. It also reported three expected dangling active-change references:

- `foreign_derivation_import.castore_provenance_audit`
- `foreign_derivation_import.executable_payload_classification`
- `foreign_derivation_import.provenance_audit_receipt`

Tracey retained unrelated repository debt and returned `valid: false`.

After accepted-spec sync, strict Cairn validation passed again. Tracey reported
266 referenced requirements and 693 accepted requirements. None of the three new
IDs remained missing or dangling. Unrelated repository debt kept the verdict at
`valid: false`.

The machine-schema checker retained 39 unrelated StageX and source-build
inventory findings. It did not report the new provenance audit sources.

Broad all-target Clippy still reaches unrelated vendored `fuse-backend-rs`
failures and `src/protected_exec_seccomp.rs`. The focused first-party Clippy
commands above passed with warnings denied.
