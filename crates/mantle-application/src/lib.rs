#![no_std]
#![doc = r#"
# Application port compile-fail cases

## Missing family port

```compile_fail
use mantle_application::BuildOperationPort;

struct BuildOnly;

impl BuildOperationPort for BuildOnly {
    fn execute_build_operation(
        &mut self,
        effect: mantle_application_core::ApplicationEffect,
    ) -> Result<mantle_application_core::ApplicationObservation, mantle_application::ApplicationPortError> {
        Ok(mantle_application_core::successful_observation(&effect))
    }
}

let command: mantle_application_core::ApplicationCommand = todo!();
let mut incomplete_ports = BuildOnly;
let _ = mantle_application::run(command, &mut incomplete_ports);
```

Application dispatch requires the complete explicit family-port set. It does not discover a missing port from global state.

## Vendor type in a port

```compile_fail
use mantle_application::BuildOperationPort;

trait InvalidBuildPort: BuildOperationPort {
    fn execute_vendor(&mut self, request: snix_store::path_info::PathInfo);
}
```

Application ports accept Mantle-owned effects and observations. A Snix `PathInfo` must remain in an outer adapter.
"#]
//! Capability-scoped application operations for Mantle command families.

// r[impl application_architecture.application_owned_ports]
// r[impl application_architecture.typed_error_ownership]
// r[impl application_architecture.effect_observation_boundary]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod application;
mod ports;

pub use application::*;
pub use ports::*;

#[cfg(test)]
mod tests;
