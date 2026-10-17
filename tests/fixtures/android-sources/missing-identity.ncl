let android = import "../../../lib/android.ncl" in
android.bind {
  observed_sha256 = "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=",
  derivation = { name = "must-not-lower", builder = "/bin/sh", args = ["-c", "true"] },
}
