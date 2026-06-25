# Tasks

- [x] [serial] I1 Define the minimal portable release verification artifact set: copied release bundle, deterministic proof receipt, and deterministic sandbox isolation evidence. r[verification_evidence.portable_release_verification_replay]
- [x] [serial] I2 Run required verification from a fresh scratch root outside the original generated bundle path with `--require-deterministic-release` and `--require-provider-fixed-point-proof`. r[verification_evidence.portable_release_verification_replay]
- [x] [serial] I3 Run a negative replay that withholds required deterministic proof evidence and confirm verification fails closed without downgrading to a weaker claim. r[verification_evidence.portable_release_verification_replay]
- [x] [serial] V1 Record exact positive and negative replay command output, verifier JSON status, and bounded non-claims in tracked Cairn evidence. r[verification_evidence.portable_release_verification_replay]
- [x] [serial] V2 Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and record exact output plus tracked status before committing. r[verification_evidence.portable_release_verification_replay]
