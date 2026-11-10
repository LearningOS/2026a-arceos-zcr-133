#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

const fn align_down(pos: usize, align: usize) -> usize {
    pos & !(align - 1)
}

const fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if !layout.align().is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let pos = align_up(self.b_pos, layout.align());
        let next = pos.checked_add(layout.size()).ok_or(AllocError::NoMemory)?;
        if next > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        self.b_pos = next;
        self.count += 1;
        NonNull::new(pos as *mut u8).ok_or(AllocError::NoMemory)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        // 单块不单独回收，等全部释放后整片退回
        self.count = self.count.saturating_sub(1);
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        (self.b_pos - self.start) + (self.end - self.p_pos)
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let align = if align_pow2 == 0 { SIZE } else { align_pow2 };
        if align % SIZE != 0 || !align.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let size = num_pages * SIZE;
        let pos = align_down(
            self.p_pos.checked_sub(size).ok_or(AllocError::NoMemory)?,
            align,
        );
        if pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        self.p_pos = pos;
        Ok(pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // 页区只增不减
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / SIZE
    }
}