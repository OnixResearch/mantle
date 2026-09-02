#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "visit"] }
walkdir = "2.5"
---

//! Reject broad store handles and raw store services outside `crunch-store`.

use quote::ToTokens;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use syn::visit::Visit;
use walkdir::WalkDir;

const STORE_SHELL_PREFIX: &str = "crates/crunch-store/";
const ROOT_SOURCE_PREFIX: &str = "src/";
const CRATES_PREFIX: &str = "crates/";
const EXAMPLES_PREFIX: &str = "examples/";
const RUST_EXTENSION: &str = "rs";
const MAX_SOURCE_FILES: usize = 20_000;
const MAX_FINDINGS: usize = 256;

const POSITIVE_FIXTURE: &str = include_str!("../fixtures/store-capability-architecture/positive/narrow-build-store.rs");
const NEGATIVE_FIXTURES: &[(&str, &str, &str)] = &[
    (
        "broad-handle",
        include_str!("../fixtures/store-capability-architecture/negative/broad-handle.rs"),
        "StoreHandle",
    ),
    (
        "raw-service-port",
        include_str!("../fixtures/store-capability-architecture/negative/raw-service-port.rs"),
        "PathInfoService",
    ),
    (
        "writable-base-authority",
        include_str!("../fixtures/store-capability-architecture/negative/writable-base-authority.rs"),
        "StoreHandleServices",
    ),
];

const FORBIDDEN_RUNTIME_IDENTIFIERS: &[&str] = &[
    "StoreHandle",
    "StoreHandleServices",
    "BlobService",
    "DirectoryService",
    "PathInfoService",
    "RedbPathInfoService",
    "ObjectStoreBlobService",
    "RedbDirectoryService",
    "blob_service",
    "directory_service",
    "pathinfo_service",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Finding {
    path: String,
    identifier: String,
    owner: String,
}

#[derive(Default)]
struct IdentifierVisitor {
    forbidden: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for IdentifierVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        for segment in &path.segments {
            self.record(&segment.ident.to_string());
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        self.record(&expression.method.to_string());
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.scan_tokens(item.tokens.clone());
        syn::visit::visit_macro(self, item);
    }
}

impl IdentifierVisitor {
    fn record(&mut self, identifier: &str) {
        if FORBIDDEN_RUNTIME_IDENTIFIERS.contains(&identifier) {
            self.forbidden.insert(identifier.to_string());
        }
    }

    fn scan_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Group(group) => self.scan_tokens(group.stream()),
                proc_macro2::TokenTree::Ident(identifier) => self.record(&identifier.to_string()),
                proc_macro2::TokenTree::Literal(_) | proc_macro2::TokenTree::Punct(_) => {}
            }
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == ["--self-test"] {
        return self_test();
    }
    let root = match arguments.as_slice() {
        [] => env::current_dir().map_err(|error| format!("resolve current directory: {error}"))?,
        [flag, root] if flag == "--root" => PathBuf::from(root),
        _ => return Err("usage: check-store-capability-architecture.rs [--root PATH] | --self-test".to_string()),
    };
    let findings = scan_repository(&root)?;
    if !findings.is_empty() {
        let rendered = findings
            .iter()
            .map(|finding| format!("{}: {} reaches forbidden {} authority", finding.path, finding.owner, finding.identifier))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!("store capability architecture has {} finding(s):\n{rendered}", findings.len()));
    }
    println!("store capability architecture verified: external runtime findings=0");
    Ok(())
}

