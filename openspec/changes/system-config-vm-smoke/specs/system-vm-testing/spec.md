## ADDED Requirements

### Requirement: VM-1 Crunch fragment output feeds NixOS VM nodes

The project MUST provide a checked-in VM bridge that runs `crunch system eval
<inventory> --stop-after fragments`, reads the resulting JSON envelope, and
turns each selected successful machine's
`machines.<name>.merged_config.data.output.nixos` subtree into a NixOS module
attrset consumable by `pkgs.testers.runNixOSTest`.
ID: systemconfig.vm.testing.bridge.eval.fragments

The bridge MUST reject a selected machine before VM boot starts when the
machine entry is missing, marked failed, lacks `merged_config`, or lacks
`output.nixos`.

#### Scenario: Missing `output.nixos` fails before boot

- GIVEN the VM bridge selects machine `server1`
- AND the crunch JSON envelope for `server1` lacks `merged_config.data.output.nixos`
- WHEN the bridge prepares the node config
- THEN the check fails before any VM boots
- AND the diagnostic names `server1` and the missing field

### Requirement: VM-2 Checked-in system-config fixtures stay the source of truth

The VM smoke suite MUST consume checked-in system-config fixtures through live
crunch evaluation instead of hand-written replacement NixOS modules.
ID: systemconfig.vm.testing.fixtures.live.crunch.output

The initial suite MUST start from `examples/system-config/inventory.ncl` and its
sibling `modules/` directory. Test-only wrappers MAY add observation tools or
headless VM defaults, but they MUST NOT restate module behavior that crunch is
supposed to provide.

#### Scenario: Example inventory boots through live crunch output

- GIVEN the checked-in `examples/system-config/` fixture
- WHEN a VM smoke check runs
- THEN crunch evaluates that inventory and module set to fragments
- AND the VM nodes import the fragments produced by crunch
- AND the test wrapper only adds observation helpers or VM defaults

### Requirement: VM-3 Single-machine service smoke proves boot and activation

The initial VM smoke suite MUST include a single-machine boot path for the
checked-in example inventory that proves both boot completion and service
activation from crunch-produced fragments.
ID: systemconfig.vm.testing.runtime.single.machine.smoke

For the example `server1` machine, the check MUST prove that the VM reaches the
boot target, `openssh` is active with the configured port, and `nginx` responds
on port `8080`.

#### Scenario: `server1` boots with sshd and nginx active

- GIVEN the checked-in example inventory is evaluated through crunch
- WHEN the VM harness boots `server1`
- THEN the machine reaches its boot target
- AND `openssh.service` is active with the configured SSH port
- AND an HTTP request to `http://localhost:8080/` succeeds

### Requirement: VM-4 Multi-machine smoke includes positive and negative network behavior

The initial VM smoke suite MUST include a multi-machine boot path that verifies
both an allowed and a denied cross-machine request derived from the checked-in
example inventory.
ID: systemconfig.vm.testing.runtime.multi.machine.positive.negative

The first multi-machine scenario MUST boot the checked-in `server1` and
`server2` example machines, prove that `server2` can reach `server1`'s nginx
service on TCP port `8080`, and prove that `server1` cannot reach `server2`'s
nginx service on TCP port `8081` because the example firewall for `server2`
allows only TCP port `80`.

#### Scenario: Example firewall distinction survives into runtime

- GIVEN the checked-in example inventory is evaluated through crunch
- WHEN the VM harness boots `server1` and `server2`
- THEN `server2` can fetch `http://server1:8080/`
- AND `server1` cannot fetch `http://server2:8081/`
- AND the denied request is recorded as an expected negative assertion

### Requirement: VM-5 VM smoke is a named heavyweight validation rail

System-config VM smoke checks MUST be exposed as named heavyweight validation
commands that are separate from ordinary first-party Rust edit-time checks.
ID: systemconfig.vm.testing.validation.heavy.rail

The checks MUST be registered in `flake.nix`, documented with their host
prerequisites, and kept out of the ordinary Rust-only edit-time quality path.

#### Scenario: Contributors can discover VM smoke without changing the ordinary rail

- GIVEN a contributor changes system-config modules or inventories
- WHEN they inspect the checked-in validation entry points
- THEN they can find the named VM smoke checks and their prerequisites
- AND the ordinary Rust-only edit-time validation path still excludes them
