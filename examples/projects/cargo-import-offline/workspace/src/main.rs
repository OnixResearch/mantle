const EXPECTED_VALUE: u8 = 42;

fn main() {
    assert_eq!(gallery_dep::value(), EXPECTED_VALUE);
    println!("cargo import gallery: {EXPECTED_VALUE}");
}
