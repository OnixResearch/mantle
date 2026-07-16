use std::error::Error;

use crunch_delta::ArtifactNode;
use crunch_delta::BlobNode;
use crunch_delta::ChunkRef;
use crunch_delta::ClosureFixture;
use crunch_delta::ContentCatalog;
use crunch_delta::DeltaAcceptanceMode;
use crunch_delta::DeltaFallbackReason;
use crunch_delta::DeltaFetchRequest;
use crunch_delta::DeltaReceiverState;
use crunch_delta::DeltaSubstitutionError;
use crunch_delta::InMemoryDeltaAuthority;
use crunch_delta::NegotiationOffer;
use crunch_delta::OutputFixture;
use crunch_delta::substitute_from_authority;
use nix_compat::store_path::StorePath;
use snix_castore::B3Digest;
use snix_castore::Node;
use snix_store::path_info::PathInfo;

const STORE_PREFIX: &str = "/nix/store";
const OUTPUT_NAME: &str = "out";
const STORE_PATH_DIGEST_BYTE: u8 = 7;
const NAR_SHA256_BYTE: u8 = 9;
const STORE_PATH_DIGEST_BYTES: usize = 20;
const NAR_SHA256_BYTES: usize = 32;
const KEYPAIR: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

fn digest(bytes: &[u8]) -> B3Digest {
    blake3::hash(bytes).into()
}

fn make_sender(shared: &[u8], changed: &[u8]) -> Result<(ClosureFixture, B3Digest), Box<dyn Error>> {
    assert!(!shared.is_empty());
    assert!(!changed.is_empty());
    let all_bytes = [shared, changed].concat();
    let blob_digest = digest(&all_bytes);
    let root = ArtifactNode::Blob(BlobNode {
        digest: blob_digest,
        size_bytes: u64::try_from(all_bytes.len())?,
        chunks: vec![
            ChunkRef {
                digest: digest(shared),
                size_bytes: u64::try_from(shared.len())?,
            },
            ChunkRef {
                digest: digest(changed),
                size_bytes: u64::try_from(changed.len())?,
            },
        ],
    });
    Ok((
        ClosureFixture {
            store_prefix: STORE_PREFIX.to_string(),
            outputs: vec![OutputFixture {
                output_id: OUTPUT_NAME.to_string(),
                root,
            }],
        },
        blob_digest,
    ))
}

fn signed_path_info(blob_digest: B3Digest, size_bytes: u64) -> Result<PathInfo, Box<dyn Error>> {
    assert!(size_bytes > 0);
    let keypair = crunch_build::load_keypair(KEYPAIR)?;
    let mut path_info = PathInfo {
        store_path: StorePath::from_name_and_digest_fixed(
            "delta-substitution-demo",
            [STORE_PATH_DIGEST_BYTE; STORE_PATH_DIGEST_BYTES],
        )?,
        node: Node::File {
            digest: blob_digest,
            size: size_bytes,
            executable: false,
        },
        references: vec![],
        nar_size: size_bytes,
        nar_sha256: [NAR_SHA256_BYTE; NAR_SHA256_BYTES],
        signatures: vec![],
        deriver: None,
        ca: None,
    };
    crunch_build::sign_pathinfo_with_store_dir(&mut path_info, &keypair.signing_key, STORE_PREFIX);
    Ok(path_info)
}

fn make_authority(shared: &[u8], changed: &[u8]) -> Result<InMemoryDeltaAuthority, Box<dyn Error>> {
    let (sender, blob_digest) = make_sender(shared, changed)?;
    let all_bytes = [shared, changed].concat();
    let mut catalog = ContentCatalog::default();
    catalog.insert_chunks(blob_digest, vec![shared.to_vec(), changed.to_vec()]);
    catalog.insert_blob(blob_digest, all_bytes.clone());
    let path_info = signed_path_info(blob_digest, u64::try_from(all_bytes.len())?)?;
    Ok(InMemoryDeltaAuthority {
        authority_prefix: "/example-cache".to_string(),
        support_delta: true,
        sender,
        catalog,
        output_name: OUTPUT_NAME.to_string(),
        delta_path_info: path_info.clone(),
        full_artifact_path_info: path_info,
    })
}

