## Implementation

- [x] [serial] I1 Define the pure repeatability matrix profile, cell plan, verdict, and report schema over release bundle metadata and declared axes. r[verification_evidence.release_repeatability_matrix]
- [x] [serial] I2 Implement the CLI shell that runs matrix cells in isolated output/store roots with recorded cache, environment, temp, user, and host-class controls. r[verification_evidence.release_repeatability_matrix]
- [x] [serial] I3 Wire matrix report output into release/global reproducibility evidence without bypassing the existing global admission evaluator. r[verification_evidence.release_repeatability_matrix]

## Verification

- [x] [serial] V1 Positive: run a small matrix where every clean fresh-store cell rebuilds the same release artifact digest and the report verdict is matched. r[verification_evidence.release_repeatability_matrix]
- [x] [serial] V2 Negative: force one cell to produce a mismatched or missing output and prove the matrix report records the blocker and global admission stays blocked. r[verification_evidence.release_repeatability_matrix]
- [x] [serial] V3 Validate stale/reused-store handling, run focused tests, format/check touched files, and run Cairn validate/gates for this change. r[verification_evidence.release_repeatability_matrix]
