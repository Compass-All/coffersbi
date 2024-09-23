use core::alloc::Layout;
use alloc::vec::Vec;
use buddy_system_allocator::{Heap, LockedHeapWithRescue, LockedFrameAllocator};
use spin::Once;
use sbi_spec::binary::SbiRet;

use crate::enclave_id::EnclaveId;

struct MemoryPool {
    start: usize,
    size: usize,
}

const ORDER: usize = 32;
const FRAME_SIZE: usize = 0x20_0000; // 2M
const FRAME_ORDER: usize = FRAME_SIZE.trailing_zeros() as usize; // 21

#[global_allocator]
static GLOBAL_HEAP: LockedHeapWithRescue::<ORDER> = LockedHeapWithRescue::<ORDER>::new(global_heap_rescue);

static FRAME_ALLOCATOR: Once<LockedFrameAllocator<FRAME_ORDER>> = Once::new();
static MEMORY_POOL: Once<MemoryPool> = Once::new();

// ---------------------------------
// Utility functions

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

// ---------------------------------

pub(crate) fn coffer_memory_init(pool_start: usize, pool_size: usize) -> SbiRet {
    log::info!("Initializing CofferSBI Memory");

    if pool_start == 0 || pool_size <= FRAME_SIZE || pool_size % FRAME_SIZE != 0 {
        log::warn!("Invalid pool address or size");
        log::warn!("pool_start: 0x{:x}, pool_size: 0x{:x}", pool_start, pool_size);
        return SbiRet::invalid_param();
    }

    MEMORY_POOL.call_once(|| MemoryPool {
        start: pool_start,
        size: pool_size,
    });
    log::info!("Pool: 0x{:x} -> 0x{:x}, size: 0x{:x}",
        pool_start, pool_start + pool_size, pool_size);

    unsafe {
        GLOBAL_HEAP.lock().init(pool_start, FRAME_SIZE);
    }
    log::info!("Global heap initialized at 0x{:x}, initial size: 0x{:x}",
        pool_start, FRAME_SIZE);

    FRAME_ALLOCATOR.call_once(|| LockedFrameAllocator::<FRAME_ORDER>::new());
    FRAME_ALLOCATOR.get().unwrap().lock()
        .add_frame(1, pool_size / FRAME_SIZE - 1);
    log::info!("Frame allocator initialized with frames {}..{}",
        1, pool_size / FRAME_SIZE - 1);

    SbiRet::success(0)
}

pub(crate) fn coffer_mem_alloc(eid: EnclaveId, size: usize) -> SbiRet {
    log::debug!("CofferSBI mem_alloc");
    log::debug!("{:?} is allocating 0x{:x} bytes", eid, size);
    // align size to FRAME_SIZE
    let aligned = align_up(size, FRAME_SIZE);
    let num_frame = aligned / FRAME_SIZE;
    if let Some(frame) = frame_allocator().lock().alloc(num_frame) {
        let paddr = frame_to_paddr(frame);
        log::debug!("Allocated 0x{:x} bytes at 0x{:x}", aligned, paddr);
        SbiRet::success(paddr)
    } else {
        log::warn!("Allocation failed. No contiguous memory found for requested size: 0x{:x}", size);
        SbiRet::denied()
    }
}

// ---------------------------------

fn global_heap_rescue(heap: &mut Heap<ORDER>, layout: &Layout) {
    if layout.size() > FRAME_SIZE {
        panic!("Trying to allocate 0x{:x} more than frame size (0x{:x})",
            layout.size(), FRAME_SIZE);
    }

    let new_frame = frame_allocator().lock().alloc(1);
    if let Some(frame) = new_frame {
        let paddr = frame_to_paddr(frame);
        unsafe {
            heap.add_to_heap(paddr, paddr + FRAME_SIZE);
        }
        log::info!("Added to heap: 0x{:x}, frame: 0x{:x}", paddr, frame);
    } else {
        panic!("Global heap out of memory");
    }
}

