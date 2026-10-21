//! Source transformations shared with the direct native continuation ABI.
//! The policies below preserve original cc option/tool/source selection.
pub(super) fn source() -> String {
    let original = include_str!("source_built_mantle_programs_native.rs");
    let transform = original
        .split_once("pub(crate) fn cc_linux_process_transform(")
        .expect("source-owned cc transformer declaration")
        .1
        .split_once("/// Preserve original fresh RandomState results")
        .expect("source-owned cc transformer boundary")
        .0;
    let transform = format!("fn cc_linux_process_transform({transform}").replace("RunError", "String");
    format!(
        r#"
const CC_PROCESS_SOURCE:&str={process:?};
fn replace_once(text:&str,before:&str,after:&str)->Result<String,String>{{if text.matches(before).count()!=1{{return Err("original cc process boundary changed".into());}}Ok(text.replacen(before,after,1))}}
fn error(message:&str)->String{{message.into()}}
{transform}
fn cc_overlay(args:&[String])->io::Result<()>{{
    let source=Path::new(&args[1]);let destination=Path::new(&args[2]);copy(source,destination,&mut 0)?;
    let selected=if args.len()==3{{vec![source.to_owned()]}}else{{args[3..].iter().map(PathBuf::from).collect()}};
    for package in selected{{let relative=package.strip_prefix(source).map_err(io::Error::other)?;let target=destination.join(relative);
        let lib=std::fs::read_to_string(package.join("src/lib.rs"))?;
        let commands=std::fs::read_to_string(package.join("src/command_helpers.rs"))?;
        for(path,text)in cc_linux_process_transform(&lib,&commands).map_err(io::Error::other)?{{std::fs::write(target.join(path),text)?;}}
        let tempfile=target.join("src/tempfile.rs");
        if tempfile.is_file(){{let text=std::fs::read_to_string(&tempfile)?;let text=replace_once(&text,"    RandomState::new().build_hasher().finish()","    crate::mantle_process::random(|| RandomState::new().build_hasher().finish())").map_err(io::Error::other)?;std::fs::write(tempfile,text)?;}}
    }}
    Ok(())
}}
"#,
        process = super::process_source()
    )
}
