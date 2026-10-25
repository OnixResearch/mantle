# Design: Resolve remote entry points through one binding step

## Goal and scope

Concentrate remote name admission into one resolve step per entry point, with a
binding table and revocation. Change nothing about payload trust.

## Current behavior

Remote builder access uses one-use tickets with quotas. Transfer sessions bind
a canonical manifest to the job, attempt, fence generation, policy digest, and
store prefix. Substituters are configured by URL with trust keys. Base stores
carry descriptors and generations. Each path validates its own name shape.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Keep per-path admission | Existing ticket, URL, and layer checks | Rejected as sufficient: revocation and visibility stay split | Revocation fixture |
| One resolve step per entry point | Binding table with generation fences | Selected direction | Resolve and revocation fixtures |
| Route everything through one service | Single admission daemon for all paths | Deferred: larger operational change than this slice needs | Not blocking |
| Kerberos-style ticket exchange | Two-party ticket protocol | Rejected: more machinery than name resolution needs | Not blocking |

## Contract and component ownership

- Pure core: name shape validation, binding selection, generation comparison,
  revocation state, and resolve decision over in-memory facts.
- Shell: the resolve endpoint per entry point, binding persistence, and
  revocation commands.
- Boundary: a resolved handle admits a session only. Payload trust stays with
  signed PathInfo, content checks, and existing admission.

## Decisions

### Decision: Resolution binds a generation

**Choice:** A resolved handle records the binding generation at resolve time.

**Rationale:** Generation fencing already exists for worker and layer
freshness. Binding it at resolve time makes a stale handle detectable without a
new mechanism.

### Decision: Revocation is a retraction, not a deletion

**Choice:** Revocation retracts the binding. Admitted content keeps its
validity.

**Rationale:** Revocation removes future authority. Rewriting past admission
would break content identity and make evidence depend on current policy.

### Decision: Fail closed before payload work

**Choice:** Name validation, binding selection, and generation checks run
before any manifest, checkpoint, or payload handling.

**Rationale:** This preserves the existing bounded-transfer rule that no
payload work happens before session admission.

## Risks / Trade-offs

- One step adds indirection. It returns the same session facts the current
  paths already produce.
- Bindings need persistence. They use the existing durable state path with
  explicit generation.
- A revoked-then-recreated name is a new binding with a new generation.

## Non-Claims

- A resolved handle proves admission at resolve time.
- It does not prove later content identity, output trust, or execution success.

## Current receiver contract and verification handoff

`crunch-remote-core::endpoint` validates complete descriptors against the current
table, exact scope/trust/prefix, half-open expiry and nonzero generation.
`remote_build::endpoint` persists tombstones, reloads under independent binding
locks, and retains a shared lease through receiver authentication and effects;
revocation cannot race an already-admitted operation or resurrect a stale table.
The builder session retains the same binding through its transfer; there is no
separate production `Transfer`-kind receiver. Configured substituter/base-layer
admission retains its independent signature and content checks, and build
admission retains independent ticket and output checks. The coordination socket
accepts mutation only from the same observed operating-system owner. ADR 0128
records the generation, retraction and fail-closed-early decisions.
The client rejects a resolved descriptor that differs from its resolve request,
a create reply with any generation other than the exact successor, or a reply
with the wrong operation. A resolved descriptor is never bearer authority.

The following real-program smoke is a handoff command, **not an executed
acceptance receipt**. Set `MANTLE_BIN` to the freshly built binary. It exercises
all four binding kinds, repeated resolution, silent mismatched-name denial,
exact revocation, independent-binding preservation and generation recreation.

```sh
MANTLE_BIN=/absolute/path/to/mantle python3 - <<'PY'
import copy, json, os, pathlib, socket, subprocess, tempfile, time
schema = "mantle-endpoint-binding-v1"
with tempfile.TemporaryDirectory(prefix="mantle-binding-smoke-") as temporary:
    root = pathlib.Path(temporary)
    path = str(root / "live.sock")
    daemon = subprocess.Popen([os.environ["MANTLE_BIN"], "--state-dir", str(root / "state"),
                               "remote", "live", "serve", "--socket", path])
    try:
        deadline = time.monotonic() + 10
        while not pathlib.Path(path).exists():
            assert daemon.poll() is None, "coordination daemon exited"
            assert time.monotonic() < deadline, "coordination socket unavailable"
            time.sleep(0.02)
        def exchange(op, **fields):
            with socket.socket(socket.AF_UNIX) as stream:
                stream.settimeout(5)
                stream.connect(path)
                stream.sendall((json.dumps(dict(op=op, schema=schema, **fields)) + "\n").encode())
                raw = stream.makefile("rb").readline(16385)
                assert len(raw) <= 16384, "unbounded endpoint frame"
                return json.loads(raw) if raw else None
        bindings = []
        for kind in ["builder-ticket", "transfer", "substituter", "base-layer"]:
            spec = dict(kind=kind, name="smoke", scope="operator-smoke",
                        trust_requirement="existing-source-specific-policy",
                        store_prefix="/mantle/store", expires_unix_s=int(time.time()) + 300)
            reply = exchange("create", spec=spec, expected_previous_generation=None)
            assert reply["op"] == "resolved" and reply["schema"] == schema
            binding = reply["binding"]
            assert binding == dict(spec=spec, generation=1)
            for _ in range(2):
                assert exchange("resolve", binding=binding)["binding"] == binding
            bindings.append(binding)
        old = bindings[0]
        forged = copy.deepcopy(old)
        forged["spec"]["trust_requirement"] = "different-policy"
        assert exchange("resolve", binding=forged) is None
        assert exchange("revoke", binding=old) == dict(op="revoked", schema=schema)
        assert exchange("resolve", binding=old) is None
        for unaffected in bindings[1:]:
            assert exchange("resolve", binding=unaffected)["binding"] == unaffected
        new = exchange("create", spec=old["spec"], expected_previous_generation=1)["binding"]
        assert new == dict(spec=old["spec"], generation=2)
        assert exchange("resolve", binding=old) is None
        assert exchange("resolve", binding=new)["binding"] == new
        print("four endpoint kinds resolved; exact revocation isolated; generation 1 fenced by 2")
    finally:
        daemon.terminate()
        daemon.wait(timeout=10)
PY
```

Run the existing `endpoint_binding_tests` production wire fixtures and
`remote_stdio_cli` integration target separately to prove rejection occurs before
manifest reads, ticket spending, output frames and actual remote execution.
The smoke above does not authenticate work or verify admitted content.
