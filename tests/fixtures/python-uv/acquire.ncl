# Online acquisition ONLY. Export/import/pin these fixed-output sources through
# Mantle's generic source bundle before network-denied environment/wheel builds.
let mantle = import "lib.ncl" in
let pins = import "upstream-wheels.json" in
std.record.map (fun _name pin => mantle.fetchurl {
  name = pin.filename,
  url = pin.url,
  hash = pin.sha256,
}) pins
