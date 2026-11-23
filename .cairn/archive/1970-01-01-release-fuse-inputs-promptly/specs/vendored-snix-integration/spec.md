## ADDED Requirements

### Requirement: An unmounted castore inputs filesystem releases its FUSE threads

r[vendored_snix.fuse_unmount_release] Mantle's vendored `FuseDaemon::unmount` MUST NOT wait for a
reference it does not own. After the session's unmount succeeds, it MUST wake every FUSE channel
before joining the server threads, so each channel leaves its request loop and closes its
`/dev/fuse` descriptor. Because the unmount is lazy, this wake is what aborts the FUSE connection;
the join MUST therefore return independently of any remaining reference to the detached mount. The
wake MUST NOT be conditional on Mantle observing or owning such a reference: the holder can be a
private mount copy in another process's namespace — for example the Nix daemon's saved sandbox
namespace — which Mantle neither sees nor controls. A successful unmount MUST still return `Ok(())`
and a failed unmount MUST still return the existing `io::Error`; the wake MUST happen only after a
successful unmount.

A stale reference to the unmounted filesystem MUST observe a disconnected filesystem: reading
through a descriptor opened before the unmount MUST fail with `ENOTCONN` rather than block or
return stale data. The bound MUST be a named constant in the regression test, and the test MUST NOT
hang when the unmount does not return within it.

#### Scenario: The inputs filesystem is still referenced

- GIVEN a mounted castore filesystem with a file whose descriptor the test keeps open
- WHEN the daemon is unmounted on another thread while that descriptor is held
- THEN `unmount()` MUST return within the named bound with success
- AND a read through the held descriptor MUST then fail with `ENOTCONN`
- AND the daemon's server threads MUST have exited before the bound.

#### Scenario: Nothing references the inputs filesystem

- GIVEN a mounted castore filesystem with no outside reference
- WHEN the daemon is unmounted
- THEN `unmount()` MUST return successfully as before
- AND the existing FUSE test behavior MUST be unchanged.

#### Scenario: The unmount fails

- GIVEN a session whose unmount returns an error
- WHEN the daemon attempts to unmount
- THEN the error MUST be propagated unchanged
- AND no wake or join MUST hide that failure.
