# Composition snapshot invariants and ordinary regression fixtures

## Composition snapshots

The composition planner now indexes snapshots by distinct exact input root before
checking its postcondition. A missing expected root returns the existing typed
`MissingSnapshot` error rather than tripping a debug-only snapshot-count assertion.
Malformed root references, unexpected roots, and duplicate snapshots retain their
existing validation precedence.

One snapshot can supply the same root at multiple distinct mounts. Each mounted
binding still counts toward the binding limit. An exact duplicate root-and-mount
binding is still rejected. The regression checks the complete two-mount tree and
its binding count. No public composition signature, error variant, identity rule,
or store-loading policy changes.

## Ordinary regression fixtures

The store-prefix admission regression asserts rejection before overlay database
creation, rather than matching incidental error-display wording.

The Linux deep-descendant seccomp regression retains real double-fork/orphan
adoption, one reaped adopted descendant, and exactly two allowed audit events.
Its orphan leaf is a real small native executable, resolved and canonicalized by
the parent, content-bound before supervision, and passed explicitly through the
middle subprocess. The no-subreaper negative scenario retains its explicit
self-executable leaf and denial behavior. Production policy, the 30-second reap
deadline, reaper behavior, and compilation profiles are unchanged. Dependency
pins and configuration are unchanged; the standalone smoke helper seeds its own
scratch lock from the existing product lock for offline vendored resolution.

## Evidence boundary

Final-source focused security checks and frozen actual CLI composition
realization/readback succeeded. The captured ordinary workspace command then
failed in three additional regressions: non-UTF-8 Cargo-import rejection,
frontend-CAS special-file rejection, and host-probe capacity. Remaining selected
workspace targets were not completed. This packet is not an all-green ordinary
quality gate or task-completion claim.

Results are bounded to their recorded candidate source and executable subjects.
They do not replay signed build-script proofs, establish full Cargo compatibility,
prove compiler correctness or global reproducibility, or authorize boot,
deployment, release, canonical publication, or lifecycle acceptance. Historical
failures and pre-review receipts remain distinct from final-source verification.
