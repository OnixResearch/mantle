# Standalone conventional GNU Awk 3.0.4 root.

let crunch = import "lib.ncl" in
let base = crunch.derivationFile "gcc-generator-base.ncl" in
let grep = crunch.derivationFile "grep-2.4-gcc.ncl" in
(import "gawk-3.0.4-gcc-factory.ncl") base grep
