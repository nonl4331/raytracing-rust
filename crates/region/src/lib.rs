#![cfg_attr(not(test), no_std)]
#![feature(strict_provenance)]
#![feature(vec_into_raw_parts)]

extern crate alloc;

use alloc::alloc::handle_alloc_error;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::cell::Cell;
use core::mem::{size_of, ManuallyDrop};
use core::ptr::NonNull;

pub mod wrappers;
pub use wrappers::*;

const ALIGNMENT: usize = 32;
const BLOCK_SIZE: usize = 512 * ALIGNMENT;

const DEFAULT_BLOCK_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(BLOCK_SIZE, ALIGNMENT) };

#[repr(C)]
struct Block {
    pub head: Cell<NonNull<u8>>,
    pub bytes_left: usize,
    pub next: Cell<Option<NonNull<Block>>>,
    pub size: usize,
}

fn align_up(current_pos: usize, alignment: usize) -> usize {
    let modu = current_pos % alignment;
    if modu == 0 {
        0
    } else {
        alignment - modu
    }
}

impl Block {
    pub fn new() -> Self {
        let alloc_ptr = unsafe { alloc::alloc::alloc(DEFAULT_BLOCK_LAYOUT) };
        if alloc_ptr.is_null() {
            handle_alloc_error(DEFAULT_BLOCK_LAYOUT);
        }

        // unwrap should never fail due to the above
        let head = unsafe { Cell::new(NonNull::new(alloc_ptr).unwrap_unchecked()) };

        Block {
            head,
            bytes_left: BLOCK_SIZE,
            next: Cell::new(None),
            size: BLOCK_SIZE,
        }
    }
    pub fn new_with_size(size: usize) -> Self {
        let layout = Layout::from_size_align(size, ALIGNMENT)
            .expect("size rounded up to the nearest align overflows!");

        let alloc_ptr = unsafe { alloc::alloc::alloc(layout) };
        if alloc_ptr.is_null() {
            handle_alloc_error(layout);
        }

        // unwrap can never fail due to the above
        let head = unsafe { Cell::new(NonNull::new(alloc_ptr).unwrap_unchecked()) };

        Block {
            head,
            bytes_left: size,
            next: Cell::new(None),
            size,
        }
    }
    pub fn new_block(&mut self, size: Option<usize>) -> NonNull<Block> {
        if self.next.get().is_some() {
            unreachable!()
        }

        let block = Box::new(if let Some(size) = size {
            Block::new_with_size(size)
        } else {
            Block::new()
        });

        let block = alloc::boxed::Box::<Block>::into_raw(block);

        // into_raw cannot return nullptr
        let ptr = unsafe { NonNull::new(block.cast()).unwrap_unchecked() };
        self.next = Cell::new(Some(ptr));
        ptr
    }
    pub fn write_head(&self) -> *mut u8 {
        self.head.get().as_ptr()
    }
    pub fn bytes_left(&self) -> usize {
        self.bytes_left
    }
    pub fn offset_write_head(&mut self, offset: usize) {
        *self.head.get_mut() = NonNull::new(unsafe { self.write_head().add(offset) }).unwrap();
        self.bytes_left -= offset;
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        match self.next.get_mut() {
            Some(block) => unsafe {
                let _ = Box::from_raw(block.as_ptr());
            },
            None => {}
        }

        let ptr = unsafe { self.write_head().sub(self.size - self.bytes_left) };
        let layout = Layout::from_size_align(self.size, ALIGNMENT).unwrap();
        unsafe {
            alloc::alloc::dealloc(ptr, layout);
        }
    }
}

#[repr(C)]
pub struct Region {
    first: NonNull<Block>,
    current: Cell<NonNull<Block>>,
}

unsafe impl Send for Region {}
unsafe impl Sync for Region {}

impl Region {
    pub fn new() -> ManuallyDrop<Self> {
        let block = Box::<Block>::into_raw(Box::new(Block::new()));

        // into_raw cannot return nullptr
        let first = unsafe { NonNull::new(block.cast()).unwrap_unchecked() };
        let current = Cell::new(first);

        ManuallyDrop::new(Self { first, current })
    }
    #[inline(always)]
    fn ptr_alloc<T: Sized>(&mut self, data: T) -> *mut T {
        self.generic_alloc(
            data,
            |data_ptr, data| unsafe { core::ptr::write(data_ptr, data) },
            size_of::<T>(),
            |block, data| Self::ptr_alloc(block, data),
        )
    }
    #[inline(always)]
    fn ptr_slice_alloc<T>(&mut self, data: &[T]) -> *mut [T] {
        let ptr = self
            .generic_alloc(
                data,
                |data_ptr, data| unsafe {
                    core::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut T, data.len())
                },
                size_of::<T>() * data.len(),
                |block, data| Self::ptr_slice_alloc(block, data).cast(),
            )
            .cast();

