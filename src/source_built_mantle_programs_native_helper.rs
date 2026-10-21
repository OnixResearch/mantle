use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn install(source: &Path, destination: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            install(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if metadata.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if destination.try_exists()? {
            if fs::read(source)? != fs::read(destination)? {
                return Err(invalid("conflicting actual native products"));
            }
        } else {
            fs::copy(source, destination)?;
        }
    } else {
        return Err(invalid("nonregular native generator product"));
    }
    Ok(())
}
fn status(path: &Path) -> io::Result<(u8, Vec<u8>)> {
    let bytes = fs::read(path)?;
    let code = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|value| value.strip_suffix('\n'))
        .and_then(|value| value.parse::<u8>().ok())
        .ok_or_else(|| invalid("invalid actual native status"))?;
    if bytes != format!("{code}\n").as_bytes() {
        return Err(invalid("noncanonical actual native status"));
    }
    Ok((code, bytes))
}
struct Decoder<'a> {
    bytes: &'a [u8],
}
impl Decoder<'_> {
    fn count(&mut self) -> io::Result<usize> {
        let bytes = self.bytes.get(..8).ok_or_else(|| invalid("truncated process request"))?;
        let count = u64::from_le_bytes(bytes.try_into().unwrap());
        self.bytes = &self.bytes[8..];
        usize::try_from(count)
            .ok()
            .filter(|value| *value <= 16 * 1024 * 1024)
            .ok_or_else(|| invalid("process count exceeds bound"))
    }
    fn word(&mut self) -> io::Result<String> {
        let count = self.count()?;
        let bytes = self.bytes.get(..count).ok_or_else(|| invalid("truncated process word"))?;
        let text = std::str::from_utf8(bytes).map_err(|_| invalid("process word is not UTF-8"))?;
        if text.contains('\0') {
            return Err(invalid("NUL process word"));
        }
        self.bytes = &self.bytes[count..];
        Ok(text.into())
    }
}
fn ring_request(args: &[String]) -> io::Result<()> {
    let bytes = fs::read(&args[2])?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(invalid("Perl request exceeds bound"));
    }
    let mut request = Decoder {
        bytes: bytes.strip_prefix(b"mantle-cc-process-v1\0").ok_or_else(|| invalid("process magic mismatch"))?,
    };
    if request.word()? != args[3] || request.count()? != 3 {
        return Err(invalid("original ring Perl program or argc changed"));
    }
    for expected in [args[4].clone(), "elf".into(), format!("/outputs/out-dir/{}", args[5])] {
        if request.word()? != expected {
            return Err(invalid("original ring Perl operands changed"));
        }
    }
    if request.word()? != "/inputs/package" {
        return Err(invalid("original ring Perl cwd changed"));
    }
    let count = request.count()?;
    let mut environment = BTreeMap::new();
    for _ in 0..count {
        let key = request.word()?;
        let value = request.word()?;
        if environment.insert(key, value).is_some() {
            return Err(invalid("duplicate Perl environment key"));
        }
    }
    if !request.bytes.is_empty() || environment.get("OUT_DIR").map(String::as_str) != Some("/outputs/out-dir") {
        return Err(invalid("original ring Perl environment changed"));
    }
    fs::write(&args[6], bytes)
}
fn capability_result(args: &[String]) -> io::Result<()> {
    let mut stdout = fs::read(&args[2])?;
    let mut stderr = fs::read(&args[3])?;
    let (compiler_code, mut actual_status) = status(Path::new(&args[4]))?;
    let envelope = Path::new(&args[5]);
    let outcome = fs::read_to_string(envelope.join("outcome.json"))?;
    if outcome.starts_with("{\"kind\":\"executed\",") {
        if compiler_code != 0 {
            return Err(invalid("assembler executed after rejected compiler phase"));
        }
        let products = envelope.join("outputs");
        stdout.extend(fs::read(products.join("stdout"))?);
        stderr.extend(fs::read(products.join("stderr"))?);
        actual_status = status(&products.join("exit-status"))?.1;
    } else if outcome.starts_with("{\"kind\":\"skipped\",") {
        if compiler_code == 0 {
            return Err(invalid("assembler skipped successful compiler phase"));
        }
    } else {
        return Err(invalid("invalid typed phase outcome"));
    }
    install(&envelope.join("state"), Path::new(&args[9]))?;
    fs::write(&args[6], stdout)?;
    fs::write(&args[7], stderr)?;
    fs::write(&args[8], actual_status)
}
fn perlasm_result(args: &[String]) -> io::Result<()> {
    let mut stdout = fs::read(&args[2])?;
    let mut stderr = fs::read(&args[3])?;
    if status(Path::new(&args[4]))?.0 != 0 {
        return Err(invalid("failed Perl generator"));
    }
    stdout.extend(fs::read(&args[5])?);
    stderr.extend(fs::read(&args[6])?);
    let (code, actual_status) = status(Path::new(&args[7]))?;
    if code != 0 {
        return Err(invalid("failed original Perl translator"));
    }
    fs::create_dir_all(&args[13])?;
    install(Path::new(&args[8]), &Path::new(&args[13]).join(&args[9]))?;
    fs::write(&args[10], stdout)?;
    fs::write(&args[11], stderr)?;
    fs::write(&args[12], actual_status)
}
fn package(args: &[String]) -> io::Result<()> {
    let source = Path::new(&args[2]);
    let destination = Path::new(&args[4]);
    fs::create_dir_all(destination)?;
    let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_name() != "pregenerated" {
            install(&entry.path(), &destination.join(entry.file_name()))?;
        }
    }
    let generated = Path::new(&args[3]);
    let pregenerated = destination.join("pregenerated");
    fs::create_dir_all(&pregenerated)?;
    let mut entries = fs::read_dir(generated)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_name() != ".mantle-cc-random" {
            install(&entry.path(), &pregenerated.join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(String::as_str) {
        Some("--ring-request") if args.len() == 7 => ring_request(&args),
        Some("--capability-result") if args.len() == 10 => capability_result(&args),
        Some("--perlasm-result") if args.len() == 14 => perlasm_result(&args),
        Some("--ring-package") if args.len() == 5 => package(&args),
        _ => Err(invalid("invalid native helper operation")),
    }
}
