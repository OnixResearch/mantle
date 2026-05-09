# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.gzip.1.2.4

Audited `bootstrap/gzip-tcc.ncl` against `steps/gzip-1.2.4/pass1.kaem`:

- source pin: `https://ftpmirror.gnu.org/gzip/gzip-1.2.4.tar.gz`
- expected output contract: installed `bin/gzip` and `bin/gunzip`
- intentional Crunch deviations: inline `makecrc` file-output patch and direct object compile/link instead of importing upstream makefile/patch files separately
- output smoke: `--help` for both tools plus a gzip/gunzip roundtrip

No host gzip, Nix-provided gzip, or legacy tool output may be substituted for bootstrap proof.
