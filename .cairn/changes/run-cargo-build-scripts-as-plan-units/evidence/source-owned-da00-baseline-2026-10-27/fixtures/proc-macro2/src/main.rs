fn main() {
    let tokens: proc_macro2::TokenStream = "native_answer".parse().expect("valid token stream");
    assert_eq!(tokens.to_string(), "native_answer");
}
