//! Producer-side NAR BLAKE3 for copied package source subtrees.
//! The worker independently measures the admitted subtree; mismatches fail admission.

use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn write_word(writer: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let length = u64::try_from(bytes.len()).map_err(|_| "unit-plan-limit: NAR string length".to_string())?;
    writer.write_all(&length.to_le_bytes()).map_err(|error| error.to_string())?;
    writer.write_all(bytes).map_err(|error| error.to_string())?;
    let padding = (8 - bytes.len() % 8) % 8;
    writer.write_all(&[0; 8][..padding]).map_err(|error| error.to_string())
}

fn visit(writer: &mut impl Write, path: &Path, depth: u32) -> Result<(), String> {
    if depth > 32 {
        return Err("unit-plan-limit: package source nesting exceeds 32".into());
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("unit-plan-source-unsupported: {}: {error}", path.display()))?;
    if metadata.is_symlink() {
        return Err(format!("unit-plan-source-unsupported: symlink {}", path.display()));
    }
    write_word(writer, b"(")?;
    write_word(writer, b"type")?;
    if metadata.is_dir() {
        write_word(writer, b"directory")?;
        let mut entries = fs::read_dir(path)
            .map_err(|error| error.to_string())?
            .map(|entry| entry.map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            let name = name.to_str().ok_or("unit-plan-source-unsupported: non-UTF8 source name")?;
            if name.is_empty() || name == "." || name == ".." {
                return Err("unit-plan-source-unsupported: invalid source entry".into());
            }
            write_word(writer, b"entry")?;
            write_word(writer, b"(")?;
            write_word(writer, b"name")?;
            write_word(writer, name.as_bytes())?;
            write_word(writer, b"node")?;
            visit(writer, &entry.path(), depth + 1)?;
            write_word(writer, b")")?;
        }
    } else if metadata.is_file() {
        if metadata.len() > 128 * 1024 * 1024 {
            return Err(format!("unit-plan-limit: package source file over 128 MiB: {}", path.display()));
        }
        write_word(writer, b"regular")?;
        if metadata.permissions().mode() & 0o111 != 0 {
            write_word(writer, b"executable")?;
            write_word(writer, b"")?;
        }
        write_word(writer, b"contents")?;
        let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
        writer.write_all(&metadata.len().to_le_bytes()).map_err(|error| error.to_string())?;
        let bytes = std::io::copy(&mut file, writer).map_err(|error| error.to_string())?;
        if bytes != metadata.len() {
            return Err("unit-plan-source-unsupported: source changed while hashing".into());
        }
        let padding = (8 - bytes as usize % 8) % 8;
        writer.write_all(&[0; 8][..padding]).map_err(|error| error.to_string())?;
        let mut probe = [0u8; 1];
        if file.read(&mut probe).map_err(|error| error.to_string())? != 0 {
            return Err("unit-plan-source-unsupported: source grew while hashing".into());
        }
    } else {
        return Err(format!("unit-plan-source-unsupported: special source node {}", path.display()));
    }
    write_word(writer, b")")
}

struct HashWriter(blake3::Hasher);
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn nar_blake3(path: &Path) -> Result<String, String> {
    let mut writer = HashWriter(blake3::Hasher::new());
    write_word(&mut writer, b"nix-archive-1")?;
    visit(&mut writer, path, 0)?;
    Ok(writer.0.finalize().to_hex().to_string())
}

pub fn copy_package(source: &Path, destination: &Path, depth: u32) -> Result<(), String> {
    if depth > 32 {
        return Err("unit-plan-limit: package source nesting exceeds 32".into());
    }
    let metadata = fs::symlink_metadata(source).map_err(|error| format!("unit-plan-source-unsupported: {error}"))?;
    if metadata.is_symlink() {
        return Err(format!("unit-plan-source-unsupported: symlink {}", source.display()));
    }
    if metadata.is_dir() {
        fs::create_dir(destination).map_err(|error| format!("unit-plan-source-unsupported: {error}"))?;
        let mut entries = fs::read_dir(source)
            .map_err(|error| error.to_string())?
            .map(|entry| entry.map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        if entries.len() > 100_000 {
            return Err("unit-plan-limit: package source entries".into());
        }
        for entry in entries {
            let name = entry.file_name();
            let name = name.to_str().ok_or("unit-plan-source-unsupported: non-UTF8 source name")?;
            if name == "target" || name == ".git" {
                continue;
            }
            copy_package(&entry.path(), &destination.join(name), depth + 1)?;
        }
        fs::set_permissions(destination, metadata.permissions()).map_err(|error| error.to_string())?;
    } else if metadata.is_file() {
        if metadata.len() > 128 * 1024 * 1024 {
            return Err("unit-plan-limit: package source file exceeds 128 MiB".into());
        }
        fs::copy(source, destination).map_err(|error| format!("unit-plan-source-unsupported: {error}"))?;
        fs::set_permissions(destination, metadata.permissions()).map_err(|error| error.to_string())?;
    } else {
        return Err(format!("unit-plan-source-unsupported: special node {}", source.display()));
    }
    Ok(())
}
