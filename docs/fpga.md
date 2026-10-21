# iCE40 synthesis, placement/routing, and packing

This is a separate hardware build adapter, not a simulation mode. The pure
`crunch-fpga-core` crate owns bounded typed inputs, stage identities, netlist pin
admission, and timing/utilization acceptance. `crunch-fpga` owns filesystem
observation and execution, reusing `crunch-hardware-simulation`'s strict
bubblewrap runner. `mantle-fpga` is the explicit frontend command. Neither the
scheduler nor the store needs an HDL-specific branch.

The Rust core boundary consumes owned data: `validate(Profile)` returns the
admitted profile, and `identities(Profile)` returns a `PlannedProfile` carrying
that same profile, `stages`, and the exact synthesis-script/PCF bytes.
`accept_netlist(NetlistRequest)` returns the admitted profile and original
owned netlist bytes; `accept_report(ReportRequest)` returns profile and report.
The shell passes ownership between stages without cloning the profile. JSON
profile, stage-identity and manifest schemas are unchanged.

## Supported contract

The only admitted target is **iCE40HX1K, TQ144** (`ice40-hx1k` / `tq144`) on
x86_64 Linux. The proof fixture is a two-clock registered 2-bit adder containing
the **same** `tiny_adder.sv` used by the existing simulation fixture. It uses
clock pin 21 at 10 MHz, inputs 78–81, and outputs 99, 98, 97. These are package
constraints for the build fixture, not authorization to connect or program a
board. There is no USB/JTAG, device programming, proprietary tool, or board
provisioning operation.

The supported synthesis cohort is:

| Role | Version | Exact proof seed root under `/nix/store/` |
|---|---|---|
| Yosys | 0.68 | `d4q316mks6inhm54c19lqgw2c7snm5hk-yosys-0.68` |
| nextpnr-ice40 | 0.10 | `zhalr5ayq69gq0bia0nj53pdnadxz86f-nextpnr-0.10` |
| IceStorm icepack | 0-unstable-2025-06-03 | `n7hvzj54m1rf80mzrzqn4mg8axmc4mgf-icestorm-0-unstable-2025-06-03` |
| Static BusyBox shell | 1.37.0 | `8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0` |
| bubblewrap | 0.11.0 | `csk28n2yj6pzwkslf3mn2gdxz33xxazr-bubblewrap-0.11.0` |

**The iCE40 database is embedded in this nextpnr executable.** nextpnr 0.10's
iCE40 frontend does not accept an external `--chipdb`. The typed database
identity therefore names HX1K/TQ144 and the BLAKE3 of the exact
`nextpnr-ice40` binary carrying that database. A different device, package, or
mismatched database digest fails admission. Changing the embedded database
requires a correspondingly changed binary pin; it invalidates P&R and packing,
not synthesis. An unrelated database file cannot be substituted or silently
ignored. This is not a second external database format.

All tools declare their executable content hash and a complete sorted list of
immutable Nix closure roots, including host runtime dependencies and Yosys's
ABC implementation. Binaries are measured with streaming BLAKE3. The adapter
requires the declared closure to exist locally; it does not resolve, fetch,
substitute, or search ambient tools. Closure paths rely on the existing Nix
store's immutability/authority, not a new source-bootstrap claim.

## Typed inputs and execution

`lib/fpga.ncl` exports `FileInput`, `Tool`, `Clock`, and `Profile`. The Rust
profile rejects unknown fields and performs semantic validation again. Inputs
include:

- individually content-pinned RTL and includes, include directories, top,
  numeric defines, and unsigned parameters;
- the exact pinned synthesis script, typed pre-synthesis passes (`check`, `opt`,
  `clean`), and flattening choice;
- exact part/package, complete pin mapping, clock ports, exact routed clock
  report names, frequency goals in kHz, and a pinned PCF file;
- placement seed, **one thread**, exact tools/closures/database, utilization
  limits, timeout and output/log bounds.

The script and PCF must match the deterministic recipes generated from the
profile byte for byte. This deliberately supports one auditable synthesis
recipe rather than arbitrary shell/Tcl hooks or timing-waiver options.
`tests/fixtures/fpga/synth.ys` and `pins.pcf` are the concrete recipe. Yosys 0.68
flattens by default; disabling flattening uses its supported `-noflatten`
option. The fixture explicitly declares `WIDTH=2`, `FPGA_BUILD=1`, and the
`include/config.vh` input.

