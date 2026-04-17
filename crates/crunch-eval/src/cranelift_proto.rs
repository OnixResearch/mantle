use cranelift::prelude::*;
use cranelift_jit::JITBuilder;
use cranelift_jit::JITModule;
use cranelift_module::Linkage;
use cranelift_module::Module;

const DEFAULT_SYSTEM: &str = "x86_64-linux";
const DEFAULT_ADDRESSING_MODE: &str = "content-addressed";
const DEFAULT_OUTPUT: &str = "out";
const MAX_SOURCE_BYTES: u32 = 16 * 1024;
const MAX_FIELD_COUNT: u32 = 6;
const MAX_ARRAY_ITEMS: u32 = 32;
const MAX_STRING_BYTES: u32 = 4096;

pub(crate) fn evaluate_flat_derivation_to_json(source: &str) -> Result<String, CraneliftProtoError> {
    let fields = parse_flat_derivation_fields(source)?;
    jit_eval_derivation(&fields)
}

struct DerivationFields {
    name: String,
    builder: String,
    system: Option<String>,
    addressing_mode: Option<String>,
    args: Vec<String>,
    outputs: Option<Vec<String>>,
}

#[derive(Debug)]
pub(crate) enum CraneliftProtoError {
    Parse(String),
    Compile(String),
    Runtime(String),
}

impl std::fmt::Display for CraneliftProtoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CraneliftProtoError::Parse(msg) => write!(f, "cranelift prototype parse error: {msg}"),
            CraneliftProtoError::Compile(msg) => write!(f, "cranelift prototype compile error: {msg}"),
            CraneliftProtoError::Runtime(msg) => write!(f, "cranelift prototype runtime error: {msg}"),
        }
    }
}

