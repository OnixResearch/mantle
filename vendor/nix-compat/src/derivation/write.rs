//! This module implements the serialisation of derivations into the
//! [ATerm][] format used by C++ Nix.
//!
//! [ATerm]: http://program-transformation.org/Tools/ATermFormat.html

use bstr::BString;
use data_encoding::HEXLOWER;

use crate::aterm::escape_bytes;
use crate::derivation::ca_kind_prefix;
use crate::derivation::output::Output;
use crate::nixbase32;
use crate::store_path::STORE_DIR_WITH_SLASH;
use crate::store_path::StorePath;

/// Write a [StorePath] to the writer in ATerm quoted format using a custom
/// store directory prefix. Same as [AtermWriteable] for StorePath but
/// parameterised over the prefix.
pub(crate) fn write_store_path_with_prefix<S: AsRef<str>>(
    writer: &mut impl Write,
    sp: &StorePath<S>,
    store_dir: &str,
) -> std::io::Result<()> {
    write_char(writer, QUOTE)?;
    writer.write_all(store_dir.as_bytes())?;
    write_char(writer, '/')?;
    writer.write_all(nixbase32::encode(sp.digest()).as_bytes())?;
    write_char(writer, '-')?;
    writer.write_all(sp.name().as_ref().as_bytes())?;
    write_char(writer, QUOTE)?;
    Ok(())
}

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io;
use std::io::Error;
use std::io::Write;

pub const DERIVATION_PREFIX: &str = "Derive";
pub const PAREN_OPEN: char = '(';
pub const PAREN_CLOSE: char = ')';
pub const BRACKET_OPEN: char = '[';
pub const BRACKET_CLOSE: char = ']';
pub const COMMA: char = ',';
pub const QUOTE: char = '"';

/// Something that can be written as ATerm.
///
/// Note that we mostly use explicit `write_*` calls
/// instead since the serialization of the items depends on
/// the context a lot.
pub(crate) trait AtermWriteable {
    fn aterm_write(&self, writer: &mut impl Write) -> std::io::Result<()>;
}

impl<S> AtermWriteable for StorePath<S>
where S: AsRef<str>
{
    fn aterm_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write_char(writer, QUOTE)?;
        writer.write_all(STORE_DIR_WITH_SLASH.as_bytes())?;
        writer.write_all(nixbase32::encode(self.digest()).as_bytes())?;
        write_char(writer, '-')?;
        writer.write_all(self.name().as_ref().as_bytes())?;
        write_char(writer, QUOTE)?;
        Ok(())
    }
}

impl AtermWriteable for String {
    fn aterm_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write_field(writer, self, true)
    }
}

impl AtermWriteable for [u8; 32] {
    fn aterm_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write_field(writer, HEXLOWER.encode(self), false)
    }
}

// Writes a character to the writer.
pub(crate) fn write_char(writer: &mut impl Write, c: char) -> io::Result<()> {
    let mut buf = [0; 4];
    let b = c.encode_utf8(&mut buf).as_bytes();
    writer.write_all(b)
}

// Write a string `s` as a quoted field to the writer.
// The `escape` argument controls whether escaping will be skipped.
// This is the case if `s` is known to only contain characters that need no
// escaping.
pub(crate) fn write_field<S: AsRef<[u8]>>(writer: &mut impl Write, s: S, escape: bool) -> io::Result<()> {
    write_char(writer, QUOTE)?;

    if !escape {
        writer.write_all(s.as_ref())?;
    } else {
        writer.write_all(&escape_bytes(s.as_ref()))?;
    }

    write_char(writer, QUOTE)?;

    Ok(())
}

fn write_array_elements<S: AsRef<[u8]>>(writer: &mut impl Write, elements: &[S]) -> Result<(), io::Error> {
    for (index, element) in elements.iter().enumerate() {
        if index > 0 {
            write_char(writer, COMMA)?;
        }

        write_field(writer, element, true)?;
    }

    Ok(())
}

pub(crate) fn write_outputs(writer: &mut impl Write, outputs: &BTreeMap<String, Output>) -> Result<(), io::Error> {
    write_outputs_with_prefix(writer, outputs, STORE_DIR_WITH_SLASH)
}

/// Like [write_outputs] but with a custom store directory prefix.
/// `store_dir_with_slash` must end with '/'.
pub(crate) fn write_outputs_with_prefix(
    writer: &mut impl Write,
    outputs: &BTreeMap<String, Output>,
    store_dir_with_slash: &str,
) -> Result<(), io::Error> {
    write_char(writer, BRACKET_OPEN)?;
    for (ii, (output_name, output)) in outputs.iter().enumerate() {
        if ii > 0 {
            write_char(writer, COMMA)?;
        }

        write_char(writer, PAREN_OPEN)?;

        let path_str =
            output.path_str_with_prefix(&store_dir_with_slash[..store_dir_with_slash.len().saturating_sub(1)]);
        let mut elements: Vec<&str> = vec![output_name, &path_str];

        let (mode_and_algo, digest) = match &output.ca_hash {
            Some(ca_hash) => (
                format!("{}{}", ca_kind_prefix(ca_hash), ca_hash.hash().algo()),
                data_encoding::HEXLOWER.encode(ca_hash.hash().digest_as_bytes()),
            ),
            None => ("".to_string(), "".to_string()),
        };

        elements.push(&mode_and_algo);
        elements.push(&digest);

        write_array_elements(writer, &elements)?;

        write_char(writer, PAREN_CLOSE)?;
    }
    write_char(writer, BRACKET_CLOSE)?;

    Ok(())
}

