# Bounded claims and blockers for Git source witness replay

Date: 2026-06-29
Change: `git-source-witness-replay`

## Proven locally

The implementation and focused validation prove the bounded Git-source replay behavior with local `file://` Git remotes and deterministic source archive fixtures:

- Release manifests can record `source_acquisition.kind = "git"`, remote URL, pinned commit, optional ref/tag, archive profile/version, and a BLAKE3 digest that matches `source_archive.digest_blake3`.
- The release source archive policy has a no-std functional core for path ordering, filtering, symlink-target validation, and submodule rejection.
- Witness planning rejects copied-source and external-archive requests when `--require-git-source` is selected.
- Witness scratch preparation can regenerate a source archive from a local Git remote, verify the generated BLAKE3 digest, extract that archive, and ignore publisher-bundled source archive bytes.
- Digest mismatch and ref-policy mismatch fail before source extraction or workflow launch.
- Witness audit metadata distinguishes `copied-source`, `external-archive-source`, and `git-derived-source` and records Git remote URL, commit, ref/tag, generated archive path/digest, status, and failure reason.

## Not proven

This session does not prove:

- Aspen1 cross-machine Git-source witness replay.
- A final imported witness reaching `quorum-satisfied` on a separately rebuilt release.
- Compiler correctness, full bootstrap correctness, generic Git hosting API correctness, or deploy success.
- Signed tag trust roots or generic Git hosting API semantics beyond Git protocol/local transports and the implemented ref/tag resolution checks.

## Aspen1 reachability blocker

A non-mutating reachability check failed because the host alias is not resolvable from this environment:

```text
$ ssh -o BatchMode=yes -o ConnectTimeout=5 aspen1 true
ssh: Could not resolve hostname aspen1: Name or service not known
```

Exit status: 255

Next best step: run the Aspen1 witness replay from an environment that has the Aspen1 host alias/SSH access, then record the source remote/ref/commit, generated source digest, witness audit, rebuilt output digest, witness key, and final `quorum-satisfied` verification before syncing and archiving this change.
