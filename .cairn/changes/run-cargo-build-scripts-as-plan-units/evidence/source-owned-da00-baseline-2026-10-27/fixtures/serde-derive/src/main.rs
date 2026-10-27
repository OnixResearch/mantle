use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    answer: i32,
}

fn main() {
    let _ = std::mem::size_of::<Fixture>();
}