impl std::error::Error for CraneliftProtoError {}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos_bytes: u32,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Result<Self, CraneliftProtoError> {
        let len_bytes = u32::try_from(source.len())
            .map_err(|_| CraneliftProtoError::Parse("source length does not fit in u32".to_string()))?;
        if len_bytes > MAX_SOURCE_BYTES {
            return Err(CraneliftProtoError::Parse(format!(
                "source too large for prototype subset: {len_bytes} > {MAX_SOURCE_BYTES} bytes"
            )));
        }
        Ok(Self {
            source,
            bytes: source.as_bytes(),
            pos_bytes: 0,
        })
    }

    fn is_eof(&self) -> bool {
        self.pos_bytes as usize >= self.bytes.len()
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos_bytes as usize).copied()
    }

    fn bump_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.pos_bytes = self.pos_bytes.saturating_add(1);
        Some(byte)
    }

    fn skip_ws(&mut self) {
        while let Some(byte) = self.peek_byte() {
            if !byte.is_ascii_whitespace() {
                return;
            }
            self.pos_bytes = self.pos_bytes.saturating_add(1);
        }
    }

    fn expect_byte(&mut self, expected: u8, context: &str) -> Result<(), CraneliftProtoError> {
        self.skip_ws();
        let Some(actual) = self.bump_byte() else {
            return Err(self.error(format!("expected `{}` for {context}, got EOF", expected as char)));
        };
        if actual != expected {
            return Err(self.error(format!("expected `{}` for {context}, got `{}`", expected as char, actual as char)));
        }
        Ok(())
    }

    fn parse_identifier(&mut self) -> Result<String, CraneliftProtoError> {
        self.skip_ws();
        let start_bytes = self.pos_bytes as usize;
        let Some(first) = self.peek_byte() else {
            return Err(self.error("expected identifier, got EOF".to_string()));
        };
        if !is_identifier_start(first) {
            return Err(self.error(format!("expected identifier start, got `{}`", first as char)));
        }
        self.pos_bytes = self.pos_bytes.saturating_add(1);
        while let Some(byte) = self.peek_byte() {
            if !is_identifier_continue(byte) {
                break;
            }
            self.pos_bytes = self.pos_bytes.saturating_add(1);
        }
        let end_bytes = self.pos_bytes as usize;
        let ident = &self.source[start_bytes..end_bytes];
        if ident.len() > 64 {
            return Err(self.error(format!("identifier too long for prototype subset: `{ident}`")));
        }
        Ok(ident.to_string())
    }

    fn parse_string_literal(&mut self) -> Result<String, CraneliftProtoError> {
        self.skip_ws();
        self.expect_byte(b'"', "string literal start")?;
        let mut out = String::new();
        while let Some(byte) = self.bump_byte() {
            match byte {
                b'"' => {
                    let len_bytes = u32::try_from(out.len())
                        .map_err(|_| self.error("string length does not fit in u32".to_string()))?;
                    if len_bytes > MAX_STRING_BYTES {
                        return Err(self.error(format!(
                            "string literal too long for prototype subset: {len_bytes} > {MAX_STRING_BYTES} bytes"
                        )));
                    }
                    return Ok(out);
                }
                b'\\' => {
                    let escaped =
                        self.bump_byte().ok_or_else(|| self.error("unterminated string escape".to_string()))?;
                    let decoded = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        other => {
                            return Err(
                                self.error(format!("unsupported escape in prototype subset: `\\{}`", other as char))
                            );
                        }
                    };
                    out.push(decoded);
                }
                other => out.push(other as char),
            }
        }
        Err(self.error("unterminated string literal".to_string()))
    }

    fn parse_enum_tag(&mut self) -> Result<String, CraneliftProtoError> {
        self.skip_ws();
        self.expect_byte(b'\'', "enum tag start")?;
        let start_bytes = self.pos_bytes as usize;
        while let Some(byte) = self.peek_byte() {
            if byte == b',' || byte == b'}' || byte == b']' || byte.is_ascii_whitespace() {
                break;
            }
            self.pos_bytes = self.pos_bytes.saturating_add(1);
        }
        let end_bytes = self.pos_bytes as usize;
        if start_bytes == end_bytes {
            return Err(self.error("empty enum tag in prototype subset".to_string()));
        }
        let tag = &self.source[start_bytes..end_bytes];
        let len_bytes =
            u32::try_from(tag.len()).map_err(|_| self.error("enum tag length does not fit in u32".to_string()))?;
        if len_bytes > MAX_STRING_BYTES {
            return Err(
                self.error(format!("enum tag too long for prototype subset: {len_bytes} > {MAX_STRING_BYTES} bytes"))
            );
        }
        Ok(tag.to_string())
    }

    fn parse_string_or_enum(&mut self) -> Result<String, CraneliftProtoError> {
        self.skip_ws();
        match self.peek_byte() {
            Some(b'"') => self.parse_string_literal(),
            Some(b'\'') => self.parse_enum_tag(),
            Some(other) => Err(self.error(format!("expected string or enum tag, got `{}`", other as char))),
            None => Err(self.error("expected string or enum tag, got EOF".to_string())),
        }
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, CraneliftProtoError> {
        self.skip_ws();
        self.expect_byte(b'[', "array start")?;
        self.skip_ws();
        let mut values = Vec::new();
        if self.peek_byte() == Some(b']') {
            self.pos_bytes = self.pos_bytes.saturating_add(1);
            return Ok(values);
        }

        loop {
            let len_items = u32::try_from(values.len())
                .map_err(|_| self.error("array item count does not fit in u32".to_string()))?;
            if len_items >= MAX_ARRAY_ITEMS {
                return Err(
                    self.error(format!("array too long for prototype subset: {len_items} >= {MAX_ARRAY_ITEMS}"))
                );
            }
            values.push(self.parse_string_literal()?);
            self.skip_ws();
            match self.peek_byte() {
                Some(b',') => {
                    self.pos_bytes = self.pos_bytes.saturating_add(1);
                    self.skip_ws();
                    if self.peek_byte() == Some(b']') {
                        self.pos_bytes = self.pos_bytes.saturating_add(1);
                        return Ok(values);
                    }
                }
                Some(b']') => {
                    self.pos_bytes = self.pos_bytes.saturating_add(1);
                    return Ok(values);
                }
                Some(other) => {
                    return Err(self.error(format!("expected `,` or `]` after array item, got `{}`", other as char)));
                }
                None => return Err(self.error("unterminated array literal".to_string())),
            }
        }
    }

    fn error(&self, msg: String) -> CraneliftProtoError {
        CraneliftProtoError::Parse(format!("{msg} at byte {}", self.pos_bytes))
    }
}

