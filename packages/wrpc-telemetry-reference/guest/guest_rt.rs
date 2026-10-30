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

/// The binding runtime surface the pinned `wit-bindgen` CLI emits for
/// `--runtime-path crate::guest_rt::rt`, without the `wit-bindgen` crate.
pub mod rt {
    use core::alloc::Layout;

    pub struct Cleanup;

    /// Frees one canonical-ABI argument buffer after the import call returns.
    pub struct CleanupGuard {
        pointer: *mut u8,
        layout: Layout,
    }

    impl Cleanup {
        pub fn new(layout: Layout) -> (*mut u8, Option<CleanupGuard>) {
            if layout.size() == 0 {
                return (core::ptr::null_mut(), None);
            }
            let pointer = unsafe { alloc::alloc::alloc(layout) };
            if pointer.is_null() {
                core::arch::wasm32::unreachable();
            }
            (pointer, Some(CleanupGuard { pointer, layout }))
        }
    }

    impl CleanupGuard {
        /// Ownership moved to the caller (export return), so skip the free.
        pub fn forget(self) {
            core::mem::forget(self);
        }
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            unsafe { alloc::alloc::dealloc(self.pointer, self.layout) }
        }
    }

    /// The generated WIT `flags` types use only this closed subset.
    pub mod bitflags {
        macro_rules! bitflags {
            (
                $(#[$outer:meta])*
                $vis:vis struct $name:ident: $bits:ty {
                    $($(#[$inner:meta])* const $flag:ident = $value:expr;)*
                }
            ) => {
                $(#[$outer])*
                $vis struct $name($bits);

                impl $name {
                    $($(#[$inner])* pub const $flag: Self = Self($value);)*

                    pub const fn empty() -> Self {
                        Self(0)
                    }

                    pub const fn bits(&self) -> $bits {
                        self.0
                    }

                    pub const fn from_bits_retain(bits: $bits) -> Self {
                        Self(bits)
                    }
                }

                impl core::ops::BitOr for $name {
                    type Output = Self;

                    fn bitor(self, other: Self) -> Self {
                        Self(self.0 | other.0)
                    }
                }
            };
        }
        pub(crate) use bitflags;
    }
}
