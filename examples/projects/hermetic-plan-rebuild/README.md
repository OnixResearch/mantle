# Hermetic plan and rebuild

This project separates advisory planning from execution, then rebuilds the same content-addressed package in two fresh stores under strict hermetic mode.

```sh
work=$(mktemp -d /tmp/mantle-hermetic-example.XXXXXX)
mkdir -p "$work"/{plan-store,plan-state,first-store,first-state,second-store,second-state}

mantle --json --store "$work/plan-store" --state-dir "$work/plan-state" \
  build --plan --strict-hermetic --no-substitute .#payload > "$work/plan.json"

MANTLE_EXAMPLE_HOST_TOKEN=first-host-value \
mantle --json --store "$work/first-store" --state-dir "$work/first-state" \
  build --strict-hermetic --no-substitute .#payload > "$work/first.json"

MANTLE_EXAMPLE_HOST_TOKEN=second-host-value \
mantle --json --store "$work/second-store" --state-dir "$work/second-state" \
  build --strict-hermetic --no-substitute .#payload > "$work/second.json"
```

The plan reports the same derivation key later executed but writes no output. Both clean builds report `hermeticity_mode = "strict"`, empty hermeticity audits, matching BLAKE3 child-environment digests, matching logical output identities, and identical `payload.txt` and `environment.txt` bytes despite different poisoned host values.

Negative and boundary checks demonstrate that `--strict-hermetic` conflicts with `--impure`, changing the declared payload changes the planned derivation identity, and selecting `/nix/store` compatibility changes store-scoped derivation identity. A plan is advisory and a matching rebuild proves only the recorded inputs and controls for these executions. It does not prove compiler correctness, universal reproducibility, or release eligibility.
