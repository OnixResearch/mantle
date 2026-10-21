//! I/O-only continuation state for the original mrustc proc-macro protocol.
pub(super) const SOURCE: &str = r###"
const MACRO_BYTES_MAX:u64=2*1024*1024*1024;
fn macro_text(path:&Path)->io::Result<String>{let m=std::fs::metadata(path)?;if !m.is_file()||m.len()>64*1024*1024{return Err(io::Error::other("macro identity exceeds bound"));}let text=std::fs::read_to_string(path)?;if text.contains('\0'){return Err(io::Error::other("NUL in macro identity"));}Ok(text)}
fn macro_same_image(left:&Path,right:&Path)->io::Result<bool>{
    let a=std::fs::metadata(left)?;let b=std::fs::metadata(right)?;
    if !a.is_file()||!b.is_file()||a.len()>MACRO_BYTES_MAX||b.len()>MACRO_BYTES_MAX{return Err(io::Error::other("macro image is not a bounded regular file"));}
    if a.len()!=b.len(){return Ok(false);}
    let mut left=std::fs::File::open(left)?;let mut right=std::fs::File::open(right)?;let mut x=[0u8;65536];let mut y=[0u8;65536];let mut remaining=a.len();
    while remaining!=0{let n=remaining.min(x.len() as u64)as usize;left.read_exact(&mut x[..n])?;right.read_exact(&mut y[..n])?;if x[..n]!=y[..n]{return Ok(false);}remaining-=n as u64;}Ok(true)
}
fn macro_complete(state:&Path)->io::Result<bool>{let p=state.join(".macro-complete");match std::fs::read(&p){Ok(v)if v==b"actual-compiler-exit-zero\n"=>Ok(true),Ok(_)=>Err(io::Error::other("invalid observed macro completion state")),Err(e)if e.kind()==io::ErrorKind::NotFound=>Ok(false),Err(e)=>Err(e)}}
fn macro_classify(args:&[String])->io::Result<i32>{
    let phase=Path::new(&args[1]);let prior=Path::new(&args[2]);let out=Path::new(&args[3]);let package=Path::new(args.last().unwrap());
    if package!=Path::new("/inputs/package"){return Err(io::Error::other("macro source world differs from declared package namespace"));}
    let observed=envelope_output(phase,"exit-status");
    if !observed.try_exists()?{if !macro_complete(prior)?{return Err(io::Error::other("skipped unfinished source compiler"));}copy(prior,out,&mut 0)?;copy(Path::new(&args[4]),Path::new(&args[5]),&mut 0)?;return Ok(0);}
    let code=status(&observed)?;if code!=0&&code!=75{return Err(io::Error::other(format!("original compiler failed with actual exit {code}")));}
    copy(&envelope_output(phase,"generated-tmp"),Path::new(&args[5]),&mut 0)?;
    let translation=envelope_output(phase,"translation");copy(&translation,&out.join("translation"),&mut 0)?;
    if code==0{std::fs::write(out.join(".macro-complete"),b"actual-compiler-exit-zero\n")?;return Ok(0);}
    let request=translation.join("macro-request");let executable=macro_text(&request.join("executable"))?;let requested=Path::new(&executable);
    let cwd=macro_text(&request.join("cwd"))?;let cwd=std::fs::canonicalize(cwd)?;if !cwd.starts_with(std::fs::canonicalize(package)?){return Err(io::Error::other("original macro cwd escapes its exact source world"));}
    let resolved=if requested.is_absolute(){requested.to_path_buf()}else{cwd.join(requested)};
    let canonical=std::fs::canonicalize(resolved)?;let root=std::fs::canonicalize(package)?;
    if !canonical.starts_with(&root){return Err(io::Error::other("original macro executable escapes its exact source world"));}
    let name=requested.file_name().and_then(|n|n.to_str()).ok_or_else(||io::Error::other("macro executable basename is not UTF-8"))?;
    let mut selected=None;
    for(index,pair)in args[6..args.len()-1].chunks_exact(2).enumerate(){
        if pair[0]==name{if selected.is_some(){return Err(io::Error::other("ambiguous original macro image inventory"));}if !macro_same_image(&canonical,Path::new(&pair[1]))?{return Err(io::Error::other("requested macro bytes differ from exact Produced image"));}selected=Some(i32::try_from(index+1).map_err(io::Error::other)?);}
    }
    let selected=selected.ok_or_else(||io::Error::other(format!("source compiler requested undeclared macro executable {executable:?}")))?;
    copy(&request,&out.join("request"),&mut 0)?;std::fs::write(out.join(".macro-selected"),format!("{selected}\n"))?;Ok(1)
}
fn macro_environment(path:&Path)->io::Result<BTreeMap<String,String>>{
    let bytes=std::fs::read(path)?;if bytes.len()>64*1024*1024{return Err(io::Error::other("macro environment exceeds bound"));}let mut rest=bytes.as_slice();let mut out=BTreeMap::new();let mut count=0;
    while !rest.is_empty(){if rest.len()<8{return Err(io::Error::other("truncated actual macro environment frame"));}let k=u32::from_le_bytes(rest[..4].try_into().unwrap())as usize;let v=u32::from_le_bytes(rest[4..8].try_into().unwrap())as usize;rest=&rest[8..];let size=k.checked_add(v).filter(|n|*n<=rest.len()).ok_or_else(||io::Error::other("invalid macro environment frame lengths"))?;
        let key=std::str::from_utf8(&rest[..k]).map_err(io::Error::other)?.to_owned();let value=std::str::from_utf8(&rest[k..size]).map_err(io::Error::other)?.to_owned();rest=&rest[size..];count+=1;
        if count>32768||key.is_empty()||key.contains(['=','\0'])||value.contains('\0')||out.insert(key,value).is_some(){return Err(io::Error::other("invalid or duplicate actual macro environment pair"));}
    }Ok(out)
}
fn macro_selected(state:&Path)->io::Result<usize>{let text=macro_text(&state.join(".macro-selected"))?;let selected=text.strip_suffix('\n').ok_or_else(||io::Error::other("unterminated original macro selection"))?.parse::<usize>().map_err(io::Error::other)?;if selected==0||text!=format!("{selected}\n"){return Err(io::Error::other("noncanonical original macro selection"));}Ok(selected-1)}
fn macro_invocation(args:&[String])->io::Result<()>{
    let state=Path::new(&args[1]);if macro_complete(state)?{return Err(io::Error::other("macro invocation after genuine compiler completion"));}
    let selected=macro_selected(state)?;let binding=args[6..].chunks_exact(2).nth(selected).ok_or_else(||io::Error::other("macro Produced image inventory changed"))?;
    let request=state.join("request");let name=macro_text(&request.join("macro-name"))?;if name.is_empty(){return Err(io::Error::other("empty original macro name"));}
    let cwd=macro_text(&request.join("cwd"))?;let cwd=std::fs::canonicalize(cwd)?;if !cwd.starts_with(std::fs::canonicalize("/inputs/package")?){return Err(io::Error::other("original macro cwd escapes its exact source world"));}
    let expected=macro_text(&request.join("executable"))?;let expected=Path::new(&expected);let expected=std::fs::canonicalize(if expected.is_absolute(){expected.to_path_buf()}else{cwd.join(expected)})?;
    if !expected.starts_with(std::fs::canonicalize("/inputs/package")?)||expected.file_name().and_then(|v|v.to_str())!=Some(binding[0].as_str())||!macro_same_image(&expected,Path::new(&binding[1]))?{return Err(io::Error::other("selected original macro differs from exact Produced image inventory"));}
    std::fs::copy(&binding[1],&args[2])?;
    let environment=macro_environment(&request.join("environment"))?;let mut data=String::new();let argument=data_word(&name,&mut data);let mut fields=Vec::new();
    for(key,value)in environment{if key.starts_with("MRUSTC_CAIRN_"){continue;}let word=if value.contains("/inputs/")||value.contains("/outputs/")||value.contains("/mantle-rust-runtime")||value.contains("/tmp"){typed_word(&value)?}else{data_word(&value,&mut data)};fields.push(format!("{}:{word}",json_string(&key)));}
    let cwd=typed_word(cwd.to_str().ok_or_else(||io::Error::other("non-UTF8 original macro cwd"))?)?;let executable_path=typed_word(expected.to_str().ok_or_else(||io::Error::other("non-UTF8 original macro executable path"))?)?;
    std::fs::write(&args[3],format!("{{\"arguments\":[{argument}],\"environment\":{{{}}},\"cwd\":{cwd},\"executable_path\":{executable_path},\"workspace_mounts\":{{\"/tmp\":{{\"output\":\"generated-tmp\",\"path\":\"\"}}}}}}",fields.join(",")))?;std::fs::write(&args[4],data)?;
    let stdin=request.join("stdin");if std::fs::metadata(&stdin)?.len()>MACRO_BYTES_MAX{return Err(io::Error::other("original macro stdin exceeds bound"));}std::fs::copy(stdin,&args[5])?;Ok(())
}
fn macro_reply(args:&[String])->io::Result<i32>{
    let state=Path::new(&args[1]);let prior=Path::new(&args[2]);let out=Path::new(&args[3]);copy(prior,out,&mut 0)?;
    if macro_complete(state)?{for phase in &args[6..]{if envelope_output(Path::new(phase),"exit-status").try_exists()?{return Err(io::Error::other("macro executed after compiler completion"));}}copy(Path::new(&args[4]),Path::new(&args[5]),&mut 0)?;return Ok(0);}
    macro_selected(state)?;let phases=&args[6..];if phases.len()!=1{return Err(io::Error::other("macro request must observe exactly one protected selected-image leaf"));}let selected=0;
    for(index,phase)in phases.iter().enumerate(){let path=envelope_output(Path::new(phase),"exit-status");let executed=path.try_exists()?;if executed!=(index==selected){return Err(io::Error::other("macro request did not execute exactly its selected Produced leaf"));}if executed&&status(&path)?!=0{return Err(io::Error::other("original macro executable failed"));}}
    copy(&envelope_output(Path::new(&phases[selected]),"generated-tmp"),Path::new(&args[5]),&mut 0)?;
    let mut ordinals=std::fs::read_dir(prior)?.map(|entry|{let entry=entry?;if !entry.file_type()?.is_dir(){return Err(io::Error::other("non-directory macro history entry"));}let name=entry.file_name().into_string().map_err(|_|io::Error::other("non-UTF8 macro history ordinal"))?;let ordinal=name.parse::<usize>().map_err(io::Error::other)?;if name!=ordinal.to_string(){return Err(io::Error::other("noncanonical macro history ordinal"));}Ok(ordinal)}).collect::<io::Result<Vec<_>>>()?;ordinals.sort_unstable();if ordinals.iter().enumerate().any(|(i,n)|i!=*n){return Err(io::Error::other("macro history is not contiguous"));}
    let reply=out.join(ordinals.len().to_string());copy(&state.join("request"),&reply,&mut 0)?;copy(&envelope_output(Path::new(&phases[selected]),"stdout"),&reply.join("stdout"),&mut 0)?;Ok(1)
}
fn macro_finish(args:&[String])->io::Result<()>{let state=Path::new(&args[1]);if !macro_complete(state)?{return Err(io::Error::other("source compiler macro continuation resource reservation exhausted before genuine completion"));}copy(&state.join("translation"),Path::new(&args[2]),&mut 0)}
"###;