fn parse_flat_derivation_fields(source: &str) -> Result<DerivationFields, CraneliftProtoError> {
    let mut parser = Parser::new(source)?;
    parser.expect_byte(b'{', "record start")?;

    let mut name: Option<String> = None;
    let mut builder: Option<String> = None;
    let mut system: Option<String> = None;
    let mut addressing_mode: Option<String> = None;
    let mut args: Option<Vec<String>> = None;
    let mut outputs: Option<Vec<String>> = None;
    let mut field_count: u32 = 0;

    loop {
        parser.skip_ws();
        if parser.peek_byte() == Some(b'}') {
            parser.bump_byte();
            break;
        }
        field_count = field_count.saturating_add(1);
        if field_count > MAX_FIELD_COUNT {
            return Err(
                parser.error(format!("too many fields for prototype subset: {field_count} > {MAX_FIELD_COUNT}"))
            );
        }

        let key = parser.parse_identifier()?;
        parser.expect_byte(b'=', "field assignment")?;
        match key.as_str() {
            "name" => set_unique_field(&mut name, parser.parse_string_literal()?, &key, &parser)?,
            "builder" => set_unique_field(&mut builder, parser.parse_string_literal()?, &key, &parser)?,
            "system" => set_unique_field(&mut system, parser.parse_string_or_enum()?, &key, &parser)?,
            "addressing_mode" => set_unique_field(&mut addressing_mode, parser.parse_string_or_enum()?, &key, &parser)?,
            "args" => set_unique_field(&mut args, parser.parse_string_array()?, &key, &parser)?,
            "outputs" => set_unique_field(&mut outputs, parser.parse_string_array()?, &key, &parser)?,
            _ => return Err(parser.error(format!("unsupported field in prototype subset: `{key}`"))),
        }

        parser.skip_ws();
        match parser.peek_byte() {
            Some(b',') => {
                parser.bump_byte();
            }
            Some(b'}') => continue,
            Some(other) => {
                return Err(parser.error(format!("expected `,` or `}}` after field, got `{}`", other as char)));
            }
            None => return Err(parser.error("unterminated record literal".to_string())),
        }
    }

    let name = name.ok_or_else(|| parser.error("prototype subset requires `name`".to_string()))?;
    let builder = builder.ok_or_else(|| parser.error("prototype subset requires `builder`".to_string()))?;
    if parser.is_eof() {
        return Ok(DerivationFields {
            name,
            builder,
            system,
            addressing_mode,
            args: args.unwrap_or_default(),
            outputs,
        });
    }
    parser.skip_ws();
    if !parser.is_eof() {
        return Err(parser.error("unexpected trailing content after derivation record".to_string()));
    }
    Ok(DerivationFields {
        name,
        builder,
        system,
        addressing_mode,
        args: args.unwrap_or_default(),
        outputs,
    })
}

fn set_unique_field<T>(
    slot: &mut Option<T>,
    value: T,
    key: &str,
    parser: &Parser<'_>,
) -> Result<(), CraneliftProtoError> {
    if slot.is_some() {
        return Err(parser.error(format!("duplicate field in prototype subset: `{key}`")));
    }
    *slot = Some(value);
    Ok(())
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit() || byte == b'-'
}

struct StringRef {
    offset_bytes: u32,
    len_bytes: u32,
}

