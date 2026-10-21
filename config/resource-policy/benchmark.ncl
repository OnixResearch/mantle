# Public fixed-input benchmark policy; exported by the driver with Nickel 1.17.0.
# Reviewed source: nixbuild/nixbench b256cd275d8c79ba485be8d317005f973879825a.
# Apache-2.0, Copyright 2025 Immutable Solutions AB; concepts only, no vendored code.
# Mantle owns these original zero-file/packaging implementations (AGPL-3.0-or-later).
{
  schema = "mantle-resource-benchmark-corpus-v1",
  source = {
    reference = "https://github.com/nixbuild/nixbench/tree/b256cd275d8c79ba485be8d317005f973879825a",
    license = "Apache-2.0",
    license_url = "https://github.com/nixbuild/nixbench/blob/b256cd275d8c79ba485be8d317005f973879825a/LICENSE",
    reuse = "Independent derivation seed versus file seed, fixed byte geometry, compression and explicit CPU requirements; not a port of fio or the hosted service.",
  },
  implementation_license = "AGPL-3.0-or-later",
  platform = "x86_64-linux",
  rounds = 3,
  wall_seconds_max = 120,
  address_space_bytes_max = 67108864,
  output_file_bytes_max = 16777216,
  measurement = "BusyBox time wait4 child-tree user/system CPU, maximum RSS and block I/O; monotonic append-only output logical bytes; builder elapsed time. No worker OOM or remote transfer collector.",
  workloads = [
    { id = "zero-file-1m", kind = "write-zero", bytes = 1048576, source_date_epoch = 1, expected = "success-exact-zero-bytes" },
    { id = "zero-file-16m", kind = "write-zero", bytes = 16777216, source_date_epoch = 1, expected = "success-exact-zero-bytes" },
    { id = "package-zero-16m", kind = "gzip-zero", bytes = 16777216, source_date_epoch = 1, expected = "success-exact-zero-bytes-and-gzip-roundtrip" },
  ],
  non_claims = [
    "no future resource sufficiency",
    "no whole-build correctness",
    "no host isolation proof",
    "no fair billing claim",
    "no production performance claim",
    "no simulated service acceptance",
    "synthetic file/compression corpus, not representative Onix system-build performance",
    "observe-only replay does not change actual scheduling or grant machine authority",
  ],
}
