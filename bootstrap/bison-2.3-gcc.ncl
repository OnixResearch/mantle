# Standalone conventional GNU Bison 2.3 root.

let crunch = import "lib.ncl" in
let base = crunch.derivationFile "gcc-generator-base.ncl" in
let grep = crunch.derivationFile "grep-2.4-gcc.ncl" in
let m4 = crunch.derivationFile "m4-1.4.7-gcc.ncl" in
(import "bison-2.3-gcc-factory.ncl") base m4 grep
