# V3 mpc output-contract smoke gate

Task-ID: V3
Covers: bootstrap.part.mpc.1.2.1

The output-contract smoke for `mpc-1.2.1` is conditional on V2 producing an output path. V2 has no output path because the part is gated on the absent `gcc-4.7.4` provider.

Future promotion must prove, on a real `mpc-1.2.1` output, that:

- `lib/libmpc.a` exists;
- `include/mpc.h` exists;
- the output was built through declared bootstrap predecessors, not host compiler fallback.

No MPC library/header smoke success is claimed here.
