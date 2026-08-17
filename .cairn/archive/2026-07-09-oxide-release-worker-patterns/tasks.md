## Tasks

- [x] [serial] Inventory Tufaceous, Tough, Buildomat, and cancel-safe-futures as reference-only inputs with license, trust, and non-claim notes. r[mantle.verification_evidence.oxide_release_worker.reference_inventory]
- [x] [serial] Specify TUF-style release repository profiles for signed metadata, target manifests, artifact tags, and compatibility checks. r[mantle.verification_evidence.oxide_release_worker.tuf_bundles]
- [x] [serial] Specify Buildomat-style ephemeral worker evidence for input identity, target profile, logs, artifacts, cleanup, and replayable job events. r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers]
- [x] [serial] Specify cancellation-safe async orchestration requirements for worker sessions, output upload, cleanup, and final receipts. r[mantle.verification_evidence.oxide_release_worker.cancel_safety]
- [x] [serial] Add positive fixtures for valid release metadata, artifact tags, worker receipts, and cooperative cancellation. r[mantle.verification_evidence.oxide_release_worker.validation]
- [x] [serial] Add negative fixtures for expired metadata, tag mismatch, untrusted policy override, cleanup failure, mid-upload cancellation, and overclaim text. r[mantle.verification_evidence.oxide_release_worker.validation]
- [x] [serial] Run focused pure tests plus Cairn proposal/design/tasks gates for this change. r[mantle.verification_evidence.oxide_release_worker.validation]
