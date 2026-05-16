# System-config example

This directory is the smallest end-to-end `mantle system` fixture checked into
the repo.

## Files

- `inventory.ncl` — two-machine inventory used by the happy-path CLI tests
- `inventory-partial-failure.ncl` — same shape, but one machine has invalid
  module settings so partial-success behavior is exercised
- `modules/sshd.ncl` — exports the selected SSH port and emits
  `output.nixos.services.openssh`
- `modules/firewall.ncl` — produces a `firewall` provider and emits
  `output.nixos.networking.firewall`
- `modules/nginx.ncl` — depends on `sshd`, consumes `firewall`, and emits
  `output.nixos.services.nginx`
- `bad-modules/bad-contract.ncl` — intentionally invalid module used by the
  negative contract test

## Module authoring pattern

Each module imports the shared Nickel contract from `lib/system_module.ncl`,
then merges in its own schema and implementation:

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

Use these optional fields when needed:

- `inputs` — named module dependencies consumed through `upstream`
- `consumes_providers` — provider arrays consumed through `providers`
- `produces_providers` — provider types produced by this module
- `priority` — merge and graph tiebreak control

## Happy-path example

Evaluate the full fixture to derivations:

```bash
mantle system eval examples/system-config/inventory.ncl
```

Stop after merged fragments:

```bash
mantle system eval examples/system-config/inventory.ncl --stop-after fragments
```

Build the assembled machine derivations:

```bash
mantle system build examples/system-config/inventory.ncl
```

The inventory omits `--modules` because the CLI defaults to the sibling
`modules/` directory next to the inventory file.

### Checked transcript

The maintained transcript quality rail executes this local walkthrough from this
example directory, using isolated store and state directories supplied by the
runner:

```mantle
mantle system eval inventory.ncl --stop-after fragments --machine server1
```

```expect
"server1"
"kind": "fragments"
"machine": "server1"
"port": 8080
```

## Negative-path examples

Partial success:

```bash
mantle --json system eval examples/system-config/inventory-partial-failure.ncl
```

That inventory gives one `sshd` instance a string `port`, so one machine fails
settings validation while the sibling machine still produces a result.

Contract failure:

```bash
mantle system eval examples/system-config/inventory.ncl \
  --modules examples/system-config/bad-modules \
  --machine server1
```

`bad-contract.ncl` sets `impl = 42`, so module validation fails with a clear
contract or shape diagnostic.