struct DataSection {
    bytes: Vec<u8>,
    name: StringRef,
    builder: StringRef,
    system: StringRef,
    addressing_mode: StringRef,
}

impl DataSection {
    fn pack(fields: &DerivationFields) -> Result<Self, CraneliftProtoError> {
        let system = fields.system.as_deref().unwrap_or(DEFAULT_SYSTEM);
        let addressing_mode = fields.addressing_mode.as_deref().unwrap_or(DEFAULT_ADDRESSING_MODE);
        let mut bytes = Vec::new();
        let name = push_string_ref(&mut bytes, &fields.name)?;
        let builder = push_string_ref(&mut bytes, &fields.builder)?;
        let system = push_string_ref(&mut bytes, system)?;
        let addressing_mode = push_string_ref(&mut bytes, addressing_mode)?;
        Ok(Self {
            bytes,
            name,
            builder,
            system,
            addressing_mode,
        })
    }
}

fn push_string_ref(bytes: &mut Vec<u8>, value: &str) -> Result<StringRef, CraneliftProtoError> {
    let offset_bytes = u32::try_from(bytes.len())
        .map_err(|_| CraneliftProtoError::Runtime("packed data offset does not fit in u32".to_string()))?;
    let len_bytes = u32::try_from(value.len())
        .map_err(|_| CraneliftProtoError::Runtime("packed data length does not fit in u32".to_string()))?;
    if len_bytes > MAX_STRING_BYTES {
        return Err(CraneliftProtoError::Runtime(format!(
            "packed string too long for prototype subset: {len_bytes} > {MAX_STRING_BYTES} bytes"
        )));
    }
    bytes.extend_from_slice(value.as_bytes());
    Ok(StringRef {
        offset_bytes,
        len_bytes,
    })
}

#[repr(C)]
struct JitOutput {
    name_ptr_u64: u64,
    name_len_u64: u64,
    builder_ptr_u64: u64,
    builder_len_u64: u64,
    system_ptr_u64: u64,
    system_len_u64: u64,
    addressing_mode_ptr_u64: u64,
    addressing_mode_len_u64: u64,
}

fn jit_eval_derivation(fields: &DerivationFields) -> Result<String, CraneliftProtoError> {
    let data = DataSection::pack(fields)?;
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;
    flag_builder.set("is_pic", "false").map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;

    let isa_builder = cranelift_native::builder().map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;

    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let mut module = JITModule::new(builder);
    let pointer_type = module.target_config().pointer_type();
    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(pointer_type));
    signature.params.push(AbiParam::new(pointer_type));
    signature.returns.push(AbiParam::new(types::I64));

    let function_id = module
        .declare_function("assemble_derivation", Linkage::Export, &signature)
        .map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;

    let mut context = module.make_context();
    context.func.signature = signature;
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut function_builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry_block = function_builder.create_block();
        function_builder.append_block_params_for_function_params(entry_block);
        function_builder.switch_to_block(entry_block);
        function_builder.seal_block(entry_block);
        let output_ptr = function_builder.block_params(entry_block)[0];
        let data_ptr = function_builder.block_params(entry_block)[1];

        store_field(&mut function_builder, pointer_type, output_ptr, data_ptr, &data.name, 0)?;
        store_field(&mut function_builder, pointer_type, output_ptr, data_ptr, &data.builder, 16)?;
        store_field(&mut function_builder, pointer_type, output_ptr, data_ptr, &data.system, 32)?;
        store_field(&mut function_builder, pointer_type, output_ptr, data_ptr, &data.addressing_mode, 48)?;

        let status = function_builder.ins().iconst(types::I64, 0);
        function_builder.ins().return_(&[status]);
        function_builder.finalize();
    }

    module
        .define_function(function_id, &mut context)
        .map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;
    module.finalize_definitions().map_err(|err| CraneliftProtoError::Compile(err.to_string()))?;

    let function_ptr = module.get_finalized_function(function_id);
    let mut output = JitOutput {
        name_ptr_u64: 0,
        name_len_u64: 0,
        builder_ptr_u64: 0,
        builder_len_u64: 0,
        system_ptr_u64: 0,
        system_len_u64: 0,
        addressing_mode_ptr_u64: 0,
        addressing_mode_len_u64: 0,
    };
    let output_ptr_i64 = (&mut output as *mut JitOutput) as i64;
    let data_ptr_i64 = data.bytes.as_ptr() as i64;

    let status = unsafe {
        let jit_fn: unsafe extern "C" fn(i64, i64) -> i64 = std::mem::transmute(function_ptr);
        jit_fn(output_ptr_i64, data_ptr_i64)
    };
    if status != 0 {
        return Err(CraneliftProtoError::Runtime(format!("prototype JIT returned non-zero status: {status}")));
    }

    output_to_json(&data.bytes, &output, fields)
}

