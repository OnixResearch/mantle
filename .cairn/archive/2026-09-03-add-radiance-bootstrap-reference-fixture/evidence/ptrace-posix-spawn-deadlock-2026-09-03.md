# Radiance protected-link diagnostic (2026-09-03)

Connected preparation succeeded:

- source bundle: `/tmp/mantle-radiance-live-source-v1.json`
  BLAKE3 `21f962ce6bf2ba89cbd94758d95abe8418cbafc243cdc81a146693d00f2539f4`
- source cohort: `/tmp/mantle-radiance-live-cohort-v1.json`
  BLAKE3 `80c82a016b6232aa4f8002624fa2fe6296ad3aac9fcf738fdbbad6c424993d4b`

The offline proof then exposed three native-build boundaries:

1. v4: the Nix clang wrapper wrote a response file through `mktemp`; the
   supervisor denied the undeclared coreutils exec. The native-build
   environment now sets `NIX_CC_USE_RESPONSE_FILE=0`.
2. v5: the wrapper's final `exec` resolved the `clang -> clang-21` symlink. The
   supervisor denied the unresolved authority. The operator now supplies the
   canonical driver binary as `--cc-driver`.
3. v6: the bootstrap-compiler link did not complete while clang and a
   multithreaded `ld.lld` descendant were inside one protected root.

## v6 observation

Process state at the stall:

- `mantle` pid 3942876 slept in `wait_for_root_status`;
- clang-21 pid 3943518 blocked in `do_wait`;
- ld.lld pid 3947289 was a traced zombie;
- one ld.lld worker was also a traced zombie;
- the tracer thread remained alive in its bounded poll loop.

This observation did not prove a general `posix_spawn` defect in the V98
supervisor. Later probes narrowed the unsafe shape to the compiler-owned,
multithreaded descendant link. Changing the V98-bound supervisor would have
required a separate successor proof.

## Accepted confinement

The Radiance adapter does not use that descendant-link shape:

- each C translation unit is one protected compiler root;
- only the compiler launcher and canonical compiler driver are allowed in that
  root;
- `ld.lld` is a separate protected root;
- the linker runs with `--threads=1`;
- the CRT and libgcc trees are explicit inputs;
- every root keeps the proof-time network filter.

The temporary env-gated ptrace diagnostic is not part of the accepted code.
The confined shape completed in live proof v14 with 50 allowed events, zero
denied events, and no V98 change or relabel. V14 receipt BLAKE3 is
`325ee17e3e216a0069c7570f02431cda4a47174652c71c6bbe050a250b72a23a`.

V14 proves the execution-shape repair. V15 adds exact compiler-driver,
CRT-tree, and libgcc-tree observations. V16 reruns that complete receipt from
implementation commit `5e35c8a518e871cbbf844598b274ddb7842c9316` before lifecycle closure.
