//! Pure-I/O continuation control. Only the shared source controller parses
//! actual process requests and binds independently produced executable images.
pub(super) const SOURCE: &str = r###"
fn status(path:&Path)->io::Result<i32>{let s=std::fs::read_to_string(path)?;let code=s.strip_suffix('\n').ok_or_else(||io::Error::other("unterminated producer status"))?.parse::<i32>().map_err(io::Error::other)?;if !(0..=255).contains(&code)||s!=format!("{code}\n"){return Err(io::Error::other("noncanonical producer status"));}Ok(code)}
fn envelope_output<'a>(envelope:&'a Path,name:&str)->PathBuf{envelope.join("outputs").join(name)}
fn classify(args:&[String])->io::Result<i32>{
    // Input is an authenticated ObservePhase envelope, not script assertions.
    let envelope=Path::new(&args[1]);let result=Path::new(&args[2]);
    std::fs::create_dir_all(result)?;
    let logical_out=std::fs::read_to_string(&args[4])?;let relative_out=logical_out.strip_prefix("/tmp/").ok_or_else(||io::Error::other("original output root is outside temporary workspace"))?;safe(relative_out)?;
    let state=envelope.join("state");
    let code_path=envelope_output(envelope,"exit-status");
    if !code_path.try_exists()?{
        // A skipped resume preserves its previously completed state. The
        // selector's completion marker was made only after an actual exit 0.
        let prior=Path::new(&args[3]);if !prior.join("out-dir").join(relative_out).join(".mantle-successor-completed").is_file(){return Err(io::Error::other("skipped unfinished original script"));}
        copy(prior,result,&mut 0)?;return Ok(0);
    }
    copy(&state,&result.join("out-dir"),&mut 0)?;
    std::fs::write(result.join("logical-out"),&logical_out)?;
    match status(&code_path)?{
        0=>{
            std::fs::write(result.join("out-dir").join(relative_out).join(".mantle-successor-completed"),b"actual-script-exit-zero\n")?;
            copy(&envelope_output(envelope,"stdout"),&result.join("stdout"),&mut 0)?;
            Ok(0)
        },
        75=>{
            let path=state.join(relative_out).join(".mantle-successor-request");
            copy(&path,&result.join("request"),&mut 0)?;
            // The shared controller resolves the authentic image/PATH and
            // validates every descriptor. No package-specific command family
            // or source-invented GCC flag inventory chooses execution here.
            Ok(1)
        },
        code=>Err(io::Error::other(format!("original script failed with actual exit {code}"))),
    }
}
fn quote_response(words:&[String])->String{let mut out=String::new();for word in words{out.push('"');for c in word.chars(){if c=='"'||c=='\\'{out.push('\\');}out.push(c);}out.push_str("\"\n");}out}
fn merge_reply(args:&[String])->io::Result<i32> {
    let state=Path::new(&args[1]);let prior=Path::new(&args[2]);let out=Path::new(&args[3]);let ordinal=&args[4];copy(prior,out,&mut 0)?;
    let logical=std::fs::read_to_string(state.join("logical-out"))?;let relative=logical.strip_prefix("/tmp/").ok_or_else(||io::Error::other("invalid original output root"))?;safe(relative)?;let root=Path::new(&args[6]);let status_path=envelope_output(root,"exit-status");
    if state.join("out-dir").join(relative).join(".mantle-successor-completed").is_file(){if status_path.try_exists()?{return Err(io::Error::other("native command executed after original script completion"));}return Ok(0);}
    if args[5]!="shared-result"{return Err(io::Error::other("original request lacks shared source result"));}let actual=status(&status_path)?;let shared=envelope_output(root,"native-request");if status(&shared.join("exit-status"))?!=actual{return Err(io::Error::other("shared source result disagrees with actual protected exit"));}
    let reply=out.join(ordinal);std::fs::create_dir_all(&reply)?;copy(&state.join("request"),&reply.join("request"),&mut 0)?;copy(&envelope_output(root,"native-files"),&reply.join("workspace"),&mut 0)?;copy(&shared.join("positions"),&reply.join("positions"),&mut 0)?;copy(&shared.join("workspace-times"),&reply.join("workspace-times"),&mut 0)?;
    for index in 0..64{let name=format!("descriptor-{index}");let path=shared.join(&name);if path.try_exists()?{copy(&path,&reply.join(name),&mut 0)?;}}
    copy(&envelope_output(root,"stdout"),&reply.join("stdout"),&mut 0)?;copy(&envelope_output(root,"stderr"),&reply.join("stderr"),&mut 0)?;std::fs::write(reply.join("status"),format!("{actual}\n"))?;Ok(1)
}
fn finish_script(args:&[String])->io::Result<()> {
    let state=Path::new(&args[1]);let logical_out=std::fs::read_to_string(state.join("logical-out"))?;let relative_out=logical_out.strip_prefix("/tmp/").ok_or_else(||io::Error::other("invalid completed logical output"))?;safe(relative_out)?;let directory=state.join("out-dir").join(relative_out);if !directory.join(".mantle-successor-completed").is_file(){return Err(io::Error::other("original script exhausted source-authored resource reservation"));}
    copy(&directory,Path::new(&args[2]),&mut 0)?;
    copy(&state.join("out-dir"),Path::new(&args[3]),&mut 0)?;
    for root in [PathBuf::from(&args[2]),Path::new(&args[3]).join(relative_out)]{for name in [".mantle-successor-completed",".mantle-successor-request",".mantle-cc-random"]{let path=root.join(name);match std::fs::symlink_metadata(&path){Ok(metadata)=>{if metadata.is_dir(){std::fs::remove_dir_all(path)?;}else{std::fs::remove_file(path)?;}},Err(error)if error.kind()==io::ErrorKind::NotFound=>(),Err(error)=>return Err(error)}}}
    let bytes=std::fs::read(state.join("stdout"))?;io::stdout().write_all(&bytes)
}
"###;