fn store_field(
    function_builder: &mut FunctionBuilder<'_>,
    pointer_type: Type,
    output_ptr: Value,
    data_ptr: Value,
    string_ref: &StringRef,
    output_offset_bytes: i32,
) -> Result<(), CraneliftProtoError> {
    let field_offset = function_builder.ins().iconst(pointer_type, i64::from(string_ref.offset_bytes));
    let field_ptr = function_builder.ins().iadd(data_ptr, field_offset);
    function_builder.ins().store(MemFlags::new(), field_ptr, output_ptr, output_offset_bytes);
    let len_value = function_builder.ins().iconst(types::I64, i64::from(string_ref.len_bytes));
    let len_offset_bytes = output_offset_bytes
        .checked_add(8)
        .ok_or_else(|| CraneliftProtoError::Runtime("output offset overflow".to_string()))?;
    function_builder.ins().store(MemFlags::new(), len_value, output_ptr, len_offset_bytes);
    Ok(())
}

fn output_to_json(
    data_bytes: &[u8],
    output: &JitOutput,
    fields: &DerivationFields,
) -> Result<String, CraneliftProtoError> {
    let name = read_output_str(data_bytes, output.name_ptr_u64, output.name_len_u64)?;
    let builder = read_output_str(data_bytes, output.builder_ptr_u64, output.builder_len_u64)?;
    let system = read_output_str(data_bytes, output.system_ptr_u64, output.system_len_u64)?;
    let addressing_mode = read_output_str(data_bytes, output.addressing_mode_ptr_u64, output.addressing_mode_len_u64)?;
    let outputs = fields.outputs.clone().unwrap_or_else(|| vec![DEFAULT_OUTPUT.to_string()]);
    let json = serde_json::json!({
        "name": name,
        "builder": builder,
        "system": system,
        "args": fields.args,
        "outputs": outputs,
        "addressing_mode": addressing_mode,
    });
    serde_json::to_string(&json).map_err(|err| CraneliftProtoError::Runtime(err.to_string()))
}

