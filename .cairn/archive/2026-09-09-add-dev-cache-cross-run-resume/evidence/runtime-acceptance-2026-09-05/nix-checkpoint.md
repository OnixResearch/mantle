# Nix checkpoint

Question: Does the recorded Nickel derivation still block evaluation?

Inspected evidence: `nickel-eval-trace.log` returned the Nickel package derivation. The command fetched `lm8nl097aagikzqh7pbj7m1n3cq0jigx-source` from a configured cache. `nickel-source-observation.log` names `666qf3k9djcfzv6lwcp4l4v7fgck77ng-source.drv` as its valid deriver. Its NAR identity is `sha256-wWwvSMKoP4Q0PkcnWbuJEcLxuiKRoykStWo5E42jjiU=`. Nix requires SHA-256 for this interoperability identity.

Decision: The old Nickel blocker no longer reproduces. The focused Nickel cohort check passed its positive and negative tests. No package pin, lockfile, or source expression changed. These observations do not establish why the old derivation became invalid or identify another process that restored it.

Owner: Mantle verification shell and the configured Nix store.

Next action: Resolve the new import-from-derivation blocker before the final whole-flake evaluation.

## Concrete counterexample and budget extension

The whole-flake replay passed Nickel evaluation but stopped at `packages.x86_64-linux.spacewasm-reference-bundle`. The missing derivation is `dv1lsa10lkdpv9gdv5za341pd8c8c6c1-mantle-spacewasm-bundler-source.drv`.

`nix/spacewasm-reference.nix:445-460` constructs that source through `runCommandLocal`, then reads its Cargo lockfile during evaluation. This is another missing intermediate derivation, not evidence of a changed SpaceWasm source or lockfile.

Permit one ordinary evaluation of all host package derivation paths to materialize their evaluation prerequisites. Then permit one further whole-flake evaluation. Both commands retain the ten-minute bound. This extension addresses the concrete intermediate-source counterexample. It does not authorize package changes, lockfile changes, broad cache removal, or a proof restart.