Sources are checked against their pins and copied into fresh stage directories
before execution. Each stage uses `--unshare-all`, a cleared environment, no
host root bind, only declared read-only tool roots, a declared `/bin/sh`, and
its own writable `/build`. No earlier stage directory is mounted writable in
a later stage. The adapter copies only the checked netlist into P&R and the
ASC into packing. Linux user namespaces and bubblewrap must actually work.

The environment fixes locale, timezone, source date epoch, Python hash seed,
OpenMP/OpenBLAS threading, home and temporary paths. nextpnr receives an
explicit nonzero seed, `--threads 1`, heap placer, and router1. The shell
sets a per-file size limit and disables core dumps. Stage completion checks
regular-file types, aggregate bytes and entry count; logs have a separate
read bound. Each action also has a wall-clock timeout. These are bounded
small-fixture actions, not a claim of complete hostile-tool resource isolation
or a general HDL compilation service.

## Acceptance is more than exit zero

After synthesis, the adapter parses the actual Yosys JSON top ports. Every
actual port bit must have exactly one typed PCF pin; missing and phantom pins
fail. Clock ports must be actual input ports. Invalid package pin numbers are
rejected by nextpnr's selected database, and their stage stderr is retained.

P&R writes `report.json` in nextpnr's actual JSON format. The adapter parses
`utilization` (`used`, `available`) and `fmax` (`achieved`, `constraint` in MHz),
not grepped log slogans or caller-supplied success flags. Every declared
clock must appear by its **exact** report name with the declared frequency,
finite positive achieved frequency, and achieved frequency at least the target.
Extra clock reports are rejected as undeclared clocks. Missing/optimized-away
required clocks are failures, not vacuous timing success. All reported resource
usage must fit device capacity, and each declared utilization ceiling must be
reported and met.

The fixed `--timing-allow-fail` nextpnr option is used **only to obtain the full
report on a violating design**. It is not an acceptance waiver: the core checks
actual facts, returns `declared-timing-violation`, and never starts icepack on
failed timing. Failure leaves bounded stage diagnostics but no successful
artifact manifest. Missing PCF files, invalid typed PCF bytes, unsupported parts,
wrong embedded databases, missing required pins/clocks, invalid package pins,
and declared utilization violations similarly fail closed.

Clock constraints model internal synchronous timing only. No external IO
setup/hold, board trace delay, oscillator accuracy, signal integrity, power,
formal equivalence, or physical device behavior is established by these facts.

## Artifacts and identities

A successful `mantle-fpga run PROFILE FRESH_ABSOLUTE_OUTPUT` emits:

- `synthesis/netlist.json` and synthesis logs;
- `place-route/routed.json`, `routed.asc`, and `report.json` containing actual
  timing **and** utilization facts, plus P&R logs;
- `packing/bitstream.bin` and packer logs;
- `manifest.json`: exact typed inputs/cohort, Merkle stage identities, parsed
  report facts, BLAKE3/size of every emitted artifact/log, explicit acceptance
  facts and non-claims.

`mantle-fpga plan PROFILE` emits the three stage identities without executing
tools. RTL/includes/top/defines/parameters/script/Yosys changes invalidate all
stages. PCF/clocks/pins/seed/P&R tool/embedded database changes invalidate P&R
and packing only. Packer changes invalidate packing only. The shell, bwrap
closure, and execution bounds are also bound into the stage graph. This is
frontend planning/evidence; it does not invent a second store or grant unsigned
shared-result publication authority.

## Real proof procedure

No real synthesis proof is claimed merely because this driver or fixture exists.
The acceptance artifact is a successful `proof.json` **from this source**.
Run the following on the admitted scratch host after building the two packages
with the repository toolchain. Keep the binary/source immutable during proof.
The ordinary integrated parent gate owns compilation, formatting, and tests.

```sh
cargo build -p crunch-fpga --bin mantle-fpga
cargo test -p crunch-fpga-core --lib
cargo test -p crunch-hardware-simulation-core -p crunch-hardware-simulation --lib
```

