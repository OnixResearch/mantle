# Build separately, then bind its store output by the native-answer-sys Cargo package ID.
let builders = import "builders/lib.ncl" in
let seed = import "examples/seed.ncl" in
let stdenv = builders.mkStdenv seed in
stdenv.mkDerivation {
  pname = "native-answer",
  version = "0.1",
  finish_gates.version.enabled = false,
  buildInputs = [seed.gcc],
  buildPhase = m%"
    printf 'int native_answer(void) { return 42; }\n' > native-answer.c
    gcc -c -o native-answer.o native-answer.c
    ar rcs libnative_answer.a native-answer.o
  "%,
  installPhase = m%"
    mkdir -p "$out/lib"
    cp libnative_answer.a "$out/lib/"
  "%,
}