fn frame_allocator() -> &'static LockedFrameAllocator<FRAME_ORDER> {
    if let Some(allocator) = FRAME_ALLOCATOR.get() {
        allocator
    } else {
        panic!("Frame allocator not initialized");
    }
}

fn frame_to_paddr(frame: usize) -> usize {
    if let Some(pool) = MEMORY_POOL.get() {
        pool.start + frame * FRAME_SIZE
    } else {
        panic!("Memory pool not initialized");
    }
}

fn _paddr_to_frame(paddr: usize) -> usize {
    if let Some(pool) = MEMORY_POOL.get() {
        (paddr - pool.start) / FRAME_SIZE
    } else {
        panic!("Memory pool not initialized");
    }
}


// test
pub(crate) fn coffer_memory_test(test_id: usize) -> SbiRet {
    match test_id {
        0 => coffer_memory_test0(),
        1 => coffer_memory_test1(),
        _ => SbiRet::not_supported(),
    }
}

fn coffer_memory_test1() -> SbiRet {
    let paddr = 0x1_2000_0000;
    let size = 0x100;
    let mut buf = vec![0; size];
    unsafe {
        core::ptr::copy(paddr as *const u8, buf.as_mut_ptr(), size);
    }
    for i in 0..size / 16 {
        let line = &buf[i * 16..(i + 1) * 16];
        let mut line_str = format!("{:x}: ", paddr + i * 16);
        for b in line {
            line_str.push_str(&format!("{:02x} ", b));
        }
        log::debug!("{}", line_str);
    }

    SbiRet::success(0)
}


fn coffer_memory_test0() -> SbiRet {
    log::debug!("CofferSBI memory test");

    let heap_total = GLOBAL_HEAP.lock().stats_total_bytes();
    log::debug!("Global heap total: 0x{:x}", heap_total);

    let heap_allocated = GLOBAL_HEAP.lock().stats_alloc_actual();
    log::debug!("1 Global heap allocated: 0x{:x}", heap_allocated);

    if let Some(frame1) = frame_allocator().lock().alloc(1) {
        log::debug!("Allocated 1 frame: 0x{:x}", frame1);
    } else {
        panic!("Frame allocator test 1 failed");
    }

    if let Some(frame2) = frame_allocator().lock().alloc(1) {
        log::debug!("Allocated 1 frame: 0x{:x}", frame2);
    } else {
        panic!("Frame allocator test 2 failed");
    }

    frame_allocator().lock().dealloc(1, 2);
    log::debug!("Deallocated frame: 0x{:x}", 1);

    if let Some(frame1) = frame_allocator().lock().alloc(2) {
        log::debug!("Allocated 2 frames: 0x{:x}", frame1);
    } else {
        panic!("Frame allocator test 3 failed");
    }

    if let Some(frame2) = frame_allocator().lock().alloc_aligned(Layout::from_size_align(7, 8).unwrap()) {
        log::debug!("Allocated aligned 7 frames: 0x{:x}", frame2);
    } else {
        panic!("Frame allocator test 4 failed");
    }

    if let Some(frame1) = frame_allocator().lock().alloc(100) {
        log::debug!("Allocated 100 frame: 0x{:x}", frame1);
    } else {
        panic!("Frame allocator test 5 failed");
    }

    let heap_allocated = GLOBAL_HEAP.lock().stats_alloc_actual();
    log::debug!("Global heap allocated: 0x{:x}", heap_allocated);

    let vec = vec![0; 10];
    log::debug!("Allocated vec: {:?}", vec);

    let heap_allocated = GLOBAL_HEAP.lock().stats_alloc_actual();
    log::debug!("2 Global heap allocated: 0x{:x}", heap_allocated);

    let len = heap_total - heap_allocated;
    let mut vec: Vec<u8> = Vec::with_capacity(len);

    let heap_allocated = GLOBAL_HEAP.lock().stats_alloc_actual();
    log::debug!("3 Global heap allocated: 0x{:x}", heap_allocated);

    vec.push(0); // should trigger OOM

    SbiRet::success(0)
}
