#!/bin/sh
# Ordinary strict derivation builder. All executable/source/runtime roots are declared inputs.
set -eu
: "${RUSTC:?admitted pinned rustc executable}"
: "${SOURCE:?admitted fixture source file}"
: "${LINKER:?admitted native linker executable}"
: "${MKDIR:?admitted mkdir executable}"
: "${out:?Mantle output root}"
"$MKDIR" "$out"
"$RUSTC" --test "$SOURCE" --edition=2021 -C instrument-coverage -C codegen-units=1 \
  -C "linker=$LINKER" -o "$out/fixture"
"$RUSTC" -vV > "$out/rustc-vv.txt"
