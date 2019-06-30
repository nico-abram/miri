// compile-flags: -Zmiri-seed=
#![feature(allocator_api)]

use std::ptr::NonNull;
use std::alloc::{Global, Alloc, Layout, System};
use std::slice;

fn check_alloc<T: Alloc>(mut allocator: T) { unsafe {
    let layout = Layout::from_size_align(20, 4).unwrap();
    let a = allocator.alloc(layout).unwrap();
    allocator.dealloc(a, layout);

    let p1 = allocator.alloc_zeroed(layout).unwrap();

    let p2 = allocator.realloc(p1, Layout::from_size_align(20, 4).unwrap(), 40).unwrap();
    let slice = slice::from_raw_parts(p2.as_ptr(), 20);
    assert_eq!(&slice, &[0_u8; 20]);

    // old size == new size
    let p3 = allocator.realloc(p2, Layout::from_size_align(40, 4).unwrap(), 40).unwrap();
    let slice = slice::from_raw_parts(p3.as_ptr(), 20);
    assert_eq!(&slice, &[0_u8; 20]);

    // old size > new size
    let p4 = allocator.realloc(p3, Layout::from_size_align(40, 4).unwrap(), 10).unwrap();
    let slice = slice::from_raw_parts(p4.as_ptr(), 10);
    assert_eq!(&slice, &[0_u8; 10]);

    allocator.dealloc(p4, Layout::from_size_align(10, 4).unwrap());
} }

unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
    ptr.add(align - (ptr as usize & (align - 1)))
}

fn allocate_with_align<T: Alloc>(mut allocator: T, size: usize, align: usize) -> *mut u8 { unsafe {
    let ptr = allocator.alloc(Layout::from_size_align(size + align, 1).unwrap()).unwrap().as_ptr();
    eprintln!("{}", format!("Before aligning: {:?}", ptr));
    let ptr = align_ptr(ptr, align);
    eprintln!("{}", format!("After aligning: {:?}", ptr));
    ptr
} }

fn check_overalign_requests<T: Alloc+Copy>(mut allocator: T) {
    let size = 8;
    // Greater than `size`.
    let align = 16;

    let iterations = 5;
    unsafe {
        let pointers: Vec<_> = (0..iterations).map(|_| {
            allocate_with_align(allocator, size, align)
        }).collect();
        for &ptr in &pointers {
            assert_eq!((ptr as usize) % align, 0,
                       "Got a pointer less aligned than requested")
        }

        // Clean up.
        for &ptr in &pointers {
            allocator.dealloc(NonNull::new(ptr).unwrap(), Layout::from_size_align(size, 1).unwrap())
        }
    }
}

fn global_to_box() {
    type T = [i32; 4];
    let l = Layout::new::<T>();
    // allocate manually with global allocator, then turn into Box and free there
    unsafe {
        let ptr = Global.alloc(l).unwrap().as_ptr() as *mut T;
        let b = Box::from_raw(ptr);
        drop(b);
    }
}

fn box_to_global() {
    type T = [i32; 4];
    let l = Layout::new::<T>();
    // allocate with the Box, then deallocate manually with global allocator
    unsafe {
        let b = Box::new(T::default());
        let ptr = Box::into_raw(b);
        Global.dealloc(NonNull::new(ptr as *mut u8).unwrap(), l);
    }
}

fn main() {
    check_alloc(System);
    check_alloc(Global);
    check_overalign_requests(System);
    check_overalign_requests(Global);
    global_to_box();
    box_to_global();
}
