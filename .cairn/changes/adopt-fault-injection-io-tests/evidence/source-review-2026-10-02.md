# Source-only review — 2026-10-02

This receipt supports **only** T1.1 (`tasks.md:3`) and the Boundary documentation task (`tasks.md:14`). Both are marked `[x]` for inspected source and documentation, not for runtime behavior. The other eight task lines—4, 5, 9, 10, 18, 19, 20, and 21—remain `[ ]`.

## Working-source identity

The shared Mantle checkout was on branch `main` at HEAD `da00f58425740adea559ef926c9dfa97b2cb8240` with a dirty working tree. HEAD does not identify the uncommitted working bytes. SHA-256 immediately before the two-checkbox edit:

| Working file | SHA-256 |
|---|---|
| `crates/crunch-store/Cargo.toml` | `a1384252566cf2a22ddd23d2052f21b4dbc742e00d9a6150aec959ecf844c6b3` |
| `Cargo.lock` | `67ea698e70075b7f100d911d8bcb0f826ed476809ae98ba762b8718911f2e737` |
| `docs/dependency-audit.md` | `7c953a38149a831ef2c1a6d9b6c201500c0d8a82f9e8460e7670afa68bb4c23a` |
| `README.md` | `c30c58e3fdfb281f29902235974066c3c9784c10db0bbbd63dfe9edf91f02596` |
| `.cairn/changes/adopt-fault-injection-io-tests/tasks.md` (before) | `e353fe82d8100df7e5fc8327258df72d94e4f6d1c4fd1fe04887fcc964d05021` |

The task file after changing only lines 3 and 14 has SHA-256 `bd138460d3cbefbe56bdce42b8983e4486bedd4a2af4f776e534f35654add4aa`. These hashes pin the inspected files, not an atomic repository-wide snapshot or a validated build.

## T1.1 — pinned development dependency and catalog

`crates/crunch-store/Cargo.toml:49-54` places `fault-injection = "=1.0.10"` under `[dev-dependencies]`. `Cargo.lock:2794-2798` records version `1.0.10`, source `registry+https://github.com/rust-lang/crates.io-index`, and checksum `9e3d175246dec3fddef3b1fcd57acdb023e4c562d032e9eccc5f246da3d7fed3`. `docs/dependency-audit.md:206-210` catalogs transport `crates.io`, plane `validation`, and test-only scope. This is a source-only declaration review; no current-root dependency-graph classification or `cargo deny` result is claimed.

## Boundary — documentation without expanded claims

`docs/dependency-audit.md:210-224` identifies the dev-only fixture scope and disclaims sandbox hermeticity, store correctness, and release eligibility. It distinguishes ordinary completed-build publication diagnostics from the required signed-publication failure `ca-realisation-untrusted` for floating CA and CA-resolved IA intermediates. `README.md:706,721` places the komora-io reference under `## References` and repeats the bounded nonclaims. The existing dirty README was inspected but **not edited** for this milestone.

## Ownership and verification boundary

Quality's owner directly reconfirmed **NO OVERLAP**: no Quality writes, lease, or planned edits to this fault `tasks.md` or `evidence/source-review-2026-10-02.md`; Quality alone owns native quality gates and their PASS counts. The separate local worktree `/home/brittonr/git/OnixResearch/.worktrees/mantle-fault-injection-20261001` remains on the uncommitted branch `cairn/adopt-fault-injection-io-tests-20261001` at the same HEAD; its dirty files are not interchangeable with the shared-main working hashes and were not edited for this review.

**No native current-root test, package, workspace, Clippy, Cairn, Nix, dependency-graph, or `cargo deny` PASS is claimed here.** This receipt does not prove `fallible!` behavior, counter isolation, fixture outcomes, shell implementation correctness, or release acceptance. Historical Quality or isolated-worktree PASS counts are not borrowed. Subsequent task completion needs Quality-owned current-source evidence.
