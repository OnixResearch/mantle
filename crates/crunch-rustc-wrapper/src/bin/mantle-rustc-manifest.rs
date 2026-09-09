use std::path::Path;

use crunch_rust_cache_core::wrapper::DeclaredInputKind;

const EXIT_USAGE: i32 = 64;
const PUBLISH_ARGUMENT_COUNT: usize = 3;
const HASH_ARGUMENT_COUNT: usize = 4;

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments.first().map(String::as_str) {
        Some("publish") if arguments.len() == PUBLISH_ARGUMENT_COUNT => publish(&arguments),
        Some("hash-path") if arguments.len() == HASH_ARGUMENT_COUNT => hash_path(&arguments),
        _ => Err(usage()),
    };
    if let Err(error) = result {
        eprintln!("mantle-rustc-manifest:{error}");
        std::process::exit(EXIT_USAGE);
    }
}

fn publish(arguments: &[String]) -> Result<(), String> {
    let published =
        crunch_rustc_wrapper::publish_manifest_to_registry(Path::new(&arguments[1]), Path::new(&arguments[2]))
            .map_err(|error| error.to_string())?;
    println!("{}", published.path.display());
    Ok(())
}

fn hash_path(arguments: &[String]) -> Result<(), String> {
    let kind = match arguments[1].as_str() {
        "file" => DeclaredInputKind::File,
        "directory" => DeclaredInputKind::Directory,
        _ => return Err(usage()),
    };
    let limit_bytes = arguments[3].parse::<u64>().map_err(|_| "limit-bytes-invalid".to_string())?;
    let digest = crunch_rustc_wrapper::declared_input_digest_blake3(Path::new(&arguments[2]), kind, limit_bytes)
        .map_err(|error| error.to_string())?;
    println!("{digest}");
    Ok(())
}

fn usage() -> String {
    "usage: mantle-rustc-manifest publish INPUT.json REGISTRY_DIR | hash-path file|directory PATH LIMIT_BYTES"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_path_rejects_unknown_kind() {
        let arguments = ["hash-path", "pipe", "/tmp/input", "1024"].map(str::to_string);
        let error = hash_path(&arguments).unwrap_err();
        assert!(error.starts_with("usage:"));
        assert!(error.contains("file|directory"));
    }

    #[test]
    fn hash_path_rejects_invalid_limit() {
        let arguments = ["hash-path", "file", "/tmp/input", "large"].map(str::to_string);
        assert_eq!(hash_path(&arguments), Err("limit-bytes-invalid".to_string()));
        assert_eq!(arguments.len(), HASH_ARGUMENT_COUNT);
    }
}
