use core::alloc::GlobalAlloc;
use core::alloc::Layout;

const BLOCK_BYTES: usize = 16;
const BLOCK_COUNT: usize = 16_384;
const BITMAP_WORDS: usize = BLOCK_COUNT / 64;

#[repr(align(16))]
struct Heap([u8; BLOCK_BYTES * BLOCK_COUNT]);
static mut HEAP: Heap = Heap([0; BLOCK_BYTES * BLOCK_COUNT]);
static mut USED: [u64; BITMAP_WORDS] = [0; BITMAP_WORDS];

// Single-threaded bounded fixture arena. Freeing canonical input/response and
// post-return buffers makes repeated actor invocations reclaim their storage.
struct Arena;

unsafe impl GlobalAlloc for Arena {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let blocks = layout.size().max(1).div_ceil(BLOCK_BYTES);
        if blocks > BLOCK_COUNT {
            return core::ptr::null_mut();
        }
        let base = core::ptr::addr_of_mut!(HEAP).cast::<u8>();
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
        let base = core::ptr::addr_of_mut!(HEAP).cast::<u8>();
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