fn scan_repository(root: &Path) -> Result<Vec<Finding>, String> {
    let mut files = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some(RUST_EXTENSION))
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(root).ok()?.to_path_buf();
            is_runtime_candidate(&relative).then_some((relative, entry.into_path()))
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    if files.len() > MAX_SOURCE_FILES {
        return Err(format!("Rust source file count exceeds {MAX_SOURCE_FILES}"));
    }
    let mut findings = Vec::new();
    for (relative, absolute) in files {
        let source = fs::read_to_string(&absolute)
            .map_err(|error| format!("read {}: {error}", absolute.display()))?;
        scan_source(&relative, &source, &mut findings)?;
        if findings.len() > MAX_FINDINGS {
            return Err(format!("store capability finding count exceeds {MAX_FINDINGS}"));
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn is_runtime_candidate(relative: &Path) -> bool {
    let path = relative.to_string_lossy();
    if path.starts_with(STORE_SHELL_PREFIX) {
        return false;
    }
    if path.contains("/tests/")
        || path.starts_with("tests/")
        || path.starts_with("vendor/")
        || path.ends_with("/test_support.rs")
    {
        return false;
    }
    path.starts_with(ROOT_SOURCE_PREFIX)
        || path.starts_with(CRATES_PREFIX)
        || path.starts_with(EXAMPLES_PREFIX)
}

fn scan_source(path: &Path, source: &str, findings: &mut Vec<Finding>) -> Result<(), String> {
    let syntax = syn::parse_file(source).map_err(|error| format!("parse {}: {error}", path.display()))?;
    scan_items(path, &syntax.items, findings)
}

fn scan_items(path: &Path, items: &[syn::Item], findings: &mut Vec<Finding>) -> Result<(), String> {
    for item in items {
        if item_is_test(item) {
            continue;
        }
        match item {
            syn::Item::Impl(item_impl) => scan_impl(path, item_impl, findings),
            syn::Item::Mod(item_mod) => {
                scan_one(path, item_mod, &format!("module {}", item_mod.ident), findings);
                if let Some((_brace, nested)) = &item_mod.content {
                    scan_items(path, nested, findings)?;
                }
            }
            _ => scan_one(path, item, item_owner(item), findings),
        }
    }
    Ok(())
}

fn scan_impl(path: &Path, item: &syn::ItemImpl, findings: &mut Vec<Finding>) {
    let owner = format!("impl {}", item.self_ty.to_token_stream());
    scan_one(path, &item.self_ty, &owner, findings);
    if let Some((_bang, trait_path, _for)) = &item.trait_ {
        scan_one(path, trait_path, &owner, findings);
    }
    for member in &item.items {
        if impl_item_is_test(member) {
            continue;
        }
        scan_one(path, member, &owner, findings);
    }
}

fn scan_one<T: ToTokens>(path: &Path, value: &T, owner: &str, findings: &mut Vec<Finding>) {
    let source = value.to_token_stream().to_string();
    let syntax = syn::parse_file(&format!("fn __scan() {{ {source}; }}"));
    let mut visitor = IdentifierVisitor::default();
    match syntax {
        Ok(file) => visitor.visit_file(&file),
        Err(_) => visitor.scan_tokens(value.to_token_stream()),
    }
    for identifier in visitor.forbidden {
        findings.push(Finding {
            path: path.to_string_lossy().into_owned(),
            identifier,
            owner: owner.to_string(),
        });
    }
}

fn item_is_test(item: &syn::Item) -> bool {
    match item {
        syn::Item::Const(item) => has_test_cfg(&item.attrs),
        syn::Item::Enum(item) => has_test_cfg(&item.attrs),
        syn::Item::ExternCrate(item) => has_test_cfg(&item.attrs),
        syn::Item::Fn(item) => has_test_cfg(&item.attrs) || has_test_attribute(&item.attrs),
        syn::Item::ForeignMod(item) => has_test_cfg(&item.attrs),
        syn::Item::Impl(item) => has_test_cfg(&item.attrs),
        syn::Item::Macro(item) => has_test_cfg(&item.attrs),
        syn::Item::Mod(item) => has_test_cfg(&item.attrs),
        syn::Item::Static(item) => has_test_cfg(&item.attrs),
        syn::Item::Struct(item) => has_test_cfg(&item.attrs),
        syn::Item::Trait(item) => has_test_cfg(&item.attrs),
        syn::Item::TraitAlias(item) => has_test_cfg(&item.attrs),
        syn::Item::Type(item) => has_test_cfg(&item.attrs),
        syn::Item::Union(item) => has_test_cfg(&item.attrs),
        syn::Item::Use(item) => has_test_cfg(&item.attrs),
        _ => false,
    }
}

fn impl_item_is_test(item: &syn::ImplItem) -> bool {
    match item {
        syn::ImplItem::Const(item) => has_test_cfg(&item.attrs),
        syn::ImplItem::Fn(item) => has_test_cfg(&item.attrs) || has_test_attribute(&item.attrs),
        syn::ImplItem::Macro(item) => has_test_cfg(&item.attrs),
        syn::ImplItem::Type(item) => has_test_cfg(&item.attrs),
        _ => false,
    }
}

fn has_test_cfg(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg") && attribute.meta.to_token_stream().to_string().split_whitespace().any(|part| {
            part.trim_matches(|character: char| !character.is_alphanumeric() && character != '_') == "test"
        })
    })
}

fn has_test_attribute(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("test")
            || attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
    })
}

fn item_owner(item: &syn::Item) -> &'static str {
    match item {
        syn::Item::Const(_) => "const",
        syn::Item::Enum(_) => "enum",
        syn::Item::ExternCrate(_) => "extern crate",
        syn::Item::Fn(_) => "function",
        syn::Item::ForeignMod(_) => "foreign module",
        syn::Item::Impl(_) => "impl",
        syn::Item::Macro(_) => "macro",
        syn::Item::Mod(_) => "module",
        syn::Item::Static(_) => "static",
        syn::Item::Struct(_) => "struct",
        syn::Item::Trait(_) => "trait",
        syn::Item::TraitAlias(_) => "trait alias",
        syn::Item::Type(_) => "type alias",
        syn::Item::Union(_) => "union",
        syn::Item::Use(_) => "use",
        _ => "item",
    }
}

fn self_test() -> Result<(), String> {
    let path = Path::new("src/fixture.rs");
    let test_only = "#[cfg(test)] mod tests { use crunch_store::StoreHandle; }";
    let mut findings = Vec::new();
    scan_source(path, POSITIVE_FIXTURE, &mut findings)?;
    if !findings.is_empty() {
        return Err("positive capability fixture produced a finding".to_string());
    }
    for (label, source, expected_identifier) in NEGATIVE_FIXTURES {
        findings.clear();
        scan_source(path, source, &mut findings)?;
        if !findings.iter().any(|finding| finding.identifier == *expected_identifier) {
            return Err(format!("negative capability fixture produced no {expected_identifier} finding: {label}"));
        }
    }
    findings.clear();
    scan_source(path, test_only, &mut findings)?;
    if !findings.is_empty() {
        return Err("cfg(test) fixture was classified as runtime authority".to_string());
    }
    println!("store capability architecture positive and negative self-tests passed");
    Ok(())
}
