## Why

Release bundle creation writes source, binaries, proof material, optional evidence, and finally the manifest directly into the requested public directory. Any mid-assembly error leaves a partial public bundle, and the next attempt rejects that now-nonempty directory. Operators must manually identify and remove partial state before retrying, while other processes can observe incomplete evidence.

A release bundle should become visible at its final path only after complete assembly and verification.

## What Changes

- Plan bundle assembly before mutation and stage all artifacts in a private sibling directory on the destination filesystem.
- Write the canonical manifest only after all artifacts are staged, then run full bundle verification against the staging root.
- Publish with one no-clobber atomic rename after successful verification.
- Leave an existing destination untouched, keep failed staging state out of the public path, and support safe retry or bounded cleanup of Mantle-owned stale staging directories.
- Add positive publish/retry fixtures and negative injected-failure, stale-stage, destination-race, symlink-destination, and verification-failure fixtures.

## Impact

- **Files**: release bundle creation planning, filesystem shell, staging/commit logic, diagnostics, fixtures, tests, and operator documentation.
- **Compatibility**: the final bundle destination must be absent at commit time; pre-created empty directories are no longer used as in-place assembly roots.
- **Safety**: readers observe either no bundle or a complete verified bundle, and a failed attempt does not poison the final path.
- **Claims**: atomic publication establishes local assembly and visibility guarantees only, not storage durability after power loss, artifact correctness, or release eligibility beyond verified bundle policy.