fn read_output_str(data_bytes: &[u8], ptr_u64: u64, len_u64: u64) -> Result<String, CraneliftProtoError> {
    let data_start_u64 = data_bytes.as_ptr() as u64;
    let data_len_u64 = u64::try_from(data_bytes.len())
        .map_err(|_| CraneliftProtoError::Runtime("data length does not fit in u64".to_string()))?;
    let data_end_u64 = data_start_u64
        .checked_add(data_len_u64)
        .ok_or_else(|| CraneliftProtoError::Runtime("data end overflow".to_string()))?;
    let ptr_end_u64 = ptr_u64
        .checked_add(len_u64)
        .ok_or_else(|| CraneliftProtoError::Runtime("string pointer overflow".to_string()))?;
    if ptr_u64 < data_start_u64 || ptr_end_u64 > data_end_u64 {
        return Err(CraneliftProtoError::Runtime(
            "prototype JIT produced string pointer outside packed data".to_string(),
        ));
    }
    let offset_u64 = ptr_u64 - data_start_u64;
    let offset_usize = usize::try_from(offset_u64)
        .map_err(|_| CraneliftProtoError::Runtime("offset does not fit in usize".to_string()))?;
    let len_usize = usize::try_from(len_u64)
        .map_err(|_| CraneliftProtoError::Runtime("length does not fit in usize".to_string()))?;
    let end_usize = offset_usize
        .checked_add(len_usize)
        .ok_or_else(|| CraneliftProtoError::Runtime("slice end overflow".to_string()))?;
    let bytes = data_bytes
        .get(offset_usize..end_usize)
        .ok_or_else(|| CraneliftProtoError::Runtime("slice range outside packed data".to_string()))?;
    let text = std::str::from_utf8(bytes)
        .map_err(|err| CraneliftProtoError::Runtime(format!("prototype output was not utf-8: {err}")))?;
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use crunch_glue::CrunchDerivation;

    use super::*;

    #[test]
    fn prototype_json_round_trips_defaults() {
        let json = evaluate_flat_derivation_to_json(
            r#"{
  name = "hello",
  builder = "/bin/sh",
}"#,
        )
        .unwrap();
        let drv: CrunchDerivation = serde_json::from_str(&json).unwrap();
        assert_eq!(drv.name, "hello");
        assert_eq!(drv.builder, "/bin/sh");
        assert_eq!(drv.system, DEFAULT_SYSTEM);
        assert_eq!(drv.addressing_mode, DEFAULT_ADDRESSING_MODE);
        assert_eq!(drv.outputs, vec![DEFAULT_OUTPUT.to_string()]);
        assert!(drv.args.is_empty());
    }

    #[test]
    fn prototype_json_keeps_explicit_fields() {
        let json = evaluate_flat_derivation_to_json(
            r#"{
  name = "demo",
  builder = "/nix/store/bash/bin/bash",
  system = 'aarch64-linux,
  addressing_mode = 'input-addressed,
  args = ["-c", "echo hi > $out"],
  outputs = ["out", "dev"],
}"#,
        )
        .unwrap();
        let drv: CrunchDerivation = serde_json::from_str(&json).unwrap();
        assert_eq!(drv.name, "demo");
        assert_eq!(drv.builder, "/nix/store/bash/bin/bash");
        assert_eq!(drv.system, "aarch64-linux");
        assert_eq!(drv.addressing_mode, "input-addressed");
        assert_eq!(drv.args, vec!["-c".to_string(), "echo hi > $out".to_string()]);
        assert_eq!(drv.outputs, vec!["out".to_string(), "dev".to_string()]);
    }

    #[test]
    fn prototype_matches_nickel_on_supported_subset() {
        let source = r#"{
  name = "roundtrip",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  addressing_mode = 'content-addressed,
  args = ["-c", "echo done"],
  outputs = ["out", "dev"],
}"#;
        let nickel_json = crate::evaluate_str_to_json(source, &[]).unwrap();
        let prototype_json = evaluate_flat_derivation_to_json(source).unwrap();
        let nickel_drv: CrunchDerivation = serde_json::from_str(&nickel_json).unwrap();
        let prototype_drv: CrunchDerivation = serde_json::from_str(&prototype_json).unwrap();
        assert_eq!(prototype_drv.name, nickel_drv.name);
        assert_eq!(prototype_drv.builder, nickel_drv.builder);
        assert_eq!(prototype_drv.system, nickel_drv.system);
        assert_eq!(prototype_drv.addressing_mode, nickel_drv.addressing_mode);
        assert_eq!(prototype_drv.args, nickel_drv.args);
        assert_eq!(prototype_drv.outputs, nickel_drv.outputs);
    }

    #[test]
    fn prototype_rejects_nested_inputs() {
        let err = evaluate_flat_derivation_to_json(
            r#"{
  name = "bad",
  builder = "/bin/sh",
  inputs = [{ name = "dep", builder = "/bin/sh" }],
}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("unsupported field"));
    }
}