        unsafe { core::slice::from_raw_parts_mut(ptr, data.len()) }
    }
    #[inline(always)]
    fn generic_alloc<T, F: Fn(*mut T, T), A: Fn(&mut Self, T) -> *mut T>(
        &mut self,
        data: T,
        write: F,
        size: usize,
        alloc: A,
    ) -> *mut T {
        let align = core::mem::align_of::<T>();

        let block = unsafe { self.current.get_mut().as_mut() };

        // write head offset taking into consideration padding before cause of alignment
        let write_offset = align_up(block.write_head().addr(), align);
        let total_size = size + write_offset;

        if total_size > block.bytes_left() {
            // check it is possible to fit data in block to avoid infinite allocation
            if size > BLOCK_SIZE {
                let ptr = block.new_block(Some(size));
                self.current = Cell::new(ptr);

                return alloc(self, data);
            }

            // allocate new block in region
            let ptr = block.new_block(None);
            self.current = Cell::new(ptr);
            return alloc(self, data);
        }

        // offset ptr head to take into consideration alignment of allocated type
        block.offset_write_head(write_offset);

        // get data ptr
        let data_ptr = block.write_head() as *mut T;

        // write data
        write(data_ptr, data);

        // offset write head since we have moved data to block
        block.offset_write_head(size);

        data_ptr
    }
    pub fn alloc<T: Sized>(&mut self, data: T) -> RegionUniq<T> {
        let data_ptr = self.ptr_alloc(data);
        unsafe { RegionUniq(&mut *data_ptr) }
    }
    pub fn alloc_slice<T: Clone>(&mut self, data: &[T]) -> RegionUniqSlice<T> {
        let data_ptr = self.ptr_slice_alloc(data);
        unsafe { RegionUniqSlice(&mut *data_ptr) }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.first.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_region() {
        let mut region = Region::new();
        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }

    #[test]
    fn single_alloc() {
        let mut region = Region::new();

        #[derive(Debug, PartialEq, Clone)]
        enum TestEnum {
            Val1(u32),
            Val2,
            Val3(f64),
        }
        #[derive(Debug, PartialEq, Clone)]
        struct TestStruct {
            a: f64,
            b: [u32; 5],
            c: Vec<TestEnum>,
        }
        let vec = vec![TestEnum::Val1(37), TestEnum::Val2, TestEnum::Val3(-1.3)];
        let (vec_ptr, len, cap) = vec.into_raw_parts();

        let data = TestStruct {
            a: 2.713,
            b: [3, 7, 123, 43124, 0],
            c: unsafe { Vec::from_raw_parts(vec_ptr, len, cap) },
        };
        let data = region.alloc(data);
        let tester = TestStruct {
            a: 2.713,
            b: [3, 7, 123, 43124, 0],
            c: unsafe { Vec::from_raw_parts(vec_ptr, len, cap) },
        };

        assert_eq!(tester.c.as_ptr(), data.c.as_ptr());
        assert_eq!(tester, *data);

        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }

    #[test]
    fn multi_alloc() {
        let mut region = Region::new();
        let mut data = Vec::new();

        const NUM_ALLOC: usize = 4;

        for i in 0..NUM_ALLOC {
            data.push(*region.alloc([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]));
        }
        for (i, element) in data.into_iter().enumerate() {
            assert_eq!(
                [i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7],
                element
            )
        }
        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }

    #[test]
    fn multiple_blocks() {
        let mut region = Region::new();
        let mut data = Vec::new();

        const NUM_ALLOC: usize = BLOCK_SIZE * 10;

        for i in 0..NUM_ALLOC {
            data.push(*region.alloc([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]));
        }
        for (i, element) in data.into_iter().enumerate() {
            assert_eq!(
                [i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7],
                element
            )
        }
        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }

    #[test]
    fn slice_allocation() {
        let mut region = Region::new();
        let data: Vec<u32> = vec![1, 2, 3, 4, 5];

        let a = region.alloc_slice(&data);
        assert_eq!([1, 2, 3, 4, 5], *a);
        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }

    #[test]
    fn large_allocation() {
        let mut region = Region::new();
        let data = [4u8; 100_000]; // investige overflow with larger values
        let a = region.alloc(data);
        assert_eq!(data, *a);
        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }

    #[test]
    fn large_slice_allocation() {
        let mut region = Region::new();
        let data = [4u8; 400_000];
        let a = region.alloc_slice(&data);
        assert_eq!(data, *a);
        unsafe { core::mem::ManuallyDrop::<_>::drop(&mut region) }
    }
}
