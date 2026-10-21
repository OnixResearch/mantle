# Apply this function to the concrete baseline.json emitted by the proof driver.
# Keep this file in its checked-in location so the relative contract import
# resolves; export the applied expression using pinned Nickel 1.17.0.
let fpga = import "../../../lib/fpga.ncl" in
fun admitted_profile => admitted_profile | fpga.Profile
