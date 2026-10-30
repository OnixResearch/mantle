//! Dependency-free guest runtime shared by this workspace's no_std components.
//! Bindings come from the pinned toolchain `wit-bindgen` CLI, so no binding
//! macro, build script or host linker enters the offline guest build.

use core::alloc::GlobalAlloc;
use core::alloc::Layout;

const BLOCK_BYTES: usize = 16;
const BLOCK_COUNT: usize = 16_384;
const BITMAP_WORDS: usize = BLOCK_COUNT / 64;

#[repr(align(16))]
struct Heap([u8; BLOCK_BYTES * BLOCK_COUNT]);
static mut HEAP: Heap = Heap([0; BLOCK_BYTES * BLOCK_COUNT]);
static mut USED: [u64; BITMAP_WORDS] = [0; BITMAP_WORDS];

// Single-threaded bounded guest arena. Freeing canonical input/response and
// post-return buffers makes repeated actor invocations reclaim their storage.
struct Arena;

unsafe impl GlobalAlloc for Arena {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let blocks = layout.size().max(1).div_ceil(BLOCK_BYTES);
        if blocks > BLOCK_COUNT {
            return core::ptr::null_mut();
        }
        let base = unsafe { core::ptr::addr_of_mut!(HEAP.0) }.cast::<u8>();
        let mut start = 0;
        let mut run = 0;
        for index in 0..BLOCK_COUNT {
            let used = unsafe { (*core::ptr::addr_of!(USED))[index / 64] & (1 << (index % 64)) != 0 };
            if used {
                run = 0;
                continue;
            }
            if run == 0 {
                if (base as usize + index * BLOCK_BYTES) % layout.align() != 0 {
                    continue;
                }
                start = index;
            }
            run += 1;
            if run == blocks {
                for block in start..start + blocks {
                    unsafe {
                        (*core::ptr::addr_of_mut!(USED))[block / 64] |= 1 << (block % 64);
                    }
                }
                return unsafe { base.add(start * BLOCK_BYTES) };
            }
        }
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let base = unsafe { core::ptr::addr_of_mut!(HEAP.0) }.cast::<u8>();
        let first = (pointer as usize - base as usize) / BLOCK_BYTES;
        let blocks = layout.size().max(1).div_ceil(BLOCK_BYTES);
        for block in first..first + blocks {
            unsafe {
                (*core::ptr::addr_of_mut!(USED))[block / 64] &= !(1 << (block % 64));
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: Arena = Arena;

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

/// no_std wasm32-wasip2 links no libc, and `memcmp` has no Wasm instruction.
/// Volatile reads keep LLVM from lowering this loop back into a `memcmp` call.
#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(left: *const u8, right: *const u8, length: usize) -> i32 {
    for index in 0..length {
        let (a, b) = unsafe { (left.add(index).read_volatile(), right.add(index).read_volatile()) };
        if a != b {
            return i32::from(a) - i32::from(b);
        }
    }
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn bcmp(left: *const u8, right: *const u8, length: usize) -> i32 {
    unsafe { memcmp(left, right, length) }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
