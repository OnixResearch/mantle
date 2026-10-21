//! Child-free source utility compiled by each predecessor Rust compiler.
#[path = "source_built_rust_programs_successors_cc.rs"]
mod cc;
#[path = "source_built_rust_programs_successors_invocation.rs"]
mod invocation;
#[path = "source_built_rust_programs_successors_macro.rs"]
mod macro_control;
#[path = "source_built_rust_programs_successors_native_helper.rs"]
mod native;
#[path = "source_built_rust_programs_successors_process.rs"]
mod process;

const SCRIPT_STARTUP: &str = r#"if let Some(path) = std::env::var_os("MANTLE_CAIRN_SCRIPT_ENV") {
            assert!(std::fs::metadata(&path).expect("script metadata frame").len() <= 64 * 1024 * 1024, "script metadata frame bound");
            let bytes = std::fs::read(path).expect("reading genuine script dependency metadata");
            let mut remaining = bytes.as_slice();
            let mut seen = std::collections::BTreeSet::new();
            while !remaining.is_empty() {
                assert!(remaining.len() >= 8, "truncated script environment frame");
                let key_length = u32::from_le_bytes([remaining[0], remaining[1], remaining[2], remaining[3]]) as usize;
                let value_length = u32::from_le_bytes([remaining[4], remaining[5], remaining[6], remaining[7]]) as usize;
                remaining = &remaining[8..];
                let length = key_length.checked_add(value_length).filter(|length| *length <= remaining.len()).expect("script environment frame lengths");
                let key = std::str::from_utf8(&remaining[..key_length]).expect("script metadata key UTF-8");
                let value = std::str::from_utf8(&remaining[key_length..length]).expect("script metadata value UTF-8");
                assert!(!key.is_empty() && !key.contains(['=', '\0']) && !value.contains('\0') && seen.insert(key) && seen.len() <= 32768, "invalid or duplicate script metadata pair");
                // Entry precedes the original main and all of its threads.
                unsafe { std::env::set_var(key, value); }
                remaining = &remaining[length..];
            }
        }
        std::env::set_current_dir(std::env::var_os("CARGO_MANIFEST_DIR").expect("original manifest directory")).expect("entering original package");
        std::fs::create_dir_all(std::env::var_os"#;

pub(super) fn source() -> String {
    let replay = process_source();
    let metadata = crate::source_built_mantle_programs::metadata::metadata_helper_source_with_native(
        &replay,
        crate::source_built_mantle_programs::compression::NATIVE_REQUEST_HELPER_SOURCE,
    )
    .replace("fn main() {", "pub fn main() {")
    .replace("fn tokens(source:", "pub fn tokens(source:")
    .replace("fn library_overlays(", "pub fn library_overlays(")
    .replace("fn instrument_script(", "pub fn instrument_script(");
    // SCRIPT_INITIALIZER is embedded as a quoted Rust string in metadata.
    // Escape this source replacement at that same layer, exactly once.
    let startup = format!("{SCRIPT_STARTUP:?}");
    let metadata = metadata.replace("std::fs::create_dir_all(std::env::var_os", &startup[1..startup.len() - 1]);
    SOURCE
        .replace("/*NATIVE_HELPER*/", native::SOURCE)
        .replace("/*METADATA_HELPER*/", &format!("mod metadata {{\n{metadata}\n}}"))
        .replace("/*CC_OVERLAY*/", &cc::source())
        .replace("/*PROCESS_OVERLAY*/", &process::source())
        .replace("/*INVOCATION_HELPER*/", invocation::SOURCE)
        .replace("/*MACRO_HELPER*/", macro_control::SOURCE)
        .replace("/*STD_PROCESS_OVERLAY*/", super::std_process::OVERLAY_SOURCE)
}

pub(super) fn process_source() -> String {
    let random = crate::source_built_mantle_programs::native::CC_PROCESS_SOURCE
        .split_once("fn next_command()")
        .expect("source-owned cc random boundary")
        .0;
    format!("{random}\npub(crate) fn output(command: &mut Command) -> io::Result<Output> {{ command.output() }}\n")
}
pub(super) const SOURCE: &str = r#"
use std::path::{Path,PathBuf};
use std::collections::BTreeMap;
use std::io::{self,Read,Write};
fn copy(source:&Path,dest:&Path,count:&mut usize)->io::Result<()> {
    *count+=1; if *count>1_000_000 { return Err(io::Error::other("source tree exceeds entry bound")); }
    let meta=std::fs::symlink_metadata(source)?;
    if meta.file_type().is_symlink() {
        let target=std::fs::read_link(source)?;
        if target.is_absolute() { return Err(io::Error::other("absolute original source symlink")); }
        if let Some(parent)=dest.parent(){std::fs::create_dir_all(parent)?;}
        std::os::unix::fs::symlink(target,dest)
    } else if meta.is_dir() {
        std::fs::create_dir_all(dest)?;
        let mut entries=std::fs::read_dir(source)?.collect::<Result<Vec<_>,_>>()?;entries.sort_by_key(|e|e.file_name());
        for entry in entries { copy(&entry.path(),&dest.join(entry.file_name()),count)?; } Ok(())
    } else if meta.is_file() {
        if let Some(parent)=dest.parent(){std::fs::create_dir_all(parent)?;}
        std::fs::copy(source,dest)?;std::fs::set_permissions(dest,meta.permissions())
    } else {Err(io::Error::other("non-source filesystem member"))}
}
fn safe(path:&str)->io::Result<()> {
    if path.is_empty() || Path::new(path).components().any(|v|!matches!(v,std::path::Component::Normal(_))) { return Err(io::Error::other("nonrelative assembly path")); } Ok(())
}
fn cargo_metadata(args:&[String])->io::Result<()>{
    let cfg=std::fs::read_to_string(&args[1])?;let root=Path::new(&args[2]);std::fs::create_dir_all(root)?;if let Some(base)=args.get(3){copy(Path::new(base),root,&mut 0)?;}
    let mut groups:BTreeMap<String,Vec<String>>=BTreeMap::new();for row in cfg.lines(){let(name,value)=match row.split_once('='){Some((name,value))=>{let value=value.strip_prefix('"').and_then(|value|value.strip_suffix('"')).filter(|value|!value.contains(['"','\\','\0'])).ok_or_else(||io::Error::other("invalid genuine rustc cfg value"))?;(name,value)},None=>(row,"")};if name.is_empty()||!name.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'_'){return Err(io::Error::other("invalid genuine rustc cfg name"));}groups.entry(format!("CARGO_CFG_{}",name.to_ascii_uppercase())).or_default().push(value.into());}
    let mut environment:BTreeMap<String,String>=groups.into_iter().map(|(key,values)|(key,values.join(","))).collect();let frame=root.join("compiler.env");if frame.try_exists()?{let bytes=std::fs::read(&frame)?;let mut rest=bytes.as_slice();while !rest.is_empty(){if rest.len()<8{return Err(io::Error::other("truncated Cargo environment"));}let key=u32::from_le_bytes(rest[..4].try_into().unwrap())as usize;let value=u32::from_le_bytes(rest[4..8].try_into().unwrap())as usize;rest=&rest[8..];let length=key.checked_add(value).filter(|length|*length<=rest.len()).ok_or_else(||io::Error::other("invalid Cargo environment lengths"))?;let name=std::str::from_utf8(&rest[..key]).map_err(io::Error::other)?;let value=std::str::from_utf8(&rest[key..length]).map_err(io::Error::other)?;if name.is_empty()||name.contains(['=','\0'])||value.contains('\0'){return Err(io::Error::other("invalid Cargo environment pair"));}environment.insert(name.into(),value.into());rest=&rest[length..];}}
    let mut out=std::fs::File::create(frame)?;for(key,value)in environment{out.write_all(&(u32::try_from(key.len()).map_err(io::Error::other)?).to_le_bytes())?;out.write_all(&(u32::try_from(value.len()).map_err(io::Error::other)?).to_le_bytes())?;out.write_all(key.as_bytes())?;out.write_all(value.as_bytes())?;}if !root.join("compiler.rsp").try_exists()?{std::fs::write(root.join("compiler.rsp"),b"")?;}Ok(())
}
/*NATIVE_HELPER*/
/*METADATA_HELPER*/
/*CC_OVERLAY*/
/*INVOCATION_HELPER*/
/*MACRO_HELPER*/
/*PROCESS_OVERLAY*/
/*STD_PROCESS_OVERLAY*/
fn main()->io::Result<()> {
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| matches!(arg.as_str(), "--library-overlays" | "--rustc-request" | "--native-request" | "--native-result") || !arg.starts_with("--")) { metadata::main(); return Ok(()); }
    match args.first().map(String::as_str) {
        Some("--select") if args.len()==3 => copy(Path::new(&args[1]),Path::new(&args[2]),&mut 0),
        Some("--empty") if args.len()==2 => std::fs::create_dir_all(&args[1]),
        Some("--cc-overlay") if args.len()>=3 => cc_overlay(&args),
        Some("--std-process-overlay") if args.len()==5 => std_process_overlay(&args),
        Some("--original-script-overlay") if args.len()==4 => original_script_overlay(&args),
        Some("--process-library") if args.len()==5 => process_library(&args),
        Some("--macro-classify") if args.len()>=9 && args.len()%2==1 => {let code=macro_classify(&args)?;std::process::exit(code)},
        Some("--macro-invocation") if args.len()>=8 && args.len()%2==0 => macro_invocation(&args),
        Some("--macro-reply") if args.len()>=7 => {let code=macro_reply(&args)?;std::process::exit(code)},
        Some("--macro-finish") if args.len()==3 => macro_finish(&args),
        Some("--provider-receipt") if args.len()==5 => provider_receipt(&args),
        Some("--merge-trees") if args.len()>=4 => merge_trees(&args),
        Some("--cargo-metadata") if matches!(args.len(),3|4) => cargo_metadata(&args),
        Some("--native-sdk") if args.len()>=2 => std::fs::write(&args[1],quote_response(&args[2..])),
        Some("--runtime-libraries") if args.len()==3 => runtime_libraries(&args),
        Some("--classify") if args.len()>=5 && args.len()%2==1 => {let code=classify(&args)?;std::process::exit(code)},
        Some("--merge-reply") if args.len()>=5 => {let code=merge_reply(&args)?;std::process::exit(code)},
        Some("--finish-script") if args.len()==4 => finish_script(&args),
        Some("--materialize-overlay") if args.len()==5 => {safe(&args[3])?;copy(Path::new(&args[1]),Path::new(&args[4]),&mut 0)?;std::fs::copy(Path::new(&args[2]).join("transformed.rs"),Path::new(&args[4]).join(&args[3]))?;Ok(())},
        Some("--assemble") if args.len()>=4 && args.len()%2==0 => {
            let out=Path::new(&args[1]);std::fs::create_dir_all(out)?;
            for pair in args[2..].chunks_exact(2){safe(&pair[0])?;copy(Path::new(&pair[1]),&out.join(&pair[0]),&mut 0)?;} Ok(())
        },
        Some("--frame") if args.len()>=2 && args.len()%2==0 => {
            let mut file=std::fs::File::create(&args[1])?;
            let mut seen=std::collections::BTreeSet::new();
            for pair in args[2..].chunks_exact(2){
                let key=&pair[0];let value=&pair[1];
                if key.is_empty() || key.contains(['=','\0']) || value.contains('\0') || !seen.insert(key){return Err(io::Error::other("invalid environment frame pair"));}
                file.write_all(&(u32::try_from(key.len()).map_err(io::Error::other)?).to_le_bytes())?;
                file.write_all(&(u32::try_from(value.len()).map_err(io::Error::other)?).to_le_bytes())?;
                file.write_all(key.as_bytes())?;file.write_all(value.as_bytes())?;
            } Ok(())
        },
        _ => Err(io::Error::other("invalid successor source utility arguments")),
    }
}
"#;
