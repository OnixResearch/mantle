# V2 mpc build gate evidence

Task-ID: V2
Covers: bootstrap.part.mpc.1.2.1

## Result

`bootstrap/mpc-1.2.1.ncl` was inspected instead of launching an expensive build against known-missing prerequisite compiler outputs. The derivation directly requires `gcc-4.7.ncl` and its builder resolves `GCC=$(find_input gcc-4.7.4)`. The archived gcc-4.7 runtime-validation evidence states that `bootstrap/gcc-4.7.ncl` itself is gated on a real `gcc-4.0.4` provider and that no gcc-4.7 build output or compiler-smoke success is claimed.

Consequently this part cannot currently produce an MPC output without violating the bootstrap trust boundary by substituting a host/Nix/legacy compiler.

Required transcript field status:

| Field | Status |
|---|---|
| command | deferred: target build gated on absent `gcc-4.7.4` provider |
| prerequisite | blocked: archived `live-bootstrap-gcc-4-7-runtime-validation` records no gcc-4.7 output |
| exit status | not run; prerequisite gate fails before target build |
| output path | none |
| failure class | prerequisite compiler absent / gcc-4.7 negative runtime boundary |
| fallback status | no host GCC, Nix GCC, or legacy-provider substitute accepted |

No `mpc-1.2.1` build success is claimed here.
