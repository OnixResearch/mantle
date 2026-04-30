## ADDED Requirements

### Requirement: TinyCC 0.9.27 compiles trivial C on amd64
Crunch MUST build `bootstrap/tinycc.ncl` into a TinyCC 0.9.27 output that can compile a trivial C source file to an object on amd64.
ID: bootstrap.part.tinycc.0.9.27.amd64.compile

The output MUST include `bin/tcc`, report `tcc version 0.9.27 (x86_64 Linux)`, complete `tcc -c hello.c -o hello.o` within the bounded smoke timeout, and produce a non-empty object file. The compiler MUST reject malformed C with a controlled nonzero exit rather than a hang, timeout, or segmentation fault. Completion evidence MUST include a source-pin audit transcript, successful `crunch build bootstrap/tinycc.ncl` transcript, version smoke transcript, positive object-compile transcript, malformed-input negative transcript, and host-leakage scan transcript.

#### Scenario: Trivial object compile succeeds

- GIVEN `bootstrap/tinycc.ncl` has been built with the documented bootstrap environment
- WHEN the produced `bin/tcc -c hello.c -o hello.o` runs on `int main(){return 0;}`
- THEN the command exits successfully
- AND `hello.o` exists and is non-empty

#### Scenario: Malformed input fails cleanly

- GIVEN the same produced compiler
- WHEN it compiles malformed C under a bounded timeout
- THEN it exits nonzero
- AND the exit status is not timeout-derived
- AND the exit status is not signal-derived
