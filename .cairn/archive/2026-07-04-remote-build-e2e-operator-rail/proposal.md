## Why

Remote-build slices now cover route planning, stdio/SSH bindings, source input sync, output-trust admission, scheduler dispatch, and observability in focused tests. Operators still need one cohesive rail that proves those pieces compose without overclaiming: plan the route, open the framed session, synchronize inputs, execute remotely, admit the signed output, and report status/log/build evidence with secrets redacted.

## What Changes

- Add a deterministic operator-facing remote-build e2e rail that drives the supported stdio/SSH-compatible binding through route planning, handshake, input sync, remote execution, signed output admission, and bounded reports.
- Bind the rail to concrete evidence classes and non-claims so a passing fixture proves composition for that rail only, not production P2P coverage or general Cargo/Nix compatibility.
- Add negative fixtures for missing output trust, stdout protocol pollution, stale or missing source input state, upload quota/privacy rejection, and invalid fallback phase handling.

## Impact

- **Files**: remote-build test harness/fixtures, report/status assertions, Cairn remote-builds spec delta, and evidence transcript.
- **Testing**: focused positive e2e rail plus negative redaction/trust/input/protocol/fallback fixtures, then Cairn validate and proposal/design/tasks gates.

## Out of Scope

- Production coordinator deployment.
- New network transport protocols beyond the existing framed stdio/SSH-compatible binding.
- Claiming remote-build reproducibility or release readiness from the fixture alone.