pub(crate) fn write_input_derivations(
    writer: &mut impl Write,
    input_derivations: &BTreeMap<impl AtermWriteable, BTreeSet<String>>,
) -> Result<(), io::Error> {
    write_char(writer, BRACKET_OPEN)?;

    for (ii, (input_derivation_aterm, output_names)) in input_derivations.iter().enumerate() {
        if ii > 0 {
            write_char(writer, COMMA)?;
        }

        write_char(writer, PAREN_OPEN)?;
        input_derivation_aterm.aterm_write(writer)?;
        write_char(writer, COMMA)?;

        write_char(writer, BRACKET_OPEN)?;
        write_array_elements(writer, &output_names.iter().map(String::as_bytes).collect::<Vec<_>>())?;
        write_char(writer, BRACKET_CLOSE)?;

        write_char(writer, PAREN_CLOSE)?;
    }

    write_char(writer, BRACKET_CLOSE)?;

    Ok(())
}

/// Like [write_input_derivations] but writes StorePath keys with a custom
/// store directory prefix. Used for the normal (non-replacement) serialization
/// path where keys are StorePaths rather than hash digests.
pub(crate) fn write_input_derivations_with_prefix(
    writer: &mut impl Write,
    input_derivations: &BTreeMap<StorePath<String>, BTreeSet<String>>,
    store_dir: &str,
) -> Result<(), io::Error> {
    write_char(writer, BRACKET_OPEN)?;

    for (ii, (drv_path, output_names)) in input_derivations.iter().enumerate() {
        if ii > 0 {
            write_char(writer, COMMA)?;
        }

        write_char(writer, PAREN_OPEN)?;
        write_store_path_with_prefix(writer, drv_path, store_dir)?;
        write_char(writer, COMMA)?;

        write_char(writer, BRACKET_OPEN)?;
        write_array_elements(writer, &output_names.iter().map(String::as_bytes).collect::<Vec<_>>())?;
        write_char(writer, BRACKET_CLOSE)?;

        write_char(writer, PAREN_CLOSE)?;
    }

    write_char(writer, BRACKET_CLOSE)?;

    Ok(())
}

pub(crate) fn write_input_sources(
    writer: &mut impl Write,
    input_sources: &BTreeSet<StorePath<String>>,
) -> Result<(), io::Error> {
    write_input_sources_with_prefix(writer, input_sources, crate::store_path::STORE_DIR)
}

/// Like [write_input_sources] but with a custom store directory prefix.
pub(crate) fn write_input_sources_with_prefix(
    writer: &mut impl Write,
    input_sources: &BTreeSet<StorePath<String>>,
    store_dir: &str,
) -> Result<(), io::Error> {
    write_char(writer, BRACKET_OPEN)?;
    write_array_elements(
        writer,
        &input_sources.iter().map(|sp| sp.to_absolute_path_with_prefix(store_dir)).collect::<Vec<_>>(),
    )?;
    write_char(writer, BRACKET_CLOSE)?;

    Ok(())
}

pub(crate) fn write_system(writer: &mut impl Write, platform: &str) -> Result<(), Error> {
    write_field(writer, platform, true)?;
    Ok(())
}

pub(crate) fn write_builder(writer: &mut impl Write, builder: &str) -> Result<(), Error> {
    write_field(writer, builder, true)?;
    Ok(())
}

pub(crate) fn write_arguments(writer: &mut impl Write, arguments: &[String]) -> Result<(), io::Error> {
    write_char(writer, BRACKET_OPEN)?;
    write_array_elements(writer, &arguments.iter().map(|s| s.as_bytes().to_vec().into()).collect::<Vec<BString>>())?;
    write_char(writer, BRACKET_CLOSE)?;

    Ok(())
}

pub(crate) fn write_environment<E, K, V>(writer: &mut impl Write, environment: E) -> Result<(), io::Error>
where
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    write_char(writer, BRACKET_OPEN)?;

    for (i, (k, v)) in environment.into_iter().enumerate() {
        if i > 0 {
            write_char(writer, COMMA)?;
        }

        write_char(writer, PAREN_OPEN)?;
        write_field(writer, k, false)?;
        write_char(writer, COMMA)?;
        write_field(writer, v, true)?;
        write_char(writer, PAREN_CLOSE)?;
    }

    write_char(writer, BRACKET_CLOSE)?;

    Ok(())
}
