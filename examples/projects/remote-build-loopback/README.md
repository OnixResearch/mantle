# Local production stdio remote-build loopback

This project exercises Mantle's production local stdio client/server path. It creates bounded bearer tickets, evaluates concrete project payloads locally, launches `remote serve --binding stdio-once --executor local-build`, streams receiver-demanded inputs and outputs, admits signed results, and exposes redacted status.

Run from this directory on Linux with bubblewrap available.

## One-use ticket and small payload

```sh
work=$(mktemp -d /tmp/mantle-remote-loopback.XXXXXX)
mkdir -p "$work/client-store" "$work/state"
now_unix_s=$(date +%s)
ticket_ttl_secs=3600

created=$(mantle --json --state-dir "$work/state" remote ticket create \
  --display-name gallery-loopback \
  --now-unix-s "$now_unix_s" \
  --ttl-secs "$ticket_ttl_secs" \
  --uses 1)
ticket_id=$(printf '%s' "$created" | jq -r .id)
revealed=$(mantle --json --state-dir "$work/state" remote ticket reveal "$ticket_id")
ticket=$(printf '%s' "$revealed" | jq -r '"\(.id):\(.secret)"')

mantle --json --store "$work/client-store" --state-dir "$work/state" \
  build .#payload --no-substitute \
  --builder gallery-builder --ticket "$ticket" > "$work/build.json"

mantle --json --state-dir "$work/state" remote status \
  --endpoint-id gallery-builder > "$work/status.json"
mantle --json --state-dir "$work/state" remote ticket inspect "$ticket_id"
```

Inspect `.outcomes[0].outputs[0].substitution`. The ordinary production route reports `mode = "streaming"`.

Also inspect the route, upload summary, transfer phase, admission phase, trust basis, artifact reference, and bounded observability fields.

Ticket list, inspect, and status output do not contain the secret. Only `ticket reveal` prints the bearer value.

The ticket has one use, so a second dispatch must fail before output admission:

```sh
mantle --json --store "$work/client-store" --state-dir "$work/state" \
  build .#payload --no-substitute \
  --builder gallery-builder --ticket "$ticket"
```

A separately created ticket can be revoked before use with `mantle --state-dir "$work/state" remote ticket revoke <id>`.

## Deterministic interruption and resume

The `.#resumable-payload` selector preserves the small default example while adding a deterministic repeated-content file large enough to cross multiple bounded production chunks. The canonical operator validation runs the checked-in project through the public build command and real production stdio client/server path:

```sh
# Run from the repository root.
nix develop -c cargo test -p mantle --test remote_transfer_production \
  'gallery_resumable_remote_transfer_' -- --nocapture --test-threads=1
```

The positive fixture:

1. Uses a fixed bounded ticket and fresh client and worker state.
2. Interrupts after one durable output acknowledgement.
3. Verifies that the client admitted no output.
4. Starts fresh client and server processes for the same fenced request.
5. Verifies the same BLAKE3 manifest identity, reused bytes, and missing-chunk progress.
6. Admits one output through the standard signed PathInfo, content, store-prefix, and attestation checks.
7. Checks `payload.txt` and all deterministic `payload.bin` bytes.

The paired negative fixture changes the acknowledged receiver chunk before retry. Resume must fail with `acknowledged-chunk-missing`, leave the client store empty, and emit no output-admission evidence.

`MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS` controls the interruption in debug builds only.

This variable is a deterministic validation seam. It is not a supported release operator control. Release binaries ignore it.

Run the broader production regression set:

```sh
nix develop -c cargo test -p mantle --test remote_transfer_production \
  'production_stdio_' -- --nocapture --test-threads=1
```

That rail also covers interrupted multi-chunk input upload, quota rejection before checkpoint/admission, delta-unavailable full-NAR fallback over bounded chunks, and an 8 MiB output.

## Claim boundary

This example proves local production framed-stdio composition, bounded receiver-driven streaming, fenced checkpoint resume from verified receiver state, repeated-content chunk handling, signed output admission, and redacted status for the checked fixtures. It does not prove exactly-once delivery, arbitrary process-kill recovery, production P2P listening, SSH deployment, independent-machine behavior, general remote-worker honesty, compiler correctness, transfer-as-output-trust, or release reproducibility.
