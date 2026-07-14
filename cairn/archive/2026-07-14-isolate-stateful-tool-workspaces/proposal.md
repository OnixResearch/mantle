## Why

Some compilers and engineering tools maintain large incremental caches, databases, or absolute-path-sensitive workspaces. Treating those bytes as invisible worker state violates Mantle's declared-action model; uploading the entire mutable workspace through CAS can cost more than recomputation; and silently reusing it can make remote results depend on execution history.

Mantle needs an explicit boundary between immutable declared cache inputs and mutable session workspaces. Operators should be able to choose practical stateful execution without letting it masquerade as hermetic shared-cache evidence.

## What Changes

- Add typed execution modes for no workspace reuse, immutable content-addressed cache snapshots, and mutable leased session workspaces.
- Treat immutable cache snapshots as ordinary declared object inputs that may participate in strong action identity and reuse.
- Bind mutable workspaces to worker, tenant/authority, action-compatibility class, toolchain refs, job/attempt/fence, stable sandbox mount point, quotas, and retention policy.
- Keep mutable workspace bytes out of shared action-result admission and strong action-correctness claims unless a separate clean rebuild proves equivalent admitted outputs.
- Add pure lease/admission/scrub planning and shell-owned workspace creation, locking, cleanup, snapshotting, and failure quarantine.
- Prevent cross-tenant reuse, stale-fence mutation, path escape, secret carryover, and unbounded retention.
- Report warm-state use, provenance, limits, cleanup result, and explicit evidence downgrade.

## Impact

- **Surfaces**: build/action policy, sandbox request construction, local and remote worker state, coordinator leases, typed Nickel config, action-result admission, GC/retention, and build reports.
- **Dependencies**: consumes fenced remote attempts and existing sandbox/output-admission primitives; remote capacity accounting may later reserve workspace disk through `account-scarce-resources-and-locality`.
- **Non-claims**: no claim that a mutable cache is hermetic, portable, deterministic, safe to share across authorities, or eligible for shared action-result reuse; no interactive IDE/GUI service.
- **Validation**: pure mode/lease tests, stale/cross-tenant/path/secret/quota negatives, warm-workspace integration, clean-rebuild comparison, cleanup/restart fixtures, and Cairn gates.
