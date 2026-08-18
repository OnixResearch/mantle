# Baseline verification

The focused baseline passed before provenance core changes.

```text
nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import
nix develop -c cargo test -p crunch-store http_closure
nix develop -c cargo test -p crunch-store provenance
```

The HTTP closure rail passed 26 tests. The provenance rail passed 21 tests with
269 filtered tests. The live export then exposed classifier cases not present in
the existing fixtures.

The host had no Guix command. Producer probing used the Nix-provided Guix 1.5.0
only to confirm that live Guix derivations were available. The selected
GuixPkgs route did not require that Guix command.
