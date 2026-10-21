//! Only the inspected autocfg stdin/spawn and UUID boundary needs a library
//! overlay. All original output/status calls use the compiled standard library.
pub(super) fn source() -> String {
    let original = include_str!("source_built_mantle_programs_rust_probes.rs");
    let stdin = original
        .split_once("const AUTOCFG_STDIN_BEFORE:")
        .expect("original autocfg stdin declaration")
        .1
        .split_once("/// Preserve actual random UUID")
        .expect("original autocfg stdin boundary")
        .0;
    format!(
        r#"
const AUTOCFG_STDIN_BEFORE:{stdin}
const AUTOCFG_SOURCE_PORT:&str={state:?};
fn original_script_overlay(args:&[String])->io::Result<()>{{
    let source=std::fs::read_to_string(&args[1])?;let transformed=metadata::instrument_script(&source).map_err(io::Error::other)?;
    emit_overlay(Path::new(&args[3]),&[(args[2].clone(),transformed)],Path::new(""))
}}
fn emit_overlay(root:&Path,files:&[(String,String)],source_root:&Path)->io::Result<()>{{
    std::fs::create_dir_all(root)?;let mut frame=std::fs::File::create(root.join("overlay.frame"))?;
    for(index,(member,text))in files.iter().enumerate(){{let name=format!("transformed-{{index}}.rs");std::fs::write(root.join(&name),text)?;
        let original=source_root.join(member).to_str().ok_or_else(||io::Error::other("non-UTF8 original source"))?.to_owned();let replacement=format!("/inputs/source-overlay/{{name}}");
        frame.write_all(&(original.len()as u32).to_le_bytes())?;frame.write_all(&(replacement.len()as u32).to_le_bytes())?;frame.write_all(original.as_bytes())?;frame.write_all(replacement.as_bytes())?;
    }}Ok(())
}}
fn process_library(args:&[String])->io::Result<()>{{
    if args[3]!="autocfg"{{return Err(io::Error::other("library needs no source process overlay; original Std owns output/status"));}}
    let source_root=Path::new(&args[1]);let package=Path::new(&args[2]);let relative=package.strip_prefix(source_root).map_err(io::Error::other)?.to_str().ok_or_else(||io::Error::other("non-UTF8 original package"))?;safe(relative)?;
    let source=std::fs::read_to_string(package.join("src/lib.rs"))?;let source=replace_once(&source,AUTOCFG_STDIN_BEFORE,AUTOCFG_STDIN_AFTER).map_err(io::Error::other)?;let source=replace_once(&source,"    hash\n}}","    mantle_cairn_script_process::uuid(hash)\n}}").map_err(io::Error::other)?;
    emit_overlay(Path::new(&args[4]),&[(format!("{{relative}}/src/lib.rs"),format!("{{source}}\n{{AUTOCFG_SOURCE_PORT}}"))],source_root)
}}
"#,
        state = AUTOCFG_SOURCE_PORT
    )
}
const AUTOCFG_SOURCE_PORT: &str = r#"
// Source-only I/O port: original formatted stdin bytes are supplied by a real
// file descriptor. No child/PID object or compiler result is synthesized.
mod mantle_cairn_script_process {
    use std::sync::atomic::{AtomicUsize,Ordering};
    static UUID_NEXT:AtomicUsize=AtomicUsize::new(0);
    static STDIN_NEXT:AtomicUsize=AtomicUsize::new(0);
    pub(crate) fn uuid(generated:u64)->u64 {
        let index=UUID_NEXT.fetch_add(1,Ordering::SeqCst);let name=format!("autocfg-uuid-{index}");let previous=std::env::var_os("MANTLE_CC_PHASE_STATE").expect("original phase workspace");let previous=std::path::Path::new(&previous).join(".mantle-cc-random").join(&name);
        let actual=match std::fs::read_to_string(previous){Ok(value)=>value.parse::<u64>().expect("original observed UUID"),Err(error)if error.kind()==std::io::ErrorKind::NotFound=>generated,Err(error)=>panic!("reading original UUID: {error}")};
        let out=std::env::var_os("OUT_DIR").expect("original OUT_DIR");let directory=std::path::Path::new(&out).join(".mantle-cc-random");std::fs::create_dir_all(&directory).expect("retaining original UUID");std::fs::write(directory.join(name),actual.to_string()).expect("retaining original UUID");actual
    }
    pub(crate) fn with_stdin(command:&mut std::process::Command,source:&[u8])->std::io::Result<std::process::Output>{
        let index=STDIN_NEXT.fetch_add(1,Ordering::SeqCst);let out=std::env::var_os("OUT_DIR").ok_or_else(||std::io::Error::other("missing original OUT_DIR"))?;let directory=std::path::Path::new(&out).join(".mantle-cc-random");std::fs::create_dir_all(&directory)?;let path=directory.join(format!("autocfg-stdin-{index}.rs"));std::fs::write(&path,source)?;command.stdin(std::process::Stdio::from(std::fs::File::open(path)?));command.output()
    }
}
"#;
