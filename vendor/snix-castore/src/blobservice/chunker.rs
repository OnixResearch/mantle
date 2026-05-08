use std::error::Error;
use std::fmt;

use fastcdc::v2020::FastCDC;

/// Physical chunking parameters for a blob chunker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkProfile {
    pub min_chunk_bytes: u32,
    pub avg_chunk_bytes: u32,
    pub max_chunk_bytes: u32,
}

impl ChunkProfile {
    pub fn new(min_chunk_bytes: u32, avg_chunk_bytes: u32, max_chunk_bytes: u32) -> Self {
        Self {
            min_chunk_bytes,
            avg_chunk_bytes,
            max_chunk_bytes,
        }
    }
}

/// A byte range selected by a content-defined chunker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkBoundary {
    pub offset: usize,
    pub length: usize,
}

impl ChunkBoundary {
    pub fn new(offset: usize, length: usize) -> Self {
        Self { offset, length }
    }

    pub fn end(self) -> Option<usize> {
        self.offset.checked_add(self.length)
    }
}

/// Pure boundary selector for finalized blob bytes.
pub trait Chunker {
    fn chunk_boundaries(&self, blob: &[u8], profile: ChunkProfile) -> Result<Vec<ChunkBoundary>, ChunkBoundaryError>;
}

/// Default FastCDC-compatible boundary selector.
#[derive(Debug, Clone, Copy, Default)]
pub struct FastCdcChunker;

impl Chunker for FastCdcChunker {
    fn chunk_boundaries(&self, blob: &[u8], profile: ChunkProfile) -> Result<Vec<ChunkBoundary>, ChunkBoundaryError> {
        validate_profile(profile)?;
        let boundaries = FastCDC::new(blob, profile.min_chunk_bytes, profile.avg_chunk_bytes, profile.max_chunk_bytes)
            .map(|chunk| ChunkBoundary::new(chunk.offset, chunk.length))
            .collect::<Vec<_>>();
        validate_chunk_boundaries(blob.len(), profile, &boundaries)?;
        Ok(boundaries)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkBoundaryError {
    InvalidProfile,
    EmptyBoundaryList,
    ZeroLength {
        index: usize,
    },
    NonContiguous {
        index: usize,
        expected_offset: usize,
        actual_offset: usize,
    },
    BoundaryOverflow {
        index: usize,
    },
    MissingBytes {
        expected_len: usize,
        actual_len: usize,
    },
    NonFinalChunkOutOfBounds {
        index: usize,
        length: usize,
        min: usize,
        max: usize,
    },
}

impl fmt::Display for ChunkBoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfile => write!(f, "chunk profile must satisfy 0 < min < avg < max"),
            Self::EmptyBoundaryList => write!(f, "chunk boundary list must not be empty"),
            Self::ZeroLength { index } => write!(f, "chunk boundary {index} has zero length"),
            Self::NonContiguous {
                index,
                expected_offset,
                actual_offset,
            } => write!(f, "chunk boundary {index} starts at {actual_offset}, expected {expected_offset}"),
            Self::BoundaryOverflow { index } => write!(f, "chunk boundary {index} overflows usize"),
            Self::MissingBytes {
                expected_len,
                actual_len,
            } => write!(f, "chunk boundaries cover {actual_len} bytes, expected {expected_len}"),
            Self::NonFinalChunkOutOfBounds {
                index,
                length,
                min,
                max,
            } => write!(f, "non-final chunk boundary {index} has length {length}, expected {min}..={max}"),
        }
    }
}

impl Error for ChunkBoundaryError {}

pub fn validate_profile(profile: ChunkProfile) -> Result<(), ChunkBoundaryError> {
    if profile.min_chunk_bytes == 0
        || profile.min_chunk_bytes >= profile.avg_chunk_bytes
        || profile.avg_chunk_bytes >= profile.max_chunk_bytes
    {
        return Err(ChunkBoundaryError::InvalidProfile);
    }
    Ok(())
}

