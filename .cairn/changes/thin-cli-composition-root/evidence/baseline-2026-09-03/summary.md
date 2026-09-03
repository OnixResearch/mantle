# Clean baseline

Source commit: `585776295972896136907c9000533cde38f86620`

- The Mantle binary built successfully.
- The 12-case CLI family parity capture completed. Ten help cases returned status 0. Two invalid-input cases returned status 2.
- Root unit tests: 2,497 passed, 4 failed, and 72 remained ignored.
- The four baseline failures are unrelated host-sensitive fixtures:
  - fake Slurm process spawn returned `external-batch-process-spawn-failed`;
  - the OCI special-file assertion received a different host error;
  - first-stage mrustc could not find host `dirname`;
  - the known seccomp descendant test ended with `adopted StageX descendants did not exit within 30000 ms`.
- Cairn validation reports `"valid": true`.

The exact failed output is preserved. Final acceptance uses focused command parity and architecture checks plus a comparison against this baseline.
