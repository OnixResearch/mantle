use crunch_build::export::export_castore_to_disk;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;

#[allow(dead_code)]
fn compile_check(
    node: &Node,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) {
    let future = export_castore_to_disk(node, dest, blob_service, directory_service);
    std::mem::drop(future);
}

#[test]
fn export_api_is_public() {}
