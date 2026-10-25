# Cargo wait diagnostics: tool probes and interrupted receives

Two fixes to the opt-in Cargo wait diagnostics written by
`mantle-rust-wait-wrapper` and its collector. Neither changes ordinary
compilation, output acceptance, scheduling or cache identity.

Cargo's target-information probe reads its source from stdin and lists every
crate type, `proc-macro` included. The wrapper counted that probe as a
proc-macro host compile. It now stays a tool probe: a launch counts as a host
compile only when it names a source file.

The collector waits for wrapper frames with a 25 ms receive timeout. Linux never
restarts such a wait after a signal; it fails with `EINTR`. The collector
treated that failure as fatal. It stopped receiving, the wrappers' later frames
were refused, and the timeline recorded one dropped record and stayed
incomplete. It now retries the receive, as the native collector already did.
In the differential `cohorts` stage this loss had removed one warm sample's
qualification and every frame of one warm-up build.
