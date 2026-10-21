//! Canonical typed argv/environment emission from genuine source requests.
pub(super) const SOURCE: &str = r###"
fn json_string(value:&str)->String{let mut out=String::from("\"");for c in value.chars(){match c{'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),'\u{8}'=>out.push_str("\\b"),'\u{c}'=>out.push_str("\\f"),c if c<' '=>out.push_str(&format!("\\u{:04x}",c as u32)),c=>out.push(c)}}out.push('"');out}
fn data_word(value:&str,data:&mut String)->String{let offset=data.len();data.push_str(value);format!("{{\"parts\":[{{\"kind\":\"text_slice\",\"name\":\"environment-data\",\"offset_bytes\":{offset},\"length_bytes\":{}}}]}}",value.len())}
fn typed_word(value:&str)->io::Result<String>{
    if value.contains('\0'){return Err(io::Error::other("NUL in source invocation operand"));}
    let mut parts=Vec::new();let mut rest=value;
    loop{
        let found=[("/inputs/",0),("/outputs/",1),("/mantle-rust-runtime",2),("/tmp",3)].iter().filter_map(|(prefix,kind)|rest.find(prefix).filter(|at|*kind!=3||rest[*at+prefix.len()..].starts_with('/')||rest[*at+prefix.len()..].is_empty()).map(|at|(at,*prefix,*kind))).min_by_key(|(at,_,_)|*at);
        let Some((at,prefix,kind))=found else{if !rest.is_empty()||parts.is_empty(){if rest.contains('/')||rest.contains('\\'){return Err(io::Error::other(format!("unbound path in original invocation operand: {value:?}")));}parts.push(format!("{{\"kind\":\"literal\",\"text\":{}}}",json_string(rest)));}break;};
        let literal=&rest[..at];if literal.contains('/')||literal.contains('\\'){return Err(io::Error::other("untyped original path prefix"));}if !literal.is_empty(){parts.push(format!("{{\"kind\":\"literal\",\"text\":{}}}",json_string(literal)));}
        let after=&rest[at+prefix.len()..];let end=after.find(|c:char|matches!(c,':'|';'|' '| '\t'|'\n'|'\r'|'\u{1f}'|','|'"'|'\'')).unwrap_or(after.len());let operand=&after[..end];rest=&after[end..];
        if matches!(kind,2|3){
            let path=operand.strip_prefix('/').unwrap_or(operand);safe_operand_path(path)?;
            parts.push(format!("{{\"kind\":{},\"mount\":{},\"path\":{}}}",json_string(if kind==2{"runtime"}else{"workspace"}),json_string(if kind==2{"/mantle-rust-runtime"}else{"/tmp"}),json_string(path)));
        }else{
            let(name,path)=operand.split_once('/').unwrap_or((operand,""));if name.is_empty(){return Err(io::Error::other("missing source operand alias"));}safe_operand_path(path)?;
            parts.push(format!("{{\"kind\":{},\"name\":{},\"path\":{}}}",json_string(if kind==0{"input"}else{"output"}),json_string(name),json_string(path)));
        }
    }
    Ok(format!("{{\"parts\":[{}]}}",parts.join(",")))
}
fn safe_operand_path(path:&str)->io::Result<()>{if !path.is_empty()&&Path::new(path).components().any(|c|!matches!(c,std::path::Component::Normal(_))){return Err(io::Error::other("noncanonical original invocation path"));}Ok(())}
fn merge_trees(args:&[String])->io::Result<()>{let out=Path::new(&args[1]);std::fs::create_dir_all(out)?;for input in &args[2..]{copy(Path::new(input),out,&mut 0)?;}Ok(())}
fn runtime_libraries(args:&[String])->io::Result<()>{let source=Path::new(&args[1]);let out=Path::new(&args[2]);std::fs::create_dir_all(out)?;let mut entries=std::fs::read_dir(source)?.collect::<io::Result<Vec<_>>>()?;entries.sort_by_key(|entry|entry.file_name());for entry in entries{if entry.file_type()?.is_file()&&entry.file_name().to_string_lossy().ends_with(".so"){copy(&entry.path(),&out.join(entry.file_name()),&mut 0)?;}}Ok(())}
fn provider_receipt(args:&[String])->io::Result<()>{let observed=std::fs::read_to_string(&args[1])?;let original=std::fs::read_to_string(&args[2])?;let version=&args[3];if original.trim()!=version||!observed.lines().any(|line|line==format!("release: {version}"))||!observed.lines().any(|line|line=="host: x86_64-unknown-linux-musl"){return Err(io::Error::other("executed compiler version or host differs from original selected release"));}let json=format!("{{\"schema\":\"mantle-rust-source-provider-v1\",\"version\":{},\"host_triple\":\"x86_64-unknown-linux-musl\",\"target_triple\":\"x86_64-unknown-linux-musl\",\"compiler_observation\":{}}}\n",json_string(version),json_string(&observed));std::fs::write(&args[4],json)}
"###;
