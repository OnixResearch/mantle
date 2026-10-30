#![no_std]
extern crate alloc;

mod client;
#[path = "../../guest_rt.rs"]
mod guest_rt;

struct Client;

impl client::Guest for Client {
    fn run() -> u32 {
        let greeting = client::wrpc_examples::hello::handler::hello();
        assert_eq!(greeting, "hello from Rust");
        42
    }
}

client::export!(Client with_types_in client);
