# Castore Export Specification

## MODIFIED Requirements

### Requirement: File Export Completeness

The system MUST write the complete blob content when exporting a file node
to disk. After writing, the file size on disk MUST equal the blob's declared
size in the castore.

#### Scenario: Large file export
- GIVEN a build output containing a file node with a 30MB blob
- WHEN `export_castore_to_disk` writes the file to the output directory
- THEN the file on disk has exactly 30MB of content
- AND the content matches the blob's BLAKE3 digest

#### Scenario: Chunked blob reassembly
- GIVEN a blob stored as multiple fastcdc chunks in the blob service
- WHEN `open_read()` is called and the stream is fully consumed
- THEN all chunks are concatenated in order
- AND the total bytes equal the original blob size

### Requirement: Export Error Reporting

The system MUST report errors when file export fails, rather than silently
producing empty or truncated files.

#### Scenario: Write size mismatch
- GIVEN an export where `tokio::io::copy` writes fewer bytes than declared
- WHEN the export function checks the byte count
- THEN it returns `Error::ExportFailed` with the path, expected size, and
  actual bytes written

#### Scenario: Root output export failure
- GIVEN a root build output whose export to disk fails
- WHEN the build pipeline processes the result
- THEN the build is reported as failed (not cached as successful)
