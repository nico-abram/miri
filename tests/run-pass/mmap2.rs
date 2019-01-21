#![feature(rustc_private)]

extern crate libc;
use libc::*;
use std::ptr;

pub unsafe fn magic_alloc() -> (*mut u8, *mut u8) {
    // Create an in-memory file:
    let mut fname = *b"/tmp/my_in_memory_fileXXXXXX\0";
    let fd: c_long = syscall(SYS_memfd_create, fname.as_mut_ptr() as *mut c_char, 0);
    assert_ne!(fd, -1);

    // Resize it to the memory page size:
    let fd = fd as c_int;
    let page_size = sysconf(_SC_PAGESIZE);
    assert_ne!(ftruncate(fd, page_size as off_t), -1);

    // Map two adjacent virtual-memory regions to the physical-memory
    // of the in-memory file. This is done in two race-free steps.
    //
    // First, allocate a large enough virtual memory region of `2 * page_size`,
    // and map it to the physical memory of the in-memory file (the second half
    // refers to the memory after the in-memory file):
    let ptr0 = mmap(
        ptr::null_mut(),
        (page_size * 2) as size_t,
        PROT_READ | PROT_WRITE,
        MAP_SHARED,
        fd,
        0,
    );
    assert_ne!(ptr0, MAP_FAILED);

    // Map the second half of the virtual memory region to the in-memory file
    // as well, creating two adjacent virtual- memory regions that refer to the
    // same physical memory:
    let ptr1 = mmap(
        (ptr0 as *mut u8).offset(page_size as isize) as *mut c_void,
        page_size as size_t,
        PROT_READ | PROT_WRITE,
        MAP_SHARED | MAP_FIXED,
        fd,
        0,
    );
    assert_ne!(ptr1, MAP_FAILED);

    // Even though the virtual-memory regions [ptr0, ptr0+page_size) and
    // [ptr1, ptr1+page_size) are disjoint, they refer to the same physical
    // memory, and do therefore overlap:
    let ptr0 = ptr0 as *mut u8;
    let ptr1 = ptr1 as *mut u8;
    (ptr0, ptr1)
}

fn main() {
    unsafe {
        let (ptr0, ptr1) = magic_alloc();
        assert_ne!(ptr0, ptr1);
        *ptr0 = 42;
        assert_eq!(*ptr1, 42);
        *ptr1 = 24;
        assert_eq!(*ptr0, 24);
        // ba dum tss
    }
}
