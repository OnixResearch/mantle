use alloc::string::String;
use alloc::string::ToString;

use crate::STREAM_SCHEMA;
use crate::types::IdentityContext;
use crate::types::OutcomeError;
use crate::types::SelectedRoot;
use crate::types::SourceSequence;

const ROOT_IDENTITY_DOMAIN: &[u8] = b"mantle-evaluation-stream-root-v1\0";
const RUN_IDENTITY_DOMAIN: &[u8] = b"mantle-evaluation-stream-run-v1\0";

pub(crate) fn root_identity(
    context: &IdentityContext,
    label: &str,
    sequence: SourceSequence,
) -> Result<String, OutcomeError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ROOT_IDENTITY_DOMAIN);
    frame_text(&mut hasher, STREAM_SCHEMA)?;
    frame_text(&mut hasher, &context.evaluator_cohort)?;
    frame_text(&mut hasher, &context.source_blake3)?;
    frame_text(&mut hasher, &context.selector)?;
    frame_text(&mut hasher, label)?;
    hasher.update(&sequence.value().to_be_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}

pub(crate) fn run_identity(context: &IdentityContext, roots: &[SelectedRoot]) -> Result<String, OutcomeError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUN_IDENTITY_DOMAIN);
    frame_text(&mut hasher, STREAM_SCHEMA)?;
    frame_text(&mut hasher, &context.evaluator_cohort)?;
    frame_text(&mut hasher, &context.source_blake3)?;
    frame_text(&mut hasher, &context.selector)?;
    for root in roots {
        frame_text(&mut hasher, &root.root_id)?;
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn frame_text(hasher: &mut blake3::Hasher, text: &str) -> Result<(), OutcomeError> {
    let length_bytes = u32::try_from(text.len()).map_err(|_| OutcomeError::ArithmeticOverflow)?;
    hasher.update(&length_bytes.to_be_bytes());
    hasher.update(text.as_bytes());
    Ok(())
}