pub fn validate_chunk_boundaries(
    blob_len: usize,
    profile: ChunkProfile,
    boundaries: &[ChunkBoundary],
) -> Result<(), ChunkBoundaryError> {
    validate_profile(profile)?;
    if blob_len == 0 {
        return Ok(());
    }
    if boundaries.is_empty() {
        return Err(ChunkBoundaryError::EmptyBoundaryList);
    }

    let min_len = profile.min_chunk_bytes as usize;
    let max_len = profile.max_chunk_bytes as usize;
    let final_index = boundaries.len() - 1;
    let mut expected_offset = 0usize;

    for (index, boundary) in boundaries.iter().copied().enumerate() {
        if boundary.length == 0 {
            return Err(ChunkBoundaryError::ZeroLength { index });
        }
        if boundary.offset != expected_offset {
            return Err(ChunkBoundaryError::NonContiguous {
                index,
                expected_offset,
                actual_offset: boundary.offset,
            });
        }
        let Some(end) = boundary.end() else {
            return Err(ChunkBoundaryError::BoundaryOverflow { index });
        };
        if index != final_index && (boundary.length < min_len || boundary.length > max_len) {
            return Err(ChunkBoundaryError::NonFinalChunkOutOfBounds {
                index,
                length: boundary.length,
                min: min_len,
                max: max_len,
            });
        }
        expected_offset = end;
    }

    if expected_offset != blob_len {
        return Err(ChunkBoundaryError::MissingBytes {
            expected_len: blob_len,
            actual_len: expected_offset,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN_CHUNK_BYTES: u32 = 64;
    const AVG_CHUNK_BYTES: u32 = 256;
    const MAX_CHUNK_BYTES: u32 = 1024;
    const TEST_BLOB_LEN: usize = 8192;

    fn test_profile() -> ChunkProfile {
        ChunkProfile::new(MIN_CHUNK_BYTES, AVG_CHUNK_BYTES, MAX_CHUNK_BYTES)
    }

    fn test_blob() -> Vec<u8> {
        (0..TEST_BLOB_LEN).map(|index| (index % 251) as u8).collect::<Vec<_>>()
    }

    #[test]
    fn fastcdc_default_boundaries_cover_blob() {
        let blob = test_blob();
        let boundaries = FastCdcChunker.chunk_boundaries(&blob, test_profile()).expect("FastCDC boundaries validate");

        validate_chunk_boundaries(blob.len(), test_profile(), &boundaries).expect("boundaries cover blob");
        assert_eq!(boundaries.first().map(|boundary| boundary.offset), Some(0));
        assert_eq!(boundaries.iter().map(|boundary| boundary.length).sum::<usize>(), blob.len());
    }

    #[test]
    fn boundary_validation_accepts_empty_blob_without_chunks() {
        validate_chunk_boundaries(0, test_profile(), &[]).expect("empty blob has no chunk boundaries");
    }

    #[test]
    fn boundary_validation_rejects_gap() {
        let err =
            validate_chunk_boundaries(256, test_profile(), &[ChunkBoundary::new(0, 128), ChunkBoundary::new(129, 127)])
                .expect_err("gap must fail validation");

        assert!(matches!(err, ChunkBoundaryError::NonContiguous {
            index: 1,
            expected_offset: 128,
            actual_offset: 129,
        }));
    }

    #[test]
    fn boundary_validation_rejects_overlap() {
        let err =
            validate_chunk_boundaries(256, test_profile(), &[ChunkBoundary::new(0, 128), ChunkBoundary::new(127, 129)])
                .expect_err("overlap must fail validation");

        assert!(matches!(err, ChunkBoundaryError::NonContiguous {
            index: 1,
            expected_offset: 128,
            actual_offset: 127,
        }));
    }

    #[test]
    fn boundary_validation_rejects_zero_length() {
        let err = validate_chunk_boundaries(128, test_profile(), &[ChunkBoundary::new(0, 0)])
            .expect_err("zero-length boundary must fail validation");

        assert_eq!(err, ChunkBoundaryError::ZeroLength { index: 0 });
    }

    #[test]
    fn boundary_validation_rejects_short_non_final_chunk() {
        let err =
            validate_chunk_boundaries(128, test_profile(), &[ChunkBoundary::new(0, 8), ChunkBoundary::new(8, 120)])
                .expect_err("short non-final boundary must fail validation");

        assert!(matches!(err, ChunkBoundaryError::NonFinalChunkOutOfBounds { index: 0, .. }));
    }
}
