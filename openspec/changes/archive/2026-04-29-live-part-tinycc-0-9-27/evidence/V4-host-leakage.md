Task-ID: V4
Covers: bootstrap.part.tinycc.0.9.27

# tinycc 0.9.27 host-leakage scan

Result: PASS.

Full transcript: `evidence/V4-host-leakage-full.log`.

Scans run:

- absolute host path scan over `bootstrap/tinycc.ncl` for `/usr/bin`, `/usr/local`, `/run/current-system`, `/home/brittonr`, `~/`, and `/nix/store`;
- unqualified utility scan over `bootstrap/tinycc.ncl` for common shell tools, excluding explicit BusyBox applet calls through `$BB`;
- build transcript hermeticity summary check.

Transcript excerpt:

```text
$ grep for absolute host paths in bootstrap/tinycc.ncl
$ grep for unqualified shell utilities in bootstrap/tinycc.ncl (excluding declared tcc tools and shell builtins)
$ build hermeticity summary
39:hermeticity: practical (no degraded facts)
$ undeclared host path scan result: none in derivation body
```

Conclusion: no undeclared host-tool, host-path, or degraded hermeticity facts were found for this derivation body. The build transcript contains host filesystem paths only as external evidence/store locations emitted by the test command, not as derivation inputs.
