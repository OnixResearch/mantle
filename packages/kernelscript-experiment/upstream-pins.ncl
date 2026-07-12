# Authoritative upstream metadata inspected on 2026-07-12.
#
# The release provides an exact source archive and binary SHA-256, but the
# source tree does not provide opam.locked or another authoritative dependency
# closure. Therefore this file is source-pin data, not compiler-success proof.

{
  schema = "mantle-kernelscript-upstream-pins-v1",
  upstream = "https://github.com/multikernel/kernelscript",
  version = "0.1.2",
  tag = "v0.1.2",
  revision = "0c80d4e4ac0029d34cbc9d65e76d78c075b64555",
  source_archive = {
    url = "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz",
    sha256 = "9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1",
    sha256_sri = "sha256-mgC5bh8SfUgGwosHbycKzcS/SoxVjKY2v9n0kmi0ecE=",
    blake3 = "439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57",
  },
  release_binary = {
    platform = "linux-x86_64",
    sha256 = "fc44f8e3638fb9e5d8db8c1f3f569e2f545c74837b6d1eae1c20e9e1f7f0e437",
    accepted_as_compiler_closure = false,
  },
  compiler_dependency_lock = {
    required_format = "opam-lock",
    authoritative_lock_available = false,
    blocker = "upstream v0.1.2 has kernelscript.opam but no opam.locked; release and CI workflows resolve mutable opam repositories",
  },
  non_claims = [
    "not-compiler-build-success",
    "not-codegen-success",
    "not-target-cohort-evidence",
    "not-release-binary-trust",
  ],
}
