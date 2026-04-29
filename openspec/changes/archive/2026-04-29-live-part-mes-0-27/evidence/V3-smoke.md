Task-ID: V3
Covers: bootstrap.part.mes.0.27

# Mes 0.27 smoke evidence

Command:

```sh
PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:$PATH" \
CARGO_TARGET_DIR=$PWD/target/cargo-script \
cargo -Zscript /tmp/mes_smoke.rs \
  target/live-part-mes-0-27/store/anqp2lm1qgppznmndhgj7ighp8fbx6wn-mes
```

Result: PASS.

Transcript: `evidence/V3-smoke-full.log`.

Assertions:

- `bin/mes-m2` exists and runs with `MES_PREFIX` + `GUILE_LOAD_PATH`.
- `bin/mescc.scm` exists and Mes can show mescc help containing `C99 compiler in Scheme`.
- `mescc.scm` compiles a trivial C source (`int main() { return 0; }`) with `-S` and emits output containing the `:main` label.
- Required library/object files are non-empty:
  - `lib/x86_64-mes/x86_64.M1`
  - `lib/x86_64-mes/crt1.o`
  - `lib/x86_64-mes/libmescc.a`
  - `lib/x86_64-mes/libc.a`
  - `lib/x86_64-mes/libc+tcc.a`
  - `lib/x86_64-mes/libtcc1.a`
  - `lib/linux/x86_64-mes/elf64-header.hex2`
- Required directories exist:
  - `include/mes-include/`
  - `mes/module/`
  - `lib/M2libc/`
- Positive execution smoke: `(display "ok")` prints `ok` under `mes-m2`.
