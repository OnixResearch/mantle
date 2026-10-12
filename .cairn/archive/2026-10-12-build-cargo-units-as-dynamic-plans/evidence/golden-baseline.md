# Isolated Cargo-unit golden baseline (not SECOND)

The immutable two-member `/nix/store/2cc8iz424blki3kgz09ajdndy0n6pjlc-baseline-workspace`
was copied to `/tmp/mantle-cargo-unit-golden-ltcb6pgm/unchanged` and a
second untouched scratch snapshot. This did not run Cargo in the Mantle
repository or touch its target, index, or HEAD. Pinned
`/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/bin/cargo`
(Cargo 1.96.0-nightly `a357df4c2`, paired with rustc 1.96.0-nightly
`c75612477`) ran `build --locked --offline --profile dev
--message-format=json` with a dedicated `CARGO_HOME`, `CARGO_TARGET_DIR`,
absolute pinned rustc, and pinned linker. `compiler-artifact.fresh` observed:

| Scratch scenario | Executed Cargo packages | Reused Cargo packages | Successful |
| --- | ---: | ---: | --- |
| Fresh two-member build | 2 (dependency, app) | 0 | yes |
| Unchanged rerun | 0 | 2 (dependency, app) | yes |
| Edit only `app/src/main.rs` | 1 (app) | 1 (dependency) | yes |

These are **Cargo baseline compiler-artifact counts**, not Mantle unit reuse.
The same pinned Mantle `--json rust-plan --root <scratch> --cargo <pinned>
--rustc <pinned> --profile dev` emitted two ready derivations, zero blockers,
and `cargo-oracle-evidence` for the unchanged and app-only-edited snapshots.
Their receipt hashes were respectively
`313cf87485fcf2d7082e1a7d19c5f8000d0aae308d52d1c17b008253439c5560`
and `1d9762ab46d85f689785ccc91828fcc0a03667f70300bd44c3621dac7e3c7d7e`.
Both measured normalized unit-graph digest
`1830a1380273bc8328c914858ac5b9ba348458e30b11cdf6fdd2e1f2b64e7b67`
(version 1, two units, maximum one direct dependency, 1406 compact JSON
bytes). The full-workspace limit measurement remains in `contract-review.md`.

The pinned `rust-plan --no-cargo-oracle` bounded native mode also reported
two ready derivations and zero blockers in the untouched second snapshot
(`465957ca667d72410d63ea2ea58125a24f68d0ec6a707b73ac42733afdec87c0`)
and app-only-edited first snapshot
(`8331dbaa89a75deaeb7da24f8d8050629a923b84e027af68c4e6d945be982cf9`).
Both receipts classified `cargo-free-bounded-topology`; their scratch roots
differ, so the receipt-hash difference is **not** edit-invalidation evidence.

## Two representative fixtures inside a detached Git worktree

The earlier scratch observations above are preparatory, not the detached
worktree's matched before/after receipts. The following two fixtures were
also measured inside one independent Git worktree before any signed-client
run.

An additional tracked two-package representative fixture,
`examples/projects/rust-workspace`, was checked out in the detached Git
worktree `/tmp/mantle-cargo-unit-golden-ltcb6pgm/isolated-worktree` at
`da00f5842` (the main worktree HEAD and index remained untouched).
Pinned offline Cargo 1.96.0-nightly with a separate Cargo home and target
reported `compiler-artifact.fresh` counts of fresh `2/0`, unchanged `0/2`,
and app-only source edit `1/1` (executed/reused). The edited executable
actually printed `edited:Hello, Mantle!`. Full Cargo output is retained as
`artifact://15158`, `artifact://15161`, and `artifact://15190`.

The pre-existing Mantle CLI's pinned Cargo-oracle `rust-plan` receipts for
that tracked fixture were `b53b42e8d2d85cf878be205478f1dcfa7f2e3dc42e89882e3866083348323a4b`
before and `25639b6aab566a4bbc0e0287406ff9fe67c0cb08bf51b67bdf2bbaf46caaa2ca`
after editing only `workspace-app/src/main.rs` (`artifact://15169`,
`artifact://15195`). Both had two ready derivations, zero blockers, maximum
one direct dependency artifact, and unchanged Cargo unit-graph BLAKE3
`54a98eae2c48bc98ce4b5c5945df8020247bcbb6d037ba0ca8a08c562dc5db28`.
The `greeting` package source digest stayed
`fc4d75d03ced853eb9d4fa4da2e68fbf45e4d22d9cb57f4ecc3f427b79c2187a`;
the edited app digest changed from
`60a3dbf6ebbf95a992dcc7c3c8692f54cf5f6b04e0783e473d437e38e1484346`
to `9e690f2de153fd591eb7f672867ff9115fd0bc5074ebd11056b05d6ed0553243`.
These are Cargo baseline measurements, not Mantle unit reuse or SECOND
signed work-reduction proof.

The immutable `/nix/store/2cc8iz424blki3kgz09ajdndy0n6pjlc-baseline-workspace`
fixture's six manifest, lockfile, and source files were copied byte-for-byte
into that same detached worktree at `cargo-unit-baseline/`; all six file
snapshot hashes matched the immutable originals. A separate Cargo home and
target with the same pinned offline toolchain reported `compiler-artifact.fresh`
counts fresh `2/0`, unchanged `0/2`, and app-only source edit `1/1`
(executed/reused; `artifact://15349`, `artifact://15351`, `artifact://15359`).
The copied and original lockfiles remained byte-identical after these runs
(SHA-256 `8d79e5a3a08009396efa41e90b8bfbede2c52c3571b485507b4ee463776d9a19`).
The edited app actually printed `unit-plan-isolated-edit`.
Cargo-oracle `rust-plan` before/after receipts (`artifact://15353`,
`artifact://15363`) had hashes
`161c269daeb4b9284d938b7cccb401b47b1de0dd602c40c6c199c3475ceb4c0f`
and `4eba41368821b9088e3d21c4b3905dae28097e8fe24745ef0b102c2787235a46`.
Each reported two ready derivations, zero blockers, maximum one direct
dependency artifact, and the same Cargo unit-graph BLAKE3
`0b46e7ac152420d903a7743ef53407f89d134e7fbe73380a91e5c2ed0022f045`.
Only the app package source digest changed;
`unit-baseline-dep` stayed
`522d676faa9b93ec226383e2086bacff44bd82f4aa7cb916e6f4bb7d68ad2d10`.

Mantle's own workspace measurement remains **925 units, 711 package
sources, maximum 65 direct dependencies, and 948146 compact Cargo unit-graph
JSON bytes**, as recorded in `contract-review.md`. A two-unit producer
plan measured 6212 JSON bytes; linear extrapolation from its mean 2651.5
serialized bytes per unit and 314 bytes per source slice, plus its remaining
fixed bytes, estimates **2676173 bytes** for 925 units and 711 sources.
This is illustrative only, not an accepted `mantle-plan-v2` size or bound:
the sampled graph has at most one direct edge, and 711 sources exceed the
generic 256-slice limit. T1.1 is a completed isolated Cargo/rust-plan golden
baseline; T4.1, T4.2, and immutable-source SECOND remain open.