async fn run_delta(
    authority: &InMemoryDeltaAuthority,
    request: &DeltaFetchRequest,
    keypair: &crunch_build::KeyPair,
    shared: &[u8],
    changed: &[u8],
) -> Result<(), Box<dyn Error>> {
    let state = tempfile::tempdir()?;
    let mut receiver = DeltaReceiverState::default();
    receiver.receiver_store.store_prefix = STORE_PREFIX.to_string();
    receiver.retained.retain_chunk(digest(shared), shared.to_vec());
    let outcome = substitute_from_authority(
        authority,
        request,
        &mut receiver,
        &NegotiationOffer::protocol_v1(),
        std::slice::from_ref(&keypair.verifying_key),
        state.path(),
        STORE_PREFIX,
    )
    .await?;
    assert_eq!(outcome.acceptance, DeltaAcceptanceMode::Delta);
    assert_eq!(outcome.transferred_bytes, u64::try_from(changed.len())?);
    assert!(outcome.transferred_bytes < outcome.full_transfer_bytes);
    println!(
        "delta: transferred={} reused={}",
        outcome.transferred_bytes,
        outcome.full_transfer_bytes.saturating_sub(outcome.transferred_bytes)
    );
    Ok(())
}

async fn run_fallback(
    authority: &InMemoryDeltaAuthority,
    request: &DeltaFetchRequest,
    keypair: &crunch_build::KeyPair,
) -> Result<(), Box<dyn Error>> {
    let mut legacy_authority = authority.clone();
    legacy_authority.support_delta = false;
    let state = tempfile::tempdir()?;
    let mut receiver = DeltaReceiverState::default();
    receiver.receiver_store.store_prefix = STORE_PREFIX.to_string();
    let outcome = substitute_from_authority(
        &legacy_authority,
        request,
        &mut receiver,
        &NegotiationOffer::protocol_v1(),
        std::slice::from_ref(&keypair.verifying_key),
        state.path(),
        STORE_PREFIX,
    )
    .await?;
    assert_eq!(outcome.acceptance, DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::LegacyCache));
    assert_eq!(outcome.transferred_bytes, outcome.full_transfer_bytes);
    println!("fallback: full artifact bytes={}", outcome.transferred_bytes);
    Ok(())
}

async fn run_missing_chunk_rejection(
    authority: &InMemoryDeltaAuthority,
    request: &DeltaFetchRequest,
    keypair: &crunch_build::KeyPair,
    shared: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut malformed = authority.clone();
    malformed.catalog = ContentCatalog::default();
    let state = tempfile::tempdir()?;
    let mut receiver = DeltaReceiverState::default();
    receiver.receiver_store.store_prefix = STORE_PREFIX.to_string();
    receiver.retained.retain_chunk(digest(shared), shared.to_vec());
    let result = substitute_from_authority(
        &malformed,
        request,
        &mut receiver,
        &NegotiationOffer::protocol_v1(),
        std::slice::from_ref(&keypair.verifying_key),
        state.path(),
        STORE_PREFIX,
    )
    .await;
    let Err(error) = result else {
        return Err("missing changed chunk unexpectedly succeeded".into());
    };
    assert!(matches!(error, DeltaSubstitutionError::MissingSenderChunk(_)));
    println!("negative: missing sender chunk rejected");
    Ok(())
}

async fn run_demo() -> Result<(), Box<dyn Error>> {
    let shared = b"shared-v1-content";
    let changed = b"changed-v2-content";
    let authority = make_authority(shared, changed)?;
    let keypair = crunch_build::load_keypair(KEYPAIR)?;
    let request = DeltaFetchRequest {
        logical_path: format!("{STORE_PREFIX}/delta-substitution-demo"),
        output_name: OUTPUT_NAME.to_string(),
    };
    run_delta(&authority, &request, &keypair, shared, changed).await?;
    run_fallback(&authority, &request, &keypair).await?;
    run_missing_chunk_rejection(&authority, &request, &keypair, shared).await
}

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run_demo())
}
