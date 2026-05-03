# TinyCC 0.9.27 static runtime REX/GOT repair diagnostic (rexgot-r2)

Task-ID: live-part-make-3-82-runtime-validation diagnostic follow-up
Covers: V2/V3 blocker narrowing for `bootstrap/make-tcc.ncl` via focused `bootstrap/diag-tcc27-static-runtime-inspect.ncl`.

## Change under test

`bootstrap/tinycc.ncl` now includes a focused TinyCC 0.9.27/Mes static-executable repair slice:

- static executable PLT-backed GOT entries remember `got_offset` so `fill_got_entry()` can materialize them without a dynamic loader;
- `fill_local_got_entries()` is guarded for missing GOT relocations and can resolve materialized static executable dynamic-relocation symbols back through the relocation section link;
- 64-bit little-endian helpers are byte-copy based (`memcpy`) to avoid predecessor shift-helper miscompilation;
- `REX_BASE` uses a bit test (`reg & 8`) rather than `reg >= 8`, because the latter treats `TREG_MEM|RCX` as a high register and emitted `mov (%r9),%rax` for `global_words[i]`.

## Command

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store" \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state" \
  bootstrap validate bootstrap/diag-tcc27-static-runtime-inspect.ncl \
  --warmup bootstrap/tinycc.ncl \
  --resume \
  --evidence-dir target/bootstrap-validation/diag-tcc27-static-runtime-rexgot-r2
```

## Result

- Validation runner: `status=passed`; warmup `bootstrap/tinycc.ncl` passed; diagnostic build/link passed.
- Host runtime probe of produced executable: `rc=21`. This is progress from the earlier `rc=139` startup/GOT segfault: the binary now prints `diag-tcc27-static-runtime`, the static function pointer slot is materialized, and PLT-backed GOT slots contain nonzero code addresses.
- Remaining blocker: functional runtime mismatch in `helper_value("beta")`, likely in the Mes/TinyCC varargs/string formatting path (`sprintf(buf, "score=%d", ...)`) rather than static ELF startup or PLT/GOT zeroing.

## Evidence

- Runner summary: `diag-tcc27-static-runtime-rexgot-r2-validation-summary.json` / `diag-tcc27-static-runtime-rexgot-r2-validation-summary.md`
- Build stdout/stderr: `diag-tcc27-static-runtime-rexgot-r2-build.stdout.log`, `diag-tcc27-static-runtime-rexgot-r2-build.stderr.log`
- TinyCC warmup logs: `diag-tcc27-static-runtime-rexgot-r2-warmup-tinycc.stdout.log`, `diag-tcc27-static-runtime-rexgot-r2-warmup-tinycc.stderr.log`
- Derivation log: `diag-tcc27-static-runtime-rexgot-r2.derivation.log`
- Runtime probe: `diag-tcc27-static-runtime-rexgot-r2-runtime-probe.log`

Output path under local Crunch store:

```text
/home/brittonr/git/crunch/crunch/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store/yjzg5hpqxvs2rav0w10b0d0f650pyhwb-diag-tcc27-static-runtime-inspect
```
