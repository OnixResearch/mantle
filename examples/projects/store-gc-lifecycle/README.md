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

Each command reopens persistent store state, so the sequence also covers restart behavior. To inspect lock contention, start `mantle --store "$work/store" --state-dir "$work/state" build .#lock-holder --no-substitute` and run `store gc` from a second terminal before the holder finishes; GC fails fast rather than racing the build. The repository workflow validation proves dry-run non-mutation, retained-root survival, unreachable-output collection, and fail-fast mutation locking. GC evidence is scoped to the selected local state directory; it is not a distributed retention claim.
