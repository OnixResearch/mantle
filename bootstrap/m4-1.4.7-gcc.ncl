# Standalone conventional GNU M4 1.4.7 root.

let crunch = import "lib.ncl" in
let base = crunch.derivationFile "gcc-generator-base.ncl" in
let grep = crunch.derivationFile "grep-2.4-gcc.ncl" in
(import "m4-1.4.7-gcc-factory.ncl") base grep
