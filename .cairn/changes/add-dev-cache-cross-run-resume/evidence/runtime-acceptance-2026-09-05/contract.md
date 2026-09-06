# Runtime acceptance continuation

## Goal and completion evidence

Complete V2, V3, and V4 without a duplicate cold attempt or changed proof authority. The active cold attempt is `dev-cold-8e941df2`. Its wrapper PID is `1623993`, its observed Mantle PID is `1624266`, and its existing watcher is Pueue `1265`.

V2 requires the cold, cached, and adoption reports, exact input and stage identities, the fresh-directory prefix checks, and unchanged release aliases. V3 requires a separate cold promoted proof with empty authority and no dev-state consumption. V4 requires current repository gates and scoped verification evidence. A process exit, a task box, or a watcher result does not establish these outcomes.

## Ownership and excluded effects

The active Mantle worktree owns the evidence and any inspected repair. Keep source commit `8e941df20b7d888929692ad5330b76ffcdb7cfa9`, its binary, its profile, and its active remote execution unchanged. Keep the failed `97f47ae2` attempt and historical V98 proof unchanged.

Do not start a successor before its predecessor report permits it. Do not clone a cache while its producer still writes it. Do not weaken source, policy, executable, checkpoint, or receipt validation. Do not change dependency pins, manually edit a lockfile, remove shared Nix state, or restart a service to hide an error.

## Nix diagnosis budget

The recorded blocker is `/nix/store/666qf3k9djcfzv6lwcp4l4v7fgck77ng-source.drv`. Diagnose it while the remote cold proof continues.

Permit three mechanism families, with at most three discriminating probes per family. Each diagnostic command has a ten-minute bound. Permit one focused gate replay after an exact repair and one whole-flake evaluation replay. A new counterexample requires a recorded budget extension. These serial review passes share one reviewer and are correlated.

| Family | Mechanism | Claim and artifact | Gap strength | State | Next check |
| --- | --- | --- | --- | --- | --- |
| Evaluator state | A stale evaluated value retains an invalid derivation context | A fresh isolated evaluation emits the exact unchanged package derivation | unknown | active | Capture the full trace and repeat only the failing attribute in isolated evaluator state |
| Missing store object | Garbage collection removed a derivation that unchanged inputs can reproduce | Nix regenerates or restores the exact derivation and admits its references | unknown | independent | Identify the producing input and its fixed identity from the trace |
| Package defect | The pinned package expression requires unavailable or inconsistent input authority | A minimal exact-input reproduction identifies a source-level blocker | unknown | independent | Inspect the producing expression before any repair |

Successful evaluation does not prove a package build or the runtime cycle. Repair only the admitted mechanism. Preserve unsuccessful commands and exact statuses.

## Runtime preparation budget

Prepare one immutable run descriptor and one report verifier for the current cohort. Use the existing planner, report, checkpoint, and bounded-copy contracts. Give the verifier positive and negative fixtures. Keep execution separate from report inspection.

The remote filesystem does not support mandatory reflinks. Cache views must use private byte copies or a supported copy mechanism. They must retain exact source bytes and explicit capacity checks. Never use writable hard links to share cache state.

Prepared commands are not runtime evidence. Keep V2, V3, and V4 open until their required artifacts pass validation. Allowed outcomes are validated, blocked, exhausted, or user-decision-required.
