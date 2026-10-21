# Bind only already-admitted immutable roots; see docs/zephyr-firmware.md.
# `bindings` has the exact { plan, runner } shape, with independent tree hashes.
fun bindings => (import "../../../lib/zephyr.ncl").mkFirmware bindings
