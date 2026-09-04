# Parameterized typed derivations for the optional Radiance reference tools.
# The live proof uses the Rust shell so ptrace and seccomp bind every exec.
let Derivation = import "../../lib/derivation.ncl" in
let profile = import "../../config/radiance-reference.ncl" in
let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^/[^\\n\\r]*$" value
) in
let Inputs = {
  shell | AbsolutePath,
  mkdir | AbsolutePath,
  cc | AbsolutePath,
  cc_driver | AbsolutePath,
  linker | AbsolutePath,
  crt_dir | AbsolutePath,
  libgcc_dir | AbsolutePath,
  bootstrap_compiler_source | AbsolutePath,
  emulator_source | AbsolutePath,
} in
let join = fun values => std.string.join " " values in
let build_tool = fun inputs build source =>
  {
    name = "radiance-reference-%{build.output_name}",
    builder = inputs.shell,
    args = [
      "-c",
      m%"
        set -eu
        test -x "$CC_DRIVER"
        test -d "$CRT_DIR"
        test -d "$LIBGCC_DIR"
        "$MKDIR" -p "$out/bin" "$TMPDIR/radiance-objects"
        cd "$SOURCE"
        objects=
        index=0
        for source_file in $SOURCES; do
          object="$TMPDIR/radiance-objects/$index.o"
          "$CC" $FLAGS -c "$source_file" -o "$object"
          objects="$objects $object"
          index=$((index + 1))
        done
        stack_size=${LINKER_FLAGS##*=}
        "$LINKER" --threads 1 --hash-style gnu --eh-frame-hdr -m elf_x86_64 -pie \
          -o "$out/bin/$OUTPUT_NAME" \
          "$CRT_DIR/Scrt1.o" "$CRT_DIR/crti.o" "$LIBGCC_DIR/crtbeginS.o" \
          -L "$CRT_DIR" -L "$LIBGCC_DIR" \
          -dynamic-linker "$CRT_DIR/ld-linux-x86-64.so.2" \
          $objects -lgcc -lgcc_eh -lc -lgcc -lgcc_eh \
          "$LIBGCC_DIR/crtendS.o" "$CRT_DIR/crtn.o" \
          -z "stack-size=$stack_size"
      "%
    ],
    env = {
      MKDIR = inputs.mkdir,
      CC = inputs.cc,
      CC_DRIVER = inputs.cc_driver,
      LINKER = inputs.linker,
      CRT_DIR = inputs.crt_dir,
      LIBGCC_DIR = inputs.libgcc_dir,
      SOURCE = source,
      SOURCES = join build.sources,
      FLAGS = join build.flags,
      LINKER_FLAGS = join build.linker_flags,
      OUTPUT_NAME = build.output_name,
      LC_ALL = "C",
      NIX_CC_USE_RESPONSE_FILE = "0",
      SOURCE_DATE_EPOCH = "1",
    },
    inputs = [
      inputs.shell,
      inputs.mkdir,
      inputs.cc,
      inputs.cc_driver,
      inputs.linker,
      inputs.crt_dir,
      inputs.libgcc_dir,
      source,
    ],
    addressing_mode = 'input-addressed,
    sandbox = 'native,
  } | Derivation in

fun inputs =>
  let checked = inputs | Inputs in
  {
    bootstrap_compiler = build_tool checked profile.native_builds.bootstrap_compiler checked.bootstrap_compiler_source,
    emulator = build_tool checked profile.native_builds.emulator checked.emulator_source,
    non_claim = profile.non_claim,
  }
