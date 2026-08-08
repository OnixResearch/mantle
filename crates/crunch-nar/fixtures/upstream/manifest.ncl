let Fixture = {
  name | String,
  path | String,
  blake3 | String,
  source_test | String,
} in
let NegativeFixture = {
  name | String,
  source_test | String,
  construction | String,
  expected_error | String,
} in
{
  schema = "mantle-nix-archive-upstream-fixtures-v1",
  upstream = {
    repository = "https://github.com/cachix/nix-archive",
    commit = "14362ab589daa4869bda744d4fbe26a1914b5491",
    package = "nix-archive",
    version = "0.1.0",
    cargo_checksum = "70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1",
    license = "Apache-2.0",
  },
  positive | Array Fixture = [
    {
      name = "regular",
      path = "regular.nar.hex",
      blake3 = "e957e7172e05065e35156c6626f6e9a40e190f427edb9be33c5802e3fd4b699f",
      source_test = "tests/dump_parity.rs::offline_regular_and_executable_goldens_match",
    },
    {
      name = "executable",
      path = "executable.nar.hex",
      blake3 = "9e4897bc6af84f92dcb504adb4ef27afd385e2df5042a5406263741fc9196dc1",
      source_test = "tests/dump_parity.rs::offline_regular_and_executable_goldens_match",
    },
    {
      name = "symlink",
      path = "symlink.nar.hex",
      blake3 = "9d042bf21abd657aca6d483b02aa5e2a418530c9b5b244c71f8b59f5bfeafd95",
      source_test = "tests/dump_parity.rs::offline_symlink_golden_matches",
    },
    {
      name = "directory",
      path = "directory.nar.hex",
      blake3 = "e4c0d9801bc407d8360843bd825398b8c9560b60ca2f4f4f35b054a21f61d651",
      source_test = "tests/dump_parity.rs::offline_directory_golden_matches",
    },
  ],
  negative | Array NegativeFixture = [
    {
      name = "bad-magic",
      source_test = "tests/dump_parity.rs::rejects_malformed_input_without_panicking",
      construction = "ASCII bytes: not a nar",
      expected_error = "BadMagic",
    },
    {
      name = "bad-padding",
      source_test = "tests/dump_parity.rs::rejects_malformed_input_without_panicking",
      construction = "Set byte offset 33 of regular.nar.hex to 1",
      expected_error = "BadPadding",
    },
    {
      name = "invalid-entry-name",
      source_test = "tests/dump_parity.rs::rejects_invalid_entry_names",
      construction = "Generate one directory entry with empty, dot, dot-dot, slash, or NUL name bytes",
      expected_error = "InvalidName",
    },
  ],
}
