#![no_std]

const ANSWER: u32 = 42;

#[unsafe(no_mangle)]
pub extern "C" fn run() -> u32 {
    ANSWER
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
