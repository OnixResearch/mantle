# Local stdio remote-build loopback

This workflow creates a one-use bearer ticket, evaluates the project locally, dispatches the concrete payload through Mantle's framed `stdio-once` remote builder, admits the signed result, and inspects redacted status.

Run from this directory on Linux with bubblewrap available.

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

Inspect the build report's route, upload summary, transfer/admission phase, signer or trust basis, artifact attestation reference, and bounded observability fields. Ticket list, inspect, and status output remain secret-redacted; only `ticket reveal` prints the bearer value.

The ticket has one use, so a second dispatch must fail before output admission:

```sh
mantle --json --store "$work/client-store" --state-dir "$work/state" \
  build .#payload --no-substitute \
  --builder gallery-builder --ticket "$ticket"
```

A separately created ticket can be revoked before use with `mantle --state-dir "$work/state" remote ticket revoke <id>`. From the repository root, the protocol and unknown-ticket CLI rail is:

```sh
nix develop -c cargo test -p mantle --test remote_stdio_cli
```

This example proves only local framed-stdio composition with explicit ticket and output-trust seams. It does not prove production P2P deployment, SSH configuration, restart-safe coordination, resumable large-artifact transfer, release reproducibility, or general remote-builder honesty.
