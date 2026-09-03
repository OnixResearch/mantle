# Vendor-type port rejection

Application ports use Mantle-owned contracts. A vendor request cannot replace a workspace observation.

```compile_fail
use mantle_rust_plan_core::StructuralWorkspaceFacts;

fn leak_vendor_request(request: snix_build::buildservice::BuildRequest) -> StructuralWorkspaceFacts {
    request
}
```
