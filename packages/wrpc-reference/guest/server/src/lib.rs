#![no_std]
extern crate alloc;

mod server;
#[path = "../../guest_rt.rs"]
mod guest_rt;

use alloc::string::String;

use server::exports::wrpc_examples::hello::handler;

struct Server;

impl handler::Guest for Server {
    fn hello() -> String {
        String::from("hello from Rust")
    }
}

impl server::Guest for Server {
    fn run() -> u32 {
        let greeting = <Self as handler::Guest>::hello();
        assert_eq!(greeting, "hello from Rust");
        assert_eq!(greeting.len(), 15);
        42
    }
}

server::export!(Server with_types_in server);
