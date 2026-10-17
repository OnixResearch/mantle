use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    answer: i32,
}

fn main() {
    let input: Fixture = serde_json::from_str("{\"answer\":42}").expect("valid fixture input");
    let tokens: proc_macro2::TokenStream = "native_answer".parse().expect("valid token stream");
    assert_eq!(tokens.to_string(), "native_answer");
    assert_eq!(input.answer, 42);
    assert!(unsafe { libc::getpid() } > 0);
    assert_eq!(native_answer_sys::answer(), input.answer);
    println!("cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42");
}
