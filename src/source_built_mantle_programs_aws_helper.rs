// Childfree execution of the pinned AWS-LC data and prefix algorithms.
// Original algorithm sources are mandatory runtime inputs, not vendor outputs.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
fn bad(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}
fn copy(from: &Path, to: &Path) -> io::Result<()> {
    if from.is_dir() {
        fs::create_dir_all(to)?;
        for e in fs::read_dir(from)? {
            let e = e?;
            copy(&e.path(), &to.join(e.file_name()))?;
        }
    } else {
        if let Some(p) = to.parent() {
            fs::create_dir_all(p)?;
        }
        fs::copy(from, to)?;
    }
    Ok(())
}
fn raw_strings(s: &str) -> Vec<&str> {
    s.split('`').enumerate().filter_map(|(i, s)| (i % 2 == 1).then_some(s)).collect()
}
fn body<'a>(source: &'a str, name: &str) -> io::Result<&'a str> {
    let start = source.find(&format!("func {name}(")).ok_or_else(|| bad("missing original Go generator function"))?;
    let rest = &source[start..];
    Ok(rest.split("\nfunc ").next().unwrap())
}
fn headers(original: &Path, symbols: &[String], prefix: &str, out: &Path) -> io::Result<()> {
    if prefix.is_empty() {
        return Err(bad("empty original prefix"));
    }
    let source = fs::read_to_string(original.join("util/make_prefix_headers.go"))?;
    fs::create_dir_all(out)?;
    for (function, name) in [
        ("writeCHeader", "boringssl_prefix_symbols.h"),
        ("writeASMHeader", "boringssl_prefix_symbols_asm.h"),
        ("writeNASMHeader", "boringssl_prefix_symbols_nasm.inc"),
    ] {
        let chunks = raw_strings(body(&source, function)?);
        if chunks.len()
            != if function == "writeCHeader" {
                3
            } else if function == "writeASMHeader" {
                3
            } else {
                4
            }
        {
            return Err(bad("changed original header templates"));
        }
        let mut result = String::new();
        result.push_str(chunks[0]);
        result.push_str(&chunks[1].replace("%s", prefix).replace("%%", "%"));
        if function == "writeCHeader" {
            result.push_str(chunks[2]);
            for symbol in symbols {
                result.push_str(&format!("#define {symbol} BORINGSSL_ADD_PREFIX(BORINGSSL_PREFIX, {symbol})\n"));
            }
            result.push_str("\n#endif // BORINGSSL_PREFIX_SYMBOLS_H\n");
        } else if function == "writeASMHeader" {
            for symbol in symbols {
                result
                    .push_str(&format!("#define _{symbol} BORINGSSL_ADD_PREFIX_MAC_ASM(BORINGSSL_PREFIX, {symbol})\n"));
            }
            result.push_str(chunks[2]);
        } else {
            result.push_str(chunks[2]);
            for symbol in symbols {
                result.push_str(&format!("%xdefine _{symbol} _ %+ BORINGSSL_PREFIX %+ _{symbol}\n"));
            }
            result.push_str("%else\n");
            for symbol in symbols {
                result.push_str(&format!("%xdefine {symbol} BORINGSSL_PREFIX %+ _{symbol}\n"));
            }
            result.push_str(&chunks[3].replace("%%", "%"));
        }
        fs::write(out.join(name), result)?;
    }
    Ok(())
}
fn err_data(original: &Path, out: &Path) -> io::Result<()> {
    let directory = original.join("crypto/err");
    let source = fs::read_to_string(directory.join("err_data_generate.go"))?;
    let libraries = source
        .split("var libraryNames = []string{")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .ok_or_else(|| bad("missing original libraryNames"))?;
    let libraries = libraries
        .lines()
        .filter_map(|s| s.trim().strip_prefix('"').and_then(|s| s.split('"').next()))
        .collect::<Vec<_>>();
    let mut names = fs::read_dir(&directory)?.map(|e| e.map(|e| e.path())).collect::<io::Result<Vec<_>>>()?;
    names.sort();
    let mut strings = BTreeMap::<String, u32>::new();
    let mut data = String::new();
    let mut entries = BTreeMap::<u32, u32>::new();
    for path in names {
        if path.extension().and_then(|s| s.to_str()) != Some("errordata") {
            continue;
        }
        for line in fs::read_to_string(path)?.lines().filter(|s| !s.is_empty()) {
            let parts = line.split(',').collect::<Vec<_>>();
            if parts.len() != 3 {
                return Err(bad("bad original error data row"));
            }
            let library =
                libraries.iter().position(|s| *s == parts[0]).ok_or_else(|| bad("unknown error library"))? as u32 + 1;
            let reason = parts[1].parse::<u32>().map_err(|_| bad("bad error reason"))?;
            if library >= 64 || reason >= 2048 {
                return Err(bad("error key overflow"));
            }
            let offset = if let Some(offset) = strings.get(parts[2]) {
                *offset
            } else {
                let offset = data.len() as u32;
                if offset > 0x7fff {
                    return Err(bad("error string overflow"));
                }
                strings.insert(parts[2].into(), offset);
                data.push_str(parts[2]);
                data.push('\0');
                offset
            };
            let key = library << 26 | reason << 15;
            if entries.insert(key, key | offset).is_some() {
                return Err(bad("duplicate original error key"));
            }
        }
    }
    let chunks = raw_strings(body(&source, "main")?);
    if chunks.len() != 1 {
        return Err(bad("changed original error output template"));
    }
    let mut result = chunks[0].to_string();
    for (i, name) in libraries.iter().enumerate() {
        result.push_str(&format!("OPENSSL_STATIC_ASSERT(ERR_LIB_{name} == {}, library_value_changed)\n", i + 1));
    }
    result.push_str(&format!(
        "OPENSSL_STATIC_ASSERT(ERR_NUM_LIBS == {}, number_of_libraries_changed)\n\n",
        libraries.len() + 1
    ));
    result.push_str("const uint32_t kOpenSSLReasonValues[] = {\n");
    for value in entries.values() {
        result.push_str(&format!("    0x{value:x},\n"));
    }
    result.push_str("};\n\nconst size_t kOpenSSLReasonValuesLen = sizeof(kOpenSSLReasonValues) / sizeof(kOpenSSLReasonValues[0]);\n\nconst char kOpenSSLReasonStringData[] =\n    \"");
    result.push_str(&data.replace('\0', "\\0\"\n    \""));
    result.push_str("\";\n\n");
    fs::write(out, result)
}
fn filtered_symbols(original: &Path, nm: &Path) -> io::Result<Vec<String>> {
    let algorithm = fs::read_to_string(original.join("util/read_symbols.go"))?;
    let skip = algorithm
        .split("var skipSymbols = []string{")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .ok_or_else(|| bad("missing original symbol exclusions"))?;
    let skip = skip
        .lines()
        .filter_map(|s| s.trim().strip_prefix('"').and_then(|s| s.split('"').next()))
        .collect::<BTreeSet<_>>();
    let mut symbols = BTreeSet::new();
    // GNU nm -g --defined-only -P: name, type, value, optional size. Archive
    // headings end in ':' and carry no symbol. Undefined weak symbols are
    // excluded by nm, matching ELF ST_BIND != LOCAL && SHN_UNDEF exclusion.
    for line in fs::read_to_string(nm)?.lines() {
        if line.trim_end().ends_with(':') {
            continue;
        }
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.is_empty() {
            continue;
        }
        if fields.len() < 3 || fields[1].len() != 1 {
            return Err(bad("unexpected original nm symbol record"));
        }
        let symbol = fields[0];
        if skip.contains(symbol)
            || symbol.starts_with("_Z")
            || symbol.starts_with("__real@")
            || symbol.starts_with("__x86.get_pc_thunk.")
            || symbol.starts_with("DW.")
        {
            continue;
        }
        symbols.insert(symbol.to_string());
    }
    Ok(symbols.into_iter().collect())
}
fn original_native(template: &Path, original: &Path, out: &Path) -> io::Result<()> {
    fs::create_dir_all(out)?;
    for entry in fs::read_dir(template)? {
        let entry = entry?;
        let name = entry.file_name();
        let text = name.to_string_lossy();
        if text == "generated-src" || text.starts_with("boringssl_prefix_symbols") {
            continue;
        }
        let source = original.join(&name);
        let destination = out.join(&name);
        if entry.path().is_dir() {
            original_native(&entry.path(), &source, &destination)?;
        } else {
            copy(&source, &destination)?;
        }
    }
    Ok(())
}
fn prepare(package: &Path, original: &Path, out: &Path) -> io::Result<()> {
    fs::create_dir_all(out)?;
    for e in fs::read_dir(package)? {
        let e = e?;
        if e.file_name() == "generated-include" || e.file_name() == "aws-lc" {
            continue;
        }
        copy(&e.path(), &out.join(e.file_name()))?;
    }
    original_native(&package.join("aws-lc"), original, &out.join("aws-lc"))?;
    let generated = out.join("aws-lc/generated-src");
    fs::create_dir_all(&generated)?;
    headers(original, &[], "awslc", &out.join("generated-include/openssl"))?;
    err_data(original, &generated.join("err_data.c"))?;
    Ok(())
}
fn perl_tree(root: &Path, out: &Path) -> io::Result<()> {
    fs::create_dir_all(out)?;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let dest = out.join(entry.file_name());
        if path.is_dir() {
            perl_tree(&path, &dest)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("pl") {
            let mut source = fs::read_to_string(&path)?;
            if matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("rsaz-2k-avx512.pl" | "rsaz-3k-avx512.pl" | "rsaz-4k-avx512.pl")
            ) {
                let boundary = "`$ENV{CC} -Wa,-v -c -o /dev/null -x assembler /dev/null 2>&1`";
                if source.matches(boundary).count() != 1 {
                    return Err(bad("changed original assembler probe boundary"));
                }
                source = source.replace(
                    boundary,
                    "do { local $/; open my $probe, '<', $ENV{MANTLE_AWS_AS_PROBE} or die $!; <$probe> }",
                );
            }
            fs::write(dest, source)?;
        }
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let arg = |i: usize| args.get(i).map(PathBuf::from).ok_or_else(|| bad("missing AWS helper operand"));
    match args.first().map(String::as_str) {
        Some("prepare") if args.len() == 4 => prepare(&arg(1)?, &arg(2)?, &arg(3)?),
        Some("perlasm-tree") if args.len() == 3 => perl_tree(&arg(1)?.join("crypto"), &arg(2)?.join("crypto")),
        Some("prefix") if args.len() == 5 => {
            let symbols = filtered_symbols(&arg(1)?, &arg(2)?)?;
            headers(&arg(1)?, &symbols, &args[3], &arg(4)?)
        }
        Some("merge") if args.len() >= 3 && args.len() % 2 == 1 => {
            let out = arg(2)?;
            copy(&arg(1)?, &out)?;
            for pair in args[3..].chunks(2) {
                if pair.len() != 2 {
                    return Err(bad("unpaired generated source"));
                }
                let dest = Path::new(&pair[1]);
                let member =
                    dest.strip_prefix(&out).map_err(|_| bad("AWS destination outside declared package output"))?;
                if member.as_os_str().is_empty()
                    || member.components().any(|c| !matches!(c, std::path::Component::Normal(_)))
                {
                    return Err(bad("invalid AWS output member"));
                }
                copy(Path::new(&pair[0]), dest)?;
            }
            Ok(())
        }
        _ => Err(bad("unknown AWS source helper mode")),
    }
}
