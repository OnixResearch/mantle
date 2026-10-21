//! Frozen deterministic fixture recipe, including complete independent postimages.
use std::collections::BTreeMap;

use mantle_rust_differential_core::Digest;
use mantle_rust_differential_core::Expected;
use mantle_rust_differential_core::Status;
use serde::Deserialize;
use serde::Serialize;

use crate::Result;
use crate::err;

pub const RECIPE: &str = "mantle-cargo-fixtures-v1";
pub type Snapshot = BTreeMap<String, Vec<u8>>;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edit {
    pub path: String,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
    pub matches: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub snapshot: Snapshot,
    pub digest: Digest,
    pub edits: Vec<Edit>,
    pub expected: Expected,
    pub force: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fixture {
    pub id: String,
    pub steps: Vec<Step>,
    pub packages: Vec<String>,
}
pub fn identity(snapshot: &Snapshot) -> Digest {
    Digest::from_slice(&serde_json::to_vec(snapshot).expect("snapshot serializable"))
}
fn put(s: &mut Snapshot, path: &str, text: &str) {
    s.insert(path.into(), text.as_bytes().to_vec());
}
fn change(s: &mut Snapshot, path: &str, before: &str, after: &str) {
    let old = String::from_utf8(s[path].clone()).expect("recipe text");
    assert_eq!(old.matches(before).count(), 1);
    put(s, path, &old.replacen(before, after, 1));
}
fn append(steps: &mut Vec<Step>, id: &str, s: &Snapshot, output: Option<i64>, code: Option<&str>, force: bool) {
    let prior = steps.last().map(|step| &step.snapshot);
    let mut paths: Vec<_> = s.keys().chain(prior.into_iter().flat_map(|p| p.keys())).cloned().collect();
    paths.sort();
    paths.dedup();
    let edits = paths
        .into_iter()
        .filter_map(|path| {
            let before = prior.and_then(|p| p.get(&path)).cloned();
            let after = s.get(&path).cloned();
            (before != after).then_some(Edit {
                path,
                before,
                after,
                matches: 1,
            })
        })
        .collect();
    steps.push(Step {
        id: id.into(),
        snapshot: s.clone(),
        digest: identity(s),
        edits,
        force,
        expected: Expected {
            status: if code.is_some() {
                Status::Rejected
            } else {
                Status::Success
            },
            diagnostic_codes: code.into_iter().map(str::to_owned).collect(),
            diagnostic_messages: code
                .map(|c| {
                    if c == "E0425" {
                        "cannot find value `missing_value` in this scope"
                    } else {
                        "mismatched types"
                    }
                    .to_owned()
                })
                .into_iter()
                .collect(),
            program_stdout: output.map(|n| format!("{n}\n").into_bytes()),
        },
    });
}
/// Complete sources and lockfile contain no registry dependencies or timestamps.
pub fn generate(id: &str) -> Result<Fixture> {
    if !["chain", "wide", "proc-macro", "generated"].contains(&id) {
        return Err("unknown fixture".into());
    }
    let count = if id == "wide" { 8 } else { 4 };
    let mut s = Snapshot::new();
    let mut packages: Vec<String> = (0..count).map(|n| format!("lib{n}")).collect();
    packages.extend(["fixture-macro".into(), "fixture-app".into()]);
    let members = packages.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(",");
    put(
        &mut s,
        "Cargo.toml",
        &format!(
            "[workspace]\nresolver=\"2\"\nmembers=[{members}]\n[profile.dev]\ndebug=0\nincremental=true\n[profile.release]\ndebug=0\nincremental=true\ncodegen-units=1\n"
        ),
    );
    put(
        &mut s,
        "fixture-macro/Cargo.toml",
        "[package]\nname=\"fixture-macro\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[lib]\nproc-macro=true\n",
    );
    put(
        &mut s,
        "fixture-macro/src/lib.rs",
        "extern crate proc_macro;\n#[proc_macro]\npub fn number(_:proc_macro::TokenStream)->proc_macro::TokenStream { \"0i64\".parse().unwrap() }\n",
    );
    for n in 0..count {
        let dependencies = if n == 0 {
            "fixture-macro={path=\"../fixture-macro\"}\n".into()
        } else if id == "wide" {
            String::new()
        } else {
            format!("lib{}={{path=\"../lib{}\"}}\n", n - 1, n - 1)
        };
        put(
            &mut s,
            &format!("lib{n}/Cargo.toml"),
            &format!("[package]\nname=\"lib{n}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\n{dependencies}"),
        );
        let body = if n == 0 {
            BASE_LIB.into()
        } else if id == "wide" {
            format!("pub fn value()->i64 {{ {} }}\n", n + 1)
        } else {
            format!("pub fn value()->i64 {{ lib{}::value()+{} }}\n", n - 1, n + 1)
        };
        put(&mut s, &format!("lib{n}/src/lib.rs"), &body);
    }
    put(
        &mut s,
        "lib0/build.rs",
        "fn main(){println!(\"cargo:rerun-if-changed=input.txt\");let n=std::fs::read_to_string(\"input.txt\").unwrap();std::fs::write(std::path::Path::new(&std::env::var_os(\"OUT_DIR\").unwrap()).join(\"generated.rs\"),format!(\"pub fn generated()->i64 {{{}}}\",n.trim())).unwrap();}\n",
    );
    put(&mut s, "lib0/input.txt", "0\n");
    put(&mut s, "lib0/src/included.rs", "0i64\n");
    s.insert("lib0/src/input.bin".into(), vec![0]);
    let app_dependencies = (0..count).map(|n| format!("lib{n}={{path=\"../lib{n}\"}}\n")).collect::<String>();
    put(
        &mut s,
        "fixture-app/Cargo.toml",
        &format!(
            "[package]\nname=\"fixture-app\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\n{app_dependencies}"
        ),
    );
    let expression = if id == "wide" {
        (0..count).map(|n| format!("lib{n}::value()")).collect::<Vec<_>>().join("+")
    } else {
        format!("lib{}::value()", count - 1)
    };
    put(&mut s, "fixture-app/src/main.rs", &format!("fn main() {{ println!(\"{{}}\", {expression}+0); }}\n"));
    let mut lock = "# Frozen offline fixture lockfile.\nversion = 3\n".to_string();
    for p in &packages {
        lock.push_str(&format!("\n[[package]]\nname = \"{p}\"\nversion = \"0.1.0\"\n"));
        let deps: Vec<String> = if p == "fixture-app" {
            (0..count).map(|n| format!("lib{n}")).collect()
        } else if p == "lib0" {
            vec!["fixture-macro".into()]
        } else if p.starts_with("lib") && id != "wide" {
            vec![format!("lib{}", p[3..].parse::<u32>().map_err(err)? - 1)]
        } else {
            vec![]
        };
        if !deps.is_empty() {
            lock.push_str(&format!(
                "dependencies = [{}]\n",
                deps.iter().map(|d| format!("\"{d}\"")).collect::<Vec<_>>().join(",")
            ));
        }
    }
    put(&mut s, "Cargo.lock", &lock);
    let base = s.clone();
    let base_value = if id == "wide" { 36 } else { 10 };
    let mut steps = vec![];
    append(&mut steps, "cold", &s, Some(base_value), None, false);
    append(&mut steps, "warm-unchanged", &s, Some(base_value), None, false);
    append(&mut steps, "forced-unchanged", &s, Some(base_value), None, true);
    change(&mut s, "lib0/src/lib.rs", "fn body()->i64 { 1 }", "fn body()->i64 { 11 }");
    append(&mut steps, "body", &s, Some(base_value + 10), None, false);
    put(&mut s, "lib0/src/interface.rs", "pub fn interface()->i64 { 2 }\n");
    change(&mut s, "lib0/src/lib.rs", "body()+opaque()", "body()+interface::interface()+opaque()");
    let text = String::from_utf8(s["lib0/src/lib.rs"].clone()).map_err(err)?;
    put(&mut s, "lib0/src/lib.rs", &format!("mod interface;\n{text}"));
    append(&mut steps, "interface", &s, Some(base_value + 12), None, false);
    change(
        &mut s,
        "lib0/src/lib.rs",
        "fn opaque()->impl Into<i64> { 0i32 }",
        "fn opaque()->impl Into<i64> { 3u32 }",
    );
    append(&mut steps, "opaque", &s, Some(base_value + 15), None, false);
    change(&mut s, "lib0/src/lib.rs", "x.into()", "x.into()+4");
    append(&mut steps, "generic", &s, Some(base_value + 19), None, false);
    change(&mut s, "lib0/src/lib.rs", "const VALUE:i64=0", "const VALUE:i64=5");
    append(&mut steps, "const", &s, Some(base_value + 24), None, false);
    change(&mut s, "lib0/src/lib.rs", "async fn future()->i64 { 0 }", "async fn future()->i64 { 6 }");
    append(&mut steps, "async", &s, Some(base_value + 30), None, false);
    change(&mut s, "lib0/src/lib.rs", "fn body()->i64 { 11 }", "fn body()->i64 { missing_value }");
    append(&mut steps, "broken", &s, None, Some("E0425"), false);
    change(&mut s, "fixture-app/src/main.rs", "+0);", "+1);");
    append(&mut steps, "still-broken", &s, None, Some("E0425"), false);
    change(&mut s, "lib0/src/lib.rs", "fn body()->i64 { missing_value }", "fn body()->i64 { 11 }");
    append(&mut steps, "repair", &s, Some(base_value + 31), None, false);
    put(&mut s, "lib0/src/structural.rs", "pub fn added()->i64 { 7 }\n");
    change(&mut s, "lib0/src/lib.rs", "body()+interface", "structural::added()+body()+interface");
    let text = String::from_utf8(s["lib0/src/lib.rs"].clone()).map_err(err)?;
    put(&mut s, "lib0/src/lib.rs", &format!("mod structural;\n{text}"));
    append(&mut steps, "structural", &s, Some(base_value + 38), None, false);
    let path = format!("lib{}/src/lib.rs", count - 1);
    let old = String::from_utf8(s[&path].clone()).map_err(err)?;
    let changed = old.replacen(&format!("{} }}", count), &format!("{} }}", count + 10), 1);
    put(&mut s, &path, &changed);
    append(&mut steps, "upstream", &s, Some(base_value + 48), None, false);
    put(&mut s, "lib0/src/included.rs", "8i64\n");
    s.insert("lib0/src/input.bin".into(), vec![1]);
    append(&mut steps, "include", &s, Some(base_value + 57), None, false);
    change(&mut s, "fixture-macro/src/lib.rs", "0i64", "9i64");
    append(&mut steps, "macro", &s, Some(base_value + 66), None, false);
    put(&mut s, "lib0/input.txt", "10\n");
    append(&mut steps, "generated", &s, Some(base_value + 76), None, false);
    change(&mut s, "lib0/src/lib.rs", "fn body()->i64 { 11 }", "fn body()->i64 { \"wrong interface\" }");
    append(&mut steps, "interface-error", &s, None, Some("E0308"), false);
    change(&mut s, "lib0/src/lib.rs", "fn body()->i64 { \"wrong interface\" }", "fn body()->i64 { 11 }");
    append(&mut steps, "interface-repair", &s, Some(base_value + 76), None, false);
    change(&mut s, "fixture-app/src/main.rs", "+1);", "+missing_value);");
    append(&mut steps, "application-error", &s, None, Some("E0425"), false);
    change(&mut s, "fixture-app/src/main.rs", "+missing_value);", "+1);");
    append(&mut steps, "application-repair", &s, Some(base_value + 76), None, false);
    let later = s.clone();
    append(&mut steps, "revert", &base, Some(base_value), None, false);
    append(&mut steps, "reapply", &later, Some(base_value + 76), None, false);
    Ok(Fixture {
        id: id.into(),
        steps,
        packages,
    })
}
const BASE_LIB: &str = "use std::{future::Future,sync::Arc,task::{Context,Poll,Wake,Waker}};\nstruct Noop; impl Wake for Noop { fn wake(self:Arc<Self>){} }\nfn poll_ready<F:Future>(f:F)->F::Output { let w=Waker::from(Arc::new(Noop)); let mut cx=Context::from_waker(&w); match Box::pin(f).as_mut().poll(&mut cx) { Poll::Ready(v)=>v, Poll::Pending=>panic!(\"unexpected pending\") } }\n#[inline(never)] fn body()->i64 { 1 }\nfn opaque()->impl Into<i64> { 0i32 }\nfn generic<T:Into<i64>>(x:T)->i64 { x.into() }\nconst VALUE:i64=0;\nasync fn future()->i64 { 0 }\ninclude!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\"));\npub fn value()->i64 { body()+opaque().into()+generic(0i32)+VALUE+poll_ready(future())+include!(\"included.rs\")+include_bytes!(\"input.bin\")[0] as i64+fixture_macro::number!()+generated() }\n";
