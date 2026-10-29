#![no_std]
extern crate alloc;

mod actor_effects;
mod allocator;

use alloc::string::String;
use alloc::vec::Vec;

struct Fixture;

impl actor_effects::Guest for Fixture {
    fn invoke(mut input: Vec<u8>) -> Result<Vec<u8>, String> {
        let offset = command_offset(&input)?;
        let Some(mode) = input.get(offset).copied() else {
            return Err(String::from("missing fixture mode"));
        };
        match mode {
            // Reuse the input allocation. The tail is opaque product-owned bytes.
            0 => {
                input.drain(..offset + 1);
                Ok(input)
            }
            1 => actor_effects::effect_request(&input[offset + 1..]),
            2 => core::arch::wasm32::unreachable(),
            // 0xff is deliberately not a valid packed Preserves value.
            3 => Ok(alloc::vec![0xff]),
            4 => workload(None),
            5 if input.len() == offset + 9 => {
                let count = u64::from_le_bytes(input[offset + 1..].try_into().unwrap());
                workload(Some(count))
            }
            _ => Err(String::from("invalid fixture mode or workload width")),
        }
    }
}

// A closed canonical packed-Preserves ByteString carrier, not a domain codec.
// Tag b2 and minimally encoded unsigned LEB128 length are the only admitted
// framing. The product independently owns canonical payload/effect admission.
fn command_offset(input: &[u8]) -> Result<usize, String> {
    if input.first() != Some(&0xb2) {
        return Err(String::from("fixture command must be a canonical ByteString"));
    }
    let mut length = 0_usize;
    for index in 0..3 {
        let byte = *input.get(index + 1).ok_or_else(|| String::from("truncated command length"))?;
        length |= usize::from(byte & 127) << (index * 7);
        if byte < 128 {
            if (index != 0 && byte == 0) || length > 65_536 || index + 2 + length != input.len() {
                return Err(String::from("noncanonical or oversized command length"));
            }
            return Ok(index + 2);
        }
    }
    Err(String::from("oversized command length"))
}

// Volatile observations prevent deletion or replacement with a constant.
// No clock, environment, RNG, or hostcall participates in workload selection.
fn workload(count: Option<u64>) -> Result<Vec<u8>, String> {
    let mut state = 0_u64;
    let mut iteration = 0_u64;
    loop {
        if count == Some(iteration) {
            let mut output = Vec::with_capacity(10);
            output.extend_from_slice(&[0xb2, 8]);
            output.extend_from_slice(&state.to_le_bytes());
            return Ok(output);
        }
        state = state.wrapping_mul(6364136223846793005).wrapping_add(iteration | 1);
        unsafe {
            core::ptr::write_volatile(&mut state, state);
        }
        iteration = iteration.wrapping_add(1);
    }
}

actor_effects::export!(Fixture with_types_in actor_effects);

#[unsafe(export_name = "cabi_realloc")]
unsafe extern "C" fn cabi_realloc(old: *mut u8, old_bytes: usize, align: usize, new_bytes: usize) -> *mut u8 {
    if new_bytes == 0 {
        return align as *mut u8;
    }
    let layout = core::alloc::Layout::from_size_align(old_bytes.max(1), align).unwrap();
    let result = if old_bytes == 0 {
        unsafe { alloc::alloc::alloc(core::alloc::Layout::from_size_align(new_bytes, align).unwrap()) }
    } else {
        unsafe { alloc::alloc::realloc(old, layout, new_bytes) }
    };
    if result.is_null() {
        core::arch::wasm32::unreachable();
    }
    result
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
