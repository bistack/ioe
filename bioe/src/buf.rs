use std::alloc::{alloc, alloc_zeroed, dealloc, Layout};
use std::os::raw::c_char;
use std::ptr::NonNull;

pub struct PagesBuffer {
    ptr: *mut c_char,
    cnt: usize,
}

impl PagesBuffer {
    pub fn new(cnt: usize, zeroed: bool) -> PagesBuffer {
        PagesBuffer {
            ptr: alloc_pages_buf(cnt, zeroed),
            cnt: cnt,
        }
    }

    pub fn free(&mut self) {
        free_pages_buf(self.ptr, self.cnt);
    }

    pub fn get_buf_ptr(&mut self) -> *mut c_char {
        self.ptr
    }
}

fn alloc_pages_buf(cnt: usize, zeroed: bool) -> *mut c_char {
    let page_size = 4096;
    let alloc_size = page_size * cnt;
    unsafe {
        if alloc_size == 0 {
            let ptr = NonNull::<c_char>::dangling();
            return ptr.as_ptr();
        }

        let align = page_size;
        let layout = Layout::from_size_align(alloc_size, align).unwrap();
        let ptr = if zeroed {
            alloc_zeroed(layout)
        } else {
            alloc(layout)
        };
        ptr as *mut c_char
    }
}

fn free_pages_buf(pages_ptr: *mut c_char, cnt: usize) {
    let align = 4096;
    let size = 4096 * cnt;
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        dealloc(pages_ptr as *mut u8, layout);
    }
}
