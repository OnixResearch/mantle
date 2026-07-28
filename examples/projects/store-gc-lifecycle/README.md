# Store roots, restart, and garbage collection

This project creates independent retained and collectible outputs for the local store lifecycle.

Run from this directory with one explicit store/state pair:

```sh
work=$(mktemp -d /tmp/mantle-gc-example.XXXXXX)
mkdir -p "$work"/{store,state}

retained=$(mantle --store "$work/store" --state-dir "$work/state" build .#retained --no-substitute | tail -1)
collectible=$(mantle --store "$work/store" --state-dir "$work/state" build .#collectible --no-substitute | tail -1)

mantle --store "$work/store" --state-dir "$work/state" store pin "/mantle/store/${retained##*/}"
mantle --store "$work/store" --state-dir "$work/state" store unpin "/mantle/store/${collectible##*/}"
mantle --store "$work/store" --state-dir "$work/state" store gc --dry-run
mantle --store "$work/store" --state-dir "$work/state" store gc
```

Each command reopens persistent store state. Thus, the sequence also covers restart behavior.

To inspect lock contention, start this command:

```sh
mantle --store "$work/store" --state-dir "$work/state" build .#lock-holder --no-substitute
```

Run `store gc` from a second terminal before the holder finishes. GC fails fast instead of racing the build.

The workflow validation checks dry-run behavior, retained roots, unreachable-output collection, and mutation locking.

GC evidence applies only to the selected local state directory. It does not prove distributed retention.
