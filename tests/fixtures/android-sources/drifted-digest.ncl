let android = import "../../../lib/android.ncl" in
let [jdk, _, _, _] = android.cohort in
android.bind {
  identity = jdk,
  observed_sha256 = "sha256-vTpJZpEuuLMO0NALDNprZUO5SdX/4AvqVMBMgeFWHYg=",
  derivation = { name = "must-not-execute", builder = "/bin/sh", args = ["-c", "true"] },
}