Preparation requires Python 3.9+, an explicitly selected absolute Nix executable
for **offline** store metadata queries, and all exact roots listed above already
admitted locally. The IceStorm output can be obtained separately from the exact
seed derivation
`/nix/store/f0xk56ygxwv0bngk6rxa6j77d0ia27ac-icestorm-0-unstable-2025-06-03.drv`;
fetching/building it is an operator prerequisite, never a fallback inside proof.
Simulation additionally requires the existing Verilator 5.046 and Clang wrapper
21.1.8 roots from [hardware simulation](hardware-simulation.md), GNU Make 4.4.1
at `/nix/store/0xw6y53ijaqwfd9c99wyaqiinychzv1f-gnumake-4.4.1`, and GNU ar 2.44
at `/nix/store/rfp8lhk4dl9syfn64rwb3h3c73426p08-binutils-2.44`.
The offline declaration step explicitly records each tool's recursive runtime
closure, including compiler headers/libraries, Perl, make, archiver, and their
loaders. A missing exact root is a hard prerequisite failure.

With `PYTHON`, `NIX`, and `ADAPTER` set by the operator to absolute executable
paths, and `RUN` to a new writable evidence parent:

```sh
mkdir "$RUN"
"$PYTHON" scripts/prove-fpga.py declare --nix "$NIX" \
  --output "$RUN/declarations.json"
"$PYTHON" scripts/prove-fpga.py pin --adapter "$ADAPTER" \
  --inputs "$RUN/declarations.json" --output "$RUN/cohort.json"
"$PYTHON" scripts/prove-fpga.py prove --adapter "$ADAPTER" \
  --inputs "$RUN/cohort.json" --repo "$PWD" --output "$RUN/proof"
```

`declare` performs offline observation only; `pin` records and validates binary
BLAKE3 pins for those exact declarations. Neither is proof. Review/freeze
`cohort.json` before `prove`. `prove` never calls Nix or the network, never
rediscovers tool versions/paths, never falls back to host commands, and requires
a fresh directory. It:

1. Produces a real bitstream through Yosys → nextpnr-ice40 → icepack.
2. Builds again from fresh stage directories and compares **bitstream bytes**.
3. Checks stage-specific invalidation for RTL, constraints, seed, Yosys,
   embedded database/nextpnr, and packer.
4. Exercises unsupported part, wrong database, absent/invalid PCF, missing
   pin/clock declarations, actual unconstrained required pin, invalid package
   pin, missing reported required clock, and an actual declared 1 GHz timing
   failure. No failure may publish a success manifest.
5. Builds/runs the existing simulation fixture for `0,0 → 0`, `1,2 → 3`, and
   `255,255 → 510`; independently builds/runs the registered FPGA wrapper
   against all 16 two-bit input pairs after two clocks.
6. Writes `proof.json` with the actual nextpnr facts, bitstream hash/size,
   invalidation/rejection observations, separate simulation observations and
   explicit non-claims. Missing tools and unexpected diagnostics are fatal.

Repeat equality applies only to these exact sources, scripts, complete cohort,
seed/thread settings, selected architecture, and two clean executions. Neither
cross-version nor cross-host-architecture reproducibility is inferred. Logs may
contain diagnostic differences and are not compared as bitstream determinism.

## Minimal Nickel example

After the proof driver writes its admitted JSON profile, a consumer can apply
this module directly (or use the embedded `fpga` stdlib export):

```nickel
let fpga = import "fpga.ncl" in
(import "baseline.json") | fpga.Profile
```

Here `baseline.json` is the concrete file produced by the driver, not an example
with invented executable hashes. `tests/fixtures/fpga/profile.ncl` is the same
contract as a reusable function of an admitted immutable profile. Export the
result with pinned Nickel 1.17.0, then pass that JSON to `mantle-fpga plan` or
`run`. Evaluation alone does not run tools or establish any acceptance claim.

## Claim separation

The synthesis manifest always records `simulation_validated=false` and
`physical_board_validated=false`. The proof bundle's independent simulation
section does not rewrite these artifact claims. Bitstream production,
tool-model timing acceptance, functional simulation observations, and physical
board validation are four different facts. None implies the others.
