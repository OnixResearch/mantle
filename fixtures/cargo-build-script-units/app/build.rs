fn main() {
    let metadata =
        std::env::var("DEP_NATIVE_ANSWER_GENERATED").expect("direct links metadata reaches dependent script");
    assert_eq!(metadata, "42");
    println!("cargo::rustc-env=NATIVE_DEP_METADATA={metadata}");
}
