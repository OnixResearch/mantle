//! I/O-only continuation state for the original mrustc proc-macro protocol.
pub(super) const SOURCE: &str = r###"
const MACRO_BYTES_MAX:u64=2*1024*1024*1024;
const MACRO_FRAME_MAX:u64=64*1024*1024;
const MACRO_ENTRIES_MAX:usize=262144;
fn macro_error(message:&str)->io::Error{io::Error::other(message)}
fn macro_bytes(path:&Path,limit:u64)->io::Result<Vec<u8>>{
    let meta=std::fs::symlink_metadata(path)?;
    if !meta.is_file()||meta.len()>limit{return Err(macro_error("macro record is not a bounded regular file"));}
    std::fs::read(path)
}
fn macro_text(path:&Path)->io::Result<String>{
    let text=String::from_utf8(macro_bytes(path,MACRO_FRAME_MAX)?).map_err(io::Error::other)?;
    if text.contains('\0'){return Err(macro_error("NUL in macro identity"));}Ok(text)
}
fn macro_number(text:&str,max:usize)->io::Result<usize>{
    let n=text.parse::<usize>().map_err(io::Error::other)?;
    if n>max||text!=n.to_string(){return Err(macro_error("noncanonical macro count"));}Ok(n)
}
fn macro_actual_status(path:&Path)->io::Result<i32>{
    let text=String::from_utf8(macro_bytes(path,4)?).map_err(io::Error::other)?;
    let number=text.strip_suffix('\n').ok_or_else(||macro_error("unterminated actual macro command status"))?;
    Ok(macro_number(number,255)? as i32)
}
fn macro_same_image(left:&Path,right:&Path)->io::Result<bool>{
    let a=std::fs::metadata(left)?;let b=std::fs::symlink_metadata(right)?;
    if !a.is_file()||!b.is_file()||a.len()>MACRO_BYTES_MAX||b.len()>MACRO_BYTES_MAX{return Err(macro_error("macro image is not a bounded regular file"));}
    if a.len()!=b.len(){return Ok(false);}
    let mut left=std::fs::File::open(left)?;let mut right=std::fs::File::open(right)?;
    let mut x=[0u8;65536];let mut y=[0u8;65536];let mut remaining=a.len();
    while remaining!=0{let n=remaining.min(x.len() as u64)as usize;left.read_exact(&mut x[..n])?;right.read_exact(&mut y[..n])?;if x[..n]!=y[..n]{return Ok(false);}remaining-=n as u64;}Ok(true)
}
fn macro_directory(path:&Path)->io::Result<()>{
    if !std::fs::symlink_metadata(path)?.is_dir(){return Err(macro_error("macro state directory is not a real directory"));}Ok(())
}
fn macro_names(path:&Path)->io::Result<Vec<String>>{
    macro_directory(path)?;let mut names=Vec::new();
    for entry in std::fs::read_dir(path)?{if names.len()==MACRO_ENTRIES_MAX{return Err(macro_error("macro tree entry bound"));}names.push(entry?.file_name().into_string().map_err(|_|macro_error("non-UTF8 macro tree member"))?);}
    names.sort();Ok(names)
}
fn macro_inventory(pairs:&[String])->io::Result<Vec<String>>{
    if pairs.is_empty()||pairs.len()%2!=0||pairs.len()/2>32768{return Err(macro_error("invalid macro image inventory layout"));}
    let mut names=std::collections::BTreeSet::new();let mut ordered=Vec::new();
    for pair in pairs.chunks_exact(2){let name=&pair[0];safe(name)?;
        if name.contains('/')||name.contains(['\n','\r','\0'])||!names.insert(name){return Err(macro_error("invalid or duplicate macro image basename"));}
        let meta=std::fs::symlink_metadata(&pair[1])?;if !meta.is_file()||meta.len()>MACRO_BYTES_MAX{return Err(macro_error("invalid Produced macro image"));}ordered.push(name.clone());
    }Ok(ordered)
}
fn macro_inventory_check(checkpoint:&Path,pairs:&[String])->io::Result<()>{
    let names=macro_inventory(pairs)?;let expected=names.iter().map(|name|format!("{name}\n")).collect::<String>();
    if macro_text(&checkpoint.join("inventory"))?!=expected{return Err(macro_error("macro Produced image inventory changed"));}Ok(())
}
fn macro_start_arguments(args:&[String])->io::Result<usize>{
    if args.len()<9||args[0]!="--macro-control"{return Err(macro_error("invalid original macro controller arguments"));}
    let slots=macro_number(&args[6],MACRO_ENTRIES_MAX)?;if slots==0{return Err(macro_error("empty original macro reservation"));}
    macro_inventory(&args[7..])?;Ok(slots)
}
fn macro_history(root:&Path)->io::Result<usize>{
    let names=macro_names(root)?;let count=names.len();
    let expected=(0..count).map(|n|n.to_string()).collect::<std::collections::BTreeSet<_>>();
    if names.iter().cloned().collect::<std::collections::BTreeSet<_>>()!=expected{return Err(macro_error("macro replay history is not contiguous"));}
    for name in names{macro_directory(&root.join(name))?;}Ok(count)
}
fn macro_checkpoint(root:&Path)->io::Result<(String,usize,usize)>{
    macro_directory(root)?;let phase=macro_text(&root.join("phase"))?;
    if !matches!(phase.as_str(),"compiler\n"|"macro\n"|"done\n"){return Err(macro_error("invalid macro controller phase"));}
    let expected=if phase=="done\n"{"done\n"}else{"paused\n"};
    if macro_text(&root.join("state"))?!=expected{return Err(macro_error("macro phase/state disagreement"));}
    let slots=macro_text(&root.join("slots"))?;let slots=macro_number(&slots,MACRO_ENTRIES_MAX)?;
    let completed=macro_text(&root.join("completed"))?;let completed=macro_number(&completed,slots)?;
    if slots==0||macro_history(&root.join("workspace/history"))?!=completed{return Err(macro_error("invalid macro reservation/history state"));}
    for name in ["tmp","translation"]{macro_directory(&root.join("workspace").join(name))?;}
    if phase=="macro\n"&&completed==slots{return Err(macro_error("macro continuation reservation exhausted"));}
    Ok((phase,slots,completed))
}
fn macro_phase(root:&Path,phase:&str)->io::Result<()>{
    std::fs::write(root.join("phase"),format!("{phase}\n"))?;
    std::fs::write(root.join("state"),if phase=="done"{b"done\n".as_slice()}else{b"paused\n".as_slice()})
}
// Workspace times are executor wire records, not timestamps guessed from stdout.
fn macro_record(bytes:&mut Vec<u8>,value:&[u8])->io::Result<()>{
    if bytes.len().checked_add(value.len()+32).filter(|n|*n<=MACRO_FRAME_MAX as usize).is_none(){return Err(macro_error("macro time frame bound"));}
    write!(bytes,"{}:",value.len())?;bytes.extend_from_slice(value);bytes.push(b',');Ok(())
}
fn macro_read_record<'a>(bytes:&mut &'a [u8])->io::Result<&'a [u8]>{
    let colon=bytes.iter().position(|b|*b==b':').ok_or_else(||macro_error("missing macro time record delimiter"))?;
    if colon==0||colon>10{return Err(macro_error("invalid macro time record length"));}
    let text=std::str::from_utf8(&bytes[..colon]).map_err(io::Error::other)?;let n=macro_number(text,MACRO_FRAME_MAX as usize)?;
    let end=colon.checked_add(1).and_then(|n0|n0.checked_add(n)).ok_or_else(||macro_error("macro record overflow"))?;
    if bytes.get(end)!=Some(&b','){return Err(macro_error("truncated macro time record"));}let record=&bytes[colon+1..end];*bytes=&bytes[end+1..];Ok(record)
}
type MacroTimes=BTreeMap<String,(i64,i64)>;
fn macro_times_read(path:&Path)->io::Result<MacroTimes>{
    let bytes=macro_bytes(path,MACRO_FRAME_MAX)?;let mut rest=bytes.as_slice();
    if macro_read_record(&mut rest)?!=b"MANTLE-WORKSPACE-TIMES-1"{return Err(macro_error("invalid workspace time frame version"));}
    let mut times=BTreeMap::new();while !rest.is_empty(){
        let name=std::str::from_utf8(macro_read_record(&mut rest)?).map_err(io::Error::other)?.to_owned();safe_operand_path(&name)?;
        let mut numbers=[0i64;2];for value in &mut numbers{let text=std::str::from_utf8(macro_read_record(&mut rest)?).map_err(io::Error::other)?;*value=text.parse().map_err(io::Error::other)?;if text!=value.to_string(){return Err(macro_error("noncanonical workspace time integer"));}}
        if !(0..1_000_000_000).contains(&numbers[1])||times.len()==MACRO_ENTRIES_MAX||times.insert(name,(numbers[0],numbers[1])).is_some(){return Err(macro_error("invalid or duplicate workspace time record"));}
    }if !times.contains_key(""){return Err(macro_error("workspace time frame lacks root"));}Ok(times)
}
fn macro_times_capture(root:&Path)->io::Result<MacroTimes>{
    use std::os::unix::fs::MetadataExt;
    fn node(root:&Path,name:&str,out:&mut MacroTimes)->io::Result<()>{
        if out.len()==MACRO_ENTRIES_MAX{return Err(macro_error("workspace time node bound"));}
        let path=root.join(name);let meta=std::fs::symlink_metadata(&path)?;
        if !meta.is_dir()&&!meta.is_file()&&!meta.file_type().is_symlink(){return Err(macro_error("unsupported macro workspace member"));}
        out.insert(name.to_owned(),(meta.mtime(),meta.mtime_nsec()));
        if meta.is_dir(){for child in macro_names(&path)?{let child=if name.is_empty(){child}else{format!("{name}/{child}")};node(root,&child,out)?;}}Ok(())
    }
    let mut out=BTreeMap::new();node(root,"",&mut out)?;Ok(out)
}
fn macro_times_checked(workspace:&Path,path:&Path)->io::Result<MacroTimes>{
    let times=macro_times_read(path)?;let actual=macro_times_capture(workspace)?;
    if !times.keys().eq(actual.keys()){return Err(macro_error("workspace time inventory differs from actual workspace"));}Ok(times)
}
// Executor traversal visits sorted children before their directory, with root
// last. Full-path BTreeMap order is not equivalent (e.g. a/file versus a-b).
fn macro_time_order(left:&str,right:&str)->std::cmp::Ordering{
    let mut left=left.split('/').filter(|part|!part.is_empty());let mut right=right.split('/').filter(|part|!part.is_empty());
    loop{match(left.next(),right.next()){
        (Some(a),Some(b))=>{let order=a.cmp(b);if order!=std::cmp::Ordering::Equal{return order;}},
        (Some(_),None)=>return std::cmp::Ordering::Less,
        (None,Some(_))=>return std::cmp::Ordering::Greater,
        (None,None)=>return std::cmp::Ordering::Equal,
    }}
}
fn macro_times_write(path:&Path,times:&MacroTimes)->io::Result<()>{
    let mut bytes=Vec::new();macro_record(&mut bytes,b"MANTLE-WORKSPACE-TIMES-1")?;
    let mut ordered=times.iter().collect::<Vec<_>>();ordered.sort_unstable_by(|a,b|macro_time_order(a.0,b.0));
    for(name,(seconds,nanos))in ordered{macro_record(&mut bytes,name.as_bytes())?;macro_record(&mut bytes,seconds.to_string().as_bytes())?;macro_record(&mut bytes,nanos.to_string().as_bytes())?;}std::fs::write(path,bytes)
}
fn macro_times_carry(workspace:&Path,prior:&MacroTimes,refresh:&[&str])->io::Result<MacroTimes>{
    let mut actual=macro_times_capture(workspace)?;
    for(name,time)in &mut actual{if !refresh.iter().any(|prefix|name.as_str()==*prefix||name.strip_prefix(*prefix).is_some_and(|rest|rest.starts_with('/'))){if let Some(old)=prior.get(name){*time=*old;}}}Ok(actual)
}
fn macro_environment(path:&Path)->io::Result<BTreeMap<String,String>>{
    let bytes=macro_bytes(path,MACRO_FRAME_MAX)?;let mut rest=bytes.as_slice();let mut out=BTreeMap::new();
    while !rest.is_empty(){if rest.len()<8{return Err(macro_error("truncated actual macro environment frame"));}
        let k=u32::from_le_bytes(rest[..4].try_into().unwrap())as usize;let v=u32::from_le_bytes(rest[4..8].try_into().unwrap())as usize;rest=&rest[8..];
        let size=k.checked_add(v).filter(|n|*n<=rest.len()).ok_or_else(||macro_error("invalid macro environment frame lengths"))?;
        let key=std::str::from_utf8(&rest[..k]).map_err(io::Error::other)?.to_owned();let value=std::str::from_utf8(&rest[k..size]).map_err(io::Error::other)?.to_owned();rest=&rest[size..];
        if out.len()==32768||key.is_empty()||key.contains(['=','\0'])||value.contains('\0')||out.insert(key,value).is_some(){return Err(macro_error("invalid or duplicate actual macro environment pair"));}
    }Ok(out)
}
fn macro_request(checkpoint:&Path,pairs:&[String])->io::Result<(String,PathBuf,PathBuf)>{
    macro_inventory_check(checkpoint,pairs)?;let request=checkpoint.join("request");macro_directory(&request)?;
    let expected=["cwd","environment","executable","macro-name","stdin"].iter().map(|s|s.to_string()).collect::<Vec<_>>();
    if macro_names(&request)?!=expected{return Err(macro_error("unexpected original macro request members"));}
    let executable=macro_text(&request.join("executable"))?;let requested=Path::new(&executable);
    let cwd=std::fs::canonicalize(macro_text(&request.join("cwd"))?)?;macro_directory(&cwd)?;
    let package=std::fs::canonicalize("/inputs/package")?;
    let resolved=std::fs::canonicalize(if requested.is_absolute(){requested.to_path_buf()}else{cwd.join(requested)})?;
    if !cwd.starts_with(&package)||!resolved.starts_with(&package){return Err(macro_error("macro request escapes exact source package"));}
    let name=requested.file_name().and_then(|s|s.to_str()).ok_or_else(||macro_error("non-UTF8 macro basename"))?;
    if resolved.file_name().and_then(|s|s.to_str())!=Some(name){return Err(macro_error("macro canonical basename differs from request"));}
    let pair=pairs.chunks_exact(2).find(|pair|pair[0]==name).ok_or_else(||macro_error("undeclared original macro image"))?;
    if !macro_same_image(&resolved,Path::new(&pair[1]))?{return Err(macro_error("requested macro differs from exact Produced image bytes"));}
    let macro_name=macro_text(&request.join("macro-name"))?;if macro_name.is_empty(){return Err(macro_error("empty original macro name"));}
    let stdin=std::fs::symlink_metadata(request.join("stdin"))?;if !stdin.is_file()||stdin.len()>MACRO_BYTES_MAX{return Err(macro_error("invalid original macro stdin"));}
    macro_environment(&request.join("environment"))?;Ok((name.to_owned(),cwd,resolved))
}
fn macro_start(args:&[String])->io::Result<i32>{
    if args.len()<12||args[2]!="--"{return Err(macro_error("invalid macro controller start layout"));}
    let original=&args[3..];let slots=macro_start_arguments(original)?;
    if original[1]!="-"||original[2]!="-"{return Err(macro_error("macro controller start cannot acknowledge synthetic results"));}
    let out=Path::new(&args[1]);std::fs::create_dir_all(out)?;
    let workspace=out.join("workspace");std::fs::create_dir_all(workspace.join("history"))?;std::fs::create_dir_all(workspace.join("translation"))?;
    macro_directory(Path::new(&original[5]))?;let tmp_times=macro_times_capture(Path::new(&original[5]))?;
    copy(Path::new(&original[5]),&workspace.join("tmp"),&mut 0)?;
    std::fs::write(out.join("compiler-config"),macro_bytes(Path::new(&original[4]),MACRO_FRAME_MAX)?)?;
    std::fs::write(out.join("inventory"),macro_inventory(&original[7..])?.iter().map(|s|format!("{s}\n")).collect::<String>())?;
    std::fs::write(out.join("slots"),slots.to_string())?;std::fs::write(out.join("completed"),b"0")?;
    let mut times=macro_times_capture(&workspace)?;for(name,time)in tmp_times{times.insert(if name.is_empty(){"tmp".into()}else{format!("tmp/{name}")},time);}
    macro_times_write(&out.join("times"),&times)?;macro_phase(out,"compiler")?;Ok(0)
}
fn macro_prepare_bindings(args:&[String],separator:usize)->io::Result<(BTreeMap<String,String>,Vec<String>)>{
    let mut bindings=BTreeMap::new();let mut command=false;let mut runtime=false;let mut roots=Vec::new();let mut at=9;
    while at<separator{match args[at].as_str(){
        "--command-inputs"=>{if command{return Err(macro_error("duplicate command input marker"));}command=true;at+=1;},
        "--runtime-inputs"=>{if !command||runtime||at+1>=separator{return Err(macro_error("invalid runtime input layout"));}runtime=true;let n=macro_number(&args[at+1],32768)?;at+=2;if at+n!=separator{return Err(macro_error("invalid runtime input count"));}let mut seen=std::collections::BTreeSet::new();for name in &args[at..separator]{if !roots.contains(name)||!seen.insert(name){return Err(macro_error("unbound or duplicate runtime input"));}roots.retain(|root|root!=name);}at=separator;},
        _=>{if runtime||at+1>=separator{return Err(macro_error("invalid controller binding pair"));}let name=&args[at];safe(name)?;if name.contains('/')||bindings.insert(name.clone(),args[at+1].clone()).is_some(){return Err(macro_error("duplicate controller input binding"));}if command{roots.push(name.clone());}at+=2;}
    }}if !command||!runtime{return Err(macro_error("missing controller binding markers"));}Ok((bindings,roots))
}
fn macro_prepare(args:&[String])->io::Result<i32>{
    if args.len()<13{return Err(macro_error("invalid macro prepare arguments"));}
    let separator=args[9..].iter().position(|s|s=="--").map(|n|n+9).ok_or_else(||macro_error("missing prepare identity separator"))?;
    let(bindings,roots)=macro_prepare_bindings(args,separator)?;let identity=&args[separator+1..];let checkpoint=Path::new(&args[1]);let(phase,_,_)=macro_checkpoint(checkpoint)?;
    let slot=&args[2];match slot.as_str(){
        "compiler"=>{if identity.len()!=3||identity[0]!="fixed-image"||identity[1]!="compiler"{return Err(macro_error("invalid fixed compiler identity"));}let m=std::fs::symlink_metadata(&identity[2])?;if !m.is_file()||m.len()>MACRO_BYTES_MAX{return Err(macro_error("invalid fixed original compiler image"));}},
        "macro"=>{if identity.len()<4||identity[0]!="inventory"||macro_number(&identity[1],32768)?.checked_mul(2)!=Some(identity.len()-2){return Err(macro_error("invalid macro selector inventory"));}macro_inventory_check(checkpoint,&identity[2..])?;},
        _=>return Err(macro_error("unknown original macro controller slot"))
    }
    for at in [4usize,8]{std::fs::create_dir_all(&args[at])?;}for at in [3usize,5,6,7]{std::fs::write(&args[at],b"")?;}
    if phase!=format!("{slot}\n"){return Ok(1);}
    let workspace=Path::new(&args[4]);copy(&checkpoint.join("workspace"),workspace,&mut 0)?;
    let prior=macro_times_checked(&checkpoint.join("workspace"),&checkpoint.join("times"))?;
    if slot=="compiler"{
        let config=bindings.get("compiler-config").ok_or_else(||macro_error("compiler configuration binding missing"))?;
        let bytes=macro_bytes(Path::new(config),MACRO_FRAME_MAX)?;if bytes!=macro_bytes(&checkpoint.join("compiler-config"),MACRO_FRAME_MAX)?{return Err(macro_error("original compiler configuration changed"));}
        std::fs::remove_dir_all(workspace.join("translation"))?;std::fs::create_dir(workspace.join("translation"))?;
        let newline=bytes.iter().position(|b|*b==b'\n').ok_or_else(||macro_error("macro compiler config lacks argv0 data frame"))?;
        std::fs::write(&args[3],&bytes[..newline])?;std::fs::write(&args[7],&bytes[newline+1..])?;
        macro_times_write(Path::new(&args[6]),&macro_times_carry(workspace,&prior,&["translation"])?)?;
    }else{
        let(_,cwd,image)=macro_request(checkpoint,&identity[2..])?;let request=checkpoint.join("request");let mut data=String::new();
        let argument=named_data_word(&macro_text(&request.join("macro-name"))?,"data",&mut data);let mut fields=Vec::new();
        for(key,value)in macro_environment(&request.join("environment"))?{if key.starts_with("MRUSTC_CAIRN_"){continue;}let word=if value.contains("/inputs/")||value.contains("/outputs/")||value.contains("/mantle-rust-runtime")||value.contains("/tmp"){typed_word(&value)?}else{named_data_word(&value,"data",&mut data)};fields.push(format!("{}:{word}",json_string(&key)));}
        let cwd=typed_word(cwd.to_str().ok_or_else(||macro_error("non-UTF8 macro cwd"))?)?;let image=typed_word(image.to_str().ok_or_else(||macro_error("non-UTF8 macro image path"))?)?;
        let roots=roots.iter().map(|root|json_string(root)).collect::<Vec<_>>().join(",");
        std::fs::write(&args[3],format!("{{\"arguments\":[{argument}],\"environment\":{{{}}},\"cwd\":{cwd},\"executable_path\":{image},\"workspace_mounts\":{{\"/tmp\":{{\"output\":\"workspace\",\"path\":\"tmp\"}}}},\"readonly_input_roots\":[{roots}]}}",fields.join(",")))?;
        std::fs::write(&args[7],data)?;std::fs::copy(request.join("stdin"),&args[5])?;macro_times_write(Path::new(&args[6]),&macro_times_carry(workspace,&prior,&[])?)?;
    }Ok(0)
}
fn macro_executable(args:&[String])->io::Result<i32>{
    if args.len()<8||args[2]!="macro"||args[4]!="inventory"||macro_number(&args[5],32768)?.checked_mul(2)!=Some(args.len()-6){return Err(macro_error("invalid macro executable arguments"));}
    let checkpoint=Path::new(&args[1]);if macro_checkpoint(checkpoint)?.0!="macro\n"{return Err(macro_error("macro image requested outside selected macro phase"));}
    let(name,_,_)=macro_request(checkpoint,&args[6..])?;let pair=args[6..].chunks_exact(2).find(|pair|pair[0]==name).ok_or_else(||macro_error("selected macro image missing"))?;
    std::fs::copy(&pair[1],&args[3])?;Ok(0)
}
// ObservePhase emits canonical JSON. Accept only the complete authenticated
// inventory or a genuine skipped envelope; absence of exit-status is not a skip.
fn macro_observation(root:&Path)->io::Result<bool>{
    let text=macro_text(&root.join("outcome.json"))?;
    let executed="{\"kind\":\"executed\",\"output_digests\":{";
    if let Some(body)=text.strip_prefix(executed).and_then(|s|s.strip_suffix("}}")){
        let expected=["descriptor-files","descriptors","exit-status","stderr","stdout","times","workspace"];
        let entries=body.split(',').collect::<Vec<_>>();if entries.len()!=expected.len(){return Err(macro_error("unexpected executed command inventory"));}
        for(entry,name)in entries.iter().zip(expected){let prefix=format!("{}:\"",json_string(name));let digest=entry.strip_prefix(&prefix).and_then(|s|s.strip_suffix('"')).ok_or_else(||macro_error("malformed executed phase inventory"))?;
            if digest.len()!=64||!digest.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err(macro_error("invalid observed phase digest"));}
            std::fs::symlink_metadata(envelope_output(root,name))?;
        }
        if macro_names(root)?!=["outcome.json","outputs"]||macro_names(&root.join("outputs"))?!=expected{return Err(macro_error("observed command has unlisted outputs"));}return Ok(true);
    }
    let body=text.strip_prefix("{\"kind\":\"skipped\",\"guard_codes\":[").and_then(|s|s.strip_suffix("]}")).ok_or_else(||macro_error("malformed actual phase observation"))?;
    let codes=body.split(',').map(|s|macro_number(s,255)).collect::<io::Result<Vec<_>>>()?;
    if codes.is_empty()||codes.len()>8||codes.windows(2).any(|v|v[0]>=v[1])||macro_names(root)?!=["outcome.json"]{return Err(macro_error("invalid genuine skipped phase observation"));}Ok(false)
}
fn macro_resume(args:&[String])->io::Result<i32>{
    if args.len()<23||args[3..7]!=["1","2","3","2"]||args[7..9]!=["compiler","0"]||args[10..12]!=["macro","0"]||args[13]!="--"{return Err(macro_error("invalid macro resume observation layout"));}
    let original=&args[14..];let slots=macro_start_arguments(original)?;let checkpoint=Path::new(&args[1]);let out=Path::new(&args[2]);let(phase,stored_slots,completed)=macro_checkpoint(checkpoint)?;
    if slots!=stored_slots||macro_bytes(Path::new(&original[4]),MACRO_FRAME_MAX)?!=macro_bytes(&checkpoint.join("compiler-config"),MACRO_FRAME_MAX)?{return Err(macro_error("macro controller original configuration changed"));}macro_inventory_check(checkpoint,&original[7..])?;
    let compiler=macro_observation(Path::new(&args[9]))?;let plugin=macro_observation(Path::new(&args[12]))?;
    if phase=="done\n"||compiler!=(phase=="compiler\n")||plugin!=(phase=="macro\n"){return Err(macro_error("macro turn did not execute exactly the selected original command"));}
    let observed=Path::new(if compiler{&args[9]}else{&args[12]});let code=macro_actual_status(&envelope_output(observed,"exit-status"))?;
    if (compiler&&code!=0&&code!=75)||(!compiler&&code!=0){return Err(macro_error("original compiler or macro command failed"));}
    copy(checkpoint,out,&mut 0)?;std::fs::remove_dir_all(out.join("workspace"))?;
    copy(&envelope_output(observed,"workspace"),&out.join("workspace"),&mut 0)?;
    let workspace=out.join("workspace");for name in ["tmp","history","translation"]{macro_directory(&workspace.join(name))?;}
    if macro_history(&workspace.join("history"))?!=completed{return Err(macro_error("original command changed macro replay history ordinals"));}
    // Replay history is readonly protocol state even though it shares the output tree.
    macro_tree_equal(&checkpoint.join("workspace/history"),&workspace.join("history"),&mut 0)?;
    let prior=macro_times_checked(&workspace,&envelope_output(observed,"times"))?;let mut refresh=Vec::new();
    if compiler{
        if out.join("request").try_exists()?{std::fs::remove_dir_all(out.join("request"))?;}
        if code==0{macro_phase(out,"done")?;}else{
            if completed==slots{return Err(macro_error("macro reservation exhausted before genuine compiler completion"));}
            copy(&workspace.join("translation/macro-request"),&out.join("request"),&mut 0)?;macro_request(out,&original[7..])?;macro_phase(out,"macro")?;
        }
    }else{
        macro_request(checkpoint,&original[7..])?;
        let reply=workspace.join("history").join(completed.to_string());copy(&checkpoint.join("request"),&reply,&mut 0)?;
        let stdout=envelope_output(observed,"stdout");let meta=std::fs::symlink_metadata(&stdout)?;if !meta.is_file()||meta.len()>MACRO_BYTES_MAX{return Err(macro_error("invalid original macro stdout"));}std::fs::copy(stdout,reply.join("stdout"))?;
        std::fs::write(out.join("completed"),(completed+1).to_string())?;macro_phase(out,"compiler")?;refresh.push(format!("history/{completed}"));
    }
    let refresh=refresh.iter().map(String::as_str).collect::<Vec<_>>();let mut times=macro_times_carry(&workspace,&prior,&refresh)?;
    if plugin{use std::os::unix::fs::MetadataExt;let meta=std::fs::symlink_metadata(workspace.join("history"))?;times.insert("history".into(),(meta.mtime(),meta.mtime_nsec()));}
    macro_times_write(&out.join("times"),&times)?;Ok(if compiler&&code==0{1}else{0})
}
fn macro_tree_equal(left:&Path,right:&Path,count:&mut usize)->io::Result<()>{
    *count+=1;if *count>MACRO_ENTRIES_MAX{return Err(macro_error("macro history comparison node bound"));}
    let a=std::fs::symlink_metadata(left)?;let b=std::fs::symlink_metadata(right)?;
    if a.is_dir()&&b.is_dir(){let names=macro_names(left)?;if names!=macro_names(right)?{return Err(macro_error("original command modified macro history"));}for name in names{macro_tree_equal(&left.join(&name),&right.join(name),count)?;}return Ok(());}
    if a.is_file()&&b.is_file()&&macro_same_image(left,right)?{return Ok(());}
    if a.file_type().is_symlink()&&b.file_type().is_symlink()&&std::fs::read_link(left)?==std::fs::read_link(right)?{return Ok(());}
    Err(macro_error("original command modified macro replay history"))
}
fn macro_finish(args:&[String])->io::Result<i32>{
    if args.len()!=6{return Err(macro_error("invalid macro finish arguments"));}let checkpoint=Path::new(&args[1]);
    if macro_checkpoint(checkpoint)?.0!="done\n"{return Err(macro_error("macro reservation exhausted without genuine compiler exit zero"));}
    copy(&checkpoint.join("workspace"),Path::new(&args[2]),&mut 0)?;
    let times=macro_times_checked(&checkpoint.join("workspace"),&checkpoint.join("times"))?;macro_times_write(Path::new(&args[3]),&times)?;
    std::fs::create_dir_all(&args[4])?;copy(checkpoint,Path::new(&args[5]),&mut 0)?;Ok(0)
}
fn macro_controller(args:&[String])->io::Result<i32>{
    match args.first().map(String::as_str){Some("--mantle-controller-start")=>macro_start(args),Some("--mantle-controller-prepare")=>macro_prepare(args),Some("--mantle-controller-executable")=>macro_executable(args),Some("--mantle-controller-resume")=>macro_resume(args),Some("--mantle-controller-finish")=>macro_finish(args),_=>Err(macro_error("unknown macro controller operation"))}
}
"###;
