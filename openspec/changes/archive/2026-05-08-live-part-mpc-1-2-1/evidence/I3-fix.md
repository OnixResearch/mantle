# I3 source/output contract hardening evidence

Task-ID: I3
Covers: bootstrap.part.mpc.1.2.1

- Added source-pin comments beside `mpc_src`: `provenance: fosslinux/live-bootstrap steps/mpc-1.2.1/sources`, `first-consumer: bootstrap/mpc-1.2.1.ncl`, and the upstream `parts.rst` heading mismatch note.
- Hardened the output contract from library-only to `lib/libmpc.a` plus installed public header `include/mpc.h`.
- No suppressed failures or `|| true` paths were found in the required output-contract checks.
