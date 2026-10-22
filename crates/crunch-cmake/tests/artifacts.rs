use std::io::Write;

use crunch_cmake::NativeInputRequest;
use crunch_cmake::inspect_architecture;
use crunch_cmake::native_input;

fn native_request(output_root: &str, role: &str, root_env: &str, store_prefix: &str) -> NativeInputRequest {
    NativeInputRequest {
        output_root: output_root.into(),
        role: role.into(),
        root_env: root_env.into(),
        store_prefix: store_prefix.into(),
    }
}

fn elf(machine: u16) -> Vec<u8> {
    let mut bytes = vec![0_u8; 64];
    bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
    bytes[18..20].copy_from_slice(&machine.to_le_bytes());
    bytes
}
#[test]
fn cross_architecture_elf_is_rejected_without_executing_it() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(&elf(183)).unwrap();
    assert!(inspect_architecture(file.path(), 183).is_ok());
    assert!(
        inspect_architecture(file.path(), 62)
            .unwrap_err()
            .to_string()
            .contains("cmake-incompatible-artifact")
    );
}
#[test]
fn static_archive_members_are_checked_not_just_the_outer_magic() {
    let mut bytes = b"!<arch>\n".to_vec();
    let header = format!("{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n", "answer.o/", 0, 0, 0, "100644", 64);
    assert_eq!(header.len(), 60);
    bytes.extend(header.as_bytes());
    bytes.extend(elf(62));
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(&bytes).unwrap();
    assert!(inspect_architecture(file.path(), 62).is_ok());
    assert!(inspect_architecture(file.path(), 183).is_err());
}
#[test]
fn thin_archives_cannot_smuggle_external_file_references() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"!<thin>\n").unwrap();
    assert!(inspect_architecture(file.path(), 62).unwrap_err().to_string().contains("thin-archive"));
}
#[test]
fn cargo_native_exports_reuse_role_validation_and_reject_reserved_environment() {
    let root = "/mantle/store/00000000000000000000000000000000-answer";
    let exported = native_input(native_request(root, "answer", "ANSWER_ROOT", "/mantle/store")).unwrap();
    let resolved = mantle_rust_plan_core::resolve_native_inputs(&exported).unwrap();
    assert_eq!(resolved.env["ANSWER_ROOT"], root);
    assert!(native_input(native_request(root, "answer", "PATH", "/mantle/store")).is_err());
    assert!(native_input(native_request("/usr/lib", "answer", "ANSWER_ROOT", "/mantle/store")).is_err());
}
