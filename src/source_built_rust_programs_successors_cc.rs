//! Source transformations shared with the direct native continuation ABI.
//! The canonical overlay descriptors own cc option/tool/source selection.
use std::fmt::Write;

use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_mantle_programs::native;
use crate::source_built_recipe::ArtifactRef;

pub(super) fn source() -> String {
    let overlays = native::cc_library_overlays(
        &ArtifactRef::Source {
            role: SourceAuthorityRole::RustSourceArchiveSet,
            path: String::new(),
        },
        "cc",
    )
    .expect("static source-relative cc overlay paths");
    let process = super::process_source();
    let mut descriptors = String::from("const CC_OVERLAYS: &[(&str, &[(&str, &str)], &[&str])] = &[\n");
    for overlay in overlays {
        let member = overlay.compiler_path.strip_prefix("cc/").expect("cc compiler package prefix");
        write!(&mut descriptors, "({member:?}, &{:?}, &[", overlay.replacements).expect("writing source string");
        for addition in overlay.append {
            // The successor's instrumented std::process owns child requests;
            // retain the shared random boundary but not native replay dispatch.
            write!(&mut descriptors, "{:?},", addition.replace(native::CC_PROCESS_SOURCE, &process))
                .expect("writing source string");
        }
        descriptors.push_str("]),\n");
    }
    descriptors.push_str("];\n");
    format!(
        r#"
{descriptors}
fn cc_overlay(args:&[String])->io::Result<()>{{
    let source=Path::new(&args[1]);let destination=Path::new(&args[2]);copy(source,destination,&mut 0)?;
    let selected=if args.len()==3{{vec![source.to_owned()]}}else{{args[3..].iter().map(PathBuf::from).collect()}};
    for package in selected{{
        let relative=package.strip_prefix(source).map_err(io::Error::other)?;let target=destination.join(relative);
        for &(member,replacements,additions) in CC_OVERLAYS{{
            let original=package.join(member);
            if member=="src/tempfile.rs" && !original.try_exists()?{{continue;}}
            let mut text=std::fs::read_to_string(original)?;
            for &(before,after) in replacements{{
                text=replace_once(&text,before,after).map_err(io::Error::other)?;
            }}
            for addition in additions{{text.push_str(addition);}}
            std::fs::write(target.join(member),text)?;
        }}
    }}
    Ok(())
}}
"#
    )
}
