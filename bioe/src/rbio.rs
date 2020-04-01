use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr::{copy, null_mut};
use std::marker::Send;

use bincode::{deserialize, serialize};

use super::cmod_kv::{
    self, kv_options, kvnvme_options_t, kvnvme_readoptions_t, kvnvme_writeoptions_t, kvnvmedb_t,
};

pub type db_t = kvnvmedb_t;

pub extern "C" fn db_open(
    dbname: *const c_char,
    newdb: *mut *mut db_t,
) -> ::std::os::raw::c_int {
    let opt = kv_options::new();
    let opt = &opt as *const kvnvme_options_t;
    let mut errptr: *mut c_char = null_mut();
    unsafe {
        let name = CStr::from_ptr(dbname);
        println!("open db: <{:?}>", name);
        *newdb = cmod_kv::kvnvme_open(opt, dbname, &mut errptr as *mut *mut c_char);
        if !errptr.is_null() {
            let err = CStr::from_ptr(errptr);
            println!("open db err ***** {:?}", err);
            return -1i32;
        }
    }

    return 0i32;
}

pub extern "C" fn db_close(db: *mut db_t) -> c_int {
    let mut errptr: *mut c_char = null_mut();
    unsafe {
        cmod_kv::kvnvme_close_err(db, &mut errptr as *mut *mut c_char);

        if !errptr.is_null() {
            let err =  CStr::from_ptr(errptr);
            println!("close db err ***** {:?}", err);
            return -1i32;
        }
    }

    return 0i32;
}

pub extern "C" fn db_read(
    db: *mut db_t,
    cid: u32,
    off4k: u32,
    buf: *mut c_char,
    buflen: isize,
) -> ::std::os::raw::c_int {
    let keylen: usize = 8;
    let mut errptr: *mut c_char = null_mut();
    let onepage: isize = 4096;

    let mut snap = cmod_kv::kv_snapshot::new();
    let ropt = kvnvme_readoptions_t::new(&mut snap);

    for i in 0..buflen {
        let rkey: u64 = ((cid as u64) << 32) | ((off4k + i as u32) as u64);
        let key = match serialize::<u64>(&rkey) {
            Ok(k) => k,
            Err(e) => {
                let err = format!("{}", e);
                print!("err: {}", &err);
                return -1;
            }
        };

        //println!("db read cid:{}, off4k:{}, key: {}, bin {:?}", cid, off4k+i as u32, rkey, &key);

        let mut vallen: usize = onepage as usize;
        unsafe {
            let data = cmod_kv::kvnvme_get(
                db,
                &ropt as *const kvnvme_readoptions_t,
                key.as_ptr() as *const i8,
                keylen,
                &mut vallen as *mut usize,
                &mut errptr as *mut *mut c_char,
            );

            if !errptr.is_null() {
                let err = CStr::from_ptr(errptr);
                println!("read db err ***** {:?}", err);
                return -1;
            }

            if data.is_null() {
                println!("db read no record found, cid:{}, off4k:{}", cid, off4k+i as u32);
                continue;
            }

            copy(data, buf.offset(i * onepage), onepage as usize);
        }

    }
    return 0;
}

pub extern "C" fn db_write(
    db: *mut db_t,
    cid: u32,
    off4k: u32,
    buf: *const c_char,
    buflen: isize,
) -> ::std::os::raw::c_int {
    let keylen: usize = 8;
    let mut errptr: *mut c_char = null_mut();
    let onepage: isize = 4096;
    let wopt = kvnvme_writeoptions_t::new();

    for i in 0..buflen {
        let rkey: u64 = ((cid as u64) << 32) | ((off4k + i as u32) as u64);
        let key = match serialize::<u64>(&rkey) {
            Ok(k) => k,
            Err(e) => {
                let err = format!("{}", e);
                print!("err: {}", &err);
                return -1;
            }
        };
        //println!("db write cid:{}, off4k:{}, key: {}, bin {:?}", cid, off4k+i as u32, rkey, key);

        let vallen: usize = onepage as usize;
        unsafe {
            cmod_kv::kvnvme_put(
                db,
                &wopt as *const kvnvme_writeoptions_t,
                key.as_ptr() as *const i8,
                keylen,
                buf.offset(i * onepage),
                vallen,
                &mut errptr as *mut *mut c_char,
            );
    
            if !errptr.is_null() {
                let err = CStr::from_ptr(errptr);
                println!("write db err ***** {:?}", err);
                return -1;
            }
        }
    }

    return 0;
}
