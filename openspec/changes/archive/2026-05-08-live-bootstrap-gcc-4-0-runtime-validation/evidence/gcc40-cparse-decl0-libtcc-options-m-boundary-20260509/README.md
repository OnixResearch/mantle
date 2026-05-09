# GCC 4.0 c-parse decl0: libtcc post-copy options_m preprocessor boundary

Date: 2026-05-09

## Context

This continues the focused restored-output local replay after these predecessor compile triggers were bypassed:

- `tcc_compile()` `CONFIG_TCC_ASM` island
- `tcc_new()` `sscanf(TCC_VERSION, "%d.%d.%d", &a, &b, &c)`
- `tcc_set_linker()` field helper calls shaped like `copy_linker_arg(&s->fini_symbol, p, 0)`

The replay uses `.crunch-drain/post621-restored-store` bound as `/crunch/store` inside bwrap. Evidence remains a focused predecessor-compile replay against restored outputs, not a fresh Crunch derivation runtime-marker proof.

## Evidence files

- `../gcc40-cparse-decl0-libtcc-options-f-prefix-20260509/focused-local-options-f-prefix.txt`
  - After removing the four `copy_linker_arg(&s->...)` field-call sites, a generated `options_f[]` prefix with all six entries compiles under both flag sets.
  - This avoids treating earlier nonmonotonic/incomplete raw line prefixes around the `options_f` area as the next semantic trigger.
- `../gcc40-cparse-decl0-libtcc-options-m-pair-20260509/focused-local-options-m-pair.txt`
  - The base through generated `options_f[]` compiles.
  - Individual `options_m[]` entries compile.
  - A two-entry `options_m[]` without the `#ifdef TCC_TARGET_X86_64` island compiles.
  - The raw upstream `options_m[]` block with the `#ifdef TCC_TARGET_X86_64` island segfaults.

## Boundary

The next local predecessor compile trigger after the `copy_linker_arg` field-call bypass is the preprocessor island inside the static `options_m[]` initializer:

```c
static const FlagDef options_m[] = {
    { offsetof(TCCState, ms_bitfields), 0, "ms-bitfields" },
#ifdef TCC_TARGET_X86_64
    { offsetof(TCCState, nosse), FD_INVERT, "sse" },
#endif
    { 0, 0, NULL }
};
```

The failing shape is not either entry alone and not the two-entry array itself: the pair without the target-gated preprocessor lines compiles. The failing boundary is the `#ifdef TCC_TARGET_X86_64` island embedded in the static initializer.

## Key transcript lines

```text
diag-libtcc-post894: compile options_m_pair_no_ifdef_common ... rc=0
diag-libtcc-post894: compile options_m_full_common ... rc=139
diag-libtcc-post894: compile options_m_pair_no_ifdef_D_ONE_SOURCE_1 ... rc=0
diag-libtcc-post894: compile options_m_full_D_ONE_SOURCE_1 ... rc=139
```

## Re-run

```sh
BASH=$(command -v bash)
nix shell nixpkgs#bubblewrap -c bwrap \
  --dev /dev --proc /proc --tmpfs /tmp --dir /crunch \
  --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store \
  --ro-bind /nix /nix --ro-bind /usr /usr \
  --ro-bind /run/current-system /run/current-system \
  --ro-bind /home /home --chdir "$PWD" "$BASH" \
  openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-options-m-pair-20260509/run-local-options-m-pair.sh
```
