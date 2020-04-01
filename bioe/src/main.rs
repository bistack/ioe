use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::time::{Instant, Duration};
use std::sync::Arc;
use std::thread;

use rayon::prelude::*;

use bioe::buf::PagesBuffer;
use bioe::rbio::{db_close, db_open, db_t};
use bioe::rbio::{db_read, db_write};

fn main() {
    open_close();
    open_write_close();
    open_read_close();
    open_write_read_close();
}

#[derive(Clone, Copy)]
struct ThreadSafeByC {
    db: *mut db_t,
    buf: *const c_char,
}

unsafe impl Send for ThreadSafeByC {}

impl ThreadSafeByC {
    pub fn new(db: *mut db_t , buf: *const c_char) -> ThreadSafeByC {
        ThreadSafeByC {
            db: db,
            buf: buf,
        }
    }

    pub fn get_db(&self) -> *mut db_t {
        self.db
    }

    pub fn get_buf(&self) -> *const c_char {
        self.buf
    }
}

// rust string is a fat pointer, with len. so it can contains \0.
const DBNAME: &str = "/dev/lnvm1:Database0\0";
const cid_cnt: u32 = 32;

fn open_close() {
    let dbname = (*DBNAME).as_ptr() as *const c_char;
    let mut db: *mut db_t = null_mut();
    let ret = db_open(dbname, &mut db as *mut *mut db_t);
    if ret != 0i32 {
        println!("open db err: {:#?}", ret);
        return;
    }

    db_close(db);
}

fn open_write_close() {
    let dbname = DBNAME.as_ptr() as *const c_char;
    let mut db: *mut db_t = null_mut();
    let ret = db_open(dbname, &mut db as *mut *mut db_t);
    if ret != 0 {
        println!("open db err {}", ret);
        return;
    }

    do_write(db);

    db_close(db);
}

fn open_read_close() {
    let dbname = DBNAME.as_ptr() as *const c_char;
    let mut db: *mut db_t = null_mut();
    let ret = db_open(dbname, &mut db as *mut *mut db_t);
    if ret != 0 {
        println!("open db err {}", ret);
        return;
    }

    do_read(db);

    db_close(db);
}

fn open_write_read_close() {
    let dbname = DBNAME.as_ptr() as *const c_char;
    let mut db: *mut db_t = null_mut();
    let ret = db_open(dbname, &mut db as *mut *mut db_t);
    if ret != 0 {
        println!("open db err {}", ret);
        return;
    }

    do_write(db);

    do_read(db);

    db_close(db);
}

fn do_write(db: *mut db_t) {
    let cid = 7;
    let off4k = 1;
    let buflen: isize = 1024 * 1024 * 128;
    let len4k = buflen / 4096;
    let cnt: u32 = cid_cnt;
    let totalKb = ((len4k as u32) *4 *cnt) as u64;

    let mut pagesbuf = PagesBuffer::new(len4k as usize, false);
    let buf = pagesbuf.get_buf_ptr();

    let num_threads = 32usize;

    let pool = match rayon::ThreadPoolBuilder::new().num_threads(num_threads).build() {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            return;
        },
    };
    let pb = &pool;
    println!("start write {} KB", totalKb);

    let ts = ThreadSafeByC::new(db, buf);
    let now = Instant::now();

    pool.scope(move |s| {
        for i in 0..cnt {
            s.spawn(move |_| {
                let ti = match pb.current_thread_index() {
                    Some(t) => t,
                    None => return,
                };
                println!("thread{} :{} start ms {}", ti, i, now.elapsed().as_millis());
                db_write(ts.get_db(), cid+i, off4k, ts.get_buf(), len4k);
                println!("thread {} end ms {}", i, now.elapsed().as_millis());
            });
        }
    });

    let elapsed = now.elapsed().as_micros();
    println!("write speed: {} KB/s ms {}", 
        ((totalKb * 1000_000) as u128) / (elapsed+1), now.elapsed().as_millis());

    pagesbuf.free();
}

fn do_read(db: *mut db_t) {
    let cid = 7;
    let off4k = 1;
    let buflen: isize = 1024 * 1024 * 128;
    let len4k = buflen / 4096;
    let cnt: u32 = cid_cnt;
    let totalKb = (len4k as u32 *4 *cnt) as u64;

    let num_threads = 32usize;

    let pool = match rayon::ThreadPoolBuilder::new().num_threads(num_threads).build() {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            return;
        },
    };
    let pb = &pool;
    println!("start read {} KB", totalKb);

    let ts = ThreadSafeByC::new(db, null_mut());
    let now = Instant::now();

    pb.scope(move |s| {
        for i in 0..cnt {
            s.spawn(move |_| {
                let mut rbuf = PagesBuffer::new(len4k as usize, false);
                let c_rbuf = rbuf.get_buf_ptr();
                db_read(ts.get_db(), cid+i, off4k, c_rbuf, len4k);
                rbuf.free();
            });
        }
    });

    let elapsed = now.elapsed().as_micros();
    if elapsed > 0 {
        println!("read speed: {} KB/s", ((totalKb * 1000_000) as u128) / (elapsed+1));
    }

}

