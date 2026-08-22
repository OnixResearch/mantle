# V38 promoted source-built proof launch

## Question

Does the exact V37 audit policy allow a fresh promoted proof to reach final receipt verification?

## Inspected evidence

- Source commit `0268f98b` contains only the evidenced middle-sed and `mkdir` floor changes.
- One current test binary replayed the complete V28, V29, and V31 audits.
- The release orchestrator BLAKE3 is `1d1e4118986f5d464ef9de6977a7e2d49f2811ee286a565bccf8dbeffd78fbb7`.
- Root-anchored rsync transfer completed with exact checksum parity.
- The paired source/vendor profile independently verified as Ready.
- Its BLAKE3 is `80a0efe29c231e131776cd06a04aa4a131e0d0334ff4d5d4b55e39b5d55385ac`.
- Leviathan had `722972631040` free bytes before launch.
- The immutable proof disk bound remains `700000000000` bytes.

## Decision

Launch another fresh strict proof. Keep no substitution, 16 jobs, no provider cache, and no resume state.

Remote pueue task `279` started at `2026-08-21T23:55:30-04:00`.

## Owner

The Mantle source-built fixed-point change owns the final receipt decision.

## Next action

Monitor task `279` with `pueue_wait`. Preserve the log, staging root, task record, attempt status, and final receipt evidence.

## Non-claims

Launch evidence does not prove completion. V2 remains open until the verified v2 receipt records a matching fixed point.
