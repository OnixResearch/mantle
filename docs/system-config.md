# System configuration

`mantle system` evaluates Nickel service modules against an inventory, merges
per-machine fragments, and optionally assembles those merged configs into normal
mantle derivations.

The phase-1 path is intentionally narrow:

- module and inventory schema live in Nickel contracts
- orchestration, dependency ordering, and merge rules live in Rust
- the first assembler backend is `nixos`
- the `nixos` backend writes `$out/system-config.json`
- the pipeline does not require a Nix runtime dependency

## Layout

A system-config fixture has two pieces:

- an inventory file, usually `inventory.ncl`
- a sibling `modules/` directory containing `*.ncl` service modules

Example tree:

```text
examples/system-config/
├── inventory.ncl
└── modules/
    ├── firewall.ncl
    ├── nginx.ncl
    └── sshd.ncl
```

If `--modules` is omitted, `mantle system eval` and `mantle system build`
default to `./modules/` relative to the inventory path.

## Inventory schema

Inventories are Nickel records validated against `lib/inventory.ncl`.

```nickel
{
  machines = {
    server1 = {
      system = "x86_64-linux",
      class = "nixos",
    },
  },
  services = {
    sshd = {
      instances = [
        {
          machine = "server1",
          role = "default",
          settings = { port = 2222 },
          tags = ["base"],
        },
      ],
    },
  },
}
```

### Machines

Each machine record contains:

- `system`: target system string such as `x86_64-linux`
- `class`: optional assembler/backend name; defaults to `nixos`

### Services

Each service key must match a module file stem in the module directory.

Each instance contains:

- `machine`: machine name from `machines`
- `role`: role name exported by the module's `interface.roles`
- `settings`: optional per-instance settings record
- `tags`: optional string tags

## Module schema

Modules are Nickel records validated against `lib/system_module.ncl`.

```nickel
let systemModule = import "../../../lib/system_module.ncl" in
(systemModule)
& {
  interface = {
    roles = {
      default = {
        port | Number = 22,
      },
    },
  },
  impl = fun { settings, machine_name, role_name, upstream, providers } => {
    output = {
      exports = {
        sshd_port = settings.port,
      },
      nixos = {
        services = {
          openssh = {
            enable = true,
            ports = [settings.port],
          },
        },
      },
    },
  },
}
```

Optional module metadata:

- `inputs = ["other-module"]`
- `consumes_providers = ["provider-type"]`
- `produces_providers = ["provider-type"]`
- `priority = 1000`

### `impl` arguments

Each module implementation receives one record with these fields:

- `settings`: merged instance settings for the chosen role
- `machine_name`: selected machine name
- `role_name`: selected role name
- `upstream`: exports from declared input modules, keyed by module name
- `providers`: arrays of provider objects, keyed by provider type

### `output` shape

The implementation returns a record containing `output`.

Important fields under `output`:

- `exports`: values exposed to downstream `upstream.<module>` lookups
- `providers`: provider objects grouped by provider type
- `nixos`: backend-specific config namespace consumed by the phase-1 `nixos`
  assembler

The collector preserves namespaces such as `output.nixos` and `output.files`
without interpretation; only the selected assembler interprets its own
namespace.

## Commands

### Evaluate

```bash
mantle system eval <inventory.ncl>
```

This runs inventory validation, module loading, evaluator ordering, fragment
merging, and assembler dry-run. The default stdout format is JSON.

Useful flags:

```bash
--modules <dir>         Module directory; default is sibling ./modules
--machine <name>        Repeatable machine filter
--assembler <name>      Override backend for the selected machines
--stop-after fragments  Stop after merged fragment collection
--stop-after derivations
--format json|nickel
--json                  Keep stdout as the main result; encode stderr diagnostics as JSON lines
```

Notes:

- `--stop-after fragments` skips assembler lookup even if `--assembler` is
  present.
- `--format nickel` is reserved but currently returns a clear unimplemented
  error.
- unknown `--format` values fail before evaluation starts.

### Build

```bash
mantle system build <inventory.ncl>
```

This runs the same pipeline through assembler selection, then submits the
assembled derivations to mantle's normal build pipeline.

Useful flags:

```bash
--modules <dir>
--machine <name>
--assembler <name>
--json
```

Under `--json`, stdout is a machine-level envelope. Successful machine entries
embed the existing `crunch-build-report-v1` payloads from `crunch-pipeline`.
Warnings and errors stay on stderr as JSON diagnostic objects.

## Example modules

The checked-in example under `examples/system-config/` demonstrates three module
patterns:

- `sshd.ncl`: no dependencies; exports the selected SSH port
- `firewall.ncl`: produces a `firewall` provider and emits firewall config
- `nginx.ncl`: depends on `sshd`, consumes `firewall`, and emits nginx config

Try it:

```bash
mantle system eval examples/system-config/inventory.ncl
mantle system eval examples/system-config/inventory.ncl --stop-after fragments
mantle system build examples/system-config/inventory.ncl
```

## Backend model

Phase 1 ships one backend:

- `nixos`: reads `output.nixos` from the merged config and emits one derivation
  per machine whose builder writes `$out/system-config.json`

Backend selection order:

1. select machines, including any `--machine` filters
2. if `--assembler <name>` is present, use that backend for every selected
   machine
3. otherwise use `machine.class`, defaulting to `nixos`

## Diagnostics and partial success

The system-config pipeline is intentionally partial-success aware.

- independent machine outcomes stay in stdout even when sibling machines fail
- warnings are tracked separately from errors
- fatal inventory validation failures produce empty stdout and a non-zero exit
- `--json` affects stderr diagnostic encoding, not the primary stdout result

In JSON mode, diagnostic objects use the same shared shape on stdout and stderr:

- `severity`
- `layer`
- `message`
- `detail`
- `machine`
- `module`

## Limits

The current implementation enforces fixed upper bounds, including:

- module count
- dependency-chain depth
- provider fan-out
- merge depth
- machine and instance counts in the inventory

Those limits fail fast with structured diagnostics instead of allowing unbounded
module graphs or fragment trees.
