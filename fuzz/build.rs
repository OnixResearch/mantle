fn main() {
    println!("cargo:rerun-if-changed=c/elf_canonicalizer_oracle.c");
    println!("cargo:rerun-if-changed=../bootstrap/stagex-elf-local-symbol-canonicalizer.c");
    println!("cargo:rerun-if-env-changed=FUZZ_SANITIZE_C");
    println!("cargo:rerun-if-env-changed=MANTLE_FUZZING");
    let mut build = cc::Build::new();
    build.file("c/elf_canonicalizer_oracle.c").warnings(true).extra_warnings(true);
    if std::env::var_os("FUZZ_SANITIZE_C").is_some() {
        build.flag("-g3").flag("-fsanitize=address,undefined").flag("-fno-sanitize-recover=all");
    }
    build.compile("stagex_elf_canonicalizer_oracle");
}
