mod mantle_native_request {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::fs;
    use std::io;
    use std::path::Component;
    use std::path::Path;
    use std::path::PathBuf;
    fn invalid(message: &str) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, message)
    }
    struct Decoder<'a> {
        bytes: &'a [u8],
    }
    impl Decoder<'_> {
        fn count(&mut self) -> io::Result<usize> {
            let bytes = self.bytes.get(..8).ok_or_else(|| invalid("truncated native request length"))?;
            let count = u64::from_le_bytes(bytes.try_into().unwrap());
            self.bytes = &self.bytes[8..];
            usize::try_from(count)
                .ok()
                .filter(|count| *count <= 16 * 1024 * 1024)
                .ok_or_else(|| invalid("native request length exceeds bound"))
        }
        fn word(&mut self) -> io::Result<String> {
            let count = self.count()?;
            let bytes = self.bytes.get(..count).ok_or_else(|| invalid("truncated native request word"))?;
            let word = std::str::from_utf8(bytes).map_err(|_| invalid("native request is not UTF-8"))?;
            if word.contains('\0') {
                return Err(invalid("native request contains NUL"));
            }
            self.bytes = &self.bytes[count..];
            Ok(word.to_owned())
        }
    }
    struct Request {
        program: String,
        arguments: Vec<String>,
        cwd: String,
        environment: BTreeMap<String, String>,
    }
    fn decode(bytes: &[u8]) -> io::Result<Request> {
        let mut decoder = Decoder {
            bytes: bytes
                .strip_prefix(b"mantle-cc-process-v1\0")
                .ok_or_else(|| invalid("native request magic mismatch"))?,
        };
        let program = decoder.word()?;
        let count = decoder.count()?;
        if count > 32768 {
            return Err(invalid("native request argc exceeds bound"));
        }
        let mut arguments = Vec::with_capacity(count);
        for _ in 0..count {
            arguments.push(decoder.word()?);
        }
        let cwd = decoder.word()?;
        let count = decoder.count()?;
        if count > 32768 {
            return Err(invalid("native request environment exceeds bound"));
        }
        let mut environment = BTreeMap::new();
        for _ in 0..count {
            let key = decoder.word()?;
            let value = decoder.word()?;
            if key.is_empty() || key.contains('=') || environment.insert(key, value).is_some() {
                return Err(invalid("invalid or duplicate native environment key"));
            }
        }
        if !decoder.bytes.is_empty() {
            return Err(invalid("trailing native request bytes"));
        }
        Ok(Request {
            program,
            arguments,
            cwd,
            environment,
        })
    }
    fn clean(path: &Path) -> io::Result<()> {
        if path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir | Component::Prefix(_)))
        {
            return Err(invalid("noncanonical native path"));
        }
        Ok(())
    }
    fn absolute(path: &str, cwd: &str) -> io::Result<PathBuf> {
        let path = Path::new(path);
        let result = if path.is_absolute() {
            path.to_owned()
        } else {
            Path::new(cwd).join(path)
        };
        clean(&result)?;
        Ok(result)
    }
    struct Resolver {
        bindings: BTreeSet<String>,
        runtime: BTreeMap<String, String>,
        work: PathBuf,
        package: PathBuf,
        mutable_work: bool,
        data: String,
    }
    impl Resolver {
        fn read(&self, path: &str, request: &Request) -> io::Result<String> {
            let path = absolute(path, &request.cwd)?;
            if let Ok(relative) = path.strip_prefix("/outputs/out-dir") {
                return Ok(Path::new("/inputs/cc-work").join(relative).to_string_lossy().into_owned());
            }
            let text = path.to_string_lossy();
            if let Ok(relative) = path.strip_prefix("/inputs") {
                let alias = relative
                    .components()
                    .next()
                    .ok_or_else(|| invalid("native path lacks input alias"))?
                    .as_os_str()
                    .to_string_lossy();
                if !self.bindings.contains(alias.as_ref()) {
                    return Err(invalid("native read names undeclared source binding"));
                }
                return Ok(text.into_owned());
            }
            if self.runtime.keys().any(|mount| {
                text == mount.as_str() || text.strip_prefix(mount.as_str()).is_some_and(|tail| tail.starts_with('/'))
            }) {
                return Ok(text.into_owned());
            }
            Err(invalid("native read operand escapes source bindings/runtime mounts"))
        }
        fn write(&self, path: &str, request: &Request, query: bool) -> io::Result<(String, PathBuf)> {
            let path = absolute(path, &request.cwd)?;
            let relative = path
                .strip_prefix("/outputs/out-dir")
                .map_err(|_| invalid("native output escapes original OUT_DIR"))?;
            if relative.as_os_str().is_empty() {
                return Err(invalid("native output is OUT_DIR itself"));
            }
            let root = if query {
                "/inputs/cc-work"
            } else {
                "/outputs/native-files"
            };
            Ok((Path::new(root).join(relative).to_string_lossy().into_owned(), relative.to_owned()))
        }
        fn data(&mut self, text: &str) -> String {
            let offset = self.data.len();
            self.data.push_str(text);
            format!(
                "{{\"kind\":\"text_slice\",\"name\":\"native-environment-data\",\"offset_bytes\":{offset},\"length_bytes\":{}}}",
                text.len()
            )
        }
        fn path_part(&self, text: &str) -> io::Result<String> {
            clean(Path::new(text))?;
            if let Some(relative) = text.strip_prefix("/inputs/") {
                let (name, path) = relative.split_once('/').unwrap_or((relative, ""));
                if !self.bindings.contains(name) {
                    return Err(invalid("typed native path names undeclared input"));
                }
                return Ok(format!("{{\"kind\":\"input\",\"name\":{},\"path\":{}}}", json(name), json(path)));
            }
            if let Some(relative) = text.strip_prefix("/outputs/native-files") {
                if !relative.is_empty() && !relative.starts_with('/') {
                    return Err(invalid("native output path prefix mismatch"));
                }
                return Ok(format!(
                    "{{\"kind\":\"output\",\"name\":\"native-files\",\"path\":{}}}",
                    json(relative.trim_start_matches('/'))
                ));
            }
            if let Some((mount, _)) = self.runtime.iter().rev().find(|(mount, _)| {
                text == mount.as_str() || text.strip_prefix(mount.as_str()).is_some_and(|tail| tail.starts_with('/'))
            }) {
                return Ok(format!(
                    "{{\"kind\":\"runtime\",\"mount\":{},\"path\":{}}}",
                    json(mount),
                    json(text.strip_prefix(mount.as_str()).unwrap().trim_start_matches('/'))
                ));
            }
            Err(invalid("unbound actual native path"))
        }
        fn word(&mut self, text: &str, source_data: bool) -> io::Result<String> {
            if text.contains('\0') {
                return Err(invalid("NUL native operand"));
            }
            let mut parts = Vec::new();
            let mut offset = 0;
            while offset < text.len() {
                let remainder = &text[offset..];
                let next = remainder
                    .find("/inputs/")
                    .into_iter()
                    .chain(remainder.find("/outputs/out-dir"))
                    .chain(remainder.find("/outputs/native-files"))
                    .chain(self.runtime.keys().filter_map(|mount| remainder.find(mount.as_str())))
                    .min();
                let Some(next) = next else {
                    if !source_data && remainder.contains(['/', '\\']) {
                        return Err(invalid("unclassified native path operand"));
                    }
                    parts.push(if source_data {
                        self.data(remainder)
                    } else {
                        literal_part(remainder)
                    });
                    break;
                };
                if next > 0 {
                    let prefix = &remainder[..next];
                    if !source_data && prefix.contains(['/', '\\']) {
                        return Err(invalid("unclassified native path prefix"));
                    }
                    parts.push(if source_data {
                        self.data(prefix)
                    } else {
                        literal_part(prefix)
                    });
                }
                let path = &remainder[next..];
                let end = path
                    .find(|ch: char| {
                        ch == ':' || ch.is_ascii_whitespace() || matches!(ch, '\u{1f}' | '"' | '\'' | ',' | ';')
                    })
                    .unwrap_or(path.len());
                let original = &path[..end];
                let mapped = if let Some(relative) = original.strip_prefix("/outputs/out-dir") {
                    format!(
                        "{}{relative}",
                        if self.mutable_work {
                            "/outputs/native-files"
                        } else {
                            "/inputs/cc-work"
                        }
                    )
                } else {
                    original.to_owned()
                };
                let trailing = mapped.ends_with('/');
                let normalized = mapped.strip_suffix('/').unwrap_or(&mapped);
                parts.push(self.path_part(normalized)?);
                let mut consumed = end;
                if trailing {
                    let following = text.as_bytes().get(offset + next + end).copied();
                    if following.is_some_and(|byte| byte != b':') {
                        return Err(invalid("typed native directory suffix requires word end or colon delimiter"));
                    }
                    parts.push("{\"kind\":\"directory_separator\"}".into());
                    if following == Some(b':') {
                        parts.push(literal_part(":"));
                        consumed += 1;
                    }
                }
                offset += next + consumed;
            }
            if parts.is_empty() {
                parts.push(if source_data { self.data("") } else { literal_part("") });
            }
            Ok(format!("{{\"parts\":[{}]}}", parts.join(",")))
        }
    }
    fn json(text: &str) -> String {
        let mut result = String::from("\"");
        for ch in text.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\u{8}' => result.push_str("\\b"),
                '\u{c}' => result.push_str("\\f"),
                ch if ch < '\u{20}' => {
                    use std::fmt::Write;
                    write!(result, "\\u{:04x}", ch as u32).unwrap();
                }
                ch => result.push(ch),
            }
        }
        result.push('"');
        result
    }
    fn literal_part(text: &str) -> String {
        format!("{{\"kind\":\"literal\",\"text\":{}}}", json(text))
    }
    fn prepare(phase: &str, request: &Request, expected_program: &str, resolver: &Resolver) -> io::Result<Vec<String>> {
        if request.program != expected_program {
            return Err(invalid("native request executable differs from source-selected tool"));
        }
        if request.environment.get("OUT_DIR").map(String::as_str) != Some("/outputs/out-dir") {
            return Err(invalid("native request changed original OUT_DIR"));
        }
        let cwd = Path::new(&request.cwd);
        clean(cwd)?;
        if !cwd.starts_with(&resolver.package) && !cwd.starts_with("/outputs/out-dir") {
            return Err(invalid("native request cwd is outside original package/OUT_DIR"));
        }
        if phase == "driver"
            && matches!(request.arguments.as_slice(), [word] if matches!(word.as_str(), "-?" | "--version" | "-dumpfullversion"))
        {
            return Ok(request.arguments.clone());
        }
        if phase == "ar-create" || phase == "ar-index" {
            let mode = if phase == "ar-create" { "cqD" } else { "sD" };
            if request.arguments.first().map(String::as_str) != Some(mode)
                || request.arguments.len() < 2
                || (phase == "ar-index" && request.arguments.len() != 2)
            {
                return Err(invalid("native archive request differs from original GNU deterministic mode"));
            }
            if phase == "ar-create" && request.arguments.len() < 3 {
                return Err(invalid("native archive create has no original members"));
            }
            let (archive, relative) = resolver.write(&request.arguments[1], request, false)?;
            let prior = resolver.work.join(&relative);
            if prior.try_exists()? {
                if !fs::symlink_metadata(&prior)?.is_file() {
                    return Err(invalid("native prior archive is not regular"));
                }
            } else if phase == "ar-index" {
                return Err(invalid("native archive index lacks original created archive"));
            }
            let mut words = vec![mode.into(), archive];
            for member in &request.arguments[2..] {
                words.push(resolver.read(member, request)?);
            }
            return Ok(words);
        }
        if phase == "preprocess" {
            if request.arguments.len() != 2
                || request.arguments[0] != "-E"
                || !request.arguments[1].ends_with("detect_compiler_family.c")
            {
                return Err(invalid("native family detector differs from original cc source"));
            }
            return Ok(vec![
                "-E".into(),
                resolver.read(&request.arguments[1], request)?,
                "-###".into(),
            ]);
        }
        if phase != "compile" && phase != "driver" {
            return Err(invalid("unknown source-selected native phase"));
        }
        if phase == "driver"
            && !request
                .arguments
                .iter()
                .any(|word| matches!(word.as_str(), "-flto=thin" | "-fsanitize-undefined-strip-path-components=-1"))
        {
            return Err(invalid("native driver probe is not a source-selected no-child rejection"));
        }
        let mut words = Vec::new();
        let mut index = 0;
        let mut outputs = 0;
        let mut sources = 0;
        let mut compile = 0;
        while index < request.arguments.len() {
            let word = &request.arguments[index];
            if word.starts_with('@') {
                return Err(invalid("nested native response file is not admitted"));
            }
            match word.as_str() {
                "-c" => {
                    compile += 1;
                    words.push(word.clone());
                }
                "-o" | "-MF" => {
                    let output_option = word == "-o";
                    index += 1;
                    let path =
                        request.arguments.get(index).ok_or_else(|| invalid("native output option lacks path"))?;
                    words.push(word.clone());
                    words.push(resolver.write(path, request, phase == "compile")?.0);
                    if output_option {
                        outputs += 1;
                    }
                }
                "-I" | "-isystem" | "-iquote" | "-include" | "-imacros" | "--sysroot" | "-B" | "-L" => {
                    index += 1;
                    let path = request
                        .arguments
                        .get(index)
                        .ok_or_else(|| invalid("native search/include option lacks path"))?;
                    words.push(word.clone());
                    words.push(resolver.read(path, request)?);
                }
                _ if word.starts_with("--sysroot=") => {
                    words.push(format!("--sysroot={}", resolver.read(&word[10..], request)?))
                }
                _ if ["-I", "-B", "-L"].iter().any(|prefix| word.starts_with(prefix) && word.len() > prefix.len()) => {
                    words.push(format!("{}{}", &word[..2], resolver.read(&word[2..], request)?))
                }
                _ if word.starts_with('-') => {
                    if (word.contains('/') || word.contains('\\'))
                        && !word.starts_with("-D")
                        && !word.starts_with("-fmacro-prefix-map=")
                        && !word.starts_with("-fdebug-prefix-map=")
                    {
                        return Err(invalid("unclassified native compiler path option"));
                    }
                    words.push(word.clone());
                }
                _ => {
                    words.push(resolver.read(word, request)?);
                    sources += 1;
                }
            }
            index += 1;
        }
        if compile != 1 || outputs != 1 || sources != 1 {
            return Err(invalid("native compile is not one original -c source and -o object"));
        }
        if phase == "compile" {
            words.push("-###".into());
        }
        Ok(words)
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
                if !fs::symlink_metadata(destination)?.is_file() || fs::read(source)? != fs::read(destination)? {
                    return Err(invalid("conflicting actual native phase files"));
                }
            } else {
                fs::copy(source, destination)?;
            }
        } else {
            return Err(invalid("nonregular actual native phase product"));
        }
        Ok(())
    }
    fn request(args: &[String]) -> io::Result<()> {
        let bytes = fs::read(&args[3])?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("native request exceeds byte bound"));
        }
        let request = decode(&bytes)?;
        let mut resolver = Resolver {
            bindings: BTreeSet::from(["package".into(), "cc-work".into()]),
            runtime: BTreeMap::new(),
            work: PathBuf::from(&args[6]),
            package: PathBuf::from(&args[7]),
            mutable_work: args[2].starts_with("ar-"),
            data: String::new(),
        };
        let mut driver_arguments = Vec::new();
        let mut index = 9;
        while index < args.len() {
            match args[index].as_str() {
                "--driver-argument" if index + 1 < args.len() => {
                    if args[2].starts_with("ar-") {
                        return Err(invalid("compiler wrapper policy cannot apply to archiver"));
                    }
                    driver_arguments.push(args[index + 1].as_str());
                    index += 2;
                }
                "--binding" if index + 2 < args.len() => {
                    let name = &args[index + 1];
                    if args[index + 2] != format!("/inputs/{name}")
                        || name.is_empty()
                        || name.contains('/')
                        || !Path::new(&args[index + 2]).try_exists()?
                    {
                        return Err(invalid("native resolver binding differs from declared input"));
                    }
                    resolver.bindings.insert(name.clone());
                    index += 3;
                }
                "--runtime" if index + 2 < args.len() => {
                    let mount = &args[index + 1];
                    clean(Path::new(mount))?;
                    if !mount.starts_with('/')
                        || resolver.runtime.insert(mount.clone(), args[index + 2].clone()).is_some()
                    {
                        return Err(invalid("invalid duplicate native runtime mount"));
                    }
                    index += 3;
                }
                _ => return Err(invalid("invalid native resolver declaration")),
            }
        }
        if resolver.work != Path::new("/inputs/cc-work") || resolver.package != Path::new("/inputs/package") {
            return Err(invalid("native source alias binding changed"));
        }
        let actual_cwd = if resolver.mutable_work {
            match Path::new(&request.cwd).strip_prefix("/outputs/out-dir") {
                Ok(relative) => Path::new("/outputs/native-files").join(relative).to_string_lossy().into_owned(),
                Err(_) => resolver.read(&request.cwd, &request)?,
            }
        } else {
            resolver.read(&request.cwd, &request)?
        };
        let normalized_cwd = Path::new(&actual_cwd).components().collect::<PathBuf>();
        let cwd = resolver.path_part(normalized_cwd.to_str().ok_or_else(|| invalid("native cwd is not UTF-8"))?)?;
        let cwd_suffix = format!(",\"cwd\":{{\"parts\":[{cwd}]}}");
        let words = prepare(&args[2], &request, &args[4], &resolver)?;
        let mut arguments = Vec::with_capacity(driver_arguments.len() + words.len());
        for word in driver_arguments.into_iter().chain(words.iter().map(String::as_str)) {
            let data = word.starts_with("-D")
                || word.starts_with("-fmacro-prefix-map=")
                || word.starts_with("-fdebug-prefix-map=");
            arguments.push(if data {
                format!("{{\"parts\":[{}]}}", resolver.data(word))
            } else {
                resolver.word(word, false)?
            });
        }
        let mut environment = Vec::with_capacity(request.environment.len());
        for (name, value) in request.environment {
            let is_path = value.starts_with('/')
                || matches!(
                    name.as_str(),
                    "OUT_DIR"
                        | "CARGO_MANIFEST_DIR"
                        | "CARGO_MANIFEST_PATH"
                        | "RUSTC"
                        | "RUSTDOC"
                        | "CC"
                        | "CXX"
                        | "AR"
                        | "AS"
                        | "LD"
                        | "NM"
                        | "PERL_EXECUTABLE"
                        | "PERL5LIB"
                        | "PATH"
                        | "LD_LIBRARY_PATH"
                        | "LIBRARY_PATH"
                        | "CPATH"
                        | "C_INCLUDE_PATH"
                        | "CPLUS_INCLUDE_PATH"
                );
            if is_path
                && value.split(':').any(|path| {
                    path.starts_with('/')
                        && !path.starts_with("/inputs/")
                        && !path.starts_with("/outputs/out-dir")
                        && !resolver.runtime.keys().any(|mount| {
                            path == mount.as_str()
                                || path.strip_prefix(mount.as_str()).is_some_and(|tail| tail.starts_with('/'))
                        })
                })
            {
                return Err(invalid("original native environment contains unbound actual directory"));
            }
            environment.push(format!("{}:{}", json(&name), resolver.word(&value, true)?));
        }
        let environment = format!("{{{}}}", environment.join(","));
        fs::write(
            &args[5],
            format!("{{\"arguments\":[{}],\"environment\":{environment}{cwd_suffix}}}", arguments.join(",")),
        )?;
        fs::write(&args[8], &resolver.data)
    }
    pub fn run(args: &[String]) -> io::Result<()> {
        match args.get(1).map(String::as_str) {
            Some("--native-request") if args.len() >= 9 => request(args),
            Some("--native-result") if args.len() >= 10 && (args.len() - 7) % 3 == 0 => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                let mut status = Vec::new();
                fs::create_dir_all(&args[5])?;
                for group in args[6..args.len() - 1].chunks_exact(3) {
                    stdout.extend(fs::read(&group[0])?);
                    stderr.extend(fs::read(&group[1])?);
                    let actual = fs::read(&group[2])?;
                    let text = std::str::from_utf8(&actual).map_err(|_| invalid("native status is not UTF-8"))?;
                    let code = text
                        .strip_suffix('\n')
                        .ok_or_else(|| invalid("native status lacks newline"))?
                        .parse::<u8>()
                        .map_err(|_| invalid("native status is not a normal exit code"))?;
                    if actual != format!("{code}\n").as_bytes() {
                        return Err(invalid("native status is not canonical"));
                    }
                    status = actual;
                }
                install(Path::new(&args[args.len() - 1]), Path::new(&args[5]))?;
                fs::write(&args[2], stdout)?;
                fs::write(&args[3], stderr)?;
                fs::write(&args[4], status)
            }
            _ => Err(invalid("invalid native helper arguments")),
        }
    }
}
