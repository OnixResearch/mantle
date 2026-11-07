//! Pure-I/O continuation control. Only the shared source controller parses
//! actual process requests and binds independently produced executable images.
pub(super) const SOURCE: &str = r###"
fn status(path:&Path)->io::Result<i32>{let s=std::fs::read_to_string(path)?;let code=s.strip_suffix('\n').ok_or_else(||io::Error::other("unterminated producer status"))?.parse::<i32>().map_err(io::Error::other)?;if !(0..=255).contains(&code)||s!=format!("{code}\n"){return Err(io::Error::other("noncanonical producer status"));}Ok(code)}
fn envelope_output<'a>(envelope:&'a Path,name:&str)->PathBuf{envelope.join("outputs").join(name)}
fn quote_response(words:&[String])->String{let mut out=String::new();for word in words{out.push('"');for c in word.chars(){if c=='"'||c=='\\'{out.push('\\');}out.push(c);}out.push_str("\"\n");}out}
fn script_frame(args:&[String])->io::Result<()> {
    let root=Path::new(&args[1]);std::fs::create_dir_all(root)?;
    copy(Path::new(&args[2]),&root.join("script"),&mut 0)?;
    let mut frame=std::fs::File::create(root.join("environment"))?;
    let mut seen=std::collections::BTreeSet::new();let mut cwd=None;
    for pair in args[3..].chunks_exact(2) {
        let key=&pair[0];let value=&pair[1];
        if key.is_empty()||key.contains(['=','\0'])||value.contains('\0')||key.len()>64*1024*1024||value.len()>64*1024*1024||!seen.insert(key){return Err(io::Error::other("invalid script environment pair"));}
        if key=="CARGO_MANIFEST_DIR" {if !Path::new(value).is_absolute(){return Err(io::Error::other("script cwd is not absolute"));}cwd=Some(value);}
        for value in [key,value] {write!(frame,"{}:",value.len())?;frame.write_all(value.as_bytes())?;frame.write_all(b",")?;}
    }
    let cwd=cwd.ok_or_else(||io::Error::other("script environment lacks Cargo manifest directory"))?;
    std::fs::write(root.join("cwd"),cwd)
}
fn finish_compact_script(args:&[String])->io::Result<()> {
    let checkpoint=Path::new(&args[1]);
    if std::fs::read(checkpoint.join("state"))?!=b"done\n"||status(&checkpoint.join("exit-status"))?!=0{return Err(io::Error::other("original script did not complete successfully"));}
    let logical_out=std::fs::read_to_string(&args[2])?;
    let relative=logical_out.strip_prefix("/tmp/").ok_or_else(||io::Error::other("invalid completed logical output"))?;safe(relative)?;
    let tmp=checkpoint.join("workspace/tmp");copy(&tmp.join(relative),Path::new(&args[3]),&mut 0)?;
    copy(&tmp,Path::new(&args[4]),&mut 0)?;
    for root in [PathBuf::from(&args[3]),Path::new(&args[4]).join(relative)] {
        for name in [".mantle-successor-request",".mantle-cc-random"] {
            let path=root.join(name);match std::fs::symlink_metadata(&path) {
                Ok(meta)=>if meta.is_dir(){std::fs::remove_dir_all(path)?;}else{std::fs::remove_file(path)?;},
                Err(error)if error.kind()==io::ErrorKind::NotFound=>(),Err(error)=>return Err(error),
            }
        }
    }
    io::copy(&mut std::fs::File::open(checkpoint.join("stdout"))?,&mut io::stdout())?;Ok(())
}
#[cfg(test)]
mod native_helper_protocol_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64,Ordering};
    static NEXT:AtomicU64=AtomicU64::new(0);
    struct Root(PathBuf);
    impl Drop for Root {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    fn root()->Root {let path=std::env::temp_dir().join(format!("mantle-native-helper-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&path).unwrap();Root(path)}
    #[test]
    fn environment_frame_requires_unique_pairs_and_manifest_cwd() {
        let root=root();let target=root.0.join("frame");let script=root.0.join("script");std::fs::write(&script,b"fixture image").unwrap();
        let mut args=vec!["--script-frame".into(),target.to_str().unwrap().into(),script.to_str().unwrap().into(),"CARGO_MANIFEST_DIR".into(),"/inputs/package/crate".into(),"FLAGS".into(),"a b\n/c".into()];
        script_frame(&args).unwrap();
        assert_eq!(std::fs::read(target.join("cwd")).unwrap(),b"/inputs/package/crate");
        assert_eq!(std::fs::read(target.join("environment")).unwrap(),b"18:CARGO_MANIFEST_DIR,21:/inputs/package/crate,5:FLAGS,6:a b\n/c,");
        args.extend(["FLAGS".into(),"changed".into()]);assert!(script_frame(&args).is_err());
        args.truncate(3);assert!(script_frame(&args).is_err());
    }
    #[test]
    fn completion_carries_all_tmp_without_publishing_protocol_state() {
        let root=root();let checkpoint=root.0.join("checkpoint");let tmp=checkpoint.join("workspace/tmp");
        let out=tmp.join("mantle-rust/unit/out");std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("generated.rs"),b"actual generated source").unwrap();
        std::fs::write(out.join(".mantle-successor-request"),b"request").unwrap();
        std::fs::create_dir(out.join(".mantle-cc-random")).unwrap();
        std::fs::write(tmp.join("transitive-state"),[0,255,1]).unwrap();
        std::fs::create_dir(checkpoint.join("workspace/.mantle-rust-controller")).unwrap();
        std::fs::write(checkpoint.join("state"),b"done\n").unwrap();std::fs::write(checkpoint.join("exit-status"),b"0\n").unwrap();
        std::fs::write(checkpoint.join("stdout"),b"").unwrap();let logical=root.0.join("logical");std::fs::write(&logical,"/tmp/mantle-rust/unit/out").unwrap();
        let destination=root.0.join("out");let generated=root.0.join("generated");
        let args=vec!["--finish-compact-script".into(),checkpoint.to_str().unwrap().into(),logical.to_str().unwrap().into(),destination.to_str().unwrap().into(),generated.to_str().unwrap().into()];
        finish_compact_script(&args).unwrap();
        assert_eq!(std::fs::read(destination.join("generated.rs")).unwrap(),b"actual generated source");
        assert_eq!(std::fs::read(generated.join("transitive-state")).unwrap(),[0,255,1]);
        for path in [destination.join(".mantle-successor-request"),generated.join("mantle-rust/unit/out/.mantle-cc-random"),generated.join(".mantle-rust-controller")] {assert!(!path.exists());}
        for (state,status) in [("active\n","0\n"),("done\n","75\n"),("done\n","1\n")] {
            std::fs::write(checkpoint.join("state"),state).unwrap();std::fs::write(checkpoint.join("exit-status"),status).unwrap();
            assert!(finish_compact_script(&args).is_err());
        }
    }
}
"###;
