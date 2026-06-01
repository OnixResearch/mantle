## Context

Mantle fetchers already have implementation tests, but examples should prove the user-facing Nickel helpers and CLI paths too. Network-free examples reduce flakes and provide safe fixtures for bad-hash and `--fix` workflows.

## Approach

1. Add fixture-backed examples for `fetchurl`, `fetchTarball`, and `fetchGit`. The fixture material can be generated inside tests or checked in as small deterministic content.
2. Prefer local `file://` URLs and test-owned local git repositories for CI-grade examples. Keep external URLs only in cookbook examples that are marked real-network.
3. Add negative examples or test fixtures with deliberately wrong hashes. Assertions should check fixed-output mismatch shape and, where supported, that `--fix` reports or writes the corrected hash without treating the bad build as success.
4. Keep hash algorithms explicit. Mantle-owned content fingerprints should prefer BLAKE3 where Mantle controls the contract; fetcher interoperability may keep SRI SHA-256 where existing fetch helper APIs require it.
5. Use functional-core helpers for fixture planning and expected outcome classification. Test shells create temp files/repos, run commands, and capture output.

## Risks

- Checking binary tar fixtures into the repo can obscure review; prefer tiny textual fixtures or generated tarballs in tests.
- `file://` behavior can diverge from remote HTTP behavior; keep one ignored real-network smoke for the public cookbook path.
- `--fix` assertions must avoid silently editing checked-in examples unless the test runs in a temp copy.

## Validation

- Offline positive tests prove file, tarball, and git examples build in temp store/state roots.
- Negative tests prove wrong hashes fail closed and do not persist bad PathInfo/output state.
- A `--fix` test uses a temp copy and verifies the corrected hash path or emitted diagnostic.
