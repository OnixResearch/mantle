use proc_macro::TokenStream;

#[proc_macro]
pub fn answer(_input: TokenStream) -> TokenStream {
    "if cfg!(feature = \"answer-two\") { 43i32 } else { 42i32 }".parse().unwrap()
}
