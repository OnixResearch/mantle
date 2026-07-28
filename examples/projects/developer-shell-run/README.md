# Developer shell and run loop

This local project packages a small executable and declares `dev` and `minimal` shell profiles.

The default profile is `dev`. Activation is explicit. It does not refresh locks, rewrite project files, or change package identity.

Run from this directory with writable store state:

```bash
work=$(mktemp -d)
mkdir -p "$work/store" "$work/state"

mantle --store "$work/store" --state-dir "$work/state" \
  run .#tool --no-substitute -- Mantle
mantle --store "$work/store" --state-dir "$work/state" \
  shell --no-substitute --command env
mantle --store "$work/store" --state-dir "$work/state" \
  shell .#minimal --no-substitute --command env
```

The package prints `operator-demo: hello, Mantle`. The default shell exports `MANTLE_EXAMPLE_PROFILE=dev`, exports its profile message, and runs a visible hook. The named minimal shell exports `MANTLE_EXAMPLE_PROFILE=minimal` without that hook.

Negative paths:

```bash
mantle --store "$work/store" --state-dir "$work/state" shell .#missing --command env
mantle --store "$work/store" --state-dir "$work/state" run .#tool --no-substitute
```

The missing profile fails during selection, and the executable rejects a missing name argument. Shell activation is convenience evidence only: it does not prove hermeticity, release reproducibility, or that arbitrary ambient tools belong to the declared build closure.
