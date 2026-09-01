# V28 GNU binutils audit-closure repair

## Goal

Verify the corrected Mantle source profile and retain the next exact protected
StageX blocker.

## V28 authority

- Source commit: `df346d30`.
- Release orchestrator BLAKE3:
  `5ca308638c657b399498e001b9506b5266aedd65d660b3bcde1db7c43004260b`.
- Refreshed profile BLAKE3:
  `bd9afb4c02dd734a5448fc940b62770041fd642053ad2217092b58d28d9a4802`.
- Replacement Mantle source BLAKE3:
  `85270b43a824c361cdcd21b36dae1433ec1b2164297dc8614110ba6c392e0bdb`.
- Profile verification: Ready with zero missing, stale, unsupported, or
  untrusted records.
- The profile contains `tools/generate_operator_command_contract.rs`.

## Result

V28 passed preflight and executed the complete GNU binutils construction. It
then failed closed during protected-audit splitting:

```text
GNU binutils event count is outside the accepted closure: observed 73980
```

The V28 inventory records the already accepted lower sed form with `4,891`
invocations. V27 used `4,892` sed invocations. The complete protected audit had
`76,528` events. The GNU binutils suffix had `73,980` events.

## Audit comparison

The change did not add an executable identity or bypass a policy decision.
The preserved V28 suffix initially reached the current lower global bound, then
exposed one existing identity-count floor:

```text
mkdir: observed 5369, previous bounds [5377, 5425]
```

The repair changes only these lower floors:

- GNU binutils protected events: `73,991` to `73,980`;
- the `4,891`-sed event range: `73,991` to `73,980`;
- source-built `mkdir` executions: `5,377` to `5,369`.

Upper bounds remain unchanged.

## Full preserved-audit validation

The ignored replay test loads the complete 85,984,081-byte V28 audit and the
exact binutils inventory. It selects the trailing `73,980` events and runs the
ordinary production validators for:

- allowed policy decisions and absolute resolved paths;
- the closed 68-identity set;
- every generated executable identity and exact count;
- every fixed executable identity and exact count;
- sed-derived launcher, utility, GNU sed, and coreutils counts;
- predecessor tool counts and accepted narrow ranges;
- generated-child producer order.

Result:

```text
preserved-binutils-audit: events=73980 sed_invocations=4891
test result: ok. 1 passed; 0 failed
```

See `preserved-audit-replay.log`. `audit-observation.txt` binds the full audit
and inventory with BLAKE3.

## Local validation

- closed event-bound test: passed;
- positive and negative `mkdir` count tests: passed;
- isolated focused Clippy: passed;
- changed-file formatting: passed;
- `git diff --check`: passed.

See `local-validation.log`.

## Non-claims

V28 stopped before provider construction, Rust bootstrap, stage1, stage2, or
receipt creation. A fresh proof is required. This repair does not admit an
unknown executable, denied event, new source, fallback path, or broader StageX
behavior.
