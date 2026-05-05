# V3 m4 musl link boundary

Focused validation of `bootstrap/m4-1.4.7-musl.ncl` now advances past the earlier direct `src/m4.c` TinyCC segmentation fault.

The derivation now emits compile markers for each m4/lib source, patches the TinyCC-sensitive varargs forwarding wrappers, forces local obstack code, copies objects to root-level names before linking, and reaches a concrete static link boundary.

Latest focused result:

```text
store error: build: nonzero exit code: exit status: 1
compile src/m4
compile src/builtin
compile src/debug
compile src/eval
compile src/format
compile src/freeze
compile src/input
compile src/macro
compile src/output
compile src/path
compile src/symtab
compile lib/cloexec
compile lib/close-stream
compile lib/dup-safer
compile lib/error
compile lib/exitfail
compile lib/fd-safer
compile lib/fopen-safer
compile lib/getopt
compile lib/getopt1
compile lib/mkstemp-safer
compile lib/regex
compile lib/obstack
compile lib/tmpfile-safer
compile lib/verror
compile lib/xalloc-die
compile lib/xasprintf
compile lib/xmalloc
link m4
objects: src-builtin.o src-debug.o src-eval.o src-format.o src-freeze.o src-input.o src-m4.o src-macro.o src-output.o src-path.o src-symtab.o lib-cloexec.o lib-close-stream.o lib-dup-safer.o lib-error.o lib-exitfail.o lib-fd-safer.o lib-fopen-safer.o lib-getopt.o lib-getopt1.o lib-mkstemp-safer.o lib-obstack.o lib-regex.o lib-tmpfile-safer.o lib-verror.o lib-xalloc-die.o lib-xasprintf.o lib-xmalloc.o

tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''
tcc: error: undefined symbol 'undefined symbol '%s''

```

This is a progress boundary, not a completed m4 validation.
